use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
};
use tracing::{info, warn};
use futures_util::StreamExt;

/// WebSocket handler - upgrades HTTP connection to WebSocket
/// This endpoint accepts WebSocket connections at /ws
pub async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    info!("WebSocket connection request received");
    ws.on_upgrade(handle_socket)
}

/// Handle an individual WebSocket connection
/// - Logs connection and disconnection
/// - Responds to Ping with Pong
/// - Handles Close messages gracefully
async fn handle_socket(mut socket: WebSocket) {
    info!("WebSocket client connected");

    // Main message loop
    while let Some(msg_result) = socket.next().await {
        match msg_result {
            Ok(msg) => {
                match msg {
                    Message::Text(text) => {
                        // For now, just echo back or ignore
                        // In EP-001-14, we'll use this for broadcasts
                        info!(message = %text, "Received text message");
                    }
                    Message::Binary(data) => {
                        info!(bytes = data.len(), "Received binary message");
                    }
                    Message::Ping(data) => {
                        // Respond to Ping with Pong
                        info!("Received Ping, sending Pong");
                        if let Err(e) = socket.send(Message::Pong(data)).await {
                            warn!(error = %e, "Failed to send Pong");
                            break;
                        }
                    }
                    Message::Pong(_) => {
                        // Client responded to our ping (if we sent one)
                        info!("Received Pong");
                    }
                    Message::Close(close_frame) => {
                        // Client requested close
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

    info!("WebSocket client disconnected");
}
