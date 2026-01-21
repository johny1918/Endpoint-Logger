use crate::proxy::interceptor::intercept_request;
use crate::proxy::ProxyState;
use crate::api::websocket::ws_handler;
use axum::Router;
use axum::routing::get;

/// Create the main application router
/// Routes are registered in priority order:
/// 1. /ws - WebSocket endpoint (must come before fallback)
/// 2. fallback - Proxy all other requests to target
pub fn proxy_router(state: ProxyState) -> Router {
    Router::new()
        // WebSocket endpoint - must be registered before fallback
        .route("/ws", get(ws_handler))
        // Fallback catches ALL other requests and proxies them
        .fallback(intercept_request)
        .with_state(state)
}
