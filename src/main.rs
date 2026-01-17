use tokio::net::TcpListener;
use endpoint_logger::{run, graceful_shutdown, storage::SqliteStorage};
use dotenvy::dotenv;
use tracing::info;
mod config;
mod utils;
use crate::config::AppConfig;
use crate::utils::logger::init_tracing;


#[tokio::main]
async fn main() -> anyhow::Result<()> {

    if let Err(e) = init_tracing() {
        eprintln!("Initialization of logger failed with error: {}", e);
    }

    // Load .env file if present
    dotenv().ok();

    // Load configuration with priority: CLI > ENV > TOML > Defaults
    let config = AppConfig::load().unwrap_or_else(|e| {
        eprintln!("Configuration Error: {}", e);
        std::process::exit(1);
    });

    config.print_config_used();

    // Initialize SQLite storage
    let storage = SqliteStorage::new(&config.database_path).await.unwrap_or_else(|e| {
        eprintln!("Failed to initialize database: {}", e);
        std::process::exit(1);
    });
    info!(database = %config.database_path, "Storage initialized");

    // Bind to proxy server port
    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.proxy_port)).await.expect("Failed to bind address");
    let handle = run(listener, config.target_url, storage).await?;
    handle.await?;
    graceful_shutdown().await?;
    Ok(())
}
