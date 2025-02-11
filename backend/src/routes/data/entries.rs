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
use itertools::Itertools;

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
    Json(CreateEntry(mut entry)): Json<CreateEntry>,
) -> ApiResult<Json<EntryId>> {
    let mut tx = pool.begin().await?;

    let user_id = db::debug_get_user_id(tx.as_mut()).await?;
    match db::check_table_ownership(tx.as_mut(), user_id, table_id).await? {
        db::Relation::Owned => {}
        db::Relation::NotOwned => return Err(ApiError::Forbidden),
        db::Relation::Absent => return Err(ApiError::NotFound),
    }

    let fields_options = db::get_fields_options(tx.as_mut(), table_id).await?;

    let mut error_messages = fields_options
        .iter()
        .filter_map(|(field_id, options)| {
            let cell = entry.remove(field_id);
            let message = validate_cell(&cell, options)?;

            Some(ErrorMessage::new(field_id.to_string(), message))
        })
        .collect_vec();

    error_messages.extend(
        entry
            .keys()
            .map(|field_id| ErrorMessage::new(field_id.to_string(), INVALID_FIELD_ID_MESSAGE)),
    );

    if error_messages.len() > 0 {
        return Err(ApiError::unprocessable_entity(error_messages));
    }

    let entry_id = db::create_entry(tx.as_mut(), table_id, entry).await?;

    Ok(Json(EntryId { entry_id }))
}

async fn update_entry() {
    todo!()
}

async fn delete_entry() {}

fn validate_cell(cell: &Option<Cell>, field_options: &FieldOptions) -> Option<&'static str> {
    if let Some(cell) = cell {
        match (cell, field_options) {
            (
                Cell::Integer { i: value },
                FieldOptions::Integer {
                    range_start,
                    range_end,
                    ..
                },
            ) => check_range(value, range_start.as_ref(), range_end.as_ref()),
            (
                Cell::Float { f: value },
                FieldOptions::Decimal {
                    range_start,
                    range_end,
                    scientific_notation,
                    number_precision,
                    number_scale,
                    ..
                },
            ) => check_range(value, range_start.as_ref(), range_end.as_ref()),
            (
                Cell::Decimal { d: value },
                FieldOptions::Money {
                    range_start,
                    range_end,
                    ..
                },
            ) => check_range(value, range_start.as_ref(), range_end.as_ref()),
            (Cell::Integer { i: value }, FieldOptions::Progress { total_steps }) => {
                if *value > *total_steps as i64 || *value < 0 {
                    Some(OUT_OF_RANGE_MESSAGE)
                } else {
                    None
                }
            }
            (
                Cell::DateTime(value),
                FieldOptions::DateTime {
                    range_start,
                    range_end,
                    date_time_format,
                    ..
                },
            ) => check_range(value, range_start.as_ref(), range_end.as_ref()),
            (Cell::Interval(_), FieldOptions::Interval { .. }) => todo!(),
            (Cell::String(_), FieldOptions::WebLink { is_required }) => None,
            (Cell::String(_), FieldOptions::Email { is_required }) => None,
            (Cell::Boolean(_), FieldOptions::Checkbox) => None,
            (
                Cell::Integer { i: value },
                FieldOptions::Enumeration {
                    values,
                    default_value,
                    ..
                },
            ) => {
                if values.contains_key(&(*value as i32)) {
                    Some(ENUMERATION_VALUE_MISSING_MESSAGE)
                } else {
                    None
                }
            }
            (Cell::Image(_), FieldOptions::Image { .. }) => todo!(),
            (Cell::File(_), FieldOptions::File { .. }) => todo!(),
            _ => Some(INVALID_TYPE_MESSAGE),
        }
    } else {
        match field_options {
            FieldOptions::Text { is_required }
            | FieldOptions::Integer { is_required, .. }
            | FieldOptions::Decimal { is_required, .. }
            | FieldOptions::Money { is_required, .. }
            | FieldOptions::DateTime { is_required, .. }
            | FieldOptions::WebLink { is_required, .. }
            | FieldOptions::Email { is_required, .. }
            | FieldOptions::Enumeration { is_required, .. }
            | FieldOptions::Image { is_required, .. }
            | FieldOptions::File { is_required, .. } => {
                if *is_required {
                    Some(IS_REQUIRED_MESSAGE)
                } else {
                    None
                }
            }
            _ => Some(INVALID_TYPE_MESSAGE),
        }
    }
}

fn check_range<T>(value: T, range_start: Option<T>, range_end: Option<T>) -> Option<&'static str>
where
    T: PartialOrd,
{
    if range_start.map_or(false, |start| value < start)
        || range_end.map_or(false, |end| value > end)
    {
        Some(OUT_OF_RANGE_MESSAGE)
    } else {
        None
    }
}
