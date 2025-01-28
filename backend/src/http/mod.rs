mod user;

use crate::config::Config;
use axum::Router;
use sqlx::PgPool;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tower_http::{
    catch_panic::CatchPanicLayer, compression::CompressionLayer,
    sensitive_headers::SetSensitiveHeadersLayer, timeout::TimeoutLayer, trace::TraceLayer,
};
use tracing::info;

#[derive(Clone)]
pub(crate) struct AppState {
    config: Arc<Config>,
    db: PgPool,
}

pub async fn serve(config: Config, db: PgPool) -> Result<(), std::io::Error> {
    let app_state = AppState {
        config: Arc::new(config),
        db,
    };

    let app = Router::new()
        .merge(user::router())
        // Enables logging. Use `RUST_LOG=tower_http=debug`
        .layer((
            // SetSensitiveHeadersLayer::new([AUTHORIZATION]),
            CompressionLayer::new(),
            TraceLayer::new_for_http().on_failure(()),
            TimeoutLayer::new(Duration::from_secs(30)),
            CatchPanicLayer::new(),
        ))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    info!("Backend running on http://{}", addr);

    axum::serve(listener, app).await
}
