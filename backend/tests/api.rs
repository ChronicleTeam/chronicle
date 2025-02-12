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
use tokio::sync::OnceCell;

static MIGRATOR: Migrator = sqlx::migrate!();

async fn setup(db: PgPool) -> Result<SocketAddr> {
    dotenvy::dotenv().ok();
    let options = db.connect_options();
    let config = Config {
        
    };

    MIGRATOR.run(&db).await?;

    routes::serve(config, db).await
}

#[tokio::test]
async fn test_root_endpoint() -> Result<()> {
    let config = Config::parse();

    let db = PgPoolOptions::new()
        .max_connections(50)
        .acquire_timeout(Duration::from_secs(3))
        .connect_with(
            PgConnectOptions::new()
                .host(&config.database_host)
                .port(config.database_port)
                .username(&config.database_user)
                .password(&config.database_password),
        )
        .await?;

    MIGRATOR.run(&db).await?;

    routes::serve(config, db).await?;

    let client = Client::new();

    let resp = client
        .get(format!("http://{}/", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert_eq!(body, "Hello, world!");

    Ok(())
}
