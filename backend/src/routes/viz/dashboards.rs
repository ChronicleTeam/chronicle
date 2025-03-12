use crate::{
    db,
    error::{ApiError, ApiResult, ErrorMessage, OnConstraint},
    model::viz::{CreateDashboard, Dashboard, UpdateDashboard},
    routes::ApiState,
    Id,
};
use axum::{
    extract::{Path, State},
    routing::{patch, post},
    Json, Router,
};

const DASHBOARD_NAME_CONFLICT: ErrorMessage =
    ErrorMessage::new_static("name", "Dashboard name already used");

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

async fn create_dashboard(
    State(ApiState { pool, .. }): State<ApiState>,
    Json(create_dashboard): Json<CreateDashboard>,
) -> ApiResult<Json<Dashboard>> {
    let user_id = db::debug_get_user_id(&pool).await?;

    let dashboard = db::create_dashboard(&pool, user_id, create_dashboard)
        .await
        .on_constraint("dashboard_user_id_name_key", |_| {
            ApiError::unprocessable_entity([DASHBOARD_NAME_CONFLICT])
        })?;

    Ok(Json(dashboard))
}

async fn update_dashboard(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(dashboard_id): Path<Id>,
    Json(update_dashboard): Json<UpdateDashboard>,
) -> ApiResult<Json<Dashboard>> {
    let user_id = db::debug_get_user_id(&pool).await?;

    db::check_dashboard_relation(&pool, user_id, dashboard_id)
        .await?
        .to_api_result()?;

    let dashboard = db::update_dashboard(&pool, dashboard_id, update_dashboard)
        .await
        .on_constraint("dashboard_user_id_name_key", |_| {
            ApiError::unprocessable_entity([DASHBOARD_NAME_CONFLICT])
        })?;

    Ok(Json(dashboard))
}

async fn delete_dashboard(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(dashboard_id): Path<Id>,
) -> ApiResult<()> {
    let user_id = db::debug_get_user_id(&pool).await?;

    db::check_dashboard_relation(&pool, user_id, dashboard_id)
        .await?
        .to_api_result()?;

    db::delete_dashboard(&pool, dashboard_id).await?;

    Ok(())
}

async fn get_dashboards(
    State(ApiState { pool, .. }): State<ApiState>,
) -> ApiResult<Json<Vec<Dashboard>>> {
    let user_id = db::debug_get_user_id(&pool).await?;

    let dashboards = db::get_dashboards(&pool, user_id).await?;

    Ok(Json(dashboards))
}
