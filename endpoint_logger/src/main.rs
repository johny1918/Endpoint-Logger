use tokio::net::TcpListener;
use endpoint_logger::run;
use dotenvy::dotenv;
mod config;
mod utils;
use crate::config::AppConfig;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env
    dotenv().ok();
    let config = AppConfig::from_env();

    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.port)).await.expect("Failed to bind address");
    let handle = run(listener).await?;
    handle.await?;
    Ok(())
}
