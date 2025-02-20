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

pub(crate) fn router() -> Router<ApiState> {
    Router::new()
        .merge(tables::router())
        .merge(fields::router())
        .merge(entries::router())
        .route("/tables/{table_id}/data", get(get_data_table))
}

async fn get_data_table(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
) -> ApiResult<Json<DataTable>> {
    
    let user_id = db::debug_get_user_id(&pool).await?;

    match db::check_table_relation(&pool, user_id, table_id).await? {
        db::Relation::Owned => {}
        db::Relation::NotOwned => return Err(ApiError::Forbidden),
        db::Relation::Absent => return Err(ApiError::NotFound),
    }

    let data_table = db::get_data_table(&pool, table_id).await?;

    Ok(Json(data_table))
}
