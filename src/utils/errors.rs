use thiserror::Error;
use axum::response::IntoResponse;
use axum::response::Response;
use reqwest::StatusCode;

// Error types for future use in the application
#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Unknown Error: {0}")]
    UnknownError(String),
    #[error("Invalid Port Range")]
    InvalidPortRange,
    #[error("Invalid URL Format")]
    InvalidURLFormat,
    #[error("Config error: {0}")]
    ConfigMissing(String),
    #[error("Merge config error: {0}")]
    MergeEnvError(String),
    #[error("Fail to validate config file: {0}")]
    ValidateConfigError(String),
    #[error("Fail to read toml config: {0}")]
    ReadConfigTomlError(String),
    #[error("Fail to validate URL: {0}")]
    ValidateURLConfig(String),
    #[error("Fail to validate PORT: {0}")]
    ValidatePORTConfig(String),
    #[error("Fail to init the logger")]
    LoggerInitFail,
    #[error("Fail to read Cargo.toml")]
    CargoTomlError,
    #[error("Graceful shutdown error: {0}")]
    GracefulShutdownError(String),
    #[error("Bad request")]
    BadRequest,
    #[error("Error 404 Not found")]
    NotFound,
    #[error("Proxy error: {0}")]
    ProxyError(String),
    #[error("Bad gateway: {0}")]
    BadGateway(String),
    #[error("Gateway timeout")]
    GatewayTimeout,
    #[error("Failed to create HTTP client")]
    ProxyStateError,
    #[error("Database error: {0}")]
    DatabaseError(String),
}

/// Convert AppError to HTTP response
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            AppError::BadGateway(msg) => (StatusCode::BAD_GATEWAY, msg),
            AppError::GatewayTimeout => (StatusCode::GATEWAY_TIMEOUT, "Gateway timeout".to_string()),
            AppError::ProxyError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        (status, message).into_response()
    }
}