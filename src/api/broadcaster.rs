use tokio::sync::broadcast::{self, Receiver, Sender};
use crate::models::proxy::LogEntry;

/// Channel capacity - how many messages can be buffered
const CHANNEL_CAPACITY: usize = 100;

/// Broadcaster wraps a tokio broadcast channel for sending log entries
/// to all connected WebSocket clients
#[derive(Clone)]
pub struct Broadcaster {
    sender: Sender<String>,
}

impl Broadcaster {
    /// Create a new Broadcaster with a broadcast channel
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(CHANNEL_CAPACITY);
        Self { sender }
    }

    /// Subscribe to receive broadcast messages
    /// Returns a Receiver that will get all future broadcasts
    pub fn subscribe(&self) -> Receiver<String> {
        self.sender.subscribe()
    }

    /// Broadcast a log entry to all connected clients
    /// Serializes the entry to JSON before sending
    /// Returns Ok(count) with number of receivers, or handles no receivers gracefully
    pub fn broadcast(&self, entry: &LogEntry) -> Result<usize, BroadcastError> {
        // Serialize log entry to JSON
        let json = serde_json::to_string(entry)
            .map_err(|e| BroadcastError::SerializationError(e.to_string()))?;

        // Send to all subscribers
        // send() returns error if there are no receivers, which is fine
        match self.sender.send(json) {
            Ok(count) => Ok(count),
            Err(_) => {
                // No active receivers - this is not an error, just no clients connected
                Ok(0)
            }
        }
    }

    /// Get the current number of subscribers
    #[allow(dead_code)]
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for Broadcaster {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum BroadcastError {
    SerializationError(String),
}

impl std::fmt::Display for BroadcastError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BroadcastError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for BroadcastError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use uuid::Uuid;
    use chrono::Utc;

    fn create_test_log_entry() -> LogEntry {
        LogEntry {
            id: Some(Uuid::new_v4()),
            request_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            method: "GET".to_string(),
            path: "/api/test".to_string(),
            query_string: None,
            status_code: 200,
            duration_ms: 42,
            request_headers: HashMap::new(),
            request_body: None,
            response_headers: HashMap::new(),
            response_body: None,
            client_ip: "127.0.0.1".to_string(),
        }
    }

    #[test]
    fn test_broadcaster_creation() {
        let broadcaster = Broadcaster::new();
        assert_eq!(broadcaster.subscriber_count(), 0);
    }

    #[test]
    fn test_subscribe_increases_count() {
        let broadcaster = Broadcaster::new();
        let _rx1 = broadcaster.subscribe();
        assert_eq!(broadcaster.subscriber_count(), 1);

        let _rx2 = broadcaster.subscribe();
        assert_eq!(broadcaster.subscriber_count(), 2);
    }

    #[test]
    fn test_broadcast_with_no_subscribers() {
        let broadcaster = Broadcaster::new();
        let entry = create_test_log_entry();

        // Should not error even with no subscribers
        let result = broadcaster.broadcast(&entry);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_broadcast_to_subscriber() {
        let broadcaster = Broadcaster::new();
        let mut rx = broadcaster.subscribe();
        let entry = create_test_log_entry();
        let request_id = entry.request_id;

        // Broadcast
        let result = broadcaster.broadcast(&entry);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        // Receive
        let received = rx.recv().await.unwrap();
        assert!(received.contains(&request_id.to_string()));
        assert!(received.contains("GET"));
        assert!(received.contains("/api/test"));
    }

    #[tokio::test]
    async fn test_broadcast_to_multiple_subscribers() {
        let broadcaster = Broadcaster::new();
        let mut rx1 = broadcaster.subscribe();
        let mut rx2 = broadcaster.subscribe();
        let entry = create_test_log_entry();

        // Broadcast
        let result = broadcaster.broadcast(&entry);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);

        // Both should receive
        let received1 = rx1.recv().await.unwrap();
        let received2 = rx2.recv().await.unwrap();
        assert_eq!(received1, received2);
    }

    #[test]
    fn test_broadcast_serializes_to_json() {
        let broadcaster = Broadcaster::new();
        let mut rx = broadcaster.subscribe();
        let entry = create_test_log_entry();

        broadcaster.broadcast(&entry).unwrap();

        // Try to receive (non-blocking check)
        let received = rx.try_recv().unwrap();

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&received).unwrap();
        assert!(parsed.is_object());
        assert!(parsed.get("method").is_some());
        assert!(parsed.get("path").is_some());
        assert!(parsed.get("status_code").is_some());
    }
}
