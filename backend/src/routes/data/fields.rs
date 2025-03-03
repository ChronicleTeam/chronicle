use super::ApiState;
use crate::{
    db,
    error::{ApiError, ApiResult, ErrorMessage, OnConstraint},
    model::data::{CreateField, Field, FieldKind, UpdateField},
    Id,
};
use anyhow::anyhow;
use axum::{
    extract::{Path, State},
    routing::{post, put},
    Json, Router,
};

const INVALID_RANGE: ErrorMessage =
    ErrorMessage::new_static("range", "Range start bound is greater than end bound");
const FIELD_NAME_CONFLICT: ErrorMessage =
    ErrorMessage::new_static("name", "Field name already used for this table");

pub(crate) fn router() -> Router<ApiState> {
    Router::new().nest(
        "/tables/{table_id}/fields",
        Router::new()
            .route("/", post(create_field).get(get_fields))
            .route("/{field_id}", put(update_field).delete(delete_field)),
    )
}

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

    let field = db::create_field(&pool, table_id, create_field.name, create_field.field_kind)
        .await
        .on_constraint("meta_field_table_id_name_key", |_| {
            ApiError::unprocessable_entity([FIELD_NAME_CONFLICT])
        })?;

    Ok(Json(field))
}

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

    let field = db::update_field(&pool, field_id, update_field.name, update_field.field_kind).await?;

    Ok(Json(field))
}

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

fn validate_field_kind(field_kind: &mut FieldKind) -> ApiResult<()> {
    match field_kind {
        FieldKind::Integer {
            range_start,
            range_end,
            ..
        } => validate_range(*range_start, *range_end)?,
        FieldKind::Decimal {
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
        FieldKind::Interval { .. } => todo!(),
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
        Err(ApiError::unprocessable_entity([INVALID_RANGE]))
    }
}
