#[derive(clap::Parser)]
pub struct Config {
    #[clap(long, env)]
    pub database_host: String,

    #[clap(long, env)]
    pub database_port: u16,

    #[clap(long, env)]
    pub database_user: String,

    #[clap(long, env)]
    pub database_password: String,
}
