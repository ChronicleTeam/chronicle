use std::collections::HashSet;

use super::ApiState;
use crate::{
    db,
    error::{ApiError, ApiResult, ErrorMessage},
    model::data::{CreateField, Field, FieldKind, SetFieldOrder, UpdateField},
    Id,
};
use anyhow::anyhow;
use axum::{
    extract::{Path, State},
    routing::{patch, post},
    Json, Router,
};
use itertools::Itertools;

const INVALID_RANGE: ErrorMessage =
    ErrorMessage::new_static("range", "Range start bound is greater than end bound");
// const FIELD_NAME_CONFLICT: ErrorMessage =
//     ErrorMessage::new_static("name", "Field name already used for this table");
const FIELD_ID_NOT_FOUND: &str = "Field ID not found";
const FIELD_ID_MISSING: &str = "Field ID missing";
const INVALID_ORDERING: &str = "Ordering number does not follow the sequence";

pub fn router() -> Router<ApiState> {
    Router::new().nest(
        "/tables/{table-id}/fields",
        Router::new()
            .route("/", post(create_field).get(get_fields))
            .route("/{field_id}", patch(update_field).delete(delete_field))
            .route("/order", patch(set_field_order)),
    )
}

/// Create a field in a table.
///
/// # Errors
/// - [`ApiError::Unauthorized`]: User not authenticated
/// - [`ApiError::Forbidden`]: User does not have access to that table or field
/// - [`ApiError::NotFound`]: Table or field not found
/// - [`ApiError::UnprocessableEntity`]:
///     - [`INVALID_RANGE`]
///
async fn create_field(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
    Json(mut create_field): Json<CreateField>,
) -> ApiResult<Json<Field>> {
    let user_id = db::debug_get_user_id(&pool).await?;
    db::check_table_relation(&pool, user_id, table_id)
        .await?
        .to_api_result()?;

    validate_field_kind(&mut create_field.field_kind)?;

    let field = db::create_field(&pool, table_id, create_field).await?;

    Ok(Json(field))
}

/// Update a field in a table.
///
/// # Errors
/// - [`ApiError::Unauthorized`]: User not authenticated
/// - [`ApiError::Forbidden`]: User does not have access to that table or field
/// - [`ApiError::NotFound`]: Table or field not found
/// - [`ApiError::UnprocessableEntity`]:
///     - [`INVALID_RANGE`]
///
async fn update_field(
    State(ApiState { pool, .. }): State<ApiState>,
    Path((table_id, field_id)): Path<(Id, Id)>,
    Json(mut update_field): Json<UpdateField>,
) -> ApiResult<Json<Field>> {
    let user_id = db::debug_get_user_id(&pool).await?;
    db::check_table_relation(&pool, user_id, table_id)
        .await?
        .to_api_result()?;
    db::check_field_relation(&pool, table_id, field_id)
        .await?
        .to_api_result()?;

    validate_field_kind(&mut update_field.field_kind)?;

    let field = db::update_field(&pool, field_id, update_field).await?;

    Ok(Json(field))
}

/// Delete a field and all cells in its respective column in the table.
///
/// # Errors
/// - [`ApiError::Unauthorized`]: User not authenticated
/// - [`ApiError::Forbidden`]: User does not have access to that table or field
/// - [`ApiError::NotFound`]: Table or field not found
///
async fn delete_field(
    State(ApiState { pool, .. }): State<ApiState>,
    Path((table_id, field_id)): Path<(Id, Id)>,
) -> ApiResult<()> {
    let user_id = db::debug_get_user_id(&pool).await?;
    db::check_table_relation(&pool, user_id, table_id)
        .await?
        .to_api_result()?;
    db::check_field_relation(&pool, table_id, field_id)
        .await?
        .to_api_result()?;

    db::delete_field(&pool, field_id).await?;

    Ok(())
}

/// Get all fields in a table.
///
/// # Errors
/// - [`ApiError::Unauthorized`]: User not authenticated
/// - [`ApiError::Forbidden`]: User does not have access to that table
/// - [`ApiError::NotFound`]: Table not found
///
async fn get_fields(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
) -> ApiResult<Json<Vec<Field>>> {
    let user_id = db::debug_get_user_id(&pool).await?;
    db::check_table_relation(&pool, user_id, table_id)
        .await?
        .to_api_result()?;

    let fields = db::get_fields(&pool, table_id).await?;

    Ok(Json(fields))
}

async fn set_field_order(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
    Json(SetFieldOrder(order)): Json<SetFieldOrder>,
) -> ApiResult<()> {
    let user_id = db::debug_get_user_id(&pool).await?;

    db::check_table_relation(&pool, user_id, table_id)
        .await?
        .to_api_result()?;

    let mut field_ids: HashSet<_> = db::get_field_ids(&pool, table_id)
        .await?
        .into_iter()
        .collect();

    let mut error_messages = order
        .iter()
        .sorted_by_key(|(_, ordering)| **ordering)
        .enumerate()
        .filter_map(|(idx, (field_id, ordering))| {
            let key = field_id.to_string();
            if !field_ids.remove(field_id) {
                Some(ErrorMessage::new(key, FIELD_ID_NOT_FOUND))
            } else if idx as i32 != *ordering {
                Some(ErrorMessage::new(key, INVALID_ORDERING))
            } else {
                None
            }
        })
        .collect_vec();

    error_messages.extend(
        field_ids
            .into_iter()
            .map(|field_id| ErrorMessage::new(field_id.to_string(), FIELD_ID_MISSING)),
    );

    if error_messages.len() > 0 {
        // return Err(ApiError::unprocessable_entity(error_messages));
    }

    db::set_field_order(&pool, order).await?;

    Ok(())
}

/// Validates [`FieldKind`] from requests.
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
            number_precision,
            number_scale,
            ..
        } => {
            validate_range(*range_start, *range_end)?;
            *number_precision = number_precision.map(|n| n.max(1));
            *number_scale = number_scale.map(|n| n.max(0));
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
            // date_time_format,
            ..
        } => validate_range(*range_start, *range_end)?,
        FieldKind::Enumeration {
            values,
            default_value,
            ..
        } => {
            if !values.contains_key(&default_value) {
                return Err(
                    anyhow!("enumeration field default value does not map to a value").into(),
                );
            }
        }
        _ => {}
    };
    Ok(())
}

/// Validates a range definition for validating fields.
fn validate_range<T>(range_start: Option<T>, range_end: Option<T>) -> ApiResult<()>
where
    T: PartialOrd,
{
    if range_start
        .zip(range_end)
        .map_or(true, |(start, end)| start <= end)
    {
        Ok(())
    } else {
        Err(ApiError::unprocessable_entity([("Range", "INVALID_RANGE")]))
    }
}
