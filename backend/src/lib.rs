mod db;
mod error;
mod io;
mod model;
mod api;

use std::{
    env, fs,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

use axum::{
    http::{HeaderValue, Method, header},
    response::Response,
};
use axum_login::AuthManagerLayerBuilder;
use serde::Deserialize;
use sqlx::{
    PgPool,
    migrate::Migrator,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::{CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tower_sessions::{
    ExpiredDeletion, Expiry, SessionManagerLayer,
    cookie::{Key, SameSite},
};
use tower_sessions_sqlx_store::PostgresStore;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    db::AuthBackend,
    model::users::{Credentials, UserRole},
};

static MIGRATOR: Migrator = sqlx::migrate!();

type Id = i32;

/// Global state for the API.
///
/// Contains the configuration ([Config]) and the
/// shared database connection ([PgPool]).
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

/// Application configuration
#[derive(Clone, Deserialize)]
struct AppConfig {
    /// Server port.
    port: u16,
    allowed_origin: Vec<String>,
    admin: Credentials,
    // /// Authentication related configuration.
    // auth: AuthConfig,
    /// Database connection info.
    database: DatabaseConfig,
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

    let app_config: AppConfig = toml::from_str(&fs::read_to_string(&env::var("CONFIG_PATH")?)?)?;

    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect_with(
            PgConnectOptions::new()
                .host(&app_config.database.host)
                .database(&app_config.database.name)
                .username(&app_config.database.username)
                .password(&app_config.database.password),
        )
        .await?;

    MIGRATOR.run(&db).await?;

    let session_store = PostgresStore::new(db.clone());
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
    let backend = AuthBackend::new(db.clone());
    let auth_layer = AuthManagerLayerBuilder::new(backend.clone(), session_layer).build();

    tokio::spawn(async move { create_admin_users(backend, app_config.admin).await.unwrap() });

    let allowed_origin = app_config
        .allowed_origin
        .into_iter()
        .map(|x| x.parse::<HeaderValue>())
        .collect::<Result<Vec<_>, _>>()?;

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
        .layer(TimeoutLayer::new(Duration::from_secs(300)))
        .map_response(set_partitioned_cookie)
        .layer(auth_layer);

    let router = api::router().layer(service).with_state(AppState { db });

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), app_config.port);
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("API docs url http://localhost:{}/docs", app_config.port);
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, router.into_make_service()).await?;

    Ok(())
}

/// Sets up tracing for debuging and monitoring.
/// Does nothing if called more than once.
fn setup_tracing() {
    static INIT: std::sync::Once = std::sync::Once::new();

    INIT.call_once(|| {
        let _subscriber = tracing_subscriber::registry()
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

/// Sets the "Partitioned" attribute of the "set-cookie" header.
fn set_partitioned_cookie(mut res: Response) -> Response {
    if let Some(set_cookie) = res.headers().get(header::SET_COOKIE) {
        if let Ok(cookie_value) = set_cookie.to_str() {
            if !cookie_value.contains("Partitioned") {
                let cookie_value = format!("{}; Partitioned", cookie_value);
                let headers = res.headers_mut();
                headers.insert(
                    header::SET_COOKIE,
                    HeaderValue::from_str(&cookie_value).unwrap(),
                );
            }
        }
    }
    res
}

/// Create the admin users from the application secrets.
async fn create_admin_users(mut backend: AuthBackend, admin_creds: Credentials) -> sqlx::Result<()> {
    if !backend.exists(&admin_creds).await? {
        let user_id = backend.create_user(admin_creds).await?.user_id;
        backend.set_role(user_id, UserRole::Admin).await?;
    }
    Ok(())
}
