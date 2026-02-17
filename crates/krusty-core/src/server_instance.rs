//! Server instance detection and management
//!
//! Tracks running Krusty server instances via a PID file at ~/.krusty/server.pid.
//! Enables the desktop app and CLI to detect and reuse an existing server.

use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::paths;

#[derive(Debug, Clone)]
pub struct ServerInstance {
    pub pid: u32,
    pub port: u16,
}

fn pid_file_path() -> PathBuf {
    paths::config_dir().join("server.pid")
}

/// Write PID file when server starts.
pub fn write_pid_file(port: u16) -> Result<()> {
    let path = pid_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = format!("{}:{}", std::process::id(), port);
    std::fs::write(&path, content).context("Failed to write server PID file")?;
    Ok(())
}

/// Remove PID file on shutdown.
pub fn remove_pid_file() {
    let _ = std::fs::remove_file(pid_file_path());
}

/// Read PID file and check if the process is still alive.
pub fn read_pid_file() -> Option<ServerInstance> {
    let path = pid_file_path();
    let content = std::fs::read_to_string(&path).ok()?;
    let parts: Vec<&str> = content.trim().split(':').collect();
    if parts.len() != 2 {
        return None;
    }

    let pid: u32 = parts[0].parse().ok()?;
    let port: u16 = parts[1].parse().ok()?;

    // Check if process is alive (Unix: kill -0)
    if !is_process_alive(pid) {
        // Stale PID file — clean it up
        let _ = std::fs::remove_file(&path);
        return None;
    }

    Some(ServerInstance { pid, port })
}

/// Check if a running server is healthy by probing /health.
pub async fn probe_health(port: u16) -> bool {
    let url = format!("http://127.0.0.1:{}/health", port);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build();

    let client = match client {
        Ok(c) => c,
        Err(_) => return false,
    };

    match client.get(&url).send().await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

/// Detect a running Krusty server instance.
/// Returns the instance info if a healthy server is found.
pub async fn detect_running_server() -> Option<ServerInstance> {
    let instance = read_pid_file()?;

    if probe_health(instance.port).await {
        Some(instance)
    } else {
        // Process alive but not responding — stale
        remove_pid_file();
        None
    }
}

#[cfg(unix)]
fn is_process_alive(pid: u32) -> bool {
    if pid > i32::MAX as u32 {
        return false;
    }
    // SAFETY: kill(pid, 0) with signal 0 only checks process existence
    // without sending a signal. The pid is guarded to fit in i32.
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

#[cfg(not(unix))]
fn is_process_alive(_pid: u32) -> bool {
    // On non-Unix, assume alive and let health check determine
    true
}
