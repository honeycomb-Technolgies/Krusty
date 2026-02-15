//! User hooks management endpoints

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, patch},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use krusty_core::agent::{UserHook, UserHookType};
use krusty_core::storage::Database;

use crate::error::AppError;
use crate::AppState;

/// Build the hooks router
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_hooks).post(create_hook))
        .route("/:id", delete(delete_hook))
        .route("/:id/toggle", patch(toggle_hook))
}

/// Hook info for API response
#[derive(Serialize)]
pub struct HookResponse {
    pub id: String,
    pub hook_type: String,
    pub tool_pattern: String,
    pub command: String,
    pub enabled: bool,
    pub created_at: String,
}

impl From<&UserHook> for HookResponse {
    fn from(hook: &UserHook) -> Self {
        Self {
            id: hook.id.clone(),
            hook_type: hook.hook_type.display_name().to_string(),
            tool_pattern: hook.tool_pattern.clone(),
            command: hook.command.clone(),
            enabled: hook.enabled,
            created_at: hook.created_at.clone(),
        }
    }
}

/// Request to create a new hook
#[derive(Deserialize)]
pub struct CreateHookRequest {
    pub hook_type: String,
    pub tool_pattern: String,
    pub command: String,
}

/// List all user hooks
async fn list_hooks(State(state): State<AppState>) -> Result<Json<Vec<HookResponse>>, AppError> {
    let manager = state.hook_manager.read().await;
    let hooks: Vec<HookResponse> = manager.hooks().iter().map(HookResponse::from).collect();
    Ok(Json(hooks))
}

/// Create a new hook
async fn create_hook(
    State(state): State<AppState>,
    Json(req): Json<CreateHookRequest>,
) -> Result<(StatusCode, Json<HookResponse>), AppError> {
    // Parse hook type
    let hook_type = UserHookType::parse(&req.hook_type)
        .ok_or_else(|| AppError::BadRequest(format!("Invalid hook type: {}", req.hook_type)))?;

    // Validate regex pattern
    if regex::Regex::new(&req.tool_pattern).is_err() {
        return Err(AppError::BadRequest(format!(
            "Invalid regex pattern: {}",
            req.tool_pattern
        )));
    }

    // Validate command not empty
    if req.command.trim().is_empty() {
        return Err(AppError::BadRequest("Command cannot be empty".to_string()));
    }

    // Create the hook
    let hook = UserHook::new(hook_type, req.tool_pattern, req.command);
    let response = HookResponse::from(&hook);

    // Save to database
    let db = Database::new(&state.db_path)?;
    let mut manager = state.hook_manager.write().await;
    manager
        .save(&db, hook)
        .map_err(|e| AppError::Internal(format!("Failed to save hook: {}", e)))?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// Toggle a hook's enabled state
async fn toggle_hook(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<HookResponse>, AppError> {
    let db = Database::new(&state.db_path)?;
    let mut manager = state.hook_manager.write().await;

    manager
        .toggle(&db, &id)
        .map_err(|e| AppError::Internal(format!("Failed to toggle hook: {}", e)))?;

    // Find the updated hook
    let hook = manager
        .hooks()
        .iter()
        .find(|h| h.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Hook {} not found", id)))?;

    Ok(Json(HookResponse::from(hook)))
}

/// Delete a hook
async fn delete_hook(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let db = Database::new(&state.db_path)?;
    let mut manager = state.hook_manager.write().await;

    manager
        .delete(&db, &id)
        .map_err(|e| AppError::Internal(format!("Failed to delete hook: {}", e)))?;

    Ok(StatusCode::NO_CONTENT)
}
