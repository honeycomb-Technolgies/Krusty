//! File operations endpoints

use std::path::Path;

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use tokio::fs;

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
    Query(query): Query<FileQuery>,
) -> Result<Json<FileResponse>, AppError> {
    let path = resolve_path(&state.working_dir, &query.path);

    // Security: ensure path is within home directory
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::Internal("Could not determine home directory".to_string()))?;
    validate_path_within(&home, &path)?;

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
    Query(query): Query<FileQuery>,
    Json(req): Json<FileWriteRequest>,
) -> Result<Json<FileWriteResponse>, AppError> {
    let path = resolve_path(&state.working_dir, &query.path);

    // Security: ensure path is within home directory
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::Internal("Could not determine home directory".to_string()))?;
    validate_path_within(&home, &path)?;

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
    Query(query): Query<TreeQuery>,
) -> Result<Json<TreeResponse>, AppError> {
    let root_path = match &query.root {
        Some(root) => resolve_path(&state.working_dir, root),
        None => state.working_dir.to_path_buf(),
    };

    // Security: ensure path is within home directory (not just working_dir)
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::Internal("Could not determine home directory".to_string()))?;
    validate_path_within(&home, &root_path)?;

    if !root_path.is_dir() {
        return Err(AppError::BadRequest(format!(
            "Path is not a directory: {}",
            root_path.display()
        )));
    }

    let entries = build_tree(&root_path, query.depth).await?;

    Ok(Json(TreeResponse {
        root: root_path.display().to_string(),
        entries,
    }))
}

/// Resolve a path relative to working directory
fn resolve_path(working_dir: &Path, path: &str) -> std::path::PathBuf {
    let path = Path::new(path);

    if path.is_absolute() {
        path.to_path_buf()
    } else {
        working_dir.join(path)
    }
}

/// Validate that path is within a base directory (prevent directory traversal)
fn validate_path_within(base: &Path, path: &Path) -> Result<(), AppError> {
    let canonical_base = base
        .canonicalize()
        .map_err(|e| AppError::Internal(format!("Failed to canonicalize base dir: {}", e)))?;

    // Find the first existing ancestor to canonicalize
    let check_path = find_existing_ancestor(path)
        .ok_or_else(|| AppError::BadRequest("Invalid path: no existing ancestor".to_string()))?;

    let canonical_check = check_path
        .canonicalize()
        .map_err(|e| AppError::Internal(format!("Failed to canonicalize path: {}", e)))?;

    if !canonical_check.starts_with(&canonical_base) {
        return Err(AppError::BadRequest(
            "Path must be within allowed directory".to_string(),
        ));
    }

    Ok(())
}

/// Walk up the path to find the first existing ancestor
fn find_existing_ancestor(path: &Path) -> Option<std::path::PathBuf> {
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

/// Recursively build directory tree
async fn build_tree(path: &Path, depth: usize) -> Result<Vec<TreeEntry>, AppError> {
    if depth == 0 {
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
        let entry_path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files and common ignore patterns
        if name.starts_with('.') || name == "node_modules" || name == "target" {
            continue;
        }

        let is_dir = entry_path.is_dir();
        let children = if is_dir && depth > 1 {
            Some(Box::pin(build_tree(&entry_path, depth - 1)).await?)
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
    Query(query): Query<BrowseQuery>,
) -> Result<Json<BrowseResponse>, AppError> {
    // Default to home directory
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::Internal("Could not determine home directory".to_string()))?;

    let current_path = match &query.path {
        Some(p) => std::path::PathBuf::from(p),
        None => home.clone(),
    };

    // Security: must be within home directory
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
