use crate::{
    AppState, Id,
    auth::AppAuthSession,
    db,
    error::{ApiError, ApiResult},
    model::{
        Cell,
        data::{CreateEntries, Entry, FieldKind, FieldMetadata, UpdateEntry},
        users::{AccessRole, AccessRoleCheck},
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
use sqlx::PgExecutor;
use std::{collections::HashMap, str::FromStr};

const IS_REQUIRED: &str = "A value is required";
const OUT_OF_RANGE: &str = "Value is out of range";
const ENUMERATION_VALUE_MISSING: &str = "Enumeration value is does not exist";
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

/// Create many entries in a table.
///
/// Can optionally take a parent entry ID.
///
/// # Errors
/// - [`ApiError::Unauthorized`]: User not authenticated
/// - [`ApiError::Forbidden`]: User does not have access to that table or
/// - [`ApiError::NotFound`]: Table or parent entry not found
/// - [`ApiError::UnprocessableEntity`]:
///     - <field_id>: [`IS_REQUIRED`]
///     - <field_id>: [`INVALID_TYPE`]
///     - <field_id>: [`ENUMERATION_VALUE_MISSING`]
///     - <field_id>: [`INVALID_FIELD_ID`]
///
async fn create_entries(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(table_id): Path<Id>,
    Json(CreateEntries { parent_id, entries }): Json<CreateEntries>,
) -> ApiResult<Json<Vec<Entry>>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::get_table_access(&db, user_id, table_id)
        .await?
        .check(AccessRole::Editor)?;

    if let Some(parent_entry_id) = parent_id {
        check_parent_id(&db, parent_entry_id, table_id).await?;
    }

    let fields = db::get_fields_metadata(&db, table_id).await?;

    let entries = entries
        .into_iter()
        .map(|cells| convert_cells(cells, &fields))
        .try_collect()?;

    let entries = db::create_entries(&db, table_id, parent_id, fields, entries).await?;

    Ok(Json(entries))
}

/// Update an entry in a table.
///
/// Can optionally take a parent entry ID.
///
/// # Errors
/// - [`ApiError::Unauthorized`]: User not authenticated
/// - [`ApiError::Forbidden`]: User does not have access to that table
/// - [`ApiError::NotFound`]: Table, entry, or parent entry not found
/// - [`ApiError::UnprocessableEntity`]:
///     - [`IS_REQUIRED`]
///     - [`INVALID_TYPE`]
///     - [`ENUMERATION_VALUE_MISSING`]
///     - [`INVALID_FIELD_ID`]
///
async fn update_entry(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path((table_id, entry_id)): Path<(Id, Id)>,
    Json(UpdateEntry { parent_id, cells }): Json<UpdateEntry>,
) -> ApiResult<Json<Entry>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::get_table_access(&db, user_id, table_id)
        .await?
        .check(AccessRole::Editor)?;
    if !db::entry_exists(&db, table_id, entry_id).await? {
        return Err(ApiError::NotFound);
    }
    if let Some(parent_entry_id) = parent_id {
        check_parent_id(&db, parent_entry_id, table_id).await?;
    }

    let fields = db::get_fields_metadata(&db, table_id).await?;

    let cells = convert_cells(cells, &fields)?;

    let entry = db::update_entry(&db, table_id, entry_id, parent_id, fields, cells).await?;

    Ok(Json(entry))
}

async fn delete_entry(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path((table_id, entry_id)): Path<(Id, Id)>,
) -> ApiResult<()> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::get_table_access(&db, user_id, table_id)
        .await?
        .check(AccessRole::Editor)?;
    if !db::entry_exists(&db, table_id, entry_id).await? {
        return Err(ApiError::NotFound);
    }

    db::delete_entry(&db, table_id, entry_id).await?;

    Ok(())
}

async fn check_parent_id(
    executor: impl PgExecutor<'_> + Clone,
    parent_entry_id: Id,
    table_id: Id,
) -> ApiResult<()> {
    let parent_table_id = db::get_table_parent_id(executor.clone(), table_id)
        .await?
        .ok_or(ApiError::UnprocessableEntity(NO_PARENT_TABLE.into()))?;
    if !db::entry_exists(executor, parent_table_id, parent_entry_id).await? {
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
        .into_iter()
        .map(|field| {
            let json_value = raw_cells.remove(&field.field_id).unwrap_or(Value::Null);
            Ok(json_to_cell(json_value, &field.field_kind)
                .map_err(|message| format!("{}: {message}", field.field_id))?)
        })
        .partition_result();

    error_messages.extend(
        raw_cells
            .keys()
            .map(|field_id| format!("{}: {INVALID_FIELD_ID}", field_id)),
    );

    if error_messages.len() > 0 {
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
                // scientific_notation,
                // number_precision,
                // number_scale,
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
                if value > *total_steps as i64 || value < 0 {
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
                // date_time_format,
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
    if range_start.map_or(false, |start| value < start)
        || range_end.map_or(false, |end| value > end)
    {
        Err(OUT_OF_RANGE)
    } else {
        Ok(())
    }
}

mod docs {
    use crate::{
        api::data::entries::{
            ENUMERATION_VALUE_MISSING, INVALID_FIELD_ID, INVALID_TYPE, IS_REQUIRED,
            NO_PARENT_TABLE, PARENT_ID_NOT_FOUND,
        },
        docs::{ENTRIES_TAG, TransformOperationExt, template},
        model::{data::Entry, users::AccessRole},
    };
    use aide::{OperationOutput, transform::TransformOperation};
    use axum::Json;
    use itertools::Itertools;
    
    const TABLE_EDITOR: [(&str, AccessRole); 1] = [("Table", AccessRole::Editor)];

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
        .map(|v| format!("<field_id> : {v}"))
        .chain([NO_PARENT_TABLE.into(), PARENT_ID_NOT_FOUND.into()])
        .join("\n\n");

        entries::<Json<Vec<Entry>>>(
            op,
            "create_entries",
            "Create many entries in a table. Can optionally take a parent entry ID.",
        )
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
