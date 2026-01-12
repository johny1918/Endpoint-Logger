use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: Option<i64>,
    pub request_id: String,
    pub timestamp: i64,
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
    pub from_timestamp: Option<i64>,
    pub to_timestamp: Option<i64>,
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