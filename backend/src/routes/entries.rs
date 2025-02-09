use super::ApiState;
use crate::{
    db, error::{ApiError, ApiResult, ErrorMessage}, model::{Cell, CreateEntry, EntryId, EntryTable, FieldOptions}, routes::validate_user_table, Id
};
use axum::{
    extract::{Path, State},
    routing::{patch, post},
    Json, Router,
};
use itertools::Itertools;

const IS_REQUIRED_MESSAGE: &str = "A value is required";
const OUT_OF_RANGE_MESSAGE: &str = "Value is out of range";
const PROGRESS_EXCEEDS_MESSAGE: &str = "Progress value exceeds total steps";
const ENUMERATION_VALUE_MISSING_MESSAGE: &str = "Enumeration value is does not exist";
const INVALID_TYPE_MESSAGE: &str = "Value is not the correct type";
const INVALID_FIELD_ID_MESSAGE: &str = "Field ID key is invalid";

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
    Json(CreateEntry(entry)): Json<CreateEntry>,
) -> ApiResult<Json<EntryId>> {
    let mut tx = pool.begin().await?;

    let user_id = db::debug_get_user_id(tx.as_mut()).await?;
    validate_user_table(tx.as_mut(), user_id, table_id).await?;

    let fields_options = db::get_fields_options(tx.as_mut(), table_id).await?;
    if fields_options.len() == 0 {
        return Err(ApiError::NotFound);
    }

    let error_messages = entry
        .iter()
        .filter_map(|(field_id, cell)| {
            let message = if let Some(field_options) = fields_options.get(field_id) {
                validate_cell(cell, field_options)?
            } else {
                INVALID_FIELD_ID_MESSAGE
            };
            Some(ErrorMessage::new(field_id.to_string(), message))
        })
        .collect_vec();

    if error_messages.len() > 0 {
        return Err(ApiError::unprocessable_entity(error_messages));
    }

    let entry_id = db::create_entry(tx.as_mut(), table_id, entry).await?;

    Ok(Json(EntryId { entry_id }))
}

async fn get_entries(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
) -> ApiResult<Json<EntryTable>> {
    let mut tx = pool.begin().await?;

    let user_id = db::debug_get_user_id(tx.as_mut()).await?;
    validate_user_table(tx.as_mut(), user_id, table_id).await?;


    


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
