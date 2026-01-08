use tokio::net::TcpListener;
use endpoint_logger::run;
use dotenvy::dotenv;
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
    

    // Bind to proxy server port
    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.proxy_port)).await.expect("Failed to bind address");
    let handle = run(listener).await?;
    handle.await?;
    Ok(())
}
