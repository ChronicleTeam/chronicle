use std::collections::HashMap;

use super::axis::validate_axes;
use crate::{
    db,
    error::{ApiError, ApiResult, ErrorMessage},
    model::viz::{ChartData, CreateAxis, CreateChart},
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
    Router::new().route("/dashboards/{dashboard_id}/charts", post(create_chart))
}

async fn create_chart(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(dashboard_id): Path<Id>,
    Json(create_chart): Json<CreateChart>,
) -> ApiResult<Json<ChartData>> {
    let user_id = db::debug_get_user_id(&pool).await?;

    db::check_dashboard_relation(&pool, user_id, create_chart.table_id)
        .await?
        .to_api_result()?;

    db::check_chart_relation(&pool, user_id, create_chart.table_id)
        .await?
        .to_api_result()?;

    match db::check_table_relation(&pool, user_id, create_chart.table_id).await? {
        db::Relation::NotOwned | db::Relation::Absent => {
            Err(ApiError::unprocessable_entity([TABLE_NOT_FOUND]))?
        }
        _ => {}
    }

    validate_axes(&pool, create_chart.table_id, &create_chart.axes).await?;

    let chart_data = db::create_chart(&pool, dashboard_id, create_chart).await?;

    Ok(Json(chart_data))
}
