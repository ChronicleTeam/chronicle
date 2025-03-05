use axum::{extract::{Path, State}, routing::post, Json, Router};
use axum_macros::debug_handler;
use crate::{db, error::ApiResult, model::viz::{ChartData, CreateChart}, routes::ApiState, Id};

pub(crate) fn router() -> Router<ApiState> {
    Router::new().nest(
        "/dashboards/{dashboard_id}/charts",
        Router::new()
            .route("/", post(create_chart)),
    )
}



async fn create_chart(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(dashboard_id): Path<Id>,
    Json(create_chart): Json<CreateChart>,
) -> ApiResult<Json<ChartData>> {
    let user_id = db::debug_get_user_id(&pool).await?;
    db::check_table_relation(&pool, user_id, create_chart.table_id)
        .await?
        .to_api_result()?;



    todo!()
}