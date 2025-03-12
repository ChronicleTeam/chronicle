use super::ApiState;
use crate::{
    db,
    error::{ApiError, ApiResult, ErrorMessage},
    model::{
        data::{CreateEntry, Entry, FieldKind, FieldMetadata, UpdateEntry},
        Cell,
    },
    Id,
};
use axum::{
    extract::{Path, State},
    routing::{post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use rust_decimal::Decimal;
use serde_json::Value;
use std::{collections::HashMap, str::FromStr};

const IS_REQUIRED: &str = "A value is required";
const OUT_OF_RANGE: &str = "Value is out of range";
const ENUMERATION_VALUE_MISSING: &str = "Enumeration value is does not exist";
const INVALID_TYPE: &str = "Value is not the correct type";
const INVALID_FIELD_ID: &str = "Field ID key is invalid";

pub fn router() -> Router<ApiState> {
    Router::new().nest(
        "/tables/{table-id}/entries",
        Router::new()
            .route("/", post(create_entry))
            .route(
                "/{entry-id}",
                put(update_entry).delete(delete_entry),
            ),
    )
}

/// Create an entry in a table.
///
/// # Errors
/// - [`ApiError::Unauthorized`]: User not authenticated
/// - [`ApiError::Forbidden`]: User does not have access to that table
/// - [`ApiError::NotFound`]: Table not found
/// - [`ApiError::UnprocessableEntity`]:
///     - [`IS_REQUIRED`]
///     - [`INVALID_TYPE`]
///     - [`ENUMERATION_VALUE_MISSING`]
///     - [`INVALID_FIELD_ID`]
///
async fn create_entry(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
    Json(CreateEntry(entry)): Json<CreateEntry>,
) -> ApiResult<Json<Entry>> {
    let user_id = db::debug_get_user_id(&pool).await?;
    db::check_table_relation(&pool, user_id, table_id)
        .await?
        .to_api_result()?;

    let fields = db::get_fields_metadata(&pool, table_id).await?;

    let entry = convert_entry(entry, fields)?;

    let entry = db::create_entry(&pool, table_id, entry).await?;

    Ok(Json(entry))
}

/// Update an entry in a table.
///
/// # Errors
/// - [`ApiError::Unauthorized`]: User not authenticated
/// - [`ApiError::Forbidden`]: User does not have access to that table
/// - [`ApiError::NotFound`]: Table or entry not found
/// - [`ApiError::UnprocessableEntity`]:
///     - [`IS_REQUIRED`]
///     - [`INVALID_TYPE`]
///     - [`ENUMERATION_VALUE_MISSING`]
///     - [`INVALID_FIELD_ID`]
///
async fn update_entry(
    State(ApiState { pool, .. }): State<ApiState>,
    Path((table_id, entry_id)): Path<(Id, Id)>,
    Json(UpdateEntry(entry)): Json<UpdateEntry>,
) -> ApiResult<Json<Entry>> {
    let user_id = db::debug_get_user_id(&pool).await?;
    db::check_table_relation(&pool, user_id, table_id)
        .await?
        .to_api_result()?;
    db::check_entry_relation(&pool, table_id, entry_id)
        .await?
        .to_api_result()?;

    let fields = db::get_fields_metadata(&pool, table_id).await?;

    let entry = convert_entry(entry, fields)?;

    let entry = db::update_entry(&pool, table_id, entry_id, entry).await?;

    Ok(Json(entry))
}

/// Delete an entry from a table.
///
/// # Errors
/// - [`ApiError::Unauthorized`]: User not authenticated
/// - [`ApiError::Forbidden`]: User does not have access to that table
/// - [`ApiError::NotFound`]: Table or entry not found
///
async fn delete_entry(
    State(ApiState { pool, .. }): State<ApiState>,
    Path((table_id, entry_id)): Path<(Id, Id)>,
) -> ApiResult<()> {
    let user_id = db::debug_get_user_id(&pool).await?;
    db::check_table_relation(&pool, user_id, table_id)
        .await?
        .to_api_result()?;
    db::check_entry_relation(&pool, table_id, entry_id)
        .await?
        .to_api_result()?;

    db::delete_entry(&pool, table_id, entry_id).await?;

    Ok(())
}

/// Convert an input entry from a request to a [`CellMap`] and
/// performs validation on each value in the entry.
fn convert_entry(
    mut entry: HashMap<Id, Value>,
    fields: Vec<FieldMetadata>,
) -> ApiResult<Vec<(Option<Cell>, FieldMetadata)>> {
    let (new_entry, mut error_messages): (Vec<_>, Vec<_>) = fields
        .into_iter()
        .map(|field| {
            let json_value = entry.remove(&field.field_id).unwrap_or(Value::Null);
            Ok((
                json_to_cell(json_value, &field.field_kind)
                    .map_err(|message| ErrorMessage::new(field.field_id.to_string(), message))?,
                field,
            ))
        })
        .partition_result();

    error_messages.extend(
        entry
            .keys()
            .map(|field_id| ErrorMessage::new(field_id.to_string(), INVALID_FIELD_ID)),
    );

    if error_messages.len() > 0 {
        return Err(ApiError::unprocessable_entity(error_messages));
    }

    Ok(new_entry)
}

/// Converts a JSON value to a [`Cell`] and returns the
/// correct error message on failure.
fn json_to_cell(value: Value, field_kind: &FieldKind) -> Result<Option<Cell>, &'static str> {
    match (value, field_kind) {
        (
            Value::Null,
            FieldKind::Text { is_required }
            | FieldKind::Integer { is_required, .. }
            | FieldKind::Float { is_required, .. }
            | FieldKind::Money { is_required, .. }
            | FieldKind::DateTime { is_required, .. }
            | FieldKind::WebLink { is_required, .. }
            | FieldKind::Email { is_required, .. }
            | FieldKind::Enumeration { is_required, .. },
        ) => {
            if *is_required {
                Err(IS_REQUIRED)
            } else {
                Ok(None)
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
                Ok(Some(Cell::Integer(value)))
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
                Ok(Some(Cell::Float(value)))
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
                Ok(Some(Cell::Decimal(value)))
            } else {
                Err(INVALID_TYPE)
            }
        }
        (Value::Number(value), FieldKind::Progress { total_steps }) => {
            if let Some(value) = value.as_i64() {
                if value > *total_steps as i64 || value < 0 {
                    Err(OUT_OF_RANGE)
                } else {
                    Ok(Some(Cell::Integer(value)))
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
                Ok(Some(Cell::DateTime(value)))
            } else {
                Err(INVALID_TYPE)
            }
        }
        (
            Value::String(value),
            FieldKind::Text { .. } | FieldKind::WebLink { .. } | FieldKind::Email { .. },
        ) => Ok(Some(Cell::String(value))),
        (Value::Bool(value), FieldKind::Checkbox) => Ok(Some(Cell::Boolean(value))),
        (Value::Number(value), FieldKind::Enumeration { values, .. }) => {
            if let Some(value) = value.as_i64() {
                if values.contains_key(&value) {
                    Ok(Some(Cell::Integer(value)))
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

/// Check that a value is within the range for validating entries.
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
