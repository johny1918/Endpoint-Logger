use axum::{Json, extract::{Path, State}};

use crate::{models::proxy::LogEntry, proxy::ProxyState, utils::errors::AppError};

/// GET /api/logs/{request_id}
/// Returns a single log entry by request_id, or 404 if not found
pub async fn get_log_by_request_id(
    Path(request_id): Path<String>,
    State(state): State<ProxyState>,
) -> Result<Json<LogEntry>, AppError> {
    let log_entry = state.storage
        .get_by_request_id(&request_id)
        .await?;

    match log_entry {
        Some(entry) => Ok(Json(entry)),
        None => Err(AppError::NotFound),
    }
}