use std::collections::HashMap;

use axum::body::Body;
use axum::extract::{Request, State};
use axum::response::Response;
use tracing::{info, error};
use uuid::Uuid;
use chrono::Utc;
use crate::proxy::forwarder::forward_request;
use crate::proxy::ProxyState;
use crate::utils::errors::AppError;
use crate::models::proxy::LogEntry;

const MAX_BODY_SIZE: usize = 100_000; // 100KB limit for body capture

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
    let request_headers_map: HashMap<String, String> = headers
        .iter()
        .filter_map(|(key, value)| {
            value.to_str().ok().map(|v| (key.to_string(), v.to_string()))
        })
        .collect();

    // Extract client IP from headers (X-Forwarded-For, X-Real-IP) or fallback
    let client_ip = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());

    // Read body bytes for logging (with 100KB limit)
    let body_bytes = axum::body::to_bytes(body, MAX_BODY_SIZE)
        .await
        .map_err(|e| AppError::ProxyError(format!("Failed to read request body: {}", e)))?;

    // Capture request body as string for logging (with truncation marker if needed)
    let request_body_str = if body_bytes.is_empty() {
        None
    } else {
        let body_str = String::from_utf8_lossy(&body_bytes).to_string();
        if body_bytes.len() >= MAX_BODY_SIZE {
            Some(format!("{}... [TRUNCATED]", body_str))
        } else {
            Some(body_str)
        }
    };

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
            let response_headers_map: HashMap<String, String> = response_headers
                .iter()
                .filter_map(|(key, value)| {
                    value.to_str().ok().map(|v| (key.to_string(), v.to_string()))
                })
                .collect();

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

