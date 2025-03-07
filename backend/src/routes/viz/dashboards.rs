use axum::{
    extract::{Path, State},
    routing::post,
    Json, Router,
};
use axum_macros::debug_handler;

use crate::{db, error::ApiResult, model::viz::{CreateDashboard, Dashboard}, routes::ApiState, Id};

pub fn router() -> Router<ApiState> {
    Router::new().route("/dashboards", post(create_dashboard))
}


#[debug_handler]
async fn create_dashboard(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(dashboard_id): Path<Id>,
    Json(create_dashboard): Json<CreateDashboard>,
) -> ApiResult<Json<Dashboard>> {

    let user_id = db::debug_get_user_id(&pool).await?;

    db::check_dashboard_relation(&pool, user_id, dashboard_id)
        .await?
        .to_api_result()?;
    
    db::create_dashboard(&pool, user_id, create_dashboard).await?;

    todo!()
}
