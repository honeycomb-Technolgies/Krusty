//! API routes

use axum::Router;

use crate::AppState;

mod chat;
mod credentials;
mod files;
mod hooks;
mod mcp;
mod models;
mod processes;
mod sessions;
mod tools;

/// Build the API router with all endpoints
pub fn api_router() -> Router<AppState> {
    Router::new()
        .nest("/sessions", sessions::router())
        .nest("/chat", chat::router())
        .nest("/models", models::router())
        .nest("/tools", tools::router())
        .nest("/files", files::router())
        .nest("/credentials", credentials::router())
        .nest("/mcp", mcp::router())
        .nest("/processes", processes::router())
        .nest("/hooks", hooks::router())
        .merge(Router::new())
}
