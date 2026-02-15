//! Krusty - The most elegant coding CLI to ever exist
//!
//! A terminal-based AI coding assistant with:
//! - Multi-provider AI with API key authentication
//! - Single-mode Chat UI with slash commands
//! - `krusty serve` — unified server + PWA + Tailscale
//! - Clean architecture from day one

use anyhow::Result;
use clap::{Parser, Subcommand};

// Re-export core modules for TUI usage
use krusty_core::{acp, agent, ai, constants, extensions, paths, plan, process, storage, tools};

mod serve;
mod tui;

/// Krusty - AI Coding Assistant
#[derive(Parser)]
#[command(name = "krusty")]
#[command(about = "The most elegant coding CLI to ever exist", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run as ACP (Agent Client Protocol) server
    ///
    /// Krusty runs as an ACP-compatible agent that communicates
    /// via JSON-RPC over stdin/stdout. This mode is used when Krusty is
    /// spawned by an ACP-compatible editor (Zed, Neovim, etc.).
    ///
    /// Uses credentials from TUI configuration, or override with env vars:
    /// - KRUSTY_PROVIDER + KRUSTY_API_KEY (+ optional KRUSTY_MODEL)
    /// - Or provider-specific: ANTHROPIC_API_KEY, OPENROUTER_API_KEY, etc.
    Acp,

    /// Start the Krusty web server with embedded PWA frontend
    ///
    /// Launches the API server with the PWA bundled into the binary.
    /// On first run, prompts for provider and API key configuration.
    /// Automatically configures Tailscale for remote access if available.
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },
}

/// Restore terminal state - called on panic or unexpected exit
fn restore_terminal() {
    use crossterm::{
        event::DisableMouseCapture,
        execute,
        terminal::{disable_raw_mode, LeaveAlternateScreen},
    };
    let _ = disable_raw_mode();
    let _ = execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Serve mode has its own logging (stdout), skip TUI logging setup
    if matches!(cli.command, Some(Commands::Serve { .. })) {
        if let Some(Commands::Serve { port }) = cli.command {
            return serve::run(port).await;
        }
    }

    // Set up panic hook to restore terminal state (TUI/ACP modes)
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        restore_terminal();
        original_hook(panic_info);
    }));

    // Initialize logging to file (not stdout/stderr which would mess up TUI)
    let log_dir = paths::logs_dir();
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("Failed to create log directory: {}", e);
    }

    #[cfg(unix)]
    let null_device = "/dev/null";
    #[cfg(windows)]
    let null_device = "NUL";

    let log_file = match std::fs::File::create(log_dir.join("krusty.log")) {
        Ok(file) => file,
        Err(e) => {
            eprintln!(
                "Failed to create log file: {}, falling back to null device",
                e
            );
            match std::fs::File::create(null_device) {
                Ok(file) => file,
                Err(e) => {
                    eprintln!(
                        "Failed to create null device {}: {}, logging disabled",
                        null_device, e
                    );
                    return Err(e.into());
                }
            }
        }
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(std::sync::Mutex::new(log_file))
        .with_ansi(false)
        .init();

    // Apply any pending update before starting TUI
    if let Ok(Some(version)) = krusty_core::updater::apply_pending_update() {
        tracing::info!("Applied pending update to v{}", version);
    }

    match cli.command {
        Some(Commands::Acp) => {
            tracing::info!("Starting Krusty in ACP server mode");
            let server = acp::AcpServer::new()?;
            server.run().await?;
        }
        Some(Commands::Serve { .. }) => unreachable!(),
        None => {
            let mut app = tui::App::new().await;
            app.run().await?;
        }
    }

    Ok(())
}
