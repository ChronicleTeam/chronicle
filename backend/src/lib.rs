pub mod config;
pub mod db;
pub mod error;
pub mod model;
pub mod routes;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

type Id = i32;

pub fn setup_tracing() {
    static INIT: std::sync::Once = std::sync::Once::new();

    INIT.call_once(|| {
        let _subscriber = tracing_subscriber::registry()
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

        // tracing::subscriber::set_global_default(subscriber)
        //     .expect("Failed to set global default tracing subscriber");
    });
}
