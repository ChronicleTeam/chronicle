use crate::{
    AppState, Id,
    db::{self, AuthSession},
    error::{ApiError, ApiResult},
    model::data::{CreateField, Field, FieldKind, SetFieldOrder, UpdateField},
};
use anyhow::anyhow;
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{patch, post},
};
use itertools::Itertools;
use std::collections::HashSet;

const INVALID_RANGE: &str = "Range start bound is greater than end bound";
const FIELD_ID_NOT_FOUND: &str = "Field ID not found";
const FIELD_ID_MISSING: &str = "Field ID missing";
const INVALID_ORDERING: &str = "Ordering number does not follow the sequence";

pub fn router() -> Router<AppState> {
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
/// - [ApiError::Unauthorized]: User not authenticated
/// - [ApiError::Forbidden]: User does not have access to that table
/// - [ApiError::NotFound]: Table not found
/// - [ApiError::UnprocessableEntity]:
///     - [INVALID_RANGE]
///
async fn create_field(
    AuthSession { user, .. }: AuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(table_id): Path<Id>,
    Json(mut create_field): Json<CreateField>,
) -> ApiResult<Json<Field>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::check_table_relation(&db, user_id, table_id)
        .await?
        .to_api_result()?;

    validate_field_kind(&mut create_field.field_kind)?;

    let field = db::create_field(&db, table_id, create_field).await?;

    Ok(Json(field))
}

/// Update a field's metadata in a table.
///
/// Will perform conversion on the cells if the field kind changes and backup the original cells.
/// Cells that fail to convert are set to null.
///
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
/// - [ApiError::Forbidden]: User does not have access to that table or field
/// - [ApiError::NotFound]: Table or field not found
/// - [ApiError::UnprocessableEntity]:
///     - [INVALID_RANGE]
///
async fn update_field(
    AuthSession { user, .. }: AuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path((table_id, field_id)): Path<(Id, Id)>,
    Json(mut update_field): Json<UpdateField>,
) -> ApiResult<Json<Field>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::check_table_relation(&db, user_id, table_id)
        .await?
        .to_api_result()?;
    db::check_field_relation(&db, table_id, field_id)
        .await?
        .to_api_result()?;

    validate_field_kind(&mut update_field.field_kind)?;

    let field = db::update_field(&db, field_id, update_field).await?;

    Ok(Json(field))
}

/// Delete a field and all cells in its respective column in the table.
///
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
/// - [ApiError::Forbidden]: User does not have access to that table or field
/// - [ApiError::NotFound]: Table or field not found
///
async fn delete_field(
    AuthSession { user, .. }: AuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path((table_id, field_id)): Path<(Id, Id)>,
) -> ApiResult<()> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::check_table_relation(&db, user_id, table_id)
        .await?
        .to_api_result()?;
    db::check_field_relation(&db, table_id, field_id)
        .await?
        .to_api_result()?;

    db::delete_field(&db, field_id).await?;

    Ok(())
}

/// Get all fields in a table.
///
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
/// - [ApiError::Forbidden]: User does not have access to that table
/// - [ApiError::NotFound]: Table not found
///
async fn get_fields(
    AuthSession { user, .. }: AuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(table_id): Path<Id>,
) -> ApiResult<Json<Vec<Field>>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::check_table_relation(&db, user_id, table_id)
        .await?
        .to_api_result()?;

    let fields = db::get_fields(&db, table_id).await?;

    Ok(Json(fields))
}

/// Set the order of all fields in a table.
/// Ordering numbers must go from `0` to `n-1` where `n` is the total number of fields
///
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
/// - [ApiError::Forbidden]: User does not have access to that table
/// - [ApiError::NotFound]: Table not found
/// - [ApiError::UnprocessableEntity]:
///   - <field_id>: [FIELD_ID_NOT_FOUND]
///   - <field_id>: [INVALID_ORDERING]
///
async fn set_field_order(
    AuthSession { user, .. }: AuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(table_id): Path<Id>,
    Json(SetFieldOrder(order)): Json<SetFieldOrder>,
) -> ApiResult<()> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::check_table_relation(&db, user_id, table_id)
        .await?
        .to_api_result()?;

    let mut field_ids: HashSet<_> = db::get_field_ids(&db, table_id)
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

    db::set_field_order(&db, order).await?;

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

/// Validates the range definition of a field.
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
        Err(ApiError::UnprocessableEntity(INVALID_RANGE.into()))
    }
}
