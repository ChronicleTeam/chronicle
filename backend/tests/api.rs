use anyhow::Result;
use chronicle::{config::Config, routes};
use clap::Parser;
use reqwest::Client;
use sqlx::{
    migrate::Migrator,
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};
use std::{net::SocketAddr, time::Duration};

static MIGRATOR: Migrator = sqlx::migrate!();

async fn setup(pool: PgPool) -> Result<SocketAddr> {
    dotenvy::dotenv().ok();
    let config = Config::try_parse()?;

    MIGRATOR.run(&pool).await?;

    routes::serve(config, pool).await
}

#[sqlx::test]
async fn test_root_endpoint(pool: PgPool) -> Result<()> {
    setup(pool).await?;

    let client = Client::new();

    Ok(())
}
