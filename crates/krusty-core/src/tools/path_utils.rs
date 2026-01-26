//! Shared path validation utilities for tool implementations

use std::path::{Path, PathBuf};

use crate::tools::registry::ToolResult;

/// Resolve and validate a path against the sandbox root
/// Returns the canonicalized path if valid, or a ToolResult error
pub fn validate_path(
    path: &str,
    sandbox_root: Option<&Path>,
    working_dir: &Path,
) -> Result<PathBuf, ToolResult> {
    let resolved = if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        working_dir.join(path)
    };

    let canonical = resolved
        .canonicalize()
        .map_err(|e| ToolResult::error(format!("Cannot resolve path '{}': {}", path, e)))?;

    if let Some(sandbox) = sandbox_root {
        if !canonical.starts_with(sandbox) {
            return Err(ToolResult::error(format!(
                "Access denied: path '{}' is outside workspace",
                path
            )));
        }
    }

    Ok(canonical)
}
