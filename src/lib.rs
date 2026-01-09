use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio::signal;
use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};
use tracing::info;

mod utils;

use crate::utils::errors::AppError;

pub async fn health_check() -> impl IntoResponse {
    info!("Health check handle alive");
    StatusCode::OK
}

pub async fn run(listener: TcpListener) -> anyhow::Result<JoinHandle<()>> {

    let app = Router::new()
        .route("/health_check", get(health_check));

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