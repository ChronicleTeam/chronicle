//! This module is responsible for building the application [`Router`]
//! and defining the logic of all route handlers to create a REST API.
//!
//! Routes represent ressources on which CRUD opertions are performed.
//!
//! HTTP methods map to different operations like this:
//! - POST: Create
//! - GET: Read
//! - PUT: Update
//! - DELETE: Delete
//!
//! See [`crate::error::ApiError`] for the errors that can be returned from the API.
//!
//! Handlers have only the following responsability
//! - Validating the input request.
//! - Calling database functions from [`crate::db`].
//! - Returning the output response.
//!
//! Handlers should not be concerned with creating SQL queries
//! and should validate every possible input. Fortunately, Axum
//! and Rust allow for strict types which reduce the amount of validation
//! necessary.

mod data;
mod users;
mod viz;

#[cfg(test)]
mod tests;

use crate::config::Config;
use axum::Router;
use sqlx::PgPool;
use std::{sync::Arc, time::Duration};
use tower_http::{
    catch_panic::CatchPanicLayer, compression::CompressionLayer, cors::CorsLayer,
    timeout::TimeoutLayer, trace::TraceLayer,
};

/// Global state for the API.
///
/// Contains the configuration ([`Config`]) and the
/// shared database connection ([`PgPool`]).
#[derive(Clone)]
struct ApiState {
    _config: Arc<Config>,
    pool: PgPool,
}

/// Create the application [`Router`].
/// It puts all routes under the `/api` path, it sets important
/// middleware layers for the back-end, and it attaches the [`ApiState`]
pub fn create_app(config: Config, pool: PgPool) -> Router {
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
        .with_state(ApiState {
            _config: Arc::new(config),
            pool,
        })
}
