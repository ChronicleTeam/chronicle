//! This module is responsible for building the application [Router]
//! and defining the logic of all route handlers to create a REST API.
//!
//! Routes represent ressources on which CRUD opertions are performed.
//!
//! HTTP methods map to different operations like this:
//! - POST: Create
//! - PUT: Create or replace
//! - GET: Read
//! - PATCH: Update
//! - DELETE: Delete
//!
//! See [crate::error::ApiError] for the errors that can be returned from the API.
//!
//! Handlers have only the following responsability
//! - Validating the input request.
//! - Calling database functions from [crate::db].
//! - Returning the output response.
//!
//! Handlers should not be concerned with creating SQL queries
//! and should validate every possible input. Fortunately, Axum
//! and Rust allow for strict types which reduce the amount of validation
//! necessary.

mod users;
mod data;
mod viz;

// #[cfg(test)]
// mod tests;

use crate::{
    config::Config, db::Backend, model::users::{Credentials, UserRole},
};
use anyhow::Result;
use axum::{
    http::{
        header::{self, SET_COOKIE},
        HeaderValue, Method,
    },
    response::Response,
    Router,
};
use axum_login::{tower_sessions::ExpiredDeletion, AuthManagerLayerBuilder};
use shuttle_runtime::SecretStore;
use sqlx::PgPool;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::CatchPanicLayer, compression::CompressionLayer, cors::CorsLayer,
    timeout::TimeoutLayer, trace::TraceLayer,
};
use tower_sessions::{
    cookie::{Key, SameSite},
    Expiry, SessionManagerLayer,
};
use tower_sessions_sqlx_store::PostgresStore;

/// Global state for the API.
///
/// Contains the configuration ([Config]) and the
/// shared database connection ([PgPool]).
#[derive(Clone)]
pub struct ApiState {
    pub config: Arc<Config>,
    pub pool: PgPool,
}

/// Create the application [Router].
/// It creates the routes under the `/api` path and configures
/// middleware layers for the back-end. The [ApiState] is then
/// attached to the router.
/// 
/// The secrets provided must contain the following keys:
/// ```toml
/// ALLOWED_ORIGIN=<url>
/// ```
/// 
/// An amount of admin accounts can be defined by repeating this pair of variables:
/// ```toml
/// <identifier>_USERNAME=<username>
/// <identifier>_PASSWORD=<password>
/// ```
/// 
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

    let allowed_origin = secrets
        .get("ALLOWED_ORIGIN")
        .expect("ALLOWED_ORIGIN secret must be set");

    tokio::spawn(async move { create_admin_users(backend, secrets).await.unwrap() });

    Ok(Router::new()
        .nest(
            "/api",
            Router::new()
                .merge(users::router())
                .merge(data::router())
                .merge(viz::router()),
        )
        .layer(auth_layer)
        .layer(ServiceBuilder::new().map_response(set_partitioned_cookie))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http().on_failure(()))
        .layer(TimeoutLayer::new(Duration::from_secs(300)))
        .layer(CatchPanicLayer::new())
        .layer(
            CorsLayer::new()
                .allow_origin([allowed_origin.parse().unwrap()])
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
                    header::CONTENT_TYPE,
                    header::AUTHORIZATION,
                ])
                .allow_credentials(true),
        )
        .with_state(api_state))
}

/// Create the admin users from the application secrets.
async fn create_admin_users(mut backend: Backend, secrets: SecretStore) -> sqlx::Result<()> {
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
            let user_id = backend.create_user(creds).await?.user_id;
            backend.set_role(user_id, UserRole::Admin).await?;
        }
    }

    Ok(())
}

/// Sets the "Partiioned" attribute of the "set-cookie" header.
fn set_partitioned_cookie(mut res: Response) -> Response {
    if let Some(set_cookie) = res.headers().get(SET_COOKIE) {
        if let Ok(cookie_value) = set_cookie.to_str() {
            if !cookie_value.contains("Partitioned") {
                let cookie_value = format!("{}; Partitioned", cookie_value);
                let headers = res.headers_mut();
                headers.insert(SET_COOKIE, HeaderValue::from_str(&cookie_value).unwrap());
            }
        }
    }
    res
}


