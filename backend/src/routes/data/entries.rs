use super::ApiState;
use crate::{
    db,
    error::{ApiError, ApiResult, ErrorMessage},
    model::data::{Cell, CreateEntry, Entry, FieldOptions, UpdateEntry},
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

const IS_REQUIRED_MESSAGE: &str = "A value is required";
const OUT_OF_RANGE_MESSAGE: &str = "Value is out of range";
const ENUMERATION_VALUE_MISSING_MESSAGE: &str = "Enumeration value is does not exist";
const INVALID_TYPE_MESSAGE: &str = "Value is not the correct type";
const INVALID_FIELD_ID_MESSAGE: &str = "Field ID key is invalid";

pub(crate) fn router() -> Router<ApiState> {
    Router::new().nest(
        "/tables/{table_id}/entries",
        Router::new()
            .route("/", post(create_entry))
            .route("/{entry_id}", put(update_entry).delete(delete_entry)),
    )
}

async fn create_entry(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
    Json(CreateEntry(entry)): Json<CreateEntry>,
) -> ApiResult<Json<Entry>> {
    let user_id = db::debug_get_user_id(&pool).await?;
    db::check_table_relation(&pool, user_id, table_id)
        .await?
        .to_api_result()?;

    let fields_options = db::get_fields_options(&pool, table_id).await?;

    let entry = convert_entry(entry, fields_options)?;

    let entry = db::create_entry(&pool, table_id, entry).await?;

    Ok(Json(entry))
}

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

    let fields_options = db::get_fields_options(&pool, table_id).await?;

    let entry = convert_entry(entry, fields_options)?;

    let entry = db::update_entry(&pool, table_id, entry_id, entry).await?;

    Ok(Json(entry))
}

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

fn convert_entry(
    mut entry: HashMap<Id, Value>,
    fields_options: HashMap<Id, FieldOptions>,
) -> ApiResult<HashMap<Id, Option<Cell>>> {
    let (new_entry, mut error_messages): (HashMap<_, _>, Vec<_>) = fields_options
        .iter()
        .map(|(field_id, options)| {
            let json_value = entry.remove(field_id).unwrap_or(Value::Null);
            json_to_cell(json_value, options)
                .map(|cell| (*field_id, cell))
                .map_err(|message| ErrorMessage::new(field_id.to_string(), message))
        })
        .partition_result();

    error_messages.extend(
        entry
            .keys()
            .map(|field_id| ErrorMessage::new(field_id.to_string(), INVALID_FIELD_ID_MESSAGE)),
    );

    if error_messages.len() > 0 {
        return Err(ApiError::unprocessable_entity(error_messages));
    }

    Ok(new_entry)
}

fn json_to_cell(value: Value, field_options: &FieldOptions) -> Result<Option<Cell>, &'static str> {
    match (value, field_options) {
        (
            Value::Null,
            FieldOptions::Text { is_required }
            | FieldOptions::Integer { is_required, .. }
            | FieldOptions::Decimal { is_required, .. }
            | FieldOptions::Money { is_required, .. }
            | FieldOptions::DateTime { is_required, .. }
            | FieldOptions::WebLink { is_required, .. }
            | FieldOptions::Email { is_required, .. }
            | FieldOptions::Enumeration { is_required, .. },
        ) => {
            if *is_required {
                Err(IS_REQUIRED_MESSAGE)
            } else {
                Ok(None)
            }
        }
        (
            Value::Number(value),
            FieldOptions::Integer {
                range_start,
                range_end,
                ..
            },
        ) => {
            if let Some(value) = value.as_i64() {
                check_range(&value, range_start.as_ref(), range_end.as_ref())?;
                Ok(Some(Cell::Integer(value)))
            } else {
                Err(INVALID_TYPE_MESSAGE)
            }
        }

        (
            Value::Number(value),
            FieldOptions::Decimal {
                range_start,
                range_end,
                scientific_notation,
                number_precision,
                number_scale,
                ..
            },
        ) => {
            if let Some(value) = value.as_f64() {
                check_range(&value, range_start.as_ref(), range_end.as_ref())?;
                Ok(Some(Cell::Float(value)))
            } else {
                Err(INVALID_TYPE_MESSAGE)
            }
        }
        (
            Value::String(value),
            FieldOptions::Money {
                range_start,
                range_end,
                ..
            },
        ) => {
            if let Ok(value) = Decimal::from_str_radix(&value, 10) {
                check_range(&value, range_start.as_ref(), range_end.as_ref())?;
                Ok(Some(Cell::Decimal(value)))
            } else {
                Err(INVALID_TYPE_MESSAGE)
            }
        }
        (Value::Number(value), FieldOptions::Progress { total_steps }) => {
            if let Some(value) = value.as_i64() {
                if value > *total_steps as i64 || value < 0 {
                    Ok(Some(Cell::Integer(value)))
                } else {
                    Err(OUT_OF_RANGE_MESSAGE)
                }
            } else {
                Err(INVALID_TYPE_MESSAGE)
            }
        }
        (
            Value::String(value),
            FieldOptions::DateTime {
                range_start,
                range_end,
                date_time_format,
                ..
            },
        ) => {
            if let Ok(value) = DateTime::<Utc>::from_str(&value) {
                check_range(&value, range_start.as_ref(), range_end.as_ref())?;
                Ok(Some(Cell::DateTime(value)))
            } else {
                Err(INVALID_TYPE_MESSAGE)
            }
        }
        (_, FieldOptions::Interval { .. }) => todo!(),
        (
            Value::String(value),
            FieldOptions::Text { .. } | FieldOptions::WebLink { .. } | FieldOptions::Email { .. },
        ) => Ok(Some(Cell::String(value))),
        (Value::Bool(value), FieldOptions::Checkbox) => Ok(Some(Cell::Boolean(value))),
        (Value::Number(value), FieldOptions::Enumeration { values, .. }) => {
            if let Some(value) = value.as_i64() {
                if values.contains_key(&value) {
                    Ok(Some(Cell::Integer(value)))
                } else {
                    Err(ENUMERATION_VALUE_MISSING_MESSAGE)
                }
            } else {
                Err(INVALID_TYPE_MESSAGE)
            }
        }
        _ => Err(INVALID_TYPE_MESSAGE),
    }
}

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
        Err(OUT_OF_RANGE_MESSAGE)
    } else {
        Ok(())
    }
}
