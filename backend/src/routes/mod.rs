mod data;
mod users;
mod viz;

#[cfg(test)]
mod tests;

use crate::config::Config;
use anyhow::Result;
use axum::Router;
use sqlx::PgPool;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tower_http::{
    catch_panic::CatchPanicLayer, compression::CompressionLayer, cors::CorsLayer,
    timeout::TimeoutLayer, trace::TraceLayer,
};
use tracing::info;

#[derive(Clone)]
struct ApiState {
    config: Arc<Config>,
    pool: PgPool,
}

fn create_app(api_state: ApiState) -> Router {
    Router::new()
        .nest(
            "/api",
            Router::new().merge(data::router()).merge(viz::router()),
        )
        // Enables logging. Use `RUST_LOG=tower_http=debug`
        .layer((
            // SetSensitiveHeadersLayer::new([AUTHORIZATION]),
            CompressionLayer::new(),
            TraceLayer::new_for_http().on_failure(()),
            TimeoutLayer::new(Duration::from_secs(30)),
            CatchPanicLayer::new(),
            CorsLayer::permissive(),
        ))
        .with_state(api_state)
}

pub async fn serve(config: Config, pool: PgPool) -> Result<SocketAddr> {
    let api_state = ApiState {
        config: Arc::new(config),
        pool,
    };

    let app = create_app(api_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    info!("Backend running on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(addr)
}
