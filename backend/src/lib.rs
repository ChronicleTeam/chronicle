mod api;
mod auth;
mod db;
mod error;
mod io;
mod model;

#[cfg_attr(coverage_nightly, coverage(off))]
mod docs;

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod test_util;

use crate::model::users::Credentials;
use axum::{
    Router,
    http::{HeaderValue, Method, header},
};
use base64::{Engine, prelude::BASE64_STANDARD};
use config::{Config, ConfigError, Environment};
use itertools::Itertools;
use serde::Deserialize;
use sqlx::{
    PgPool,
    migrate::Migrator,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::CatchPanicLayer, compression::CompressionLayer, cors::CorsLayer,
    timeout::TimeoutLayer, trace::TraceLayer,
};
use tower_sessions::cookie::Key;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static MIGRATOR: Migrator = sqlx::migrate!();

type Id = i32;

/// Global state for the API.
///
/// Contains the shared database connection ([PgPool]).
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

/// Application configuration
#[derive(Clone, Deserialize)]
struct AppConfig {
    /// Server port.
    port: u16,
    #[serde(deserialize_with = "env_list")]
    allowed_origin: Vec<String>,
    #[serde(deserialize_with = "base64_session_key")]
    session_key: Key,
    admin: Credentials,
    /// Database connection info.
    database: DatabaseConfig,
}

impl AppConfig {
    fn build() -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?
            .try_deserialize()
    }
}

#[derive(Clone, Deserialize)]
struct DatabaseConfig {
    host: String,
    name: String,
    username: String,
    password: String,
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
pub async fn serve() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    setup_tracing();

    let config = AppConfig::build()?;

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.port);
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("API docs url http://localhost:{}/docs", config.port);
    tracing::info!("listening on {}", listener.local_addr()?);

    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect_with(
            PgConnectOptions::new()
                .host(&config.database.host)
                .database(&config.database.name)
                .username(&config.database.username)
                .password(&config.database.password),
        )
        .await?;
    MIGRATOR.run(&db).await?;

    auth::set_admin_user(&db, config.admin).await?;

    let router = api::router();
    let router = docs::init(router)?;
    let router = auth::init(router, db.clone(), config.session_key).await?;
    let router = init_layers(router, config.allowed_origin)?;
    let router = router.with_state(AppState { db });

    axum::serve(listener, router.into_make_service()).await?;
    Ok(())
}

fn init_layers(
    router: Router<AppState>,
    allowed_origin: Vec<String>,
) -> anyhow::Result<Router<AppState>> {
    let allowed_origin: Vec<_> = allowed_origin
        .iter()
        .map(|v| HeaderValue::from_str(v))
        .try_collect()?;
    let service = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http().on_failure(()))
        .layer(
            CorsLayer::new()
                .allow_origin(allowed_origin)
                .allow_methods([
                    Method::POST,
                    Method::GET,
                    Method::PATCH,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                    Method::HEAD,
                ])
                .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
                .allow_credentials(true),
        )
        .layer(CompressionLayer::new())
        .layer(CatchPanicLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(300)));
    Ok(router.layer(service))
}

/// Sets up tracing for debuging and monitoring.
/// Does nothing if called more than once.
fn setup_tracing() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
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
    });
}

fn env_list<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

fn base64_session_key<'de, D>(deserializer: D) -> Result<Key, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Key::from(
        &BASE64_STANDARD
            .decode(s)
            .map_err(serde::de::Error::custom)?,
    ))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use std::time::Duration;

    use crate::AppConfig;
    use reqwest::StatusCode;

    #[test]
    fn build_app_config() -> anyhow::Result<()> {
        dotenvy::from_filename_override("example.env")?;
        let _config = AppConfig::build()?;
        Ok(())
    }

    #[sqlx::test]
    async fn serve() -> anyhow::Result<()> {
        dotenvy::from_filename_override("example.env")?;
        tokio::task::spawn(async move { crate::serve().await.unwrap() });
        tokio::time::sleep(Duration::from_secs(2)).await;
        let response = reqwest::get("http://localhost:5000/api").await?;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        Ok(())
    }
}
