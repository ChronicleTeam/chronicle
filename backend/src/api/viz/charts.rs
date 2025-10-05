use crate::{
    AppState, Id,
    auth::AppAuthSession,
    db,
    error::{ApiError, ApiResult},
    model::viz::{Chart, ChartData, CreateChart, UpdateChart},
};
use aide::{NoApi, axum::ApiRouter};
use axum::{
    Json,
    extract::{Path, State},
    routing::{get, patch, post},
};
use axum_login::AuthSession;

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new().nest(
        "/dashboards/{dashboard-id}/charts",
        ApiRouter::new()
            .route("/", post(create_chart).get(get_charts))
            .route("/{chart-id}", patch(update_chart).delete(delete_chart))
            .route("/{chart-id}/data", get(get_chart_data)),
    )
}

/// Create a blank chart.
///
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
/// - [ApiError::Forbidden]: User does not have access to this dashboard or table
/// - [ApiError::NotFound]: Dashboard or table not found
///
async fn create_chart(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(dashboard_id): Path<Id>,
    Json(create_chart): Json<CreateChart>,
) -> ApiResult<Json<Chart>> {
    let user_id = user.ok_or(ApiError::Forbidden)?.user_id;

    db::check_dashboard_relation(&db, user_id, dashboard_id)
        .await?
        .to_api_result()?;
    db::check_table_relation(&db, user_id, create_chart.table_id)
        .await?
        .to_api_result()?;

    let chart = db::create_chart(&db, dashboard_id, create_chart).await?;

    Ok(Json(chart))
}

/// Update a chart's metadata.
///
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
/// - [ApiError::Forbidden]: User does not have access to this dashboard or chart
/// - [ApiError::NotFound]: Dashboard or chart not found
///
async fn update_chart(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path((dashboard_id, chart_id)): Path<(Id, Id)>,
    Json(update_chart): Json<UpdateChart>,
) -> ApiResult<Json<Chart>> {
    let user_id = user.ok_or(ApiError::Forbidden)?.user_id;

    db::check_dashboard_relation(&db, user_id, dashboard_id)
        .await?
        .to_api_result()?;
    db::check_chart_relation(&db, dashboard_id, chart_id)
        .await?
        .to_api_result()?;

    let chart = db::update_chart(&db, chart_id, update_chart).await?;

    Ok(Json(chart))
}

/// Delete a chart and its axes.
///
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
/// - [ApiError::Forbidden]: User does not have access to this dashboard or chart
/// - [ApiError::NotFound]: Dashboard or chart not found
///
async fn delete_chart(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path((dashboard_id, chart_id)): Path<(Id, Id)>,
) -> ApiResult<()> {
    let user_id = user.ok_or(ApiError::Forbidden)?.user_id;

    db::check_dashboard_relation(&db, user_id, dashboard_id)
        .await?
        .to_api_result()?;
    db::check_chart_relation(&db, dashboard_id, chart_id)
        .await?
        .to_api_result()?;

    db::delete_chart(&db, chart_id).await?;

    Ok(())
}

/// Get all charts for this dashboard.
///
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
/// - [ApiError::Forbidden]: User does not have access to this dashboard
/// - [ApiError::NotFound]: Dashboard not found
///
async fn get_charts(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(dashboard_id): Path<Id>,
) -> ApiResult<Json<Vec<Chart>>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::check_dashboard_relation(&db, user_id, dashboard_id)
        .await?
        .to_api_result()?;

    let charts = db::get_charts(&db, dashboard_id).await?;

    Ok(Json(charts))
}

/// Get the chart's metadata, axes metadata, and data points.
///
/// Used for building and displaying the chart.
///
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
/// - [ApiError::Forbidden]: User does not have access to this dashboard or chart
/// - [ApiError::NotFound]: Dashboard or chart not found
///
async fn get_chart_data(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path((dashboard_id, chart_id)): Path<(Id, Id)>,
) -> ApiResult<Json<ChartData>> {
    let user_id = user.ok_or(ApiError::Forbidden)?.user_id;

    db::check_dashboard_relation(&db, user_id, dashboard_id)
        .await?
        .to_api_result()?;
    db::check_chart_relation(&db, dashboard_id, chart_id)
        .await?
        .to_api_result()?;

    let chart_data = db::get_chart_data(&db, chart_id).await?;

    Ok(Json(chart_data))
}
