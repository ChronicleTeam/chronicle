use crate::{
    db::{self, AuthSession},
    error::{ApiError, ApiResult},
    model::viz::{CreateDashboard, Dashboard, UpdateDashboard},
    routes::ApiState,
    Id,
};
use axum::{
    extract::{Path, State},
    routing::{patch, post},
    Json, Router,
};

pub fn router() -> Router<ApiState> {
    Router::new().nest(
        "/dashboards",
        Router::new()
            .route("/", post(create_dashboard).get(get_dashboards))
            .route(
                "/{dashboard-id}",
                patch(update_dashboard).delete(delete_dashboard),
            ),
    )
}

/// Create a blank dashboard.
/// 
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
/// 
async fn create_dashboard(
    AuthSession { user, .. }: AuthSession,
    State(ApiState { pool, .. }): State<ApiState>,
    Json(create_dashboard): Json<CreateDashboard>,
) -> ApiResult<Json<Dashboard>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    let dashboard = db::create_dashboard(&pool, user_id, create_dashboard).await?;

    Ok(Json(dashboard))
}

/// Update a dashboard's metadata.
/// 
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
/// - [ApiError::Forbidden]: User does not have access to this dashboard
/// - [ApiError::NotFound]: Dashboard not found
/// 
async fn update_dashboard(
    AuthSession { user, .. }: AuthSession,
    State(ApiState { pool, .. }): State<ApiState>,
    Path(dashboard_id): Path<Id>,
    Json(update_dashboard): Json<UpdateDashboard>,
) -> ApiResult<Json<Dashboard>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::check_dashboard_relation(&pool, user_id, dashboard_id)
        .await?
        .to_api_result()?;

    let dashboard = db::update_dashboard(&pool, dashboard_id, update_dashboard).await?;

    Ok(Json(dashboard))
}

/// Delete a dashboard and all of it's charts and chart axes.
/// 
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
/// - [ApiError::Forbidden]: User does not have access to this dashboard
/// - [ApiError::NotFound]: Dashboard not found
/// 
async fn delete_dashboard(
    AuthSession { user, .. }: AuthSession,
    State(ApiState { pool, .. }): State<ApiState>,
    Path(dashboard_id): Path<Id>,
) -> ApiResult<()> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::check_dashboard_relation(&pool, user_id, dashboard_id)
        .await?
        .to_api_result()?;

    db::delete_dashboard(&pool, dashboard_id).await?;

    Ok(())
}

/// Get all dashboards of the user.
/// 
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
/// 
async fn get_dashboards(
    AuthSession { user, .. }: AuthSession,
    State(ApiState { pool, .. }): State<ApiState>,
) -> ApiResult<Json<Vec<Dashboard>>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    let dashboards = db::get_dashboards(&pool, user_id).await?;

    Ok(Json(dashboards))
}
