use axum::{
    routing::get,
    Router,
    Json,
};
use serde::Serialize;
use tokio::net::TcpListener;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()>  {
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/api/hello", get(api_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("Backend running on http://{}", addr);

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn root_handler() -> &'static str {
    "Axum backend is live!"
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
