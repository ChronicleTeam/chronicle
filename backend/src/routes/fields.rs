use super::ApiState;
use crate::{
    error::ApiResult,
    model::{CreateField, FieldId},
    query, Id,
};
use axum::{
    extract::{Path, State},
    routing::post,
    Json, Router,
};
use axum_macros::debug_handler;

pub(crate) fn router() -> Router<ApiState> {
    Router::new().nest(
        "/tables/{table_id}/fields",
        Router::new().route("/", post(create_field)),
    )
}

async fn create_field(
    Path(table_id): Path<Id>,
    State(ApiState { pool, .. }): State<ApiState>,
    Json(create_field): Json<CreateField>,
) -> ApiResult<Json<FieldId>> {
    create_field.options.validate()?;

    let field_id =
        query::insert_field(&pool, table_id, create_field.name, create_field.options).await?;

    Ok(Json(FieldId { field_id }))
}
