#[tokio::main]
async fn main() -> anyhow::Result<()> {
    chronicle::serve().await
}
