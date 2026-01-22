use reqwest::Client;
use std::time::Duration;

use crate::api::broadcaster::Broadcaster;
use crate::storage::SqliteStorage;
use crate::utils::errors::AppError;

/// Shared application state for the proxy
/// Contains the HTTP client (with connection pooling), target URL, storage, and broadcaster
#[derive(Clone)]
pub struct ProxyState {
    /// Reusable HTTP client with connection pooling and 30s timeout
    pub client: Client,
    /// Target application URL to forward requests to
    pub target_url: String,
    /// SQLite storage for log entries
    pub storage: SqliteStorage,
    /// Broadcast channel for real-time WebSocket updates
    pub broadcaster: Broadcaster,
}

impl ProxyState {
    /// Create new proxy state with configured target URL and storage
    pub fn new(target_url: String, storage: SqliteStorage) -> Self {
        // Create HTTP client with 30 second timeout (EP-001-06 requirement)
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10) // Connection pooling
            .build()
            .map_err(|_| AppError::ProxyStateError).unwrap_or_default();

        // Create broadcaster for WebSocket clients
        let broadcaster = Broadcaster::new();

        Self {
            client,
            target_url,
            storage,
            broadcaster,
        }
    }
}
