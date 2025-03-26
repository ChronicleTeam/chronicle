use std::sync::Arc;

use chronicle::{
    config::Config,
    routes::{self, ApiState},
};
use sqlx::{migrate::Migrator, PgPool};

static MIGRATOR: Migrator = sqlx::migrate!();

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    // Load database URL from .env for development
    #[cfg(debug_assertions)]
    dotenvy::dotenv().ok();

    MIGRATOR.run(&pool).await.unwrap();

    let router = routes::create_app(ApiState {
        _config: Arc::new(Config {
            database_url: String::new(),
            hmac_key: String::new(),
        }),
        pool,
    });

    Ok(router.into())
}
