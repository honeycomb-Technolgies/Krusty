//! Tool execution endpoint

use std::path::PathBuf;

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;

use krusty_core::tools::registry::ToolContext;

use crate::error::AppError;
use crate::types::{ToolExecuteRequest, ToolExecuteResponse};
use crate::AppState;

/// Build the tools router
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_tools))
        .route("/execute", post(execute_tool))
}

/// Tool info for API response
#[derive(Serialize)]
pub struct ToolResponse {
    pub name: String,
    pub description: String,
}

/// List all available tools
async fn list_tools(State(state): State<AppState>) -> Json<Vec<ToolResponse>> {
    let tools = state.tool_registry.get_ai_tools().await;

    let response: Vec<ToolResponse> = tools
        .into_iter()
        .map(|t| ToolResponse {
            name: t.name,
            description: t.description,
        })
        .collect();

    Json(response)
}

/// Execute a tool
async fn execute_tool(
    State(state): State<AppState>,
    Json(req): Json<ToolExecuteRequest>,
) -> Result<Json<ToolExecuteResponse>, AppError> {
    // Determine working directory
    let working_dir = req
        .working_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| (*state.working_dir).clone());

    // Create tool context
    let ctx = ToolContext {
        working_dir,
        process_registry: Some(state.process_registry.clone()),
        plan_mode: false,
        ..Default::default()
    };

    // Execute tool
    let result = state
        .tool_registry
        .execute(&req.tool_name, req.params, &ctx)
        .await
        .ok_or_else(|| AppError::NotFound(format!("Tool '{}' not found", req.tool_name)))?;

    Ok(Json(ToolExecuteResponse {
        output: result.output,
        is_error: result.is_error,
    }))
}
