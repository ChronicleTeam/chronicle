mod user;

use std::time::Duration;
use crate::config::Config;
use axum::Router;
use sqlx::PgPool;
use tower_http::{catch_panic::CatchPanicLayer, compression::CompressionLayer, sensitive_headers::SetSensitiveHeadersLayer, timeout::TimeoutLayer, trace::TraceLayer};

pub async fn serve(config: Config, db: PgPool) -> anyhow::Result<()> {
    let t = Router::new()
        .merge(user::router())
        // Enables logging. Use `RUST_LOG=tower_http=debug`
        .layer((
            // SetSensitiveHeadersLayer::new([AUTHORIZATION]),
            // CompressionLayer::new(),
            TraceLayer::new_for_http().on_failure(()),
            TimeoutLayer::new(Duration::from_secs(30)),
            CatchPanicLayer::new(),
        ))
        .with_state(db);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("Backend running on http://{}", addr);

    axum::serve(listener, app).await?
}
