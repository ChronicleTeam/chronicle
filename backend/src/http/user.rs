use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;

use super::AppState;

pub(crate) fn router() -> Router<AppState> {
    // By having each module responsible for setting up its own routing,
    // it makes the root module a lot cleaner.
    Router::new()
        .route("/", get(root_handler))
        .route("/api/hello", post(api_handler))
        .route("/test-db", get(test_db))
}

async fn root_handler() -> &'static str {
    "Axum backend is live!"
}

async fn test_db(
    State(AppState { pool, .. }): State<AppState>,
) -> Result<String, (StatusCode, String)> {
    sqlx::query_scalar("SELECT 'hello world from pg'")
        .fetch_one(&pool)
        .await
        .map_err(internal_error)
}

#[derive(Serialize)]
struct ApiResponse {
    message: String,
}

async fn api_handler() -> Json<ApiResponse> {
    Json(ApiResponse {
        message: "Hello from Axum API!".to_string(),
    })
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
