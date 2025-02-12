
pub mod config;
pub mod routes;
pub mod error;
pub mod model;
pub mod db;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

type Id = i32;

pub fn setup_tracing() {
    tracing_subscriber::registry()
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
}

#[cfg(test)]
fn setup_test() {
    
}