use super::{validate_user_table, ApiState};
use crate::{
    db,
    error::{ApiError, ApiResult, ErrorMessage, OnConstraint},
    model::data::{CreateField, Field, FieldId, FieldOptions},
    Id,
};
use anyhow::anyhow;
use axum::{
    extract::{Path, State},
    routing::{patch, post},
    Json, Router,
};

const INVALID_RANGE: ErrorMessage = ErrorMessage::new_static("range", "Range start bound is greater than end bound");
const FIELD_NAME_CONFLICT: ErrorMessage = ErrorMessage::new_static("name", "Field name already used for this table");

pub(crate) fn router() -> Router<ApiState> {
    Router::new().nest(
        "/tables/{table_id}/fields",
        Router::new()
            .route("/", post(create_field).get(get_fields))
            .route("/{field_id}", patch(update_field).delete(delete_field)),
    )
}

async fn create_field(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
    Json(mut create_field): Json<CreateField>,
) -> ApiResult<Json<FieldId>> {
    let mut tx = pool.begin().await?;

    let user_id = db::debug_get_user_id(tx.as_mut()).await?;
    validate_user_table(tx.as_mut(), user_id, table_id).await?;

    match &create_field.options {
        FieldOptions::Integer {
            range_start,
            range_end,
            ..
        } => validate_range(*range_start, *range_end),
        FieldOptions::Decimal {
            range_start,
            range_end,
            number_precision,
            number_scale,
            ..
        } => validate_range(*range_start, *range_end),
        FieldOptions::Money {
            range_start,
            range_end,
            ..
        } => validate_range(*range_start, *range_end),
        FieldOptions::DateTime {
            range_start,
            range_end,
            // date_time_format,
            ..
        } => validate_range(*range_start, *range_end),
        FieldOptions::Interval { .. } => Ok(()),
        FieldOptions::Enumeration {
            values,
            default_value,
            ..
        } => {
            if !values.contains_key(&default_value) {
                Err(anyhow!("enumeration field default value does not map to a value").into())
            } else {
                Ok(())
            }
        }
        // FieldOptions::CreationDate { date_time_format } => Ok(()),
        // FieldOptions::ModificationDate { date_time_format } => Ok(()),
        _ => Ok(()),
    }?;

    if let FieldOptions::Decimal {
        number_precision: Some(number_precision),
        ..
    } = &mut create_field.options
    {
        *number_precision = u32::max(*number_precision, 1)
    }

    let field_id = db::create_field(&pool, table_id, create_field.name, create_field.options)
        .await
        .on_constraint("meta_field_table_id_name_key", |_| {
            ApiError::unprocessable_entity([FIELD_NAME_CONFLICT])
        })?;

    Ok(Json(FieldId { field_id }))
}

async fn get_fields(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
) -> ApiResult<Json<Vec<Field>>> {
    let mut tx = pool.begin().await?;

    let user_id = db::debug_get_user_id(tx.as_mut()).await?;
    validate_user_table(tx.as_mut(), user_id, table_id).await?;

    let fields = db::get_fields(tx.as_mut(), table_id).await?;

    tx.commit().await?;
    Ok(Json(fields))
}


async fn update_field(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
) -> ApiResult<Json<Field>> {
    todo!()
}

async fn delete_field(
    State(ApiState { pool, .. }): State<ApiState>,
    Path((table_id, field_id)): Path<(Id, Id)>,
) -> ApiResult<()> {
    let mut tx = pool.begin().await?;

    let user_id = db::debug_get_user_id(tx.as_mut()).await?;
    validate_user_table(tx.as_mut(), user_id, table_id).await?;
    
    _ = db::get_field_table_id(tx.as_mut(), field_id)
        .await?
        .filter(|field_table_id| *field_table_id == table_id)
        .ok_or(ApiError::NotFound)?;

    db::delete_field(tx.as_mut(), field_id).await?;

    tx.commit().await?;
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
