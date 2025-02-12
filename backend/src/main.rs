use chronicle::{config::Config, routes, setup_tracing};
use clap::Parser;
use sqlx::{migrate::Migrator, postgres::PgPoolOptions};
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static MIGRATOR: Migrator = sqlx::migrate!(); // Points to the migrations folder

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing();

    dotenvy::dotenv().ok();

    let config = Config::try_parse()?;

    let pool = PgPoolOptions::new()
        .max_connections(50)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&config.database_url)
        .await?;

    MIGRATOR.run(&pool).await?;

    routes::serve(config, pool).await?;

    Ok(())
}
