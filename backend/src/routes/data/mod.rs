mod entries;
mod fields;
mod tables;

use crate::{
    db,
    error::{ApiError, ApiResult},
    model::data::DataTable,
    routes::ApiState,
    Id,
};
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use sqlx::PgExecutor;

use super::users;

pub(crate) fn router() -> Router<ApiState> {
    Router::new()
        .merge(tables::router())
        .merge(fields::router())
        .merge(users::router())
        .route("tables/{table_id}/data", get(get_data_table))
}

pub async fn validate_user_table(
    executor: impl PgExecutor<'_>,
    user_id: Id,
    table_id: Id,
) -> ApiResult<()> {
    let table_user_id = db::data::get_table_user_id(executor, table_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if table_user_id != user_id {
        return Err(ApiError::Forbidden);
    }

    Ok(())
}

async fn get_data_table(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
) -> ApiResult<Json<DataTable>> {
    let mut tx = pool.begin().await?;

    let user_id = db::debug_get_user_id(tx.as_mut()).await?;
    validate_user_table(tx.as_mut(), user_id, table_id).await?;

    todo!()
}
