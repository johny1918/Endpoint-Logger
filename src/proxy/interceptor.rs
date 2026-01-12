use axum::extract::{Request, State};
use axum::response::Response;
use tracing::{info, error};
use crate::proxy::forwarder::forward_request;
use crate::proxy::ProxyState;
use crate::utils::errors::AppError;

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

    // Extract request components
    let method = req.method().clone();
    let uri = req.uri();
    let path = uri.path();
    let query = uri.query().unwrap_or("");

    // Build full path with query string
    let path_and_query = if query.is_empty() {
        path.to_string()
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

    // Forward the request to target application
    let result = forward_request(
        &state.client,
        target_url.clone(),
        method.clone(),
        headers,
        body,
    ).await;

    match result {
        Ok((status, response_headers, response_body)) => {
            info!(
                method = %method,
                path = %path_and_query,
                status = %status,
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

