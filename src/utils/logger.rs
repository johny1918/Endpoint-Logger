use crate::utils::errors::AppError;

pub fn init_tracing() -> anyhow::Result<(), AppError>{
    tracing_subscriber::fmt()
        .json()
        .flatten_event(true)
        .try_init().map_err(|_| AppError::LoggerInitFail)

}