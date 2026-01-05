use anyhow::Ok;
use tokio::net::TcpListener;
use endpoint_logger::run;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3000").await.expect("Failed to bind address");
    run(listener).await?;
    Ok(())
}
