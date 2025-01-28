use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use chronicle::{config::Config, http};
use clap::Parser;
use serde::Serialize;
use sqlx::{
    migrate::Migrator,
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};
use std::{net::SocketAddr, time::Duration};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static MIGRATOR: Migrator = sqlx::migrate!(); // Points to the migrations folder

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let config = Config::parse();

    let db = PgPoolOptions::new()
        .max_connections(50)
        .acquire_timeout(Duration::from_secs(3))
        .connect_with(
            PgConnectOptions::new()
                .host(&config.database_host)
                .port(config.database_port)
                .username(&config.database_user)
                .password(&config.database_password),
        )
        .await?;

    MIGRATOR.run(&db).await?;

    

    http::serve(config, db)
}

async fn root_handler() -> &'static str {
    "Axum backend is live!"
}

async fn test_db(State(pool): State<PgPool>) -> Result<String, (StatusCode, String)> {
    sqlx::query_scalar("select 'hello world from pg'")
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
