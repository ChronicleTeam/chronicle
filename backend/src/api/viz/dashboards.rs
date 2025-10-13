use crate::{
    auth::AppAuthSession, db, error::{ApiError, ApiResult}, model::{users::{AccessRole, AccessRoleCheck}, viz::{CreateDashboard, Dashboard, UpdateDashboard}}, AppState, Id
};
use aide::{NoApi, axum::ApiRouter};
use axum::{
    Json,
    extract::{Path, State},
    routing::{patch, post},
};
use axum_login::AuthSession;

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new().nest(
        "/dashboards",
        ApiRouter::new()
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
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Json(create_dashboard): Json<CreateDashboard>,
) -> ApiResult<Json<Dashboard>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;

    let dashboard = db::create_dashboard(&db, create_dashboard).await?;
    db::create_table_access(tx.as_mut(), [(user_id, AccessRole::Owner)], dashboard.dashboard_id).await?;

    tx.commit().await?;
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
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(dashboard_id): Path<Id>,
    Json(update_dashboard): Json<UpdateDashboard>,
) -> ApiResult<Json<Dashboard>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;

    db::get_dashboard_access(tx.as_mut(), user_id, dashboard_id)
        .await?
        .check(AccessRole::Editor)?;

    let dashboard = db::update_dashboard(&db, dashboard_id, update_dashboard).await?;

    tx.commit().await?;
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
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(dashboard_id): Path<Id>,
) -> ApiResult<()> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;

    db::get_dashboard_access(tx.as_mut(), user_id, dashboard_id)
        .await?
        .check(AccessRole::Owner)?;

    tx.commit().await?;
    db::delete_dashboard(&db, dashboard_id).await?;

    Ok(())
}

/// Get all dashboards of the user.
///
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
///
async fn get_dashboards(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
) -> ApiResult<Json<Vec<Dashboard>>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    let dashboards = db::get_dashboards(&db, user_id).await?;

    Ok(Json(dashboards))
}
