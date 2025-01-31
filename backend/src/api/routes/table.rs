use axum::{extract::State, routing::post, Json, Router};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use sqlx::any;

use crate::api::{error::ApiResult, Id};

use super::ApiState;

pub(crate) fn router() -> Router<ApiState> {
    Router::new().nest("/table", Router::new().route("/create", post(create_table)))
}

#[derive(Deserialize, Serialize)]
struct TableBody {
    table_id: Id,
}

#[derive(Deserialize)]
struct CreateTable {
    name: String,
    description: String,
}

#[debug_handler]
async fn create_table(
    State(ApiState { pool, .. }): State<ApiState>,
    Json(mut request): Json<CreateTable>,
) -> ApiResult<Json<TableBody>> {
    let user_id = 1; // TESTING

    Ok(Json(TableBody { table_id: 0 }))
}
