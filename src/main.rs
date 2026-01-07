use tokio::net::TcpListener;
use endpoint_logger::run;
use dotenvy::dotenv;
mod config;
mod utils;
use crate::config::AppConfig;


#[tokio::main]
async fn main() -> anyhow::Result<()> {

    // Load .env file if present
    dotenv().ok();

    // Load configuration with priority: CLI > ENV > TOML > Defaults
    let config = AppConfig::load().unwrap_or_else(|e| {
        eprintln!("Configuration Error: {}", e);
        std::process::exit(1);
    });

    // Bind to proxy server port
    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.proxy_port)).await.expect("Failed to bind address");
    let handle = run(listener).await?;
    handle.await?;
    Ok(())
}
