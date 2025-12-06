use crate::{
    AppState, Id,
    api::NO_DATA_IN_REQUEST_BODY,
    auth::AppAuthSession,
    db,
    error::{ApiError, ApiResult},
    model::{
        Cell,
        access::{AccessRole, AccessRoleCheck, Resource},
        data::{CreateEntries, Entry, FieldKind, FieldMetadata, SelectTable, UpdateEntry},
    },
};
use aide::{
    NoApi,
    axum::{
        ApiRouter,
        routing::{patch_with, post_with},
    },
};
use axum::{
    Json,
    extract::{Path, State},
};
use axum_login::AuthSession;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use rust_decimal::Decimal;
use serde_json::Value;
use sqlx::{Acquire, Postgres};
use std::{collections::HashMap, str::FromStr};

const IS_REQUIRED: &str = "A value is required";
const OUT_OF_RANGE: &str = "Value is out of range";
const ENUMERATION_VALUE_MISSING: &str = "Enumeration value does not exist";
const INVALID_TYPE: &str = "Value is not the correct type";
const INVALID_FIELD_ID: &str = "Field ID key is invalid";
const PARENT_ID_NOT_FOUND: &str = "Entry parent ID not found";
const NO_PARENT_TABLE: &str = "This table has no parent table";

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new().nest(
        "/tables/{table_id}/entries",
        ApiRouter::new()
            .api_route("/", post_with(create_entries, docs::create_entries))
            .api_route(
                "/{entry_id}",
                patch_with(update_entry, docs::update_entry)
                    .delete_with(delete_entry, docs::delete_entry),
            ),
    )
}

async fn create_entries(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectTable { table_id }): Path<SelectTable>,
    Json(CreateEntries { parent_id, entries }): Json<CreateEntries>,
) -> ApiResult<Json<Vec<Entry>>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;

    db::get_access_role(tx.as_mut(), Resource::Table, table_id, user_id)
        .await?
        .check(AccessRole::Editor)?;

    if entries.is_empty() {
        return Err(ApiError::BadRequest(NO_DATA_IN_REQUEST_BODY.into()));
    }

    if let Some(parent_entry_id) = parent_id {
        check_parent_id(tx.as_mut(), parent_entry_id, table_id).await?;
    }

    let fields = db::get_fields_metadata(tx.as_mut(), table_id).await?;
    let entries = entries
        .into_iter()
        .map(|cells| convert_cells(cells, &fields))
        .try_collect()?;

    let entries = db::create_entries(tx.as_mut(), table_id, parent_id, fields, entries).await?;

    tx.commit().await?;
    Ok(Json(entries))
}

async fn update_entry(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path((table_id, entry_id)): Path<(Id, Id)>,
    Json(UpdateEntry { parent_id, cells }): Json<UpdateEntry>,
) -> ApiResult<Json<Entry>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;

    db::get_access_role(tx.as_mut(), Resource::Table, table_id, user_id)
        .await?
        .check(AccessRole::Editor)?;
    if !db::entry_exists(tx.as_mut(), table_id, entry_id).await? {
        return Err(ApiError::NotFound);
    }
    if let Some(parent_entry_id) = parent_id {
        check_parent_id(tx.as_mut(), parent_entry_id, table_id).await?;
    }

    let fields = db::get_fields_metadata(tx.as_mut(), table_id).await?;

    let cells = convert_cells(cells, &fields)?;

    let entry = db::update_entry(tx.as_mut(), table_id, entry_id, parent_id, fields, cells).await?;

    tx.commit().await?;
    Ok(Json(entry))
}

async fn delete_entry(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path((table_id, entry_id)): Path<(Id, Id)>,
) -> ApiResult<()> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;

    db::get_access_role(tx.as_mut(), Resource::Table, table_id, user_id)
        .await?
        .check(AccessRole::Editor)?;
    if !db::entry_exists(tx.as_mut(), table_id, entry_id).await? {
        return Err(ApiError::NotFound);
    }

    db::delete_entry(tx.as_mut(), table_id, entry_id).await?;

    tx.commit().await?;
    Ok(())
}

async fn check_parent_id(
    conn: impl Acquire<'_, Database = Postgres>,
    parent_entry_id: Id,
    table_id: Id,
) -> ApiResult<()> {
    let mut tx = conn.begin().await?;
    let parent_table_id = db::get_table_parent_id(tx.as_mut(), table_id)
        .await?
        .ok_or(ApiError::UnprocessableEntity(NO_PARENT_TABLE.into()))?;

    if !db::entry_exists(tx.as_mut(), parent_table_id, parent_entry_id).await? {
        return Err(ApiError::UnprocessableEntity(PARENT_ID_NOT_FOUND.into()));
    }
    Ok(())
}

/// Convert raw JSON cell values to a list of cells.
fn convert_cells(
    mut raw_cells: HashMap<Id, Value>,
    fields: &[FieldMetadata],
) -> ApiResult<Vec<Cell>> {
    let (new_cells, mut error_messages): (Vec<_>, Vec<_>) = fields
        .iter()
        .map(|field| {
            let json_value = raw_cells.remove(&field.field_id).unwrap_or(Value::Null);
            json_to_cell(json_value, &field.field_kind)
                .map_err(|message| format!("{}: {message}", field.field_id))
        })
        .partition_result();

    error_messages.extend(
        raw_cells
            .keys()
            .map(|field_id| format!("{field_id}: {INVALID_FIELD_ID}")),
    );

    if !error_messages.is_empty() {
        return Err(ApiError::UnprocessableEntity(error_messages.join(", ")));
    }

    Ok(new_cells)
}

/// Converts a JSON value to a [`Cell`] and return the correct error message on failure.
fn json_to_cell(value: Value, field_kind: &FieldKind) -> Result<Cell, &'static str> {
    match (value, field_kind) {
        (
            Value::Null,
            FieldKind::Text { is_required }
            | FieldKind::Integer { is_required, .. }
            | FieldKind::Float { is_required, .. }
            | FieldKind::Money { is_required, .. }
            | FieldKind::DateTime { is_required, .. }
            | FieldKind::WebLink { is_required, .. }
            | FieldKind::Enumeration { is_required, .. },
        ) => {
            if *is_required {
                Err(IS_REQUIRED)
            } else {
                Ok(Cell::Null)
            }
        }
        (
            Value::Number(value),
            FieldKind::Integer {
                range_start,
                range_end,
                ..
            },
        ) => {
            if let Some(value) = value.as_i64() {
                check_range(&value, range_start.as_ref(), range_end.as_ref())?;
                Ok(Cell::Integer(value))
            } else {
                Err(INVALID_TYPE)
            }
        }

        (
            Value::Number(value),
            FieldKind::Float {
                range_start,
                range_end,
                ..
            },
        ) => {
            if let Some(value) = value.as_f64() {
                check_range(&value, range_start.as_ref(), range_end.as_ref())?;
                Ok(Cell::Float(value))
            } else {
                Err(INVALID_TYPE)
            }
        }
        (
            Value::String(value),
            FieldKind::Money {
                range_start,
                range_end,
                ..
            },
        ) => {
            if let Ok(value) = Decimal::from_str_radix(&value, 10) {
                check_range(&value, range_start.as_ref(), range_end.as_ref())?;
                Ok(Cell::Decimal(value))
            } else {
                Err(INVALID_TYPE)
            }
        }
        (Value::Number(value), FieldKind::Progress { total_steps }) => {
            if let Some(value) = value.as_i64() {
                if value > *total_steps || value < 0 {
                    Err(OUT_OF_RANGE)
                } else {
                    Ok(Cell::Integer(value))
                }
            } else {
                Err(INVALID_TYPE)
            }
        }
        (
            Value::String(value),
            FieldKind::DateTime {
                range_start,
                range_end,
                ..
            },
        ) => {
            if let Ok(value) = DateTime::<Utc>::from_str(&value) {
                check_range(&value, range_start.as_ref(), range_end.as_ref())?;
                Ok(Cell::DateTime(value))
            } else {
                Err(INVALID_TYPE)
            }
        }
        (Value::String(value), FieldKind::Text { .. } | FieldKind::WebLink { .. }) => {
            Ok(Cell::String(value))
        }
        (Value::Bool(value), FieldKind::Checkbox) => Ok(Cell::Boolean(value)),
        (Value::Number(value), FieldKind::Enumeration { values, .. }) => {
            if let Some(value) = value.as_i64() {
                if values.contains_key(&value) {
                    Ok(Cell::Integer(value))
                } else {
                    Err(ENUMERATION_VALUE_MISSING)
                }
            } else {
                Err(INVALID_TYPE)
            }
        }
        _ => Err(INVALID_TYPE),
    }
}

/// Check that cell value is within the range specified by the field options.
fn check_range<T>(
    value: &T,
    range_start: Option<&T>,
    range_end: Option<&T>,
) -> Result<(), &'static str>
where
    T: PartialOrd,
{
    if range_start.is_some_and(|start| value < start) || range_end.is_some_and(|end| value > end) {
        Err(OUT_OF_RANGE)
    } else {
        Ok(())
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
mod docs {
    use crate::{
        api::{
            NO_DATA_IN_REQUEST_BODY,
            data::entries::{
                ENUMERATION_VALUE_MISSING, INVALID_FIELD_ID, INVALID_TYPE, IS_REQUIRED,
                NO_PARENT_TABLE, PARENT_ID_NOT_FOUND,
            },
        },
        docs::{ENTRIES_TAG, TransformOperationExt, template},
        model::{
            access::{AccessRole, Resource},
            data::Entry,
        },
    };
    use aide::{OperationOutput, transform::TransformOperation};
    use axum::Json;
    use itertools::Itertools;

    const TABLE_EDITOR: [(Resource, AccessRole); 1] = [(Resource::Table, AccessRole::Editor)];

    fn entries<'a, R: OperationOutput>(
        op: TransformOperation<'a>,
        summary: &'a str,
        description: &'a str,
    ) -> TransformOperation<'a> {
        template::<R>(op, summary, description, true, ENTRIES_TAG)
    }

    pub fn create_entries(op: TransformOperation) -> TransformOperation {
        let errors = [
            IS_REQUIRED,
            INVALID_TYPE,
            ENUMERATION_VALUE_MISSING,
            INVALID_FIELD_ID,
        ]
        .into_iter()
        .map(|v| format!("<field_id>: {v}"))
        .chain([NO_PARENT_TABLE.into(), PARENT_ID_NOT_FOUND.into()])
        .join("\n\n");

        entries::<Json<Vec<Entry>>>(
            op,
            "create_entries",
            "Create many entries in a table. Can optionally take a parent entry ID.",
        )
        .response_description::<400, String>(NO_DATA_IN_REQUEST_BODY)
        .response_description::<404, ()>("Table not found")
        .response_description::<422, String>(&errors)
        .required_access(TABLE_EDITOR)
    }

    pub fn update_entry(op: TransformOperation) -> TransformOperation {
        let errors = [
            IS_REQUIRED,
            INVALID_TYPE,
            ENUMERATION_VALUE_MISSING,
            INVALID_FIELD_ID,
            NO_PARENT_TABLE,
            PARENT_ID_NOT_FOUND,
        ]
        .join("\n\n");
        entries::<()>(
            op,
            "update_entry",
            "Update an entry in a table. Can optionally take a parent entry ID.",
        )
        .response_description::<404, ()>("Table not found\n\nEntry not found")
        .response_description::<422, String>(&errors)
        .required_access(TABLE_EDITOR)
    }

    pub fn delete_entry(op: TransformOperation) -> TransformOperation {
        entries::<()>(op, "delete_entry", "Delete an entry from a table.")
            .response_description::<404, ()>("Table not found\n\nEntry not found")
            .required_access(TABLE_EDITOR)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use crate::{
        Id, db,
        model::{
            Cell,
            access::{AccessRole, Resource},
            data::{
                CreateEntries, CreateField, CreateTable, FieldIdentifier, FieldKind, FieldMetadata,
                TableIdentifier, UpdateEntry,
            },
        },
        test_util,
    };
    use chrono::{DateTime, Utc};
    use itertools::Itertools;
    use num_traits::FromPrimitive;
    use rust_decimal::Decimal;
    use serde::Serialize;
    use serde_json::{Value, json};
    use sqlx::{PgPool, Row, types::Json};
    use std::{collections::HashMap, str::FromStr};

    #[sqlx::test]
    async fn create_entries(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let parent_table_id = db::create_table(
            &db,
            CreateTable {
                name: "parent".into(),
                description: "".into(),
                parent_id: None,
            },
        )
        .await?
        .table_id;
        let field = FieldMetadata::from_field(
            db::create_field(
                &db,
                parent_table_id,
                CreateField {
                    name: "abc".into(),
                    field_kind: FieldKind::Checkbox,
                },
            )
            .await?,
        );
        let parent_entry_id = db::create_entries(
            &db,
            parent_table_id,
            None,
            vec![field],
            vec![vec![Cell::Boolean(true)]],
        )
        .await?
        .first()
        .unwrap()
        .entry_id;
        let table_id = db::create_table(
            &db,
            CreateTable {
                name: "child".into(),
                description: "".into(),
                parent_id: Some(parent_table_id),
            },
        )
        .await?
        .table_id;
        let field_id = db::create_field(
            &db,
            table_id,
            CreateField {
                name: "abc".into(),
                field_kind: FieldKind::Checkbox,
            },
        )
        .await?
        .field_id;

        let path = format!("/api/tables/{table_id}/entries");

        let create_entries = CreateEntries {
            parent_id: Some(parent_entry_id),
            entries: vec![HashMap::from_iter([(field_id, json!(false))])],
        };

        server
            .post(&path)
            .json(&create_entries)
            .await
            .assert_status_unauthorized();

        let user = db::create_user(&db, "test".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &user).await;
        test_util::test_access_control(
            &db,
            Resource::Table,
            table_id,
            user.user_id,
            AccessRole::Editor,
            async || server.post(&path).json(&create_entries).await,
        )
        .await;

        server
            .post("/api/tables/1000/entries")
            .json(&create_entries)
            .await
            .assert_status_not_found();

        let entry_value = true;
        let create_entries = CreateEntries {
            parent_id: Some(parent_entry_id),
            entries: vec![HashMap::from_iter([(field_id, json!(entry_value))])],
        };
        let response = server.post(&path).json(&create_entries).await;
        response.assert_status_ok();
        let entries: Vec<Value> = response.json();
        assert_eq!(entries.len(), 1);

        let entry_1 = entries.into_iter().next().unwrap();
        let entry_id_1: Id = serde_json::from_value(entry_1.get("entry_id").unwrap().clone())?;
        let parent_id_1: Option<Id> =
            serde_json::from_value(entry_1.get("parent_id").unwrap().clone())?;
        let created_at_1: DateTime<Utc> =
            serde_json::from_value(entry_1.get("created_at").unwrap().clone())?;
        let updated_at_1: Option<DateTime<Utc>> =
            serde_json::from_value(entry_1.get("updated_at").unwrap().clone())?;
        let value_1: bool = serde_json::from_value(
            entry_1
                .get("cells")
                .unwrap()
                .get(field_id.to_string())
                .unwrap()
                .clone(),
        )?;

        assert_eq!(parent_id_1, create_entries.parent_id);
        assert_eq!(value_1, entry_value);

        let table_ident = TableIdentifier::new(table_id, "data_table");
        let field_ident = FieldIdentifier::new(field_id);
        let entry_2 = sqlx::query(&format!(
            r#"SELECT * FROM {table_ident} WHERE entry_id = $1"#
        ))
        .bind(entry_id_1)
        .fetch_one(&db)
        .await?;
        assert_eq!(entry_2.get::<Id, _>("entry_id"), entry_id_1);
        assert_eq!(entry_2.get::<Option<Id>, _>("parent_id"), parent_id_1);
        assert_eq!(entry_2.get::<DateTime<Utc>, _>("created_at"), created_at_1);
        assert_eq!(
            entry_2.get::<Option<DateTime<Utc>>, _>("updated_at"),
            updated_at_1
        );
        assert_eq!(
            entry_2.get::<bool, _>(field_ident.unquote().as_str()),
            value_1
        );

        let no_data = CreateEntries {
            parent_id: Some(parent_entry_id),
            entries: vec![],
        };
        server
            .post(&path)
            .json(&no_data)
            .await
            .assert_status_bad_request();

        let wrong_parent_id = CreateEntries {
            parent_id: Some(1000),
            entries: vec![HashMap::from_iter([(field_id, json!(false))])],
        };
        server
            .post(&path)
            .json(&wrong_parent_id)
            .await
            .assert_status_unprocessable_entity();

        let invalid_type = CreateEntries {
            parent_id: Some(parent_entry_id),
            entries: vec![HashMap::from_iter([(field_id, json!(null))])],
        };
        server
            .post(&path)
            .json(&invalid_type)
            .await
            .assert_status_unprocessable_entity();
        Ok(())
    }

    #[sqlx::test]
    async fn update_entry(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let parent_table_id = db::create_table(
            &db,
            CreateTable {
                name: "parent".into(),
                description: "".into(),
                parent_id: None,
            },
        )
        .await?
        .table_id;
        let field = FieldMetadata::from_field(
            db::create_field(
                &db,
                parent_table_id,
                CreateField {
                    name: "abc".into(),
                    field_kind: FieldKind::Checkbox,
                },
            )
            .await?,
        );
        let mut parent_entry_iter = db::create_entries(
            &db,
            parent_table_id,
            None,
            vec![field],
            vec![vec![Cell::Boolean(true)], vec![Cell::Boolean(true)]],
        )
        .await?
        .into_iter();
        let parent_entry_id_1 = parent_entry_iter.next().unwrap().entry_id;
        let parent_entry_id_2 = parent_entry_iter.next().unwrap().entry_id;
        let table_id = db::create_table(
            &db,
            CreateTable {
                name: "child".into(),
                description: "".into(),
                parent_id: Some(parent_table_id),
            },
        )
        .await?
        .table_id;
        let field = FieldMetadata::from_field(
            db::create_field(
                &db,
                table_id,
                CreateField {
                    name: "abc".into(),
                    field_kind: FieldKind::Checkbox,
                },
            )
            .await?,
        );
        let field_id = field.field_id;
        let entry_id = db::create_entries(
            &db,
            table_id,
            Some(parent_entry_id_1),
            vec![field],
            vec![vec![Cell::Boolean(true)]],
        )
        .await?
        .first()
        .unwrap()
        .entry_id;

        let path = format!("/api/tables/{table_id}/entries/{entry_id}");

        let update_entry = UpdateEntry {
            parent_id: Some(parent_entry_id_1),
            cells: HashMap::from_iter([(field_id, json!(true))]),
        };

        server
            .patch(&path)
            .json(&update_entry)
            .await
            .assert_status_unauthorized();

        let user = db::create_user(&db, "test".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &user).await;
        test_util::test_access_control(
            &db,
            Resource::Table,
            table_id,
            user.user_id,
            AccessRole::Editor,
            async || server.patch(&path).json(&update_entry).await,
        )
        .await;

        for path_wrong in [
            format!("/api/tables/1000/entries/{entry_id}"),
            format!("/api/tables/{table_id}/entries/1000"),
        ] {
            server
                .patch(&path_wrong)
                .json(&update_entry)
                .await
                .assert_status_not_found();
        }

        let entry_value = false;
        let update_entry = UpdateEntry {
            parent_id: Some(parent_entry_id_2),
            cells: HashMap::from_iter([(field_id, json!(entry_value))]),
        };
        let response = server.patch(&path).json(&update_entry).await;
        response.assert_status_ok();
        let entry_1: Value = response.json();
        let entry_id_1: Id = serde_json::from_value(entry_1.get("entry_id").unwrap().clone())?;
        let parent_id_1: Option<Id> =
            serde_json::from_value(entry_1.get("parent_id").unwrap().clone())?;
        let created_at_1: DateTime<Utc> =
            serde_json::from_value(entry_1.get("created_at").unwrap().clone())?;
        let updated_at_1: Option<DateTime<Utc>> =
            serde_json::from_value(entry_1.get("updated_at").unwrap().clone())?;
        let value_1: bool = serde_json::from_value(
            entry_1
                .get("cells")
                .unwrap()
                .get(field_id.to_string())
                .unwrap()
                .clone(),
        )?;

        assert_eq!(parent_id_1, update_entry.parent_id);
        assert_eq!(value_1, entry_value);

        let table_ident = TableIdentifier::new(table_id, "data_table");
        let field_ident = FieldIdentifier::new(field_id);
        let entry_2 = sqlx::query(&format!(
            r#"SELECT * FROM {table_ident} WHERE entry_id = $1"#
        ))
        .bind(entry_id_1)
        .fetch_one(&db)
        .await?;
        assert_eq!(entry_2.get::<Id, _>("entry_id"), entry_id_1);
        assert_eq!(entry_2.get::<Option<Id>, _>("parent_id"), parent_id_1);
        assert_eq!(entry_2.get::<DateTime<Utc>, _>("created_at"), created_at_1);
        assert_eq!(
            entry_2.get::<Option<DateTime<Utc>>, _>("updated_at"),
            updated_at_1
        );
        assert_eq!(
            entry_2.get::<bool, _>(field_ident.unquote().as_str()),
            value_1
        );

        let wrong_parent_id = UpdateEntry {
            parent_id: Some(1000),
            cells: HashMap::from_iter([(field_id, json!(false))]),
        };
        server
            .patch(&path)
            .json(&wrong_parent_id)
            .await
            .assert_status_unprocessable_entity();

        let invalid_type = UpdateEntry {
            parent_id: Some(parent_entry_id_2),
            cells: HashMap::from_iter([(field_id, json!(null))]),
        };
        server
            .patch(&path)
            .json(&invalid_type)
            .await
            .assert_status_unprocessable_entity();
        Ok(())
    }

    #[sqlx::test]
    async fn delete_entry(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let table_id = db::create_table(
            &db,
            CreateTable {
                name: "test".into(),
                description: "".into(),
                parent_id: None,
            },
        )
        .await?
        .table_id;
        let field = FieldMetadata::from_field(
            db::create_field(
                &db,
                table_id,
                CreateField {
                    name: "abc".into(),
                    field_kind: FieldKind::Checkbox,
                },
            )
            .await?,
        );

        server
            .delete("/api/tables/1000/entries/1000")
            .await
            .assert_status_unauthorized();

        let user = db::create_user(&db, "test".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &user).await;
        test_util::test_access_control(
            &db,
            Resource::Table,
            table_id,
            user.user_id,
            AccessRole::Editor,
            async || {
                let entry_id = db::create_entries(
                    &db,
                    table_id,
                    None,
                    vec![field.clone()],
                    vec![vec![Cell::Boolean(true)]],
                )
                .await
                .unwrap()
                .first()
                .unwrap()
                .entry_id;
                server
                    .delete(&format!("/api/tables/{table_id}/entries/{entry_id}"))
                    .await
            },
        )
        .await;

        let entry_id = db::create_entries(
            &db,
            table_id,
            None,
            vec![field],
            vec![vec![Cell::Boolean(true)]],
        )
        .await
        .unwrap()
        .first()
        .unwrap()
        .entry_id;

        for path_wrong in [
            format!("/api/tables/1000/entries/{entry_id}"),
            format!("/api/tables/{table_id}/entries/1000"),
        ] {
            server.delete(&path_wrong).await.assert_status_not_found();
        }

        let path = format!("/api/tables/{table_id}/entries/{entry_id}");

        server.delete(&path).await.assert_status_ok();
        let table_ident = TableIdentifier::new(table_id, "data_table");
        let not_exists: bool = sqlx::query_scalar(&format!(
            r#"SELECT NOT EXISTS (SELECT 1 FROM {table_ident} WHERE entry_id = $1)"#
        ))
        .bind(entry_id)
        .fetch_one(&db)
        .await?;
        assert!(not_exists);

        server.delete(&path).await.assert_status_not_found();
        Ok(())
    }

    #[sqlx::test]
    async fn check_parent_id(db: PgPool) -> anyhow::Result<()> {
        let parent_table_id = db::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "parent".into(),
                description: "".into(),
            },
        )
        .await?
        .table_id;
        let field = FieldMetadata::from_field(
            db::create_field(
                &db,
                parent_table_id,
                CreateField {
                    name: "test".into(),
                    field_kind: FieldKind::Checkbox,
                },
            )
            .await?,
        );
        let parent_entry_id = db::create_entries(
            &db,
            parent_table_id,
            None,
            vec![field],
            vec![vec![Cell::Boolean(true)]],
        )
        .await?
        .first()
        .unwrap()
        .entry_id;

        let table_id = db::create_table(
            &db,
            CreateTable {
                parent_id: Some(parent_table_id),
                name: "child".into(),
                description: "".into(),
            },
        )
        .await?
        .table_id;

        let mut conn = db.acquire().await?;
        super::check_parent_id(conn.as_mut(), parent_entry_id, parent_table_id)
            .await
            .unwrap_err();
        super::check_parent_id(conn.as_mut(), parent_entry_id + 1, table_id)
            .await
            .unwrap_err();
        super::check_parent_id(conn.as_mut(), parent_entry_id, table_id)
            .await
            .unwrap();
        Ok(())
    }

    fn check_range_data<'a, T>(
        ok_value: &'a T,
        low_value: &'a T,
        high_value: &'a T,
        range_start: &'a T,
        range_end: &'a T,
    ) -> Vec<(&'a T, Option<&'a T>, Option<&'a T>, bool)>
    where
        T: PartialOrd,
    {
        vec![
            (ok_value, None, None, true),
            (low_value, None, None, true),
            (high_value, None, None, true),
            (ok_value, Some(range_start), None, true),
            (low_value, Some(range_start), None, false),
            (high_value, Some(range_start), None, true),
            (ok_value, None, Some(range_end), true),
            (low_value, None, Some(range_end), true),
            (high_value, None, Some(range_end), false),
            (ok_value, Some(range_start), Some(range_end), true),
            (low_value, Some(range_start), Some(range_end), false),
            (high_value, Some(range_start), Some(range_end), false),
        ]
    }

    #[test]
    fn convert_cells() {
        let text_value = "abcdef";
        let integer_value = 3;
        let float_value = 0.5;
        let money_value = "20000";
        let progress_value = 60;
        let date_time_value = "2020-01-20 00:00:00Z";
        let web_link_value = "https://example.com";
        let checkbox_value = false;
        let enumeration_value = 2;

        let ok_values: HashMap<i32, Value> = [
            json!(text_value),
            json!(integer_value),
            json!(float_value),
            json!(money_value),
            json!(progress_value),
            json!(date_time_value),
            json!(web_link_value),
            json!(checkbox_value),
            json!(enumeration_value),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, v)| (i as i32, v))
        .collect();

        let err_values: HashMap<i32, Value> = [
            json!(null),
            json!(6),
            json!(10.0),
            json!("100"),
            json!(101),
            json!("2015-01-01 00:00:00Z"),
            json!(null),
            json!("no"),
            json!(3),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, v)| (i as i32, v))
        .collect();

        let fields = [
            FieldKind::Text { is_required: true },
            FieldKind::Integer {
                is_required: true,
                range_start: Some(1),
                range_end: Some(5),
            },
            FieldKind::Float {
                is_required: true,
                range_start: Some(0.0),
                range_end: Some(1.0),
            },
            FieldKind::Money {
                is_required: true,
                range_start: Some(Decimal::from_i32(1_000).unwrap()),
                range_end: Some(Decimal::from_i32(100_000).unwrap()),
            },
            FieldKind::Progress { total_steps: 100 },
            FieldKind::DateTime {
                is_required: true,
                range_start: Some(DateTime::from_str("2018-01-01 00:00:00Z").unwrap()),
                range_end: Some(DateTime::from_str("2030-12-31 00:00:00Z").unwrap()),
            },
            FieldKind::WebLink { is_required: true },
            FieldKind::Checkbox,
            FieldKind::Enumeration {
                is_required: true,
                values: HashMap::from_iter([(0, "A".into()), (1, "B".into()), (2, "C".into())]),
                default_value: 0,
            },
        ]
        .into_iter()
        .enumerate()
        .map(|(field_id, field_kind)| FieldMetadata {
            field_id: field_id as i32,
            field_kind: Json(field_kind),
        })
        .collect_vec();

        let expected_cells = [
            Cell::String(text_value.into()),
            Cell::Integer(integer_value),
            Cell::Float(float_value),
            Cell::Decimal(Decimal::from_str_exact(money_value).unwrap()),
            Cell::Integer(progress_value),
            Cell::DateTime(DateTime::from_str(date_time_value).unwrap()),
            Cell::String(web_link_value.into()),
            Cell::Boolean(checkbox_value),
            Cell::Integer(enumeration_value),
        ];

        let cells = super::convert_cells(ok_values.clone(), &fields).unwrap();
        assert_eq!(expected_cells.len(), cells.len());
        assert!(
            cells
                .into_iter()
                .zip(expected_cells)
                .all(|(c1, c2)| c1 == c2)
        );

        for (id, err_value) in err_values.iter() {
            let mut raw_cells = ok_values.clone();
            *raw_cells.get_mut(id).unwrap() = err_value.clone();
            super::convert_cells(raw_cells, &fields).unwrap_err();
        }

        super::convert_cells(err_values, &fields).unwrap_err();
    }

    #[test]
    fn json_to_cell() {
        fn test_is_required<F>(get_field_kind: F)
        where
            F: Fn(bool) -> FieldKind,
        {
            assert!(super::json_to_cell(Value::Null, &get_field_kind(true)).is_err());
            assert_eq!(
                super::json_to_cell(Value::Null, &get_field_kind(false)).unwrap(),
                Cell::Null
            );
        }

        fn test_numeric<T, S, F, C>(
            ok_value: T,
            low_value: T,
            high_value: T,
            range_start: T,
            range_end: T,
            invalid_value: S,
            get_field_kind: F,
            get_cell: C,
        ) where
            T: PartialOrd + Serialize + Clone,
            S: Serialize,
            F: Fn(Option<T>, Option<T>) -> FieldKind,
            C: Fn(T) -> Cell,
        {
            for (value, range_start, range_end, is_ok) in
                check_range_data(&ok_value, &low_value, &high_value, &range_start, &range_end)
            {
                let cell = super::json_to_cell(
                    serde_json::to_value(value).unwrap(),
                    &get_field_kind(range_start.cloned(), range_end.cloned()),
                );
                if is_ok {
                    assert_eq!(cell.unwrap(), get_cell(value.clone()))
                } else {
                    assert!(cell.is_err())
                }
            }
            assert!(
                super::json_to_cell(
                    serde_json::to_value(invalid_value).unwrap(),
                    &get_field_kind(None, None),
                )
                .is_err()
            );
        }

        test_is_required(|is_required| FieldKind::Text { is_required });
        test_is_required(|is_required| FieldKind::Integer {
            is_required,
            range_start: None,
            range_end: None,
        });
        test_is_required(|is_required| FieldKind::Float {
            is_required,
            range_start: None,
            range_end: None,
        });
        test_is_required(|is_required| FieldKind::Money {
            is_required,
            range_start: None,
            range_end: None,
        });
        test_is_required(|is_required| FieldKind::DateTime {
            is_required,
            range_start: None,
            range_end: None,
        });
        test_is_required(|is_required| FieldKind::WebLink { is_required });
        test_is_required(|is_required| FieldKind::Enumeration {
            is_required,
            values: HashMap::new(),
            default_value: 0,
        });

        test_numeric(
            10,
            0,
            1000,
            1,
            100,
            "10",
            |range_start, range_end| FieldKind::Integer {
                is_required: true,
                range_start,
                range_end,
            },
            Cell::Integer,
        );
        test_numeric(
            0.5,
            -0.01,
            100.0,
            0.0,
            1.0,
            "0.5",
            |range_start, range_end| FieldKind::Float {
                is_required: true,
                range_start,
                range_end,
            },
            Cell::Float,
        );
        test_numeric(
            Decimal::from_f32(50.25).unwrap(),
            Decimal::from_f32(-10.5).unwrap(),
            Decimal::from_str_exact("999999999999.99").unwrap(),
            Decimal::from_f32(0.0).unwrap(),
            Decimal::from_f32(100.0).unwrap(),
            50.25,
            |range_start, range_end| FieldKind::Money {
                is_required: true,
                range_start,
                range_end,
            },
            Cell::Decimal,
        );
        test_numeric(
            "2012-12-12 00:00:00Z",
            "2012-10-31 00:00:00Z",
            "2013-01-02 00:00:00Z",
            "2012-11-01 00:00:00Z",
            "2013-01-01 00:00:00Z",
            "2012-12-12",
            |range_start, range_end| FieldKind::DateTime {
                is_required: true,
                range_start: range_start.map(|s| DateTime::from_str(s).unwrap()),
                range_end: range_end.map(|s| DateTime::from_str(s).unwrap()),
            },
            |value| Cell::DateTime(DateTime::from_str(value).unwrap()),
        );

        let progress_field = FieldKind::Progress { total_steps: 100 };
        for (value, is_ok) in [
            (10, true),
            (0, true),
            (100, true),
            (-1, false),
            (200, false),
        ] {
            let cell = super::json_to_cell(serde_json::to_value(value).unwrap(), &progress_field);
            if is_ok {
                assert_eq!(cell.unwrap(), Cell::Integer(value))
            } else {
                assert!(cell.is_err())
            }
        }
        assert!(super::json_to_cell(serde_json::to_value("10").unwrap(), &progress_field).is_err());

        assert_eq!(
            super::json_to_cell(serde_json::to_value(true).unwrap(), &FieldKind::Checkbox).unwrap(),
            Cell::Boolean(true)
        );

        let enumeration_field = FieldKind::Enumeration {
            is_required: true,
            values: HashMap::from_iter([
                (0, "High".into()),
                (1, "Medium".into()),
                (2, "Low".into()),
            ]),
            default_value: 0,
        };
        for (value, is_ok) in [(-1, false), (0, true), (1, true), (2, true), (3, false)] {
            let cell =
                super::json_to_cell(serde_json::to_value(value).unwrap(), &enumeration_field);
            if is_ok {
                assert_eq!(cell.unwrap(), Cell::Integer(value))
            } else {
                assert!(cell.is_err())
            }
        }
        assert!(
            super::json_to_cell(serde_json::to_value("0").unwrap(), &enumeration_field).is_err()
        );
    }

    #[test]
    fn check_range() {
        for (value, range_start, range_end, is_ok) in check_range_data(&0, &-1, &2, &0, &1) {
            assert_eq!(
                super::check_range(value, range_start, range_end).is_ok(),
                is_ok
            );
        }
    }
}
