use super::ApiState;
use crate::{
    error::{ApiError, ApiResult, ErrorMessage},
    model::{Cell, CreateEntry, EntryId, FieldOptions},
    Id,
};
use axum::{
    extract::{Path, State},
    routing::{patch, post},
    Json, Router,
};
use itertools::Itertools;
use serde::de::value;

const IS_REQUIRED_MESSAGE: &str = "A value is required";
const OUT_OF_RANGE_MESSAGE: &str = "Value is out of range";
const PROGRESS_EXCEEDS_MESSAGE: &str = "Progress value exceeds total steps";
const ENUMERATION_VALUE_MISSING_MESSAGE: &str = "Enumeration value is does not exist";
const INVALID_TYPE_MESSAGE: &str = "Value is not the correct type";

pub(crate) fn router() -> Router<ApiState> {
    Router::new().nest(
        "/tables/{table_id}/entries",
        Router::new()
            .route("/", post(create_entry).get(get_entries))
            .route("/{entry_id}", patch(update_entry).delete(delete_entry)),
    )
}

async fn create_entry(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
    Json(CreateEntry(create_entry)): Json<CreateEntry>,
) -> ApiResult<Json<EntryId>> {
    let tx = pool.begin().await?;

    let error_messages = create_entry.iter().filter_map(|(field_id, cell)| {
        Some(ErrorMessage::new(
            field_id.to_string(),
            validate_cell(cell, todo!())?,
        ))
    }).collect_vec();
    if error_messages.len() > 0 {
        return Err(ApiError::unprocessable_entity(error_messages));
    }
    

    todo!()
}

async fn get_entries() {
    todo!()
}

async fn update_entry() {
    todo!()
}

async fn delete_entry() {}

fn validate_cell(cell: &Cell, field_options: &FieldOptions) -> Option<&'static str> {
    match (cell, field_options) {
        (Cell::Text(value), FieldOptions::Text { is_required }) => {
            check_required(value.as_ref(), *is_required)
        }

        (
            Cell::Integer(value),
            FieldOptions::Integer {
                is_required,
                range_start,
                range_end,
            },
        ) => check_required(value.as_ref(), *is_required)
            .or_else(|| check_range(value.as_ref(), range_start.as_ref(), range_end.as_ref())),
        (
            Cell::Decimal(value),
            FieldOptions::Decimal {
                is_required,
                range_start,
                range_end,
                scientific_notation,
                number_precision,
                number_scale,
            },
        ) => check_required(value.as_ref(), *is_required)
            .or_else(|| check_range(value.as_ref(), range_start.as_ref(), range_end.as_ref())),
        (
            Cell::Money(value),
            FieldOptions::Money {
                is_required,
                range_start,
                range_end,
            },
        ) => check_required(value.as_ref(), *is_required)
            .or_else(|| check_range(value.as_ref(), range_start.as_ref(), range_end.as_ref())),
        (Cell::Progress(value), FieldOptions::Progress { total_steps }) => {
            if value.map_or(false, |v| v > *total_steps) {
                Some(PROGRESS_EXCEEDS_MESSAGE)
            } else {
                None
            }
        }
        (
            Cell::DateTime(value),
            FieldOptions::DateTime {
                is_required,
                range_start,
                range_end,
                date_time_format,
            },
        ) => check_required(value.as_ref(), *is_required)
            .or_else(|| check_range(value.as_ref(), range_start.as_ref(), range_end.as_ref())),
        (Cell::Interval(_), FieldOptions::Interval { is_required }) => todo!(),
        (Cell::WebLink(value), FieldOptions::WebLink { is_required }) => {
            check_required(value.as_ref(), *is_required)
        }
        (Cell::Email(value), FieldOptions::Email { is_required }) => {
            check_required(value.as_ref(), *is_required)
        }
        (Cell::Checkbox(_), FieldOptions::Checkbox) => None,
        (
            Cell::Enumeration(value),
            FieldOptions::Enumeration {
                is_required,
                values,
                default_value,
            },
        ) => check_required(*value, *is_required).or_else(|| {
            if value.map_or(false, |v| values.contains_key(&v)) {
                Some(ENUMERATION_VALUE_MISSING_MESSAGE)
            } else {
                None
            }
        }),
        (Cell::Image(_), FieldOptions::Image { is_required }) => todo!(),
        (Cell::File(_), FieldOptions::File { is_required }) => todo!(),
        _ => Some(INVALID_TYPE_MESSAGE),
    }
}

fn check_required<T>(value: Option<T>, is_required: bool) -> Option<&'static str> {
    if matches!(value, None) && is_required {
        Some(IS_REQUIRED_MESSAGE)
    } else {
        None
    }
}

fn check_range<T>(
    value: Option<T>,
    range_start: Option<T>,
    range_end: Option<T>,
) -> Option<&'static str>
where
    T: PartialOrd,
{
    if value.map_or(false, |v| {
        range_start.map_or(false, |start| v < start) || range_end.map_or(false, |end| v > end)
    }) {
        Some(OUT_OF_RANGE_MESSAGE)
    } else {
        None
    }
}
