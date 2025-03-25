use chronicle::{config::Config, routes};
use sqlx::{migrate::Migrator, PgPool};

static MIGRATOR: Migrator = sqlx::migrate!();

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {

    MIGRATOR.run(&pool).await.unwrap();

    let router = routes::create_app(Config { database_url: String::new() }, pool);

    Ok(router.into())
}
