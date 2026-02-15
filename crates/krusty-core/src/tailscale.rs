//! Tailscale integration for network access
//!
//! Detects Tailscale, resolves device URLs, and manages `tailscale serve`
//! for exposing the Krusty server with automatic HTTPS on the tailnet.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct TailscaleInfo {
    pub dns_name: String,
    pub tailnet_name: String,
    pub online: bool,
}

#[derive(Deserialize)]
struct TailscaleStatus {
    #[serde(rename = "Self")]
    self_node: Option<SelfNode>,
    #[serde(rename = "CurrentTailnet")]
    current_tailnet: Option<TailnetInfo>,
}

#[derive(Deserialize)]
struct SelfNode {
    #[serde(rename = "DNSName")]
    dns_name: String,
    #[serde(rename = "Online")]
    online: bool,
}

#[derive(Deserialize)]
struct TailnetInfo {
    #[serde(rename = "Name")]
    name: String,
}

/// Check if the `tailscale` CLI is available on PATH.
pub fn is_installed() -> bool {
    which::which("tailscale").is_ok()
}

/// Get current Tailscale device info by running `tailscale status --json`.
pub fn device_info() -> Result<TailscaleInfo> {
    let output = Command::new("tailscale")
        .args(["status", "--json"])
        .output()
        .context("Failed to run `tailscale status --json`")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("tailscale status failed: {}", stderr.trim());
    }

    let status: TailscaleStatus =
        serde_json::from_slice(&output.stdout).context("Failed to parse tailscale status JSON")?;

    let self_node = status
        .self_node
        .context("No self node in tailscale status")?;

    let tailnet = status
        .current_tailnet
        .context("No tailnet info in tailscale status")?;

    // DNSName comes with trailing dot, strip it
    let dns_name = self_node.dns_name.trim_end_matches('.').to_string();

    Ok(TailscaleInfo {
        dns_name,
        tailnet_name: tailnet.name,
        online: self_node.online,
    })
}

/// Get the HTTPS URL for this device on the tailnet.
pub fn device_url(_port: u16) -> Result<String> {
    let info = device_info()?;
    if !info.online {
        anyhow::bail!("Tailscale is installed but this device is offline");
    }
    // When using `tailscale serve`, Tailscale handles TLS on port 443
    // so the URL is just https://<dns-name> without a port
    Ok(format!("https://{}", info.dns_name))
}

/// Run `tailscale serve --bg <port>` to expose a local port on the tailnet with HTTPS.
///
/// This sets up a reverse proxy: tailnet HTTPS traffic → localhost:<port>.
/// The `--bg` flag runs it in the background persistently.
pub fn serve(port: u16) -> Result<()> {
    let output = Command::new("tailscale")
        .args(["serve", "--bg", &format!("{}", port)])
        .output()
        .context("Failed to run `tailscale serve`")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("tailscale serve failed: {}", stderr.trim());
    }

    tracing::info!("Tailscale serve configured: HTTPS → localhost:{}", port);
    Ok(())
}

/// Stop `tailscale serve` for the given port.
pub fn serve_stop(port: u16) -> Result<()> {
    let output = Command::new("tailscale")
        .args(["serve", "--remove", &format!("{}", port)])
        .output()
        .context("Failed to run `tailscale serve --remove`")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::warn!("tailscale serve --remove failed: {}", stderr.trim());
    }

    Ok(())
}

/// Full Tailscale setup: detect, expose, return URL.
/// Returns None if Tailscale is not available (non-fatal).
pub fn setup_tailscale_serve(port: u16) -> Option<String> {
    if !is_installed() {
        return None;
    }

    match device_info() {
        Ok(info) if !info.online => {
            tracing::warn!("Tailscale installed but device is offline");
            None
        }
        Ok(_) => {
            if let Err(e) = serve(port) {
                tracing::warn!("Failed to setup tailscale serve: {}", e);
                // Still try to return the URL — serve might already be configured
            }
            device_url(port).ok()
        }
        Err(e) => {
            tracing::warn!("Tailscale detection failed: {}", e);
            None
        }
    }
}
