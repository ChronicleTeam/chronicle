#[derive(clap::Parser, Clone)]
pub struct Config {
    #[clap(long, env)]
    pub database_url: String,
}
