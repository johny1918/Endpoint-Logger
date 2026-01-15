use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: Option<Uuid>,
    pub request_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub method: String,
    pub path: String,
    pub query_string: Option<String>,
    pub status_code: u16,
    pub duration_ms: u64,
    pub request_headers: HashMap<String, String>,
    pub request_body: Option<String>,
    pub response_headers: HashMap<String, String>,
    pub response_body: Option<String>,
    pub client_ip: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFilter {
    pub from_timestamp: DateTime<Utc>,
    pub to_timestamp: DateTime<Utc>,
    pub methods: Vec<String>,
    pub paths: Vec<String>,
    pub status_codes: Vec<u16>,
    pub search_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    pub total_requests: u64,
    pub success_rate: f64,
    pub error_rate: f64,
    pub avg_response_time: f64,
    pub requests_by_endpoint: HashMap<String, u64>,
    pub requests_by_status: HashMap<u16, u64>,
}


impl LogEntry {
    pub fn new(
        method: String,
        path: String,
        query_string: Option<String>,
        status_code: u16,
        client_ip: String,
    ) -> Self {
        Self {
            id: Some(Uuid::new_v4()),
            request_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            method,
            path,
            query_string,
            status_code,
            duration_ms: 0,
            request_headers: HashMap::new(),
            request_body: None,
            response_headers: HashMap::new(),
            response_body: None,
            client_ip,
        }
    }
    
    // Then add setters for optional data
    pub fn with_request_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.request_headers = headers;
        self
    }
    
    pub fn with_request_body(mut self, body: String) -> Self {
        self.request_body = Some(body);
        self
    }
    
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }
}