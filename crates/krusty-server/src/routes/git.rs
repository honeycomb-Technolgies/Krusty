//! Git status and branch/worktree endpoints.

use std::path::{Path, PathBuf};

use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::types::{
    GitBranchResponse, GitBranchesResponse, GitCheckoutRequest, GitQuery, GitStatusResponse,
    GitWorktreeResponse, GitWorktreesResponse,
};
use crate::AppState;

/// Build the git router.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/status", get(get_status))
        .route("/branches", get(list_branches))
        .route("/worktrees", get(list_worktrees))
        .route("/checkout", post(checkout_branch))
}

async fn get_status(
    State(state): State<AppState>,
    user: Option<CurrentUser>,
    Query(query): Query<GitQuery>,
) -> Result<Json<GitStatusResponse>, AppError> {
    let path = resolve_git_path(&state, user.as_ref(), query.path.as_deref())?;
    let status = krusty_core::git::status(&path).map_err(to_bad_request)?;

    if let Some(status) = status {
        return Ok(Json(to_status_response(status)));
    }

    Ok(Json(GitStatusResponse {
        in_repo: false,
        repo_root: None,
        branch: None,
        head: None,
        upstream: None,
        branch_files: 0,
        branch_additions: 0,
        branch_deletions: 0,
        pr_number: None,
        ahead: 0,
        behind: 0,
        staged: 0,
        modified: 0,
        untracked: 0,
        conflicted: 0,
        total_changes: 0,
    }))
}

async fn list_branches(
    State(state): State<AppState>,
    user: Option<CurrentUser>,
    Query(query): Query<GitQuery>,
) -> Result<Json<GitBranchesResponse>, AppError> {
    let path = resolve_git_path(&state, user.as_ref(), query.path.as_deref())?;
    let repo_root = krusty_core::git::resolve_repo_root(&path)
        .map_err(to_bad_request)?
        .ok_or_else(|| AppError::BadRequest("Path is not inside a git repository".to_string()))?;

    let branches = krusty_core::git::branches(&path)
        .map_err(to_bad_request)?
        .unwrap_or_default()
        .into_iter()
        .map(|b| GitBranchResponse {
            name: b.name,
            is_current: b.is_current,
            upstream: b.upstream,
        })
        .collect();

    Ok(Json(GitBranchesResponse {
        repo_root: repo_root.display().to_string(),
        branches,
    }))
}

async fn list_worktrees(
    State(state): State<AppState>,
    user: Option<CurrentUser>,
    Query(query): Query<GitQuery>,
) -> Result<Json<GitWorktreesResponse>, AppError> {
    let path = resolve_git_path(&state, user.as_ref(), query.path.as_deref())?;
    let repo_root = krusty_core::git::resolve_repo_root(&path)
        .map_err(to_bad_request)?
        .ok_or_else(|| AppError::BadRequest("Path is not inside a git repository".to_string()))?;

    let worktrees = krusty_core::git::worktrees(&path)
        .map_err(to_bad_request)?
        .unwrap_or_default()
        .into_iter()
        .map(|wt| GitWorktreeResponse {
            path: wt.path.display().to_string(),
            branch: wt.branch,
            head: wt.head,
            is_current: wt.is_current,
        })
        .collect();

    Ok(Json(GitWorktreesResponse {
        repo_root: repo_root.display().to_string(),
        worktrees,
    }))
}

async fn checkout_branch(
    State(state): State<AppState>,
    user: Option<CurrentUser>,
    Json(req): Json<GitCheckoutRequest>,
) -> Result<Json<GitStatusResponse>, AppError> {
    let path = resolve_git_path(&state, user.as_ref(), req.path.as_deref())?;

    krusty_core::git::checkout(&path, &req.branch, req.create, req.start_point.as_deref())
        .map_err(to_bad_request)?;

    let status = krusty_core::git::status(&path)
        .map_err(to_bad_request)?
        .ok_or_else(|| AppError::BadRequest("Path is not inside a git repository".to_string()))?;

    Ok(Json(to_status_response(status)))
}

fn to_bad_request(err: anyhow::Error) -> AppError {
    AppError::BadRequest(err.to_string())
}

fn resolve_git_path(
    state: &AppState,
    user: Option<&CurrentUser>,
    requested: Option<&str>,
) -> Result<PathBuf, AppError> {
    let workspace_base = user
        .and_then(|u| u.0.home_dir.clone())
        .unwrap_or_else(|| (*state.working_dir).clone());

    let path = match requested.map(str::trim).filter(|p| !p.is_empty()) {
        Some(raw) => {
            let candidate = PathBuf::from(raw);
            if candidate.is_absolute() {
                candidate
            } else {
                workspace_base.join(candidate)
            }
        }
        None => workspace_base.clone(),
    };

    let allowed_root = user
        .and_then(|u| u.0.home_dir.clone())
        .or_else(dirs::home_dir)
        .unwrap_or(workspace_base);
    validate_path_within(&allowed_root, &path)?;

    Ok(path)
}

fn validate_path_within(base: &Path, path: &Path) -> Result<(), AppError> {
    let canonical_base = base
        .canonicalize()
        .map_err(|e| AppError::BadRequest(format!("Invalid workspace root: {}", e)))?;

    let check_path = find_existing_ancestor(path).ok_or_else(|| {
        AppError::BadRequest(format!(
            "Path does not exist and has no existing ancestor: {}",
            path.display()
        ))
    })?;

    let canonical_check = check_path
        .canonicalize()
        .map_err(|e| AppError::BadRequest(format!("Invalid path: {}", e)))?;

    if !canonical_check.starts_with(&canonical_base) {
        return Err(AppError::BadRequest(
            "Path must be within allowed workspace".to_string(),
        ));
    }

    Ok(())
}

fn find_existing_ancestor(path: &Path) -> Option<PathBuf> {
    let mut current = path.to_path_buf();
    loop {
        if current.exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn to_status_response(status: krusty_core::git::GitStatusSummary) -> GitStatusResponse {
    let total_changes = status.total_changes();
    GitStatusResponse {
        in_repo: true,
        repo_root: Some(status.repo_root.display().to_string()),
        branch: status.branch,
        head: status.head,
        upstream: status.upstream,
        branch_files: status.branch_files,
        branch_additions: status.branch_additions,
        branch_deletions: status.branch_deletions,
        pr_number: status.pr_number,
        ahead: status.ahead,
        behind: status.behind,
        staged: status.staged,
        modified: status.modified,
        untracked: status.untracked,
        conflicted: status.conflicted,
        total_changes,
    }
}
