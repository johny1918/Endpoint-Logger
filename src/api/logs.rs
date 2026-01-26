use axum::{Json, extract::{Path, Query, State}};
use serde::Deserialize;

use crate::{models::proxy::LogEntry, proxy::ProxyState, utils::errors::AppError};

/// Query parameters for get_recent_logs
#[derive(Deserialize)]
pub struct RecentLogsQuery {
    /// Maximum number of logs to return (default: 50)
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_limit() -> u32 {
    50
}

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

/// GET /api/logs?limit=N
/// Returns recent log entries, ordered by timestamp descending
/// Default limit is 50 if not specified
pub async fn get_recent_logs(
    Query(params): Query<RecentLogsQuery>,
    State(state): State<ProxyState>,
) -> Result<Json<Vec<LogEntry>>, AppError> {
    let logs = state.storage
        .query_recent(params.limit)
        .await?;

    Ok(Json(logs))
}