//! Process management endpoints

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;

use krusty_core::process::ProcessInfo;

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::AppState;

/// Build the processes router
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_processes))
        .route("/:id", get(get_process))
        .route("/:id/kill", post(kill_process))
        .route("/:id/suspend", post(suspend_process))
        .route("/:id/resume", post(resume_process))
}

/// Process info for API response
#[derive(Serialize)]
pub struct ProcessResponse {
    pub id: String,
    pub command: String,
    pub description: Option<String>,
    pub pid: Option<u32>,
    pub status: String,
    pub elapsed_secs: u64,
}

impl From<ProcessInfo> for ProcessResponse {
    fn from(p: ProcessInfo) -> Self {
        Self {
            id: p.id,
            command: p.command,
            description: p.description,
            pid: p.pid,
            status: format!("{:?}", p.status),
            elapsed_secs: p.started_at.elapsed().as_secs(),
        }
    }
}

/// List all background processes (user-scoped in multi-tenant mode)
async fn list_processes(
    State(state): State<AppState>,
    user: Option<CurrentUser>,
) -> Json<Vec<ProcessResponse>> {
    let processes: Vec<ProcessResponse> = match user.and_then(|u| u.0.user_id) {
        Some(user_id) => state.process_registry.list_for_user(&user_id).await,
        None => state.process_registry.list().await,
    }
    .into_iter()
    .map(Into::into)
    .collect();
    Json(processes)
}

/// Get a specific process (user-scoped in multi-tenant mode)
async fn get_process(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: Option<CurrentUser>,
) -> Result<Json<ProcessResponse>, AppError> {
    let process = match user.and_then(|u| u.0.user_id) {
        Some(user_id) => state.process_registry.get_for_user(&user_id, &id).await,
        None => state.process_registry.get(&id).await,
    }
    .ok_or_else(|| AppError::NotFound(format!("Process {} not found", id)))?;

    Ok(Json(process.into()))
}

/// Kill a process (user-scoped in multi-tenant mode)
async fn kill_process(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: Option<CurrentUser>,
) -> Result<StatusCode, AppError> {
    let result = match user.and_then(|u| u.0.user_id) {
        Some(user_id) => state.process_registry.kill_for_user(&user_id, &id).await,
        None => state.process_registry.kill(&id).await,
    };

    result.map_err(|e| AppError::BadRequest(e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

/// Suspend a process (user-scoped in multi-tenant mode)
async fn suspend_process(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: Option<CurrentUser>,
) -> Result<StatusCode, AppError> {
    let result = match user.and_then(|u| u.0.user_id) {
        Some(user_id) => state.process_registry.suspend_for_user(&user_id, &id).await,
        None => state.process_registry.suspend(&id).await,
    };

    result.map_err(|e| AppError::BadRequest(e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

/// Resume a suspended process (user-scoped in multi-tenant mode)
async fn resume_process(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: Option<CurrentUser>,
) -> Result<StatusCode, AppError> {
    let result = match user.and_then(|u| u.0.user_id) {
        Some(user_id) => state.process_registry.resume_for_user(&user_id, &id).await,
        None => state.process_registry.resume(&id).await,
    };

    result.map_err(|e| AppError::BadRequest(e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}
