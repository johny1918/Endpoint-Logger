use crate::proxy::interceptor::intercept_request;
use crate::proxy::ProxyState;
use axum::Router;

/// Create proxy router with catch-all fallback
/// This catches ALL requests (any method, any path) and forwards them to the target
pub fn proxy_router(state: ProxyState) -> Router {
    Router::new()
        // Use fallback to catch ALL requests regardless of path or method
        .fallback(intercept_request)
        .with_state(state)
}