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
}