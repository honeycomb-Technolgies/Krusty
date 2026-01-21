//! ACP Server - Main entry point for ACP mode
//!
//! Handles the stdio transport and message routing for ACP protocol.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use agent_client_protocol::{AgentSideConnection, Client};
use anyhow::Result;
use tokio::io::{stdin, stdout};
use tokio::sync::mpsc;
use tokio::task::LocalSet;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use tracing::{error, info, warn};

use super::agent::KrustyAgent;
use crate::tools::ToolRegistry;

/// ACP Server configuration
#[derive(Debug, Clone, Default)]
pub struct AcpServerConfig {
    /// Working directory override
    pub working_dir: Option<std::path::PathBuf>,
}

/// ACP Server that runs Krusty as an ACP-compatible agent
pub struct AcpServer {
    agent: Arc<KrustyAgent>,
    #[allow(dead_code)]
    config: AcpServerConfig,
}

impl AcpServer {
    /// Create a new ACP server with default configuration
    pub fn new() -> Result<Self> {
        Ok(Self {
            agent: Arc::new(KrustyAgent::new()),
            config: AcpServerConfig::default(),
        })
    }

    /// Create with custom tool registry
    pub fn with_tools(tools: Arc<ToolRegistry>) -> Result<Self> {
        Ok(Self {
            agent: Arc::new(KrustyAgent::with_tools(tools)),
            config: AcpServerConfig::default(),
        })
    }

    /// Create with configuration
    pub fn with_config(config: AcpServerConfig) -> Result<Self> {
        Ok(Self {
            agent: Arc::new(KrustyAgent::new()),
            config,
        })
    }

    /// Run the ACP server (blocks until connection closes)
    ///
    /// This method takes over stdin/stdout for ACP communication.
    /// All logging should go to stderr.
    pub async fn run(self) -> Result<()> {
        info!("Starting Krusty ACP server");

        // Create the agent-side connection
        // Note: ACP connections are not Send, so we need LocalSet
        let local = LocalSet::new();

        local
            .run_until(async move {
                // Create notification channel
                let (tx, mut rx) = mpsc::unbounded_channel();

                // Give the sender to the agent
                self.agent.set_notification_channel(tx).await;

                // Get stdin/stdout for transport, wrapped for futures compatibility
                let stdin = stdin().compat();
                let stdout = stdout().compat_write();

                // Spawn function for the connection
                let spawn_fn = |fut: Pin<Box<dyn Future<Output = ()>>>| {
                    tokio::task::spawn_local(fut);
                };

                // Create connection with our agent
                let (connection, io_task) = AgentSideConnection::new(
                    self.agent,
                    stdout,
                    stdin,
                    spawn_fn,
                );

                info!("ACP connection established, waiting for requests...");

                // Spawn task to forward notifications to the connection
                tokio::task::spawn_local(async move {
                    while let Some(notification) = rx.recv().await {
                        if let Err(e) = connection.session_notification(notification).await {
                            warn!("Failed to forward notification: {}", e);
                        }
                    }
                });

                // Run the IO task
                if let Err(e) = io_task.await {
                    error!("ACP connection error: {}", e);
                    return Err(anyhow::anyhow!("ACP connection error: {}", e));
                }

                info!("ACP connection closed");
                Ok(())
            })
            .await
    }

    /// Get a reference to the agent
    pub fn agent(&self) -> &KrustyAgent {
        &self.agent
    }
}

impl Default for AcpServer {
    fn default() -> Self {
        Self::new().expect("Failed to create default ACP server")
    }
}

/// Check if we should run in ACP mode
///
/// Returns true if stdin is not a TTY (likely being spawned by an editor)
/// and the `--acp` flag is present.
#[allow(dead_code)]
pub fn should_run_acp_mode(force_acp: bool) -> bool {
    if force_acp {
        return true;
    }

    // Auto-detect: if stdin is not a TTY, we might be in ACP mode
    // But only if explicitly requested or detected
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = AcpServer::new().unwrap();
        assert_eq!(server.agent().sessions().session_count(), 0);
    }

    #[test]
    fn test_acp_mode_detection() {
        assert!(should_run_acp_mode(true));
        assert!(!should_run_acp_mode(false));
    }
}
