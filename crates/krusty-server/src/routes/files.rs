//! File operations endpoints

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use tokio::fs;

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::types::{
    BrowseEntry, BrowseQuery, BrowseResponse, FileQuery, FileResponse, FileWriteRequest,
    FileWriteResponse, TreeEntry, TreeQuery, TreeResponse,
};
use crate::AppState;

/// Build the files router
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(read_file).put(write_file))
        .route("/tree", get(get_tree))
        .route("/browse", get(browse_directories))
}

/// Read a file's contents
async fn read_file(
    State(state): State<AppState>,
    user: Option<CurrentUser>,
    Query(query): Query<FileQuery>,
) -> Result<Json<FileResponse>, AppError> {
    let workspace = workspace_base(&state, user.as_ref());
    let path = resolve_path(&workspace, &query.path);

    // Security: ensure path is within allowed root
    let allowed_root = allowed_root(user.as_ref());
    validate_path_within(&allowed_root, &path)?;

    let metadata = fs::metadata(&path).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AppError::NotFound(format!("File not found: {}", query.path))
        } else {
            AppError::Internal(format!("Failed to read file metadata: {}", e))
        }
    })?;

    if metadata.is_dir() {
        return Err(AppError::BadRequest(format!(
            "Path is a directory: {}",
            query.path
        )));
    }

    let content = fs::read_to_string(&path)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read file: {}", e)))?;

    Ok(Json(FileResponse {
        path: query.path,
        content,
        size: metadata.len(),
    }))
}

/// Write content to a file
async fn write_file(
    State(state): State<AppState>,
    user: Option<CurrentUser>,
    Query(query): Query<FileQuery>,
    Json(req): Json<FileWriteRequest>,
) -> Result<Json<FileWriteResponse>, AppError> {
    const MAX_FILE_WRITE_SIZE: usize = 100 * 1024 * 1024; // 100MB
    if req.content.len() > MAX_FILE_WRITE_SIZE {
        return Err(AppError::BadRequest(
            "File content exceeds maximum size of 100MB".to_string(),
        ));
    }

    let workspace = workspace_base(&state, user.as_ref());
    let path = resolve_path(&workspace, &query.path);

    // Security: ensure path is within allowed root
    let allowed_root = allowed_root(user.as_ref());
    validate_path_within(&allowed_root, &path)?;

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create directories: {}", e)))?;
    }

    let bytes = req.content.as_bytes();
    fs::write(&path, bytes)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to write file: {}", e)))?;

    Ok(Json(FileWriteResponse {
        path: query.path,
        bytes_written: bytes.len(),
    }))
}

/// Get directory tree
async fn get_tree(
    State(state): State<AppState>,
    user: Option<CurrentUser>,
    Query(query): Query<TreeQuery>,
) -> Result<Json<TreeResponse>, AppError> {
    let workspace = workspace_base(&state, user.as_ref());
    let root_path = match &query.root {
        Some(root) => resolve_path(&workspace, root),
        None => workspace.clone(),
    };

    // Security: ensure path is within allowed root
    let allowed_root = allowed_root(user.as_ref());
    validate_path_within(&allowed_root, &root_path)?;

    if !root_path.is_dir() {
        return Err(AppError::BadRequest(format!(
            "Path is not a directory: {}",
            root_path.display()
        )));
    }

    let counter = Arc::new(AtomicUsize::new(0));
    let entries = build_tree(&root_path, query.depth, &counter).await?;

    Ok(Json(TreeResponse {
        root: root_path.display().to_string(),
        entries,
    }))
}

/// Derive the workspace base directory from user context (multi-tenant) or app state (single-tenant).
fn workspace_base(state: &AppState, user: Option<&CurrentUser>) -> PathBuf {
    user.and_then(|u| u.0.home_dir.clone())
        .unwrap_or_else(|| (*state.working_dir).clone())
}

/// Derive the allowed root for path validation.
///
/// In multi-tenant mode the user's workspace directory is the boundary;
/// in single-tenant mode we fall back to the system home directory.
fn allowed_root(user: Option<&CurrentUser>) -> PathBuf {
    user.and_then(|u| u.0.home_dir.clone())
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("/"))
}

/// Resolve a path relative to working directory
fn resolve_path(working_dir: &Path, path: &str) -> PathBuf {
    let path = Path::new(path);

    if path.is_absolute() {
        path.to_path_buf()
    } else {
        working_dir.join(path)
    }
}

fn validate_path_within(base: &Path, path: &Path) -> Result<(), AppError> {
    crate::utils::paths::validate_path_within(base, path)
}

const MAX_TREE_ENTRIES: usize = 10_000;

/// Recursively build directory tree
async fn build_tree(
    path: &Path,
    depth: usize,
    counter: &Arc<AtomicUsize>,
) -> Result<Vec<TreeEntry>, AppError> {
    if depth == 0 || counter.load(Ordering::Relaxed) >= MAX_TREE_ENTRIES {
        return Ok(vec![]);
    }

    let mut entries = Vec::new();
    let mut read_dir = fs::read_dir(path)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read directory: {}", e)))?;

    while let Some(entry) = read_dir
        .next_entry()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read directory entry: {}", e)))?
    {
        if counter.load(Ordering::Relaxed) >= MAX_TREE_ENTRIES {
            break;
        }

        let entry_path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files and common ignore patterns
        if name.starts_with('.') || name == "node_modules" || name == "target" {
            continue;
        }

        counter.fetch_add(1, Ordering::Relaxed);

        let is_dir = entry_path.is_dir();
        let children = if is_dir && depth > 1 {
            Some(Box::pin(build_tree(&entry_path, depth - 1, counter)).await?)
        } else {
            None
        };

        entries.push(TreeEntry {
            name,
            path: entry_path.display().to_string(),
            is_dir,
            children,
        });
    }

    // Sort: directories first, then alphabetically
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    Ok(entries)
}

/// Browse directories for project selection (not restricted to working dir)
async fn browse_directories(
    user: Option<CurrentUser>,
    Query(query): Query<BrowseQuery>,
) -> Result<Json<BrowseResponse>, AppError> {
    // In multi-tenant mode, scope to user's workspace; otherwise use home dir
    let home = user
        .as_ref()
        .and_then(|u| u.0.home_dir.clone())
        .or_else(dirs::home_dir)
        .ok_or_else(|| AppError::Internal("Could not determine home directory".to_string()))?;

    let current_path = match &query.path {
        Some(p) => PathBuf::from(p),
        None => home.clone(),
    };

    // Security: must be within allowed root
    let canonical_home = home
        .canonicalize()
        .map_err(|e| AppError::Internal(format!("Failed to canonicalize home: {}", e)))?;

    let canonical_current = current_path.canonicalize().map_err(|_| {
        AppError::NotFound(format!("Directory not found: {}", current_path.display()))
    })?;

    if !canonical_current.starts_with(&canonical_home) {
        return Err(AppError::BadRequest(
            "Path must be within home directory".to_string(),
        ));
    }

    if !canonical_current.is_dir() {
        return Err(AppError::BadRequest(format!(
            "Path is not a directory: {}",
            current_path.display()
        )));
    }

    // Get parent (if not at home)
    let parent = if canonical_current != canonical_home {
        canonical_current.parent().map(|p| p.display().to_string())
    } else {
        None
    };

    // List directories only
    let mut directories = Vec::new();
    let mut read_dir = fs::read_dir(&canonical_current)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read directory: {}", e)))?;

    while let Some(entry) = read_dir
        .next_entry()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read entry: {}", e)))?
    {
        let path = entry.path();

        // Only directories
        if !path.is_dir() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().into_owned();

        // Skip hidden directories
        if name.starts_with('.') {
            continue;
        }

        directories.push(BrowseEntry {
            name,
            path: path.display().to_string(),
        });
    }

    // Sort alphabetically
    directories.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(Json(BrowseResponse {
        current: canonical_current.display().to_string(),
        parent,
        directories,
    }))
}
