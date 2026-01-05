use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};


async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}


#[tokio::main]
async fn main() {
    let app = Router::new()
            .route("/health_check", get(health_check));
    
    let listener = tokio::net::TcpListener
                ::bind("127.0.0.1:3000").await.expect("Failed to bind address");
    
    println!("Server running on http://{:?}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.expect("Failed to start axum server");
}
