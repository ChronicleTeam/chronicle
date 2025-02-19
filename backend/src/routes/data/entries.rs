use std::{collections::HashMap, str::FromStr};

use super::ApiState;
use crate::{
    db,
    error::{ApiError, ApiResult, ErrorMessage},
    model::data::{Cell, CreateEntry, EntryId, FieldOptions},
    Id,
};
use axum::{
    extract::{Path, State},
    routing::{patch, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use rust_decimal::Decimal;
use serde::de::value;
use serde_json::{from_str, Number, Value};

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
            .route("/{entry_id}", patch(update_entry).delete(delete_entry)),
    )
}

async fn create_entry(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
    Json(CreateEntry(mut raw_entry)): Json<CreateEntry>,
) -> ApiResult<Json<EntryId>> {
    let mut tx = pool.begin().await?;

    let user_id = db::debug_get_user_id(tx.as_mut()).await?;
    match db::check_table_ownership(tx.as_mut(), user_id, table_id).await? {
        db::Relation::Owned => {}
        db::Relation::NotOwned => return Err(ApiError::Forbidden),
        db::Relation::Absent => return Err(ApiError::NotFound),
    }

    let fields_options = db::get_fields_options(tx.as_mut(), table_id).await?;

    let (entry, mut error_messages): (HashMap<_, _>, Vec<_>) = fields_options
        .iter()
        .map(|(field_id, options)| {
            let json_value = raw_entry.remove(field_id).unwrap_or(Value::Null);
            json_value_to_cell(json_value, options)
                .map(|cell| (*field_id, cell))
                .map_err(|message| ErrorMessage::new(field_id.to_string(), message))
        })
        .partition_result();

    error_messages.extend(
        raw_entry
            .keys()
            .map(|field_id| ErrorMessage::new(field_id.to_string(), INVALID_FIELD_ID_MESSAGE)),
    );

    if error_messages.len() > 0 {
        return Err(ApiError::unprocessable_entity(error_messages));
    }

    let entry_id = db::create_entry(tx.as_mut(), table_id, entry).await?;

    tx.commit().await?;

    Ok(Json(EntryId { entry_id }))
}

async fn update_entry() {
    todo!()
}

async fn delete_entry() {}

fn json_value_to_cell(
    value: Value,
    field_options: &FieldOptions,
) -> Result<Option<Cell>, &'static str> {
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
            | FieldOptions::Enumeration { is_required, .. }
            | FieldOptions::Image { is_required, .. }
            | FieldOptions::File { is_required, .. },
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
        (Value::String(value),FieldOptions::Text { .. } | FieldOptions::WebLink { .. } | FieldOptions::Email { .. }) => {
            Ok(Some(Cell::String(value)))
        }
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
        (_, FieldOptions::Image { .. }) => todo!(),
        (_, FieldOptions::File { .. }) => todo!(),
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
