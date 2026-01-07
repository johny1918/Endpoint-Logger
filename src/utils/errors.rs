use thiserror::Error;

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
}