/// API configuration.
///
/// Contains the database URL to be used in handlers.
#[derive(clap::Parser, Clone)]
pub struct Config {
    #[clap(long, env)]
    pub database_url: String,

    #[clap(long, env)]
    pub hmac_key: String,
}
