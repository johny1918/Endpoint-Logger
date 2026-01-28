use tokio::sync::mpsc::{self, Receiver, Sender};
use tracing::{info, error, warn};

use crate::api::broadcaster::Broadcaster;
use crate::models::proxy::LogEntry;
use crate::storage::SqliteStorage;

/// Channel buffer size for log entries
const CHANNEL_BUFFER_SIZE: usize = 1000;

/// Sender for log entries - cloneable and cheap to pass around
#[derive(Clone)]
pub struct LogSender {
    sender: Sender<LogEntry>,
}

impl LogSender {
    /// Send a log entry to the worker
    /// Returns immediately without waiting for processing
    pub fn send(&self, entry: LogEntry) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            if let Err(e) = sender.send(entry).await {
                warn!("Failed to send log entry to worker: {}", e);
            }
        });
    }
}

/// Background worker that processes log entries
/// Receives entries from the channel, inserts to database, and broadcasts to WebSocket clients
pub struct LogWorker {
    receiver: Receiver<LogEntry>,
    storage: SqliteStorage,
    broadcaster: Broadcaster,
}

impl LogWorker {
    /// Create a new LogWorker and its corresponding LogSender
    /// Returns (LogSender, LogWorker) - sender goes to ProxyState, worker runs in background
    pub fn new(storage: SqliteStorage, broadcaster: Broadcaster) -> (LogSender, Self) {
        let (sender, receiver) = mpsc::channel(CHANNEL_BUFFER_SIZE);

        let log_sender = LogSender { sender };
        let worker = Self {
            receiver,
            storage,
            broadcaster,
        };

        (log_sender, worker)
    }

    /// Run the worker loop
    /// Processes log entries until the channel is closed
    pub async fn run(mut self) {
        info!("Log worker started");

        while let Some(entry) = self.receiver.recv().await {
            self.process_entry(&entry).await;
        }

        info!("Log worker shutting down");
    }

    /// Process a single log entry
    /// Inserts to database and broadcasts to WebSocket clients
    async fn process_entry(&self, entry: &LogEntry) {
        // Insert to database
        match self.storage.insert(entry).await {
            Ok(row_id) => {
                info!(
                    row_id = row_id,
                    request_id = %entry.request_id,
                    "Log entry inserted to database"
                );
            }
            Err(e) => {
                error!(
                    request_id = %entry.request_id,
                    error = %e,
                    "Failed to insert log entry to database"
                );
            }
        }

        // Broadcast to WebSocket clients
        match self.broadcaster.broadcast(entry) {
            Ok(count) => {
                if count > 0 {
                    info!(
                        clients = count,
                        request_id = %entry.request_id,
                        "Broadcasted log entry to WebSocket clients"
                    );
                }
            }
            Err(e) => {
                warn!(
                    request_id = %entry.request_id,
                    error = %e,
                    "Failed to broadcast log entry"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;
    use uuid::Uuid;

    fn create_test_entry() -> LogEntry {
        LogEntry {
            id: Some(Uuid::new_v4()),
            request_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            method: "GET".to_string(),
            path: "/test".to_string(),
            query_string: None,
            status_code: 200,
            duration_ms: 50,
            request_headers: HashMap::new(),
            request_body: None,
            response_headers: HashMap::new(),
            response_body: None,
            client_ip: "127.0.0.1".to_string(),
        }
    }

    #[tokio::test]
    async fn test_log_sender_is_cloneable() {
        let storage = SqliteStorage::new(":memory:").await.unwrap();
        let broadcaster = Broadcaster::new();
        let (sender, _worker) = LogWorker::new(storage, broadcaster);

        // LogSender should be cloneable
        let _sender2 = sender.clone();
    }

    #[tokio::test]
    async fn test_worker_processes_entry() {
        let storage = SqliteStorage::new(":memory:").await.unwrap();
        let broadcaster = Broadcaster::new();
        let (sender, worker) = LogWorker::new(storage.clone(), broadcaster);

        // Spawn worker in background
        let worker_handle = tokio::spawn(worker.run());

        // Send an entry
        let entry = create_test_entry();
        let request_id = entry.request_id.to_string();
        sender.send(entry);

        // Give worker time to process
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Verify entry was inserted
        let retrieved = storage.get_by_request_id(&request_id).await.unwrap();
        assert!(retrieved.is_some());

        // Drop sender to close channel and stop worker
        drop(sender);
        worker_handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_worker_handles_multiple_entries() {
        let storage = SqliteStorage::new(":memory:").await.unwrap();
        let broadcaster = Broadcaster::new();
        let (sender, worker) = LogWorker::new(storage.clone(), broadcaster);

        // Spawn worker
        let worker_handle = tokio::spawn(worker.run());

        // Send multiple entries
        for _ in 0..5 {
            sender.send(create_test_entry());
        }

        // Give worker time to process
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Verify all entries were inserted
        let count = storage.count().await.unwrap();
        assert_eq!(count, 5);

        drop(sender);
        worker_handle.await.unwrap();
    }
}
