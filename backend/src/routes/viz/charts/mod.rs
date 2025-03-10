mod axes;

use crate::{
    db,
    error::{ApiResult, ErrorMessage},
    model::viz::{Chart, CreateChart},
    routes::ApiState,
    Id,
};
use axum::{
    extract::{Path, State},
    routing::post,
    Json, Router,
};
use axum_macros::debug_handler;

const TABLE_NOT_FOUND: ErrorMessage = ErrorMessage::new_static("table_id", "Table not found");

pub fn router() -> Router<ApiState> {
    Router::new()
        .merge(axes::router())
        .route("/dashboards/{dashboard-id}/charts", post(create_chart))
}

async fn create_chart(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(dashboard_id): Path<Id>,
    Json(create_chart): Json<CreateChart>,
) -> ApiResult<Json<Chart>> {
    let user_id = db::debug_get_user_id(&pool).await?;

    db::check_dashboard_relation(&pool, user_id, dashboard_id)
        .await?
        .to_api_result()?;

    let chart = db::create_chart(&pool, dashboard_id, create_chart).await?;

    Ok(Json(chart))
}
