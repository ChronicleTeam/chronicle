use axum::{extract::{Path, State}, routing::post, Json, Router};
use axum_macros::debug_handler;
use crate::{error::ApiResult, model::viz::{CreateChart}, routes::ApiState};

pub(crate) fn router() -> Router<ApiState> {
    Router::new().nest(
        "/plots",
        Router::new()
            .route("/", post(create_chart)),
    )
}



async fn create_chart(
    State(ApiState { pool, .. }): State<ApiState>,
    Json(create_plot): Json<CreateChart>,
) -> ApiResult<Json<()>> {
    todo!()
}