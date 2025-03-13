use crate::{
    db,
    error::ApiResult,
    model::viz::{Chart, ChartData, CreateChart, UpdateChart},
    routes::ApiState,
    Id,
};
use axum::{
    extract::{Path, State},
    routing::{get, patch, post},
    Json, Router,
};

pub fn router() -> Router<ApiState> {
    Router::new().nest(
        "/dashboards/{dashboard-id}/charts",
        Router::new()
            .route("/", post(create_chart))
            .route("/{chart-id}", patch(update_chart).delete(delete_chart))
            .route("/{chart-id}/data", get(get_chart_data))
    )
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

    db::check_table_relation(&pool, user_id, create_chart.table_id)
        .await?
        .to_api_result()?;

    let chart = db::create_chart(&pool, dashboard_id, create_chart).await?;

    Ok(Json(chart))
}

async fn update_chart(
    State(ApiState { pool, .. }): State<ApiState>,
    Path((dashboard_id, chart_id)): Path<(Id, Id)>,
    Json(update_chart): Json<UpdateChart>,
) -> ApiResult<Json<Chart>> {
    let user_id = db::debug_get_user_id(&pool).await?;

    db::check_dashboard_relation(&pool, user_id, dashboard_id)
        .await?
        .to_api_result()?;

    db::check_chart_relation(&pool, dashboard_id, chart_id)
        .await?
        .to_api_result()?;

    let chart = db::update_chart(&pool, chart_id, update_chart).await?;

    Ok(Json(chart))
}

async fn delete_chart(
    State(ApiState { pool, .. }): State<ApiState>,
    Path((dashboard_id, chart_id)): Path<(Id, Id)>,
) -> ApiResult<()> {
    let user_id = db::debug_get_user_id(&pool).await?;

    db::check_dashboard_relation(&pool, user_id, dashboard_id)
        .await?
        .to_api_result()?;

    db::check_chart_relation(&pool, dashboard_id, chart_id)
        .await?
        .to_api_result()?;

    db::delete_chart(&pool, chart_id).await?;

    Ok(())
}

async fn get_chart_data(
    State(ApiState { pool, .. }): State<ApiState>,
    Path((dashboard_id, chart_id)): Path<(Id, Id)>,
) -> ApiResult<Json<ChartData>> {
    let user_id = db::debug_get_user_id(&pool).await?;

    db::check_dashboard_relation(&pool, user_id, dashboard_id)
        .await?
        .to_api_result()?;


    db::check_chart_relation(&pool, dashboard_id, chart_id)
        .await?
        .to_api_result()?;

    let chart_data = db::get_chart_data(&pool, chart_id).await?;

    Ok(Json(chart_data))
}
