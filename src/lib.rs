use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};
use tracing::info;

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