use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio::signal;
use tracing::info;
use crate::routes::proxy::proxy_router;
use crate::proxy::ProxyState;
use crate::storage::SqliteStorage;
use crate::api::broadcaster::Broadcaster;
use crate::logging::LogWorker;

mod api;
mod utils;
mod proxy;
mod routes;
mod models;
mod logging;
pub mod storage;

use crate::utils::errors::AppError;

/// Run the HTTP proxy server
/// Takes the target_url from config and starts the server
/// Spawns a background worker for async logging
pub async fn run(listener: TcpListener, target_url: String, storage: SqliteStorage) -> anyhow::Result<JoinHandle<()>> {

    // Create broadcaster for WebSocket clients
    let broadcaster = Broadcaster::new();

    // Create log worker and sender for async logging pipeline
    let (log_sender, log_worker) = LogWorker::new(storage.clone(), broadcaster.clone());

    // Spawn the log worker in the background
    tokio::spawn(log_worker.run());

    // Create shared state with HTTP client, target URL, storage (for reads), and log sender (for writes)
    let state = ProxyState::new(target_url, storage, log_sender, broadcaster);

    // Create router with state
    let app = proxy_router(state);

    let handle = tokio::spawn(async move {
        println!("Server running on http://{:?}", listener.local_addr().unwrap());

        if let Err(e) = axum::serve(listener, app).await {
            println!("Failed to start server because of {}", e)
        }
    });

    Ok(handle)
}

pub async fn graceful_shutdown() -> Result<(), AppError> {
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Graceful shutdown");
            Ok(())
        },
        Err(e) => {
            Err(AppError::GracefulShutdownError(e.to_string()))
        },
    }
}