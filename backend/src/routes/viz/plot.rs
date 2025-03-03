use axum::{extract::{Path, State}, routing::post, Json, Router};
use crate::{error::ApiResult, model::viz::{CreatePlot, Plot}, routes::ApiState};

pub(crate) fn router() -> Router<ApiState> {
    Router::new().nest(
        "/plots",
        Router::new()
            .route("/", post(create_plot)),
    )
}



async fn create_plot(
    State(ApiState { pool, .. }): State<ApiState>,
    Json(create_plot): Json<CreatePlot>,
) -> ApiResult<Json<Plot>> {
    todo!()
}