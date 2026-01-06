use thiserror::Error;


#[derive(Error, Debug)]
pub enum AppError {
    #[error("Unknown Error: {0}")]
    UnknownError(String),
    #[error("Invalid Port Range")]
    InvalidPortRange,
    #[error("Invalid URL Format")]
    InvalidURLFormat,
}