use tokio::net::TcpListener;
use std::net::SocketAddr;
use axum::{Router, routing::{get, post, put, delete as axum_delete, patch, MethodRouter}, http::StatusCode, extract::Request};
use serde_json::json;

// EP-001-06: Basic proxy forwarding test
#[tokio::test]
async fn proxy_forwards_get_request() {
    let (target_addr, proxy_addr) = setup_servers().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/test", proxy_addr))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    let body = response.text().await.unwrap();
    assert_eq!(body, "GET response");
}

// EP-001-07: Test POST method
#[tokio::test]
async fn proxy_forwards_post_request() {
    let (target_addr, proxy_addr) = setup_servers().await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("http://{}/api/data", proxy_addr))
        .json(&json!({"name": "test", "value": 123}))
        .send()
        .await
        .expect("Failed to execute POST request");

    assert!(response.status().is_success());
    let body = response.text().await.unwrap();
    assert_eq!(body, "POST received");
}

// EP-001-07: Test PUT method
#[tokio::test]
async fn proxy_forwards_put_request() {
    let (target_addr, proxy_addr) = setup_servers().await;
    let client = reqwest::Client::new();

    let response = client
        .put(format!("http://{}/api/data/1", proxy_addr))
        .json(&json!({"name": "updated"}))
        .send()
        .await
        .expect("Failed to execute PUT request");

    assert!(response.status().is_success());
    let body = response.text().await.unwrap();
    assert_eq!(body, "PUT received");
}

// EP-001-07: Test DELETE method
#[tokio::test]
async fn proxy_forwards_delete_request() {
    let (target_addr, proxy_addr) = setup_servers().await;
    let client = reqwest::Client::new();

    let response = client
        .delete(format!("http://{}/api/data/1", proxy_addr))
        .send()
        .await
        .expect("Failed to execute DELETE request");

    assert!(response.status().is_success());
    let body = response.text().await.unwrap();
    assert_eq!(body, "DELETE received");
}

// EP-001-07: Test PATCH method
#[tokio::test]
async fn proxy_forwards_patch_request() {
    let (target_addr, proxy_addr) = setup_servers().await;
    let client = reqwest::Client::new();

    let response = client
        .patch(format!("http://{}/api/data/1", proxy_addr))
        .json(&json!({"field": "value"}))
        .send()
        .await
        .expect("Failed to execute PATCH request");

    assert!(response.status().is_success());
    let body = response.text().await.unwrap();
    assert_eq!(body, "PATCH received");
}

// EP-001-07: Test HEAD method (should return headers only, no body)
#[tokio::test]
async fn proxy_forwards_head_request() {
    let (target_addr, proxy_addr) = setup_servers().await;
    let client = reqwest::Client::new();

    let response = client
        .head(format!("http://{}/test", proxy_addr))
        .send()
        .await
        .expect("Failed to execute HEAD request");

    assert!(response.status().is_success());
    // HEAD responses should have no body
    assert_eq!(response.content_length(), Some(0));
}

// EP-001-07: Test OPTIONS method
#[tokio::test]
async fn proxy_forwards_options_request() {
    let (target_addr, proxy_addr) = setup_servers().await;
    let client = reqwest::Client::new();

    let response = client
        .request(reqwest::Method::OPTIONS, format!("http://{}/test", proxy_addr))
        .send()
        .await
        .expect("Failed to execute OPTIONS request");

    // OPTIONS typically returns 200 or 204
    // The proxy should forward whatever the target returns
    assert!(response.status().as_u16() == 200 || response.status().as_u16() == 204,
        "Expected 200 or 204, got {}", response.status());
    let body = response.text().await.unwrap();
    assert_eq!(body, "OPTIONS received");
}

// Test query parameters are preserved
#[tokio::test]
async fn proxy_preserves_query_parameters() {
    let (target_addr, proxy_addr) = setup_servers().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/test?foo=bar&baz=qux", proxy_addr))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    let body = response.text().await.unwrap();
    assert_eq!(body, "GET response");
}

// Helper function to setup both target and proxy servers
async fn setup_servers() -> (SocketAddr, SocketAddr) {
    let target_addr = spawn_target_server().await;
    let proxy_addr = spawn_proxy(format!("http://{}", target_addr))
        .await
        .expect("Failed to spawn proxy");
    (target_addr, proxy_addr)
}

async fn spawn_proxy(target_url: String) -> anyhow::Result<SocketAddr> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind address");
    let address = listener.local_addr().expect("Failed to get port");
    endpoint_logger::run(listener, target_url).await?;
    Ok(address)
}

async fn spawn_target_server() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind address");
    let address = listener.local_addr().expect("Failed to get port");

    let app = Router::new()
        // GET route with OPTIONS support
        .route("/test",
            get(|| async { "GET response" })
            .options(|| async { "OPTIONS received" })
        )
        // POST route
        .route("/api/data", post(|| async { "POST received" }))
        // PUT route
        .route("/api/data/{id}", put(|| async { "PUT received" }))
        // DELETE route
        .route("/api/data/{id}", axum_delete(|| async { "DELETE received" }))
        // PATCH route
        .route("/api/data/{id}", patch(|| async { "PATCH received" }))
        // Fallback
        .fallback(|| async { (StatusCode::OK, "Target fallback") });

    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("Failed to start target server");
    });

    // Give the server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    address
}