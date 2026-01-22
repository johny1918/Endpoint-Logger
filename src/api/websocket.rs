use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use tokio::sync::broadcast::Receiver;
use tracing::{info, warn};
use futures_util::{StreamExt, SinkExt};

use crate::proxy::ProxyState;

/// WebSocket handler - upgrades HTTP connection to WebSocket
/// Subscribes to the broadcast channel to receive log entries
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<ProxyState>,
) -> Response {
    info!("WebSocket connection request received");

    // Subscribe to broadcast channel before upgrading
    let rx = state.broadcaster.subscribe();

    ws.on_upgrade(move |socket| handle_socket(socket, rx))
}

/// Handle an individual WebSocket connection
/// - Subscribes to broadcast channel for log entries
/// - Forwards broadcast messages to the client
/// - Responds to Ping with Pong
/// - Handles Close messages gracefully
async fn handle_socket(socket: WebSocket, mut rx: Receiver<String>) {
    info!("WebSocket client connected");

    // Split socket into sender and receiver
    let (mut sender, mut receiver) = socket.split();

    // Spawn task to forward broadcast messages to WebSocket client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                // Client disconnected
                break;
            }
        }
        sender
    });

    // Handle incoming messages from client
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg_result) = receiver.next().await {
            match msg_result {
                Ok(msg) => {
                    match msg {
                        Message::Text(text) => {
                            info!(message = %text, "Received text message");
                        }
                        Message::Binary(data) => {
                            info!(bytes = data.len(), "Received binary message");
                        }
                        Message::Ping(_) => {
                            // Axum handles ping/pong automatically
                            info!("Received Ping");
                        }
                        Message::Pong(_) => {
                            info!("Received Pong");
                        }
                        Message::Close(close_frame) => {
                            if let Some(cf) = close_frame {
                                info!(
                                    code = %cf.code,
                                    reason = %cf.reason,
                                    "WebSocket client requested close"
                                );
                            } else {
                                info!("WebSocket client requested close");
                            }
                            break;
                        }
                    }
                }
                Err(e) => {
                    warn!(error = %e, "WebSocket error");
                    break;
                }
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();
        }
        _ = &mut recv_task => {
            send_task.abort();
        }
    }

    info!("WebSocket client disconnected");
}
