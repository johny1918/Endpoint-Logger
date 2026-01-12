use axum::body::Body;
use axum::http::{HeaderMap, Method, StatusCode};
use reqwest::Client;
use crate::utils::errors::AppError;

/// Forward the request to the target application
/// Preserves method, headers, body, and query parameters
/// Returns the target's response (status, headers, body)
pub async fn forward_request(
    client: &Client,
    target_url: String,
    method: Method,
    headers: HeaderMap,
    body: Body,
) -> Result<(StatusCode, HeaderMap, Body), AppError> {

    // Convert axum body to bytes
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|e| AppError::ProxyError(format!("Failed to read request body: {}", e)))?;

    // Convert Axum method to reqwest method
    let reqwest_method = match method {
        Method::GET => reqwest::Method::GET,
        Method::POST => reqwest::Method::POST,
        Method::PUT => reqwest::Method::PUT,
        Method::DELETE => reqwest::Method::DELETE,
        Method::PATCH => reqwest::Method::PATCH,
        Method::HEAD => reqwest::Method::HEAD,
        Method::OPTIONS => reqwest::Method::OPTIONS,
        _ => reqwest::Method::GET, // Fallback for other methods
    };

    // Build the request to the target
    let mut request_builder = client
        .request(reqwest_method, &target_url)
        .body(body_bytes.to_vec());

    // Copy headers from original request (skip host header)
    for (key, value) in headers.iter() {
        if key != "host" {
            if let Ok(val) = value.to_str() {
                request_builder = request_builder.header(key.as_str(), val);
            }
        }
    }

    // Send the request to the target
    let response = request_builder
        .send()
        .await
        .map_err(|e| {
            // Handle connection errors (target unreachable)
            if e.is_timeout() {
                AppError::GatewayTimeout
            } else if e.is_connect() {
                AppError::BadGateway(format!("Target unreachable: {}", e))
            } else {
                AppError::ProxyError(format!("Request failed: {}", e))
            }
        })?;

    // Extract response status
    let status = response.status();
    let axum_status = StatusCode::from_u16(status.as_u16())
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    // Extract response headers
    let mut response_headers = HeaderMap::new();
    for (key, value) in response.headers().iter() {
        if let Ok(header_name) = axum::http::HeaderName::from_bytes(key.as_str().as_bytes()) {
            if let Ok(header_value) = axum::http::HeaderValue::from_bytes(value.as_bytes()) {
                response_headers.insert(header_name, header_value);
            }
        }
    }

    // Extract response body
    let response_bytes = response
        .bytes()
        .await
        .map_err(|e| AppError::ProxyError(format!("Failed to read response body: {}", e)))?;

    let response_body = Body::from(response_bytes.to_vec());

    Ok((axum_status, response_headers, response_body))
}