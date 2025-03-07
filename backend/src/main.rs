use chronicle::{config::Config, routes, setup_tracing};
use clap::Parser;
use sqlx::{migrate::Migrator, postgres::PgPoolOptions};
use std::time::Duration;

static MIGRATOR: Migrator = sqlx::migrate!(); // Points to the migrations folder

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing();

    // Load database URL from .env for development
    dotenvy::dotenv().ok();

    // Parse configs from environment variables
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
