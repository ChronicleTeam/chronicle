mod entries;
mod fields;
mod tables;

use crate::{db, error::ApiResult, model::data::TableData, routes::ApiState, Id};
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};

pub(crate) fn router() -> Router<ApiState> {
    Router::new()
        .merge(tables::router())
        .merge(fields::router())
        .merge(entries::router())
        .route("/tables/{table_id}/data", get(get_table_data))
}

async fn get_table_data(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
) -> ApiResult<Json<TableData>> {
    let user_id = db::debug_get_user_id(&pool).await?;
    db::check_table_relation(&pool, user_id, table_id)
        .await?
        .to_api_result()?;

    let data_table = db::get_table_data(&pool, table_id).await?;

    Ok(Json(data_table))
}
