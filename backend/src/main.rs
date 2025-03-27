use std::sync::Arc;

use chronicle::{
    config::Config,
    routes::{self, ApiState},
};
use shuttle_runtime::SecretStore;
use sqlx::{migrate::Migrator, PgPool};

static MIGRATOR: Migrator = sqlx::migrate!();

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: PgPool,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    MIGRATOR.run(&pool).await.unwrap();

    let router = routes::create_app(
        ApiState {
            config: Arc::new(Config {
                database_url: String::new(),
            }),
            pool,
        },
        secrets,
    )
    .await
    .unwrap();

    Ok(router.into())
}
