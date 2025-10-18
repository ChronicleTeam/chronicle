mod api;
mod auth;
mod db;
mod docs;
mod error;
mod io;
mod model;

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
    println!("{:?}", config.allowed_origin);

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
        let _subscriber = tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                    format!(
                        "{}=debug,tower_http=debug,axum::rejection=trace,tower_sessions=trace",
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
pub mod test {
    use crate::{
        AppConfig, AppState, Id, api,
        auth::{self, AppAuthSession},
        error::{ApiResult, IntoAnyhow},
        init_layers,
        model::users::User,
    };
    use aide::openapi::OpenApi;
    use axum::{
        Json, Router,
        http::header::SET_COOKIE,
        routing::{get, post},
    };
    use axum_test::TestServer;
    use password_auth::generate_hash;
    use sqlx::PgPool;

    async fn login(mut session: AppAuthSession, Json(user): Json<User>) -> ApiResult<()> {
        session.login(&user).await.anyhow()?;
        Ok(())
    }

    async fn get_auth_user(session: AppAuthSession) -> Json<Option<User>> {
        Json(session.user.clone())
    }

    async fn router_setup(config: AppConfig, db: PgPool) -> anyhow::Result<Router> {
        let app = api::router().finish_api(&mut OpenApi::default());
        let app = app.nest(
            "/test",
            Router::new()
                .route("/login", post(login))
                .route("/user", get(get_auth_user)),
        );
        let app = auth::init(app, db.clone(), config.session_key).await?;
        let app = init_layers(app, config.allowed_origin)?;
        Ok(app.with_state(AppState { db }))
    }

    pub async fn test_server(db: PgPool) -> anyhow::Result<TestServer> {
        let config = AppConfig::build()?;
        let server = TestServer::new(router_setup(config, db).await?)?;
        Ok(server)
    }

    pub async fn test_server_logged_in(
        db: PgPool,
        username: &str,
        password: &str,
        is_admin: bool,
    ) -> anyhow::Result<(TestServer, Id)> {
        let config = AppConfig::build()?;
        let app = router_setup(config, db.clone()).await?;

        let user: User = sqlx::query_as(
            r#"
                INSERT INTO app_user (username, password_hash, is_admin)
                VALUES ($1, $2, $3)
                RETURNING *
            "#,
        )
        .bind(username)
        .bind(generate_hash(password))
        .bind(is_admin)
        .fetch_one(&db)
        .await?;

        let mut server = TestServer::new(app)?;
        server.save_cookies();
        let response = server.post("/test/login").json(&user).await;
        response.assert_status_ok();
        response.assert_contains_header(SET_COOKIE);
        println!("{:?}\n", response.cookies());

        Ok((server, user.user_id))
    }
}
