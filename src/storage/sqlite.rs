use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite, Row};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::models::proxy::LogEntry;
use crate::utils::errors::AppError;

/// SQLite storage for log entries
/// Uses connection pooling and WAL mode for concurrent access
#[derive(Clone)]
pub struct SqliteStorage {
    pool: Pool<Sqlite>,
}

impl SqliteStorage {
    /// Create a new SqliteStorage instance
    /// Creates the database file if it doesn't exist
    /// Initializes the schema and enables WAL mode
    pub async fn new(database_path: &str) -> Result<Self, AppError> {
        // Build connection string with create_if_missing
        let connection_string = format!("sqlite://{}?mode=rwc", database_path);

        // Create connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&connection_string)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to connect to database: {}", e)))?;

        let storage = Self { pool };

        // Initialize schema
        storage.initialize_schema().await?;

        Ok(storage)
    }

    /// Initialize the database schema
    /// Creates tables and indexes if they don't exist
    /// Enables WAL mode for better concurrent performance
    async fn initialize_schema(&self) -> Result<(), AppError> {
        // Enable WAL mode for better concurrent read/write performance
        sqlx::query("PRAGMA journal_mode=WAL;")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to enable WAL mode: {}", e)))?;

        // Create log_entries table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS log_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                request_id TEXT UNIQUE NOT NULL,
                timestamp INTEGER NOT NULL,
                method TEXT NOT NULL,
                path TEXT NOT NULL,
                query_string TEXT,
                status_code INTEGER NOT NULL,
                duration_ms INTEGER NOT NULL,
                request_headers TEXT,
                request_body TEXT,
                response_headers TEXT,
                response_body TEXT,
                client_ip TEXT,
                created_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create table: {}", e)))?;

        // Create indexes for common queries
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_timestamp ON log_entries(timestamp)")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to create timestamp index: {}", e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_path ON log_entries(path)")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to create path index: {}", e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_status ON log_entries(status_code)")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to create status index: {}", e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_method ON log_entries(method)")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to create method index: {}", e)))?;

        Ok(())
    }

    /// Insert a log entry into the database
    /// Returns the inserted row ID
    pub async fn insert(&self, entry: &LogEntry) -> Result<i64, AppError> {
        let request_headers_json = serde_json::to_string(&entry.request_headers)
            .map_err(|e| AppError::DatabaseError(format!("Failed to serialize request headers: {}", e)))?;

        let response_headers_json = serde_json::to_string(&entry.response_headers)
            .map_err(|e| AppError::DatabaseError(format!("Failed to serialize response headers: {}", e)))?;

        let timestamp = entry.timestamp.timestamp();
        let created_at = Utc::now().timestamp();

        let result = sqlx::query(
            r#"
            INSERT INTO log_entries (
                request_id, timestamp, method, path, query_string,
                status_code, duration_ms, request_headers, request_body,
                response_headers, response_body, client_ip, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(entry.request_id.to_string())
        .bind(timestamp)
        .bind(&entry.method)
        .bind(&entry.path)
        .bind(&entry.query_string)
        .bind(entry.status_code as i32)
        .bind(entry.duration_ms as i64)
        .bind(&request_headers_json)
        .bind(&entry.request_body)
        .bind(&response_headers_json)
        .bind(&entry.response_body)
        .bind(&entry.client_ip)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to insert log entry: {}", e)))?;

        Ok(result.last_insert_rowid())
    }

    /// Query recent log entries ordered by timestamp descending
    pub async fn query_recent(&self, limit: u32) -> Result<Vec<LogEntry>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, request_id, timestamp, method, path, query_string,
                   status_code, duration_ms, request_headers, request_body,
                   response_headers, response_body, client_ip
            FROM log_entries
            ORDER BY timestamp DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to query log entries: {}", e)))?;

        let mut entries = Vec::with_capacity(rows.len());
        for row in rows {
            let entry = self.row_to_log_entry(&row)?;
            entries.push(entry);
        }

        Ok(entries)
    }

    /// Get a specific log entry by request_id
    pub async fn get_by_request_id(&self, request_id: &str) -> Result<Option<LogEntry>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, request_id, timestamp, method, path, query_string,
                   status_code, duration_ms, request_headers, request_body,
                   response_headers, response_body, client_ip
            FROM log_entries
            WHERE request_id = ?
            "#,
        )
        .bind(request_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to query log entry: {}", e)))?;

        match row {
            Some(row) => Ok(Some(self.row_to_log_entry(&row)?)),
            None => Ok(None),
        }
    }

    /// Get the total count of log entries
    pub async fn count(&self) -> Result<u64, AppError> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM log_entries")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to count log entries: {}", e)))?;

        let count: i64 = row.get("count");
        Ok(count as u64)
    }

    /// Delete log entries older than the specified timestamp
    pub async fn delete_older_than(&self, timestamp: DateTime<Utc>) -> Result<u64, AppError> {
        let result = sqlx::query("DELETE FROM log_entries WHERE timestamp < ?")
            .bind(timestamp.timestamp())
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to delete old entries: {}", e)))?;

        Ok(result.rows_affected())
    }

    /// Convert a database row to a LogEntry
    fn row_to_log_entry(&self, row: &sqlx::sqlite::SqliteRow) -> Result<LogEntry, AppError> {
        let id: i64 = row.get("id");
        let request_id_str: String = row.get("request_id");
        let timestamp_unix: i64 = row.get("timestamp");
        let method: String = row.get("method");
        let path: String = row.get("path");
        let query_string: Option<String> = row.get("query_string");
        let status_code: i32 = row.get("status_code");
        let duration_ms: i64 = row.get("duration_ms");
        let request_headers_json: Option<String> = row.get("request_headers");
        let request_body: Option<String> = row.get("request_body");
        let response_headers_json: Option<String> = row.get("response_headers");
        let response_body: Option<String> = row.get("response_body");
        let client_ip: String = row.get("client_ip");

        // Parse UUID
        let request_id = Uuid::parse_str(&request_id_str)
            .map_err(|e| AppError::DatabaseError(format!("Invalid request_id UUID: {}", e)))?;

        // Parse timestamp
        let timestamp = DateTime::from_timestamp(timestamp_unix, 0)
            .ok_or_else(|| AppError::DatabaseError("Invalid timestamp".to_string()))?;

        // Parse headers JSON
        let request_headers: HashMap<String, String> = request_headers_json
            .map(|json| serde_json::from_str(&json).unwrap_or_default())
            .unwrap_or_default();

        let response_headers: HashMap<String, String> = response_headers_json
            .map(|json| serde_json::from_str(&json).unwrap_or_default())
            .unwrap_or_default();

        Ok(LogEntry {
            id: Some(Uuid::from_u128(id as u128)), // Using row id as a simple mapping
            request_id,
            timestamp,
            method,
            path,
            query_string,
            status_code: status_code as u16,
            duration_ms: duration_ms as u64,
            request_headers,
            request_body,
            response_headers,
            response_body,
            client_ip,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use uuid::Uuid;
    use chrono::Utc;

    async fn create_test_storage() -> SqliteStorage {
        // Use in-memory database for tests
        SqliteStorage::new(":memory:").await.unwrap()
    }

    fn create_test_log_entry() -> LogEntry {
        let mut request_headers = HashMap::new();
        request_headers.insert("content-type".to_string(), "application/json".to_string());

        let mut response_headers = HashMap::new();
        response_headers.insert("content-type".to_string(), "application/json".to_string());

        LogEntry {
            id: Some(Uuid::new_v4()),
            request_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            method: "GET".to_string(),
            path: "/api/users".to_string(),
            query_string: Some("page=1".to_string()),
            status_code: 200,
            duration_ms: 45,
            request_headers,
            request_body: Some(r#"{"name": "test"}"#.to_string()),
            response_headers,
            response_body: Some(r#"{"id": 1, "name": "test"}"#.to_string()),
            client_ip: "192.168.1.100".to_string(),
        }
    }

    #[tokio::test]
    async fn test_storage_creation() {
        let storage = create_test_storage().await;
        let count = storage.count().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_insert_and_query() {
        let storage = create_test_storage().await;
        let entry = create_test_log_entry();
        let request_id = entry.request_id;

        // Insert
        let row_id = storage.insert(&entry).await.unwrap();
        assert!(row_id > 0);

        // Count
        let count = storage.count().await.unwrap();
        assert_eq!(count, 1);

        // Query recent
        let entries = storage.query_recent(10).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].request_id, request_id);
        assert_eq!(entries[0].method, "GET");
        assert_eq!(entries[0].path, "/api/users");
        assert_eq!(entries[0].status_code, 200);
    }

    #[tokio::test]
    async fn test_get_by_request_id() {
        let storage = create_test_storage().await;
        let entry = create_test_log_entry();
        let request_id = entry.request_id;

        storage.insert(&entry).await.unwrap();

        // Find by request_id
        let found = storage.get_by_request_id(&request_id.to_string()).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().request_id, request_id);

        // Not found
        let not_found = storage.get_by_request_id(&Uuid::new_v4().to_string()).await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_query_recent_ordering() {
        let storage = create_test_storage().await;

        // Insert 3 entries
        for i in 0..3 {
            let mut entry = create_test_log_entry();
            entry.path = format!("/api/test/{}", i);
            storage.insert(&entry).await.unwrap();
        }

        // Query recent - should be in reverse order (newest first)
        let entries = storage.query_recent(10).await.unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].path, "/api/test/2");
        assert_eq!(entries[1].path, "/api/test/1");
        assert_eq!(entries[2].path, "/api/test/0");
    }

    #[tokio::test]
    async fn test_query_recent_limit() {
        let storage = create_test_storage().await;

        // Insert 5 entries
        for _ in 0..5 {
            let entry = create_test_log_entry();
            storage.insert(&entry).await.unwrap();
        }

        // Query with limit
        let entries = storage.query_recent(3).await.unwrap();
        assert_eq!(entries.len(), 3);
    }

    #[tokio::test]
    async fn test_headers_serialization() {
        let storage = create_test_storage().await;
        let entry = create_test_log_entry();
        let request_id = entry.request_id;

        storage.insert(&entry).await.unwrap();

        let found = storage.get_by_request_id(&request_id.to_string()).await.unwrap().unwrap();
        assert_eq!(found.request_headers.get("content-type"), Some(&"application/json".to_string()));
        assert_eq!(found.response_headers.get("content-type"), Some(&"application/json".to_string()));
    }

    #[tokio::test]
    async fn test_optional_fields() {
        let storage = create_test_storage().await;

        let entry = LogEntry {
            id: Some(Uuid::new_v4()),
            request_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            method: "GET".to_string(),
            path: "/api/test".to_string(),
            query_string: None,
            status_code: 200,
            duration_ms: 10,
            request_headers: HashMap::new(),
            request_body: None,
            response_headers: HashMap::new(),
            response_body: None,
            client_ip: "127.0.0.1".to_string(),
        };

        let request_id = entry.request_id;
        storage.insert(&entry).await.unwrap();

        let found = storage.get_by_request_id(&request_id.to_string()).await.unwrap().unwrap();
        assert!(found.query_string.is_none());
        assert!(found.request_body.is_none());
        assert!(found.response_body.is_none());
    }

    #[tokio::test]
    async fn test_delete_older_than() {
        let storage = create_test_storage().await;

        // Insert an entry
        let entry = create_test_log_entry();
        storage.insert(&entry).await.unwrap();

        // Delete entries older than now (should delete nothing since entry was just created)
        let deleted = storage.delete_older_than(Utc::now()).await.unwrap();
        // Entry might be deleted if timestamp is slightly in the past

        // Insert another entry and delete entries older than far future
        let entry2 = create_test_log_entry();
        storage.insert(&entry2).await.unwrap();

        // Delete entries older than far future (should delete all)
        let future = Utc::now() + chrono::Duration::hours(1);
        let deleted = storage.delete_older_than(future).await.unwrap();
        assert!(deleted >= 1);
    }
}
