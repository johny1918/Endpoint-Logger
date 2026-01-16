use std::collections::HashMap;

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::HeaderMap;
use axum::response::Response;
use tracing::{info, error};
use uuid::Uuid;
use chrono::Utc;
use crate::proxy::forwarder::forward_request;
use crate::proxy::ProxyState;
use crate::utils::errors::AppError;
use crate::models::proxy::LogEntry;

const MAX_BODY_SIZE: usize = 100_000; // 100KB limit for body capture

/// Extract client IP from request headers
/// Priority: X-Forwarded-For (first IP) > X-Real-IP > "unknown"
fn extract_client_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

/// Convert HeaderMap to HashMap<String, String> for logging
fn headers_to_map(headers: &HeaderMap) -> HashMap<String, String> {
    headers
        .iter()
        .filter_map(|(key, value)| {
            value.to_str().ok().map(|v| (key.to_string(), v.to_string()))
        })
        .collect()
}

/// Capture request body as string with truncation marker if exceeds limit
fn capture_body_string(body_bytes: &[u8], max_size: usize) -> Option<String> {
    if body_bytes.is_empty() {
        None
    } else {
        let body_str = String::from_utf8_lossy(body_bytes).to_string();
        if body_bytes.len() >= max_size {
            Some(format!("{}... [TRUNCATED]", body_str))
        } else {
            Some(body_str)
        }
    }
}

/// Intercept incoming request and forward it to the target application
/// This is the main proxy handler that:
/// 1. Extracts request details (method, path, headers, body)
/// 2. Builds target URL
/// 3. Forwards to target application
/// 4. Returns target's response to client
pub async fn intercept_request(
    State(state): State<ProxyState>,
    req: Request,
) -> Result<Response, AppError> {

    // Extract request components (clone before consuming req)
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let query = req.uri().query().unwrap_or("").to_string();

    // Build full path with query string
    let path_and_query = if query.is_empty() {
        path.clone()
    } else {
        format!("{}?{}", path, query)
    };

    // Build target URL: base_url + path + query
    let target_url = format!("{}{}", state.target_url, path_and_query);

    info!(
        method = %method,
        path = %path_and_query,
        target = %target_url,
        "Proxying request"
    );

    // Extract headers and body
    let (parts, body) = req.into_parts();
    let headers = parts.headers;

    // Capture request headers as HashMap for logging
    let request_headers_map = headers_to_map(&headers);

    // Extract client IP from headers (X-Forwarded-For, X-Real-IP) or fallback
    let client_ip = extract_client_ip(&headers);

    // Read body bytes for logging (with 100KB limit)
    let body_bytes = axum::body::to_bytes(body, MAX_BODY_SIZE)
        .await
        .map_err(|e| AppError::ProxyError(format!("Failed to read request body: {}", e)))?;

    // Capture request body as string for logging (with truncation marker if needed)
    let request_body_str = capture_body_string(&body_bytes, MAX_BODY_SIZE);

    // Recreate body for forwarding
    let forward_body = Body::from(body_bytes.to_vec());

    // Forward the request to target application
    let result = forward_request(
        &state.client,
        target_url.clone(),
        method.clone(),
        headers,
        forward_body,
    ).await;

    

    match result {
        Ok((status, response_headers, response_body)) => {

            // Convert HeaderMap to HashMap<String, String> for response headers
            let response_headers_map = headers_to_map(&response_headers);

            // Create log entry for this request/response
            let log_entry = LogEntry {
                id: Some(Uuid::new_v4()),
                request_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                method: method.to_string(),
                path: path.to_string(),
                query_string: if query.is_empty() { None } else { Some(query.to_string()) },
                status_code: status.as_u16(),
                duration_ms: 0, // TODO: Will calculate in EP-001-09
                request_headers: request_headers_map,
                request_body: request_body_str.clone(),
                response_headers: response_headers_map,
                response_body: None, // TODO: Will capture in EP-001-09
                client_ip: client_ip.clone(),
            };

            // Log the structured data (for now, just log the basic info)
            info!(
                method = %method,
                path = %path_and_query,
                status = %status,
                request_id = %log_entry.request_id,
                "Request completed successfully"
            );

            // Build response with target's status, headers, and body
            let mut response = Response::new(response_body);
            *response.status_mut() = status;
            *response.headers_mut() = response_headers;

            Ok(response)
        }
        Err(e) => {
            error!(
                method = %method,
                path = %path_and_query,
                target = %target_url,
                error = %e,
                "Request failed"
            );
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    // ==================== extract_client_ip tests ====================

    #[test]
    fn test_extract_client_ip_from_x_forwarded_for() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("192.168.1.100"));

        assert_eq!(extract_client_ip(&headers), "192.168.1.100");
    }

    #[test]
    fn test_extract_client_ip_from_x_forwarded_for_multiple_ips() {
        let mut headers = HeaderMap::new();
        // X-Forwarded-For can contain multiple IPs, first one is the client
        headers.insert("x-forwarded-for", HeaderValue::from_static("192.168.1.100, 10.0.0.1, 172.16.0.1"));

        assert_eq!(extract_client_ip(&headers), "192.168.1.100");
    }

    #[test]
    fn test_extract_client_ip_from_x_forwarded_for_with_spaces() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("  192.168.1.100  , 10.0.0.1"));

        assert_eq!(extract_client_ip(&headers), "192.168.1.100");
    }

    #[test]
    fn test_extract_client_ip_from_x_real_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", HeaderValue::from_static("10.0.0.50"));

        assert_eq!(extract_client_ip(&headers), "10.0.0.50");
    }

    #[test]
    fn test_extract_client_ip_prefers_x_forwarded_for_over_x_real_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("192.168.1.100"));
        headers.insert("x-real-ip", HeaderValue::from_static("10.0.0.50"));

        // X-Forwarded-For should take priority
        assert_eq!(extract_client_ip(&headers), "192.168.1.100");
    }

    #[test]
    fn test_extract_client_ip_returns_unknown_when_no_headers() {
        let headers = HeaderMap::new();

        assert_eq!(extract_client_ip(&headers), "unknown");
    }

    // ==================== headers_to_map tests ====================

    #[test]
    fn test_headers_to_map_empty() {
        let headers = HeaderMap::new();
        let map = headers_to_map(&headers);

        assert!(map.is_empty());
    }

    #[test]
    fn test_headers_to_map_single_header() {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/json"));

        let map = headers_to_map(&headers);

        assert_eq!(map.len(), 1);
        assert_eq!(map.get("content-type"), Some(&"application/json".to_string()));
    }

    #[test]
    fn test_headers_to_map_multiple_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        headers.insert("authorization", HeaderValue::from_static("Bearer token123"));
        headers.insert("accept", HeaderValue::from_static("*/*"));

        let map = headers_to_map(&headers);

        assert_eq!(map.len(), 3);
        assert_eq!(map.get("content-type"), Some(&"application/json".to_string()));
        assert_eq!(map.get("authorization"), Some(&"Bearer token123".to_string()));
        assert_eq!(map.get("accept"), Some(&"*/*".to_string()));
    }

    // ==================== capture_body_string tests ====================

    #[test]
    fn test_capture_body_string_empty_body() {
        let body_bytes: &[u8] = &[];

        assert_eq!(capture_body_string(body_bytes, 100), None);
    }

    #[test]
    fn test_capture_body_string_small_body() {
        let body_bytes = b"Hello, World!";

        let result = capture_body_string(body_bytes, 100);

        assert_eq!(result, Some("Hello, World!".to_string()));
    }

    #[test]
    fn test_capture_body_string_json_body() {
        let body_bytes = br#"{"name": "test", "value": 123}"#;

        let result = capture_body_string(body_bytes, 100);

        assert_eq!(result, Some(r#"{"name": "test", "value": 123}"#.to_string()));
    }

    #[test]
    fn test_capture_body_string_truncates_at_limit() {
        let body_bytes = b"0123456789"; // 10 bytes

        let result = capture_body_string(body_bytes, 10);

        // Should have truncation marker
        assert!(result.is_some());
        let body = result.unwrap();
        assert!(body.ends_with("... [TRUNCATED]"));
    }

    #[test]
    fn test_capture_body_string_no_truncation_under_limit() {
        let body_bytes = b"0123456789"; // 10 bytes

        let result = capture_body_string(body_bytes, 11); // limit is 11

        assert_eq!(result, Some("0123456789".to_string()));
    }

    #[test]
    fn test_capture_body_string_handles_utf8() {
        let body_bytes = "Hello, 世界! 🚀".as_bytes();

        let result = capture_body_string(body_bytes, 100);

        assert_eq!(result, Some("Hello, 世界! 🚀".to_string()));
    }

    #[test]
    fn test_capture_body_string_handles_invalid_utf8() {
        // Invalid UTF-8 sequence
        let body_bytes: &[u8] = &[0x80, 0x81, 0x82];

        let result = capture_body_string(body_bytes, 100);

        // Should use replacement characters for invalid UTF-8
        assert!(result.is_some());
        assert!(result.unwrap().contains('\u{FFFD}')); // Unicode replacement character
    }

    // ==================== MAX_BODY_SIZE constant test ====================

    #[test]
    fn test_max_body_size_is_100kb() {
        assert_eq!(MAX_BODY_SIZE, 100_000);
    }
}
