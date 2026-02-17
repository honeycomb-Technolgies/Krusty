//! Lightweight auth middleware for self-host deployments.
//!
//! This keeps request-level user context optional:
//! - No auth headers => single-tenant local mode.
//! - `X-User-Id` + optional `X-Workspace-Dir` => scoped multi-user mode.

use axum::{
    async_trait,
    extract::{ConnectInfo, FromRequestParts, Request, State},
    http::{request::Parts, StatusCode},
    middleware::Next,
    response::Response,
};
use std::net::SocketAddr;
use std::path::PathBuf;

use crate::AppState;

/// User context attached to request extensions by middleware.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Option<String>,
    pub home_dir: Option<PathBuf>,
}

impl AuthenticatedUser {
    pub fn local() -> Self {
        Self {
            user_id: None,
            home_dir: None,
        }
    }
}

/// Extractor for routes that want user context.
pub struct CurrentUser(pub AuthenticatedUser);

#[async_trait]
impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthenticatedUser>()
            .cloned()
            .map(CurrentUser)
            .ok_or((StatusCode::UNAUTHORIZED, "Not authenticated"))
    }
}

/// Middleware that attaches optional user info to request extensions.
pub async fn auth_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    mut request: Request,
    next: Next,
) -> Response {
    let mut user = AuthenticatedUser::local();

    if let Some(user_id) = request
        .headers()
        .get("X-User-Id")
        .and_then(|v| v.to_str().ok())
        .filter(|v| !v.trim().is_empty())
    {
        user.user_id = Some(user_id.to_string());
        user.home_dir = request
            .headers()
            .get("X-Workspace-Dir")
            .and_then(|v| v.to_str().ok())
            .map(PathBuf::from)
            .or_else(|| Some((*state.working_dir).clone()));
    }

    if !addr.ip().is_loopback() {
        tracing::debug!(
            "External request from {} accepted in self-host mode",
            addr.ip()
        );
    }

    request.extensions_mut().insert(user);
    next.run(request).await
}
