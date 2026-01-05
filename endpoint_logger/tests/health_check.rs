use tokio::net::TcpListener;
use std::net::SocketAddr;

#[tokio::test]
async fn health_check_works() {
    //Arrange
    let server = spawn_app().await.expect("Failed to spawn our app");
    let client = reqwest::Client::new();

    //Act
    let response = client
        .get(format!("http://{}/health_check", server))
        .send()
        .await
        .expect("Failed to execute request");

    //Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

async fn spawn_app() -> anyhow::Result<SocketAddr> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind address");
    let address = listener.local_addr().expect("Failed to get port");
    endpoint_logger::run(listener).await?;
    Ok(address)
}