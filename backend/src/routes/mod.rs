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

mod auth;
mod data;
mod viz;

#[cfg(test)]
mod tests;

use crate::{
    config::Config,
    users::{Backend, Credentials},
};
use anyhow::Result;
use axum::{
    http::{header, Method},
    Router,
};
use axum_login::{tower_sessions::ExpiredDeletion, AuthManagerLayerBuilder};
use shuttle_runtime::SecretStore;
use sqlx::PgPool;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tower_http::{
    catch_panic::CatchPanicLayer, compression::CompressionLayer, cors::CorsLayer,
    timeout::TimeoutLayer, trace::TraceLayer,
};
use tower_sessions::{cookie::{Key, SameSite}, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;

/// Global state for the API.
///
/// Contains the configuration ([`Config`]) and the
/// shared database connection ([`PgPool`]).
#[derive(Clone)]
pub struct ApiState {
    pub config: Arc<Config>,
    pub pool: PgPool,
}

/// Create the application [`Router`].
/// It puts all routes under the `/api` path, it sets important
/// middleware layers for the back-end, and it attaches the [`ApiState`]
pub async fn create_app(
    api_state: ApiState,
    secrets: SecretStore,
) -> Result<Router, Box<dyn std::error::Error>> {
    let session_store = PostgresStore::new(api_state.pool.clone());
    session_store.migrate().await?;

    let _deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );

    // Generate a cryptographic key to sign the session cookie.
    let key = Key::generate();

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(true)
        .with_same_site(SameSite::None)
        .with_expiry(Expiry::OnInactivity(time::Duration::days(1)))
        .with_signed(key);

    // Auth service.
    //
    // This combines the session layer with our backend to establish the auth
    // service which will provide the auth session as a request extension.
    let backend = Backend::new(api_state.pool.clone());
    let auth_layer = AuthManagerLayerBuilder::new(backend.clone(), session_layer).build();

    let allowed_origin = secrets.get("ALLOWED_ORIGIN").expect("ALLOWED_ORIGIN secret must be set");

    tokio::spawn(async move { register_default_users(backend, secrets).await.unwrap() });

    Ok(Router::new()
        .nest(
            "/api",
            Router::new()
                .merge(auth::router())
                .merge(data::router())
                .merge(viz::router()),
        )
        .layer(auth_layer)
        .layer((
            CompressionLayer::new(),
            TraceLayer::new_for_http().on_failure(()),
            TimeoutLayer::new(Duration::from_secs(300)),
            CatchPanicLayer::new(),
            CorsLayer::new()
                .allow_origin([allowed_origin.parse().unwrap()]) // Adjust to your frontend origin
                .allow_methods([
                    Method::POST,
                    Method::GET,
                    Method::PATCH,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                    Method::HEAD,
                ])
                .allow_headers([
                    header::CONTENT_TYPE,  // Needed for JSON, forms, and multipart
                    header::AUTHORIZATION, // Needed for Bearer tokens
                ])
                .allow_credentials(true),
        ))
        .with_state(api_state))
}

async fn register_default_users(mut backend: Backend, secrets: SecretStore) -> sqlx::Result<()> {
    let mut usernames: HashMap<String, String> = HashMap::new();
    let mut passwords: HashMap<String, String> = HashMap::new();
    for (key, value) in secrets {
        if key.ends_with("USERNAME") {
            let key = key.replace("USERNAME", "");
            usernames.insert(key, value);
        } else if key.ends_with("PASSWORD") {
            let key = key.replace("PASSWORD", "");
            passwords.insert(key, value);
        }
    }

    for creds in usernames.into_iter().filter_map(|(key, username)| {
        Some(Credentials {
            username,
            password: passwords.remove(&key)?,
        })
    }) {
        if !backend.exists(&creds).await? {
            _ = backend.create_user(creds).await?;
        }
    }

    Ok(())
}
