//! Auto-updater module for Krusty
//!
//! Checks for updates from git, builds in background, and prepares for restart.

mod checker;

pub use checker::{build_update, check_for_updates, detect_repo_path, UpdateInfo, UpdateStatus};
