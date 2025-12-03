use crate::{
    AppState,
    api::NO_DATA_IN_REQUEST_BODY,
    auth::AppAuthSession,
    db,
    error::{ApiError, ApiResult},
    model::{
        access::{AccessRole, AccessRoleCheck, Resource},
        data::{
            CreateField, Field, FieldKind, SelectField, SelectTable, SetFieldOrder, UpdateField,
        },
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
use itertools::Itertools;
use std::collections::HashSet;

const INVALID_RANGE: &str = "Range start bound is greater than end bound";
const ENUMERATION_INVALID_DEFAULT: &str = "Enumeration field default value does not exist";
const FIELD_ID_NOT_FOUND: &str = "Field ID not found";
const FIELD_ID_MISSING: &str = "Field ID missing";
const INVALID_ORDERING: &str = "Ordering number does not follow the sequence";

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new().nest(
        "/tables/{table_id}/fields",
        ApiRouter::new()
            .api_route(
                "/",
                post_with(create_field, docs::create_field).get_with(get_fields, docs::get_fields),
            )
            .api_route(
                "/{field_id}",
                patch_with(update_field, docs::update_field)
                    .delete_with(delete_field, docs::delete_field),
            )
            .api_route("/order", patch_with(set_field_order, docs::set_field_order)),
    )
}

async fn create_field(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectTable { table_id }): Path<SelectTable>,
    Json(mut create_field): Json<CreateField>,
) -> ApiResult<Json<Field>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;

    db::get_access_role(tx.as_mut(), Resource::Table, table_id, user_id)
        .await?
        .check(AccessRole::Owner)?;

    validate_field_kind(&mut create_field.field_kind)?;

    let field = db::create_field(tx.as_mut(), table_id, create_field).await?;

    tx.commit().await?;
    Ok(Json(field))
}

async fn update_field(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectField { table_id, field_id }): Path<SelectField>,
    Json(mut update_field): Json<UpdateField>,
) -> ApiResult<Json<Field>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;

    db::get_access_role(tx.as_mut(), Resource::Table, table_id, user_id)
        .await?
        .check(AccessRole::Owner)?;

    if !db::field_exists(tx.as_mut(), table_id, field_id).await? {
        return Err(ApiError::NotFound);
    };

    validate_field_kind(&mut update_field.field_kind)?;

    let field = db::update_field(tx.as_mut(), field_id, update_field).await?;

    tx.commit().await?;
    Ok(Json(field))
}

async fn delete_field(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectField { table_id, field_id }): Path<SelectField>,
) -> ApiResult<()> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;

    db::get_access_role(tx.as_mut(), Resource::Table, table_id, user_id)
        .await?
        .check(AccessRole::Owner)?;

    if !db::field_exists(tx.as_mut(), table_id, field_id).await? {
        return Err(ApiError::NotFound);
    };

    db::delete_field(tx.as_mut(), field_id).await?;

    tx.commit().await?;
    Ok(())
}

async fn get_fields(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectTable { table_id }): Path<SelectTable>,
) -> ApiResult<Json<Vec<Field>>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::get_access_role(&db, Resource::Table, table_id, user_id)
        .await?
        .check(AccessRole::Viewer)?;

    let fields = db::get_fields(&db, table_id).await?;

    Ok(Json(fields))
}

async fn set_field_order(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectTable { table_id }): Path<SelectTable>,
    Json(SetFieldOrder(order)): Json<SetFieldOrder>,
) -> ApiResult<()> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;

    db::get_access_role(tx.as_mut(), Resource::Table, table_id, user_id)
        .await?
        .check(AccessRole::Owner)?;

    if order.is_empty() {
        return Err(ApiError::BadRequest(NO_DATA_IN_REQUEST_BODY.into()));
    }

    let mut field_ids: HashSet<_> = db::get_field_ids(tx.as_mut(), table_id)
        .await?
        .into_iter()
        .collect();

    let mut error_messages = order
        .iter()
        .sorted_by_key(|(_, ordering)| **ordering)
        .enumerate()
        .filter_map(|(idx, (field_id, ordering))| {
            if !field_ids.remove(field_id) {
                Some(format!("{field_id}: {FIELD_ID_NOT_FOUND}"))
            } else if idx as i32 != *ordering {
                Some(format!("{field_id}: {INVALID_ORDERING}"))
            } else {
                None
            }
        })
        .collect_vec();
    error_messages.extend(
        field_ids
            .into_iter()
            .map(|field_id| format!("{field_id}: {FIELD_ID_MISSING}")),
    );

    if !error_messages.is_empty() {
        return Err(ApiError::UnprocessableEntity(error_messages.join(", ")));
    }

    db::set_field_order(tx.as_mut(), order).await?;

    tx.commit().await?;
    Ok(())
}

/// Validates a request [FieldKind].
fn validate_field_kind(field_kind: &mut FieldKind) -> ApiResult<()> {
    match field_kind {
        FieldKind::Integer {
            range_start,
            range_end,
            ..
        } => validate_range(*range_start, *range_end)?,
        FieldKind::Float {
            range_start,
            range_end,
            ..
        } => {
            validate_range(*range_start, *range_end)?;
        }
        FieldKind::Money {
            range_start,
            range_end,
            ..
        } => validate_range(*range_start, *range_end)?,
        FieldKind::Progress { total_steps } => {
            *total_steps = (*total_steps).max(1);
        }
        FieldKind::DateTime {
            range_start,
            range_end,
            ..
        } => validate_range(*range_start, *range_end)?,
        FieldKind::Enumeration {
            values,
            default_value,
            ..
        } => {
            if !values.contains_key(default_value) {
                return Err(ApiError::UnprocessableEntity(
                    ENUMERATION_INVALID_DEFAULT.into(),
                ));
            }
        }
        _ => {}
    };
    Ok(())
}

/// Validates the range definition of a field.
fn validate_range<T>(range_start: Option<T>, range_end: Option<T>) -> ApiResult<()>
where
    T: PartialOrd,
{
    if range_start
        .zip(range_end)
        .is_none_or(|(start, end)| start <= end)
    {
        Ok(())
    } else {
        Err(ApiError::UnprocessableEntity(INVALID_RANGE.into()))
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
mod docs {
    use crate::{
        api::{
            NO_DATA_IN_REQUEST_BODY,
            data::fields::{FIELD_ID_NOT_FOUND, INVALID_ORDERING, INVALID_RANGE},
        },
        docs::{FIELDS_TAG, TransformOperationExt, template},
        model::{
            access::{AccessRole, Resource},
            data::Field,
        },
    };
    use aide::{OperationOutput, transform::TransformOperation};
    use axum::Json;

    const TABLE_OWNER: [(Resource, AccessRole); 1] = [(Resource::Table, AccessRole::Owner)];
    const TABLE_VIEWER: [(Resource, AccessRole); 1] = [(Resource::Table, AccessRole::Viewer)];

    fn fields<'a, R: OperationOutput>(
        op: TransformOperation<'a>,
        summary: &'a str,
        description: &'a str,
    ) -> TransformOperation<'a> {
        template::<R>(op, summary, description, true, FIELDS_TAG)
            .response_description::<404, ()>("Table not found")
    }

    fn select_fields<'a, R: OperationOutput>(
        op: TransformOperation<'a>,
        summary: &'a str,
        description: &'a str,
    ) -> TransformOperation<'a> {
        fields::<R>(op, summary, description)
            .response_description::<404, ()>("Table not found\n\nField not found")
    }

    pub fn create_field(op: TransformOperation) -> TransformOperation {
        fields::<Json<Field>>(op, "create_field", "Create a field in a table.")
            .response_description::<422, String>(INVALID_RANGE)
            .required_access(TABLE_OWNER)
    }

    pub fn update_field(op: TransformOperation) -> TransformOperation {
        select_fields::<Json<Field>>(op, "update_field", "Update a field's metadata in a table.")
            .response_description::<422, String>(INVALID_RANGE)
            .required_access(TABLE_OWNER)
    }

    pub fn delete_field(op: TransformOperation) -> TransformOperation {
        select_fields::<()>(
            op,
            "delete_field",
            "Delete a field and all cells in its respective column in the table.",
        )
        .required_access(TABLE_OWNER)
    }
    pub fn get_fields(op: TransformOperation) -> TransformOperation {
        select_fields::<Json<Vec<Field>>>(op, "get_fields", "Get all fields in a table.")
            .required_access(TABLE_VIEWER)
    }

    pub fn set_field_order(op: TransformOperation) -> TransformOperation {
        select_fields::<()>(
            op,
            "set_field_order",
            "Set the order of all fields in a table. Ordering numbers must go from `0` to `n-1` where `n` is the total number of fields",
        )
        .response_description::<400, String>(NO_DATA_IN_REQUEST_BODY)
        .response_description::<422, String>(&format!(
            "<field_id>: {FIELD_ID_NOT_FOUND}\n\n<field_id>: {INVALID_ORDERING}"
        ))
        .required_access(TABLE_OWNER)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use std::collections::HashMap;

    use chrono::DateTime;
    use rust_decimal::Decimal;
    use serde_json::json;
    use sqlx::PgPool;

    use crate::{
        db,
        model::{
            access::{AccessRole, Resource},
            data::{CreateField, CreateTable, Field, FieldKind, SetFieldOrder, UpdateField},
        },
        test_util,
    };

    #[sqlx::test]
    async fn create_field(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let table_id = db::create_table(
            &db,
            CreateTable {
                name: "Test".into(),
                description: "".into(),
                parent_id: None,
            },
        )
        .await?
        .table_id;
        let path = format!("/api/tables/{table_id}/fields");

        let create_field = CreateField {
            name: "abc".into(),
            field_kind: FieldKind::Checkbox,
        };
        server
            .post(&path)
            .json(&create_field)
            .await
            .assert_status_unauthorized();

        let user = db::create_user(&db, "test".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &user).await;
        test_util::test_access_control(
            &db,
            Resource::Table,
            table_id,
            user.user_id,
            AccessRole::Owner,
            async || server.post(&path).json(&create_field).await,
        )
        .await;

        server
            .post(&format!("/api/tables/1000/fields"))
            .json(&create_field)
            .await
            .assert_status_not_found();

        let create_field = CreateField {
            name: "def".into(),
            field_kind: FieldKind::Checkbox,
        };
        let response = server.post(&path).json(&create_field).await;
        response.assert_status_ok();
        let field_1: Field = response.json();
        assert_eq!(field_1.name, create_field.name);
        assert_eq!(field_1.field_kind.0, create_field.field_kind);
        let field_2: Field = sqlx::query_as(r#"SELECT * FROM meta_field WHERE field_id = $1"#)
            .bind(field_1.field_id)
            .fetch_one(&db)
            .await?;
        assert_eq!(field_1, field_2);

        let invalid_range = CreateField {
            name: "ghj".into(),
            field_kind: FieldKind::Integer {
                is_required: false,
                range_start: Some(1),
                range_end: Some(-1),
            },
        };
        server
            .post(&path)
            .json(&invalid_range)
            .await
            .assert_status_unprocessable_entity();
        Ok(())
    }

    #[sqlx::test]
    async fn update_field(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let table_id = db::create_table(
            &db,
            CreateTable {
                name: "Test".into(),
                description: "".into(),
                parent_id: None,
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
        let path = format!("/api/tables/{table_id}/fields/{field_id}");

        let update_field = UpdateField {
            name: "def".into(),
            field_kind: FieldKind::Checkbox,
        };
        server
            .patch(&path)
            .json(&update_field)
            .await
            .assert_status_unauthorized();

        let user = db::create_user(&db, "test".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &user).await;
        test_util::test_access_control(
            &db,
            Resource::Table,
            table_id,
            user.user_id,
            AccessRole::Owner,
            async || server.patch(&path).json(&update_field).await,
        )
        .await;

        for path_wrong in [
            format!("/api/tables/{table_id}/fields/1000"),
            format!("/api/tables/1000/fields/{field_id}"),
        ] {
            server
                .patch(&path_wrong)
                .json(&update_field)
                .await
                .assert_status_not_found();
        }

        let update_field = UpdateField {
            name: "ghj".into(),
            field_kind: FieldKind::Text { is_required: false },
        };
        let response = server.patch(&path).json(&update_field).await;
        response.assert_status_ok();
        let field_1: Field = response.json();
        assert_eq!(field_1.name, update_field.name);
        assert_eq!(field_1.field_kind.0, update_field.field_kind);
        let mut field_2: Field = sqlx::query_as(r#"SELECT * FROM meta_field WHERE field_id = $1"#)
            .bind(field_1.field_id)
            .fetch_one(&db)
            .await?;
        field_2.updated_at = None;
        assert_eq!(field_1, field_2);

        let create_field = UpdateField {
            name: "ghj".into(),
            field_kind: FieldKind::Enumeration {
                is_required: false,
                values: HashMap::from_iter([(0, "A".into())]),
                default_value: 1,
            },
        };
        server
            .patch(&path)
            .json(&create_field)
            .await
            .assert_status_unprocessable_entity();
        Ok(())
    }

    #[sqlx::test]
    async fn delete_field(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let table_id = db::create_table(
            &db,
            CreateTable {
                name: "Test".into(),
                description: "".into(),
                parent_id: None,
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
        let path = format!("/api/tables/{table_id}/fields/{field_id}");

        server.delete(&path).await.assert_status_unauthorized();

        let user = db::create_user(&db, "test".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &user).await;
        test_util::test_access_control(
            &db,
            Resource::Table,
            table_id,
            user.user_id,
            AccessRole::Owner,
            async || {
                let field_id = db::create_field(
                    &db,
                    table_id,
                    CreateField {
                        name: "abc".into(),
                        field_kind: FieldKind::Checkbox,
                    },
                )
                .await
                .unwrap()
                .field_id;
                server
                    .delete(&format!("/api/tables/{table_id}/fields/{field_id}"))
                    .await
            },
        )
        .await;

        for path_wrong in [
            format!("/api/tables/{table_id}/fields/1000"),
            format!("/api/tables/1000/fields/{field_id}"),
        ] {
            server.delete(&path_wrong).await.assert_status_not_found();
        }

        server.delete(&path).await.assert_status_ok();
        let not_exists: bool = sqlx::query_scalar(
            r#"SELECT NOT EXISTS (SELECT 1 FROM meta_field WHERE field_id = $1)"#,
        )
        .bind(field_id)
        .fetch_one(&db)
        .await?;
        assert!(not_exists);

        server.delete(&path).await.assert_status_not_found();
        Ok(())
    }

    #[sqlx::test]
    async fn get_fields(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let table_id = db::create_table(
            &db,
            CreateTable {
                name: "Test".into(),
                description: "".into(),
                parent_id: None,
            },
        )
        .await?
        .table_id;
        let path = format!("/api/tables/{table_id}/fields");

        server.get(&path).await.assert_status_unauthorized();

        let user = db::create_user(&db, "test".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &user).await;
        test_util::test_access_control(
            &db,
            Resource::Table,
            table_id,
            user.user_id,
            AccessRole::Viewer,
            async || server.get(&path).await,
        )
        .await;

        server
            .get(&format!("/api/tables/1000/fields"))
            .await
            .assert_status_not_found();

        let response = server.get(&path).await;
        response.assert_status_ok();
        response.assert_json(&json!([]));

        let fields_1 = db::create_fields(
            &db,
            table_id,
            vec![
                CreateField {
                    name: "A".into(),
                    field_kind: FieldKind::Checkbox,
                },
                CreateField {
                    name: "B".into(),
                    field_kind: FieldKind::Checkbox,
                },
                CreateField {
                    name: "C".into(),
                    field_kind: FieldKind::Checkbox,
                },
            ],
        )
        .await?;
        let response = server.get(&path).await;
        response.assert_status_ok();
        let fields_2: Vec<Field> = response.json();
        test_util::assert_eq_vec(fields_1, fields_2, |f| f.field_id);
        Ok(())
    }

    #[sqlx::test]
    async fn set_field_order(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let table_id = db::create_table(
            &db,
            CreateTable {
                name: "Test".into(),
                description: "".into(),
                parent_id: None,
            },
        )
        .await?
        .table_id;
        let fields = db::create_fields(
            &db,
            table_id,
            vec![
                CreateField {
                    name: "A".into(),
                    field_kind: FieldKind::Checkbox,
                },
                CreateField {
                    name: "B".into(),
                    field_kind: FieldKind::Checkbox,
                },
                CreateField {
                    name: "C".into(),
                    field_kind: FieldKind::Checkbox,
                },
            ],
        )
        .await?;
        let path = format!("/api/tables/{table_id}/fields/order");

        let set_field_order = SetFieldOrder(HashMap::from_iter(
            fields.iter().map(|f| (f.field_id, f.ordering)),
        ));
        server
            .patch(&path)
            .json(&set_field_order)
            .await
            .assert_status_unauthorized();

        let user = db::create_user(&db, "test".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &user).await;
        test_util::test_access_control(
            &db,
            Resource::Table,
            table_id,
            user.user_id,
            AccessRole::Owner,
            async || server.patch(&path).json(&set_field_order).await,
        )
        .await;

        server
            .patch(&format!("/api/tables/1000/fields/order"))
            .json(&set_field_order)
            .await
            .assert_status_not_found();

        let set_field_order = SetFieldOrder(HashMap::from_iter(
            fields
                .iter()
                .map(|f| (f.field_id, (f.ordering + 1) % fields.len() as i32)),
        ));
        server
            .patch(&path)
            .json(&set_field_order)
            .await
            .assert_status_ok();

        let fields: Vec<Field> = sqlx::query_as(r#"SELECT * FROM meta_field WHERE table_id = $1"#)
            .bind(table_id)
            .fetch_all(&db)
            .await?;
        assert!(fields.iter().all(|f| {
            set_field_order
                .0
                .get(&f.field_id)
                .map_or(false, |ordering| f.ordering == *ordering)
        }));

        let wrong_ordering = set_field_order
            .0
            .iter()
            .map(|(field_id, ordering)| (*field_id, ordering + 1))
            .collect();
        server
            .patch(&path)
            .json(&SetFieldOrder(wrong_ordering))
            .await
            .assert_status_unprocessable_entity();

        let wrong_field_id = set_field_order
            .0
            .iter()
            .map(|(field_id, ordering)| (field_id + 1, *ordering))
            .collect();
        server
            .patch(&path)
            .json(&SetFieldOrder(wrong_field_id))
            .await
            .assert_status_unprocessable_entity();

        server
            .patch(&path)
            .json(&SetFieldOrder(HashMap::new()))
            .await
            .assert_status_bad_request();
        Ok(())
    }

    fn validate_range_data<T>(lower: T, higher: T) -> Vec<((Option<T>, Option<T>), bool)>
    where
        T: PartialOrd + Copy,
    {
        assert!(lower < higher);
        vec![
            ((None, None), true),
            ((None, Some(higher)), true),
            ((Some(lower), None), true),
            ((Some(higher), Some(higher)), true),
            ((Some(lower), Some(higher)), true),
            ((Some(higher), Some(lower)), false),
        ]
    }

    #[test]
    fn validate_field_kind() {
        for ((range_start, range_end), is_ok) in validate_range_data(0, 10) {
            assert_eq!(
                super::validate_field_kind(&mut FieldKind::Integer {
                    is_required: true,
                    range_start,
                    range_end
                })
                .is_ok(),
                is_ok
            );
        }
        for ((range_start, range_end), is_ok) in validate_range_data(-1.0, 1.0) {
            assert_eq!(
                super::validate_field_kind(&mut FieldKind::Float {
                    is_required: true,
                    range_start,
                    range_end
                })
                .is_ok(),
                is_ok
            );
        }
        for ((range_start, range_end), is_ok) in
            validate_range_data::<Decimal>(1000.into(), 2000.into())
        {
            assert_eq!(
                super::validate_field_kind(&mut FieldKind::Money {
                    is_required: true,
                    range_start,
                    range_end
                })
                .is_ok(),
                is_ok
            );
        }
        for ((range_start, range_end), is_ok) in validate_range_data(
            DateTime::from_timestamp_secs(0).unwrap(),
            DateTime::from_timestamp_secs(1).unwrap(),
        ) {
            assert_eq!(
                super::validate_field_kind(&mut FieldKind::DateTime {
                    is_required: true,
                    range_start,
                    range_end
                })
                .is_ok(),
                is_ok
            );
        }
        for total_steps in [-10, 0, 1, 10] {
            super::validate_field_kind(&mut FieldKind::Progress { total_steps }).unwrap();
        }
        for (default_value, is_ok) in [(-1, false), (0, true), (2, false)] {
            assert_eq!(
                super::validate_field_kind(&mut FieldKind::Enumeration {
                    is_required: true,
                    values: HashMap::from_iter([(0, "A".into()), (1, "B".into())]),
                    default_value,
                })
                .is_ok(),
                is_ok
            );
        }
    }

    #[test]
    fn validate_range() {
        for ((range_start, range_end), is_ok) in validate_range_data(0, 10) {
            assert_eq!(super::validate_range(range_start, range_end).is_ok(), is_ok);
        }
    }
}
