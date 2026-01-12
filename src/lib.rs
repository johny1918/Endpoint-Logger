use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio::signal;
use tracing::info;
use crate::routes::proxy::proxy_router;
use crate::proxy::ProxyState;

mod utils;
mod proxy;
mod routes;

use crate::utils::errors::AppError;

/// Run the HTTP proxy server
/// Takes the target_url from config and starts the server
pub async fn run(listener: TcpListener, target_url: String) -> anyhow::Result<JoinHandle<()>> {

    // Create shared state with HTTP client and target URL
    let state = ProxyState::new(target_url);

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