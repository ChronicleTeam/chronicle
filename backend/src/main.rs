use std::sync::Arc;

use chronicle::{
    config::Config,
    routes::{self, ApiState},
};
use shuttle_runtime::SecretStore;
use sqlx::migrate::Migrator;

static MIGRATOR: Migrator = sqlx::migrate!();

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres(local_uri = "{secrets.DATABASE_URL}")] database_url: String,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    let pool = sqlx::PgPool::connect(&database_url).await.unwrap();

    MIGRATOR.run(&pool).await.expect("Migration error");

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
