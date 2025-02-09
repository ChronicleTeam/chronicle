mod entries;
mod fields;
mod tables;
mod users;

use crate::{config::Config, db, error::{ApiError, ApiResult}, Id};
use axum::Router;
use sqlx::{PgExecutor, PgPool};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tower_http::{
    catch_panic::CatchPanicLayer, compression::CompressionLayer,
    sensitive_headers::SetSensitiveHeadersLayer, timeout::TimeoutLayer, trace::TraceLayer,
};
use tracing::info;

#[derive(Clone)]
pub(crate) struct ApiState {
    config: Arc<Config>,
    pool: PgPool,
}

pub async fn serve(config: Config, pool: PgPool) -> Result<(), std::io::Error> {
    let app_state = ApiState {
        config: Arc::new(config),
        pool,
    };

    let app = Router::new()
        .nest(
            "/api",
            Router::new()
                .merge(users::router())
                .merge(tables::router())
                .merge(fields::router())
                .merge(entries::router()),
        )
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

pub async fn validate_user_table(executor: impl PgExecutor<'_>, user_id: Id, table_id: Id) -> ApiResult<()> {
    let table_user_id = db::get_table_user_id(executor, table_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if table_user_id != user_id {
        return Err(ApiError::Forbidden);
    }

    Ok(())
}
