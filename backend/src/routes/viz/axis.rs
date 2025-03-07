use std::collections::HashMap;
use sqlx::PgPool;
use crate::{db, error::{ApiError, ApiResult, ErrorMessage}, model::{data::FieldKind, viz::{Aggregate, AxisKind, CreateAxis}}, Id};

const FIELD_NOT_FOUND: ErrorMessage =
    ErrorMessage::new_static("field_id", "Field not found");


pub async fn validate_axes(pool: &PgPool, table_id: Id, axes: &[CreateAxis]) -> ApiResult<()>  {
    let axis_fields: HashMap<_, _> = db::get_fields(pool, table_id)
        .await?
        .into_iter()
        .map(|field| (field.field_id, field.field_kind.0))
        .collect();

    for CreateAxis {
        field_id,
        axis_kind,
        aggregate,
    } in axes
    {
        validate_axis(
            &axis_kind,
            &aggregate,
            axis_fields.get(&field_id).ok_or(ApiError::unprocessable_entity([FIELD_NOT_FOUND]))?,
        )?;
    }

    Ok(())
}


fn validate_axis(axis_kind: &AxisKind, aggregate: &Option<Aggregate>, field_kind: &FieldKind) -> ApiResult<()> {
    

    todo!()
}