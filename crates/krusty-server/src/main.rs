//! Krusty Server
//!
//! Self-hosted API server for chat, tools, sessions, and local workspace access.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{http::Method, middleware, routing::get, Json, Router};
use serde::Serialize;
use tokio::sync::RwLock;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use krusty_core::agent::UserHookManager;
use krusty_core::ai::client::{AiClient, AiClientConfig};
use krusty_core::ai::models::{create_model_registry, ModelMetadata, SharedModelRegistry};
use krusty_core::ai::providers::{builtin_providers, get_provider, ProviderId};
use krusty_core::constants;
use krusty_core::mcp::McpManager;
use krusty_core::paths;
use krusty_core::process::ProcessRegistry;
use krusty_core::skills::SkillsManager;
use krusty_core::storage::credentials::CredentialStore;
use krusty_core::storage::Database;
use krusty_core::tools::implementations::register_all_tools;
use krusty_core::tools::registry::ToolRegistry;

mod auth;
mod error;
mod routes;
mod types;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    /// SQLite database path (opened per request).
    pub db_path: Arc<PathBuf>,
    /// Default working directory for file/tools APIs.
    pub working_dir: Arc<PathBuf>,
    /// Base AI client (None when no credentials configured).
    pub ai_client: Option<Arc<AiClient>>,
    /// Tool registry with built-in tools.
    pub tool_registry: Arc<ToolRegistry>,
    /// Process registry for background commands.
    pub process_registry: Arc<ProcessRegistry>,
    /// Model registry for available models.
    pub model_registry: SharedModelRegistry,
    /// API credential store.
    pub credential_store: Arc<RwLock<CredentialStore>>,
    /// MCP manager for local MCP servers.
    pub mcp_manager: Arc<McpManager>,
    /// User hooks manager.
    pub hook_manager: Arc<RwLock<UserHookManager>>,
    /// Skills manager.
    pub skills_manager: Arc<RwLock<SkillsManager>>,
}

/// Parse provider from environment value.
fn parse_provider(s: &str) -> Option<ProviderId> {
    match s.to_ascii_lowercase().as_str() {
        "minimax" => Some(ProviderId::MiniMax),
        "openrouter" => Some(ProviderId::OpenRouter),
        "z_ai" | "zai" => Some(ProviderId::ZAi),
        "openai" => Some(ProviderId::OpenAI),
        _ => None,
    }
}

/// Build an AI client from configured credentials and env overrides.
fn create_ai_client(credentials: &CredentialStore) -> Option<AiClient> {
    let provider = std::env::var("KRUSTY_PROVIDER")
        .ok()
        .as_deref()
        .and_then(parse_provider)
        .unwrap_or(ProviderId::MiniMax);

    let provider_cfg = get_provider(provider)?;
    let model =
        std::env::var("KRUSTY_MODEL").unwrap_or_else(|_| provider_cfg.default_model().to_string());

    let auth = credentials.get_auth(&provider).or_else(|| {
        let env_key = match provider {
            ProviderId::MiniMax => "MINIMAX_API_KEY",
            ProviderId::OpenRouter => "OPENROUTER_API_KEY",
            ProviderId::ZAi => "Z_AI_API_KEY",
            ProviderId::OpenAI => "OPENAI_API_KEY",
        };
        std::env::var(env_key).ok()
    });

    let api_key = match auth {
        Some(key) => key,
        None => {
            tracing::warn!(
                "No credentials found for provider {}; chat API will be unavailable until credentials are configured",
                provider
            );
            return None;
        }
    };

    let config = if provider == ProviderId::OpenAI {
        AiClientConfig::for_openai_with_auth_detection(&model, credentials)
    } else {
        AiClientConfig {
            model,
            max_tokens: constants::ai::MAX_OUTPUT_TOKENS,
            base_url: Some(provider_cfg.base_url.clone()),
            auth_header: provider_cfg.auth_header,
            provider_id: provider,
            api_format: Default::default(),
            custom_headers: provider_cfg.custom_headers.clone(),
        }
    };

    Some(AiClient::new(config, api_key))
}

/// Initialize models in the shared registry.
async fn initialize_models(registry: &SharedModelRegistry, credentials: &CredentialStore) {
    for provider in builtin_providers() {
        let models: Vec<ModelMetadata> = provider
            .models
            .iter()
            .map(|m| {
                let mut model = ModelMetadata::new(&m.id, &m.display_name, provider.id)
                    .with_context(m.context_window, m.max_output);

                if let Some(reasoning) = m.reasoning {
                    model = model.with_thinking(reasoning);
                }

                model.supports_tools = provider.supports_tools;
                model
            })
            .collect();

        registry.set_models(provider.id, models).await;
    }

    if let Some(api_key) = credentials.get(&ProviderId::OpenRouter) {
        match krusty_core::ai::openrouter::fetch_models(api_key).await {
            Ok(models) => {
                tracing::info!("Fetched {} OpenRouter models", models.len());
                registry.set_models(ProviderId::OpenRouter, models).await;
            }
            Err(e) => tracing::warn!("Failed to fetch OpenRouter models: {}", e),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let db_path = paths::config_dir().join("krusty.db");
    let working_dir = std::env::current_dir()?;
    let _db = Database::new(&db_path)?;

    let credential_store_inner = CredentialStore::load().unwrap_or_default();
    let credential_store = Arc::new(RwLock::new(credential_store_inner.clone()));
    let ai_client = create_ai_client(&credential_store_inner).map(Arc::new);

    let process_registry = Arc::new(ProcessRegistry::new());
    let tool_registry = Arc::new(ToolRegistry::new());
    register_all_tools(&tool_registry).await;

    let model_registry = create_model_registry();
    initialize_models(&model_registry, &credential_store_inner).await;

    let mcp_manager = Arc::new(McpManager::new(working_dir.clone()));
    if let Err(e) = mcp_manager.load_config().await {
        tracing::warn!("Failed to load MCP config: {}", e);
    } else if let Err(e) = mcp_manager.connect_all().await {
        tracing::warn!("Failed to connect MCP servers: {}", e);
    }

    let mut hook_manager_inner = UserHookManager::new();
    if let Ok(db) = Database::new(&db_path) {
        if let Err(e) = hook_manager_inner.load(&db) {
            tracing::warn!("Failed to load hooks: {}", e);
        }
    }

    let state = AppState {
        db_path: Arc::new(db_path),
        working_dir: Arc::new(working_dir.clone()),
        ai_client,
        tool_registry,
        process_registry,
        model_registry,
        credential_store,
        mcp_manager,
        hook_manager: Arc::new(RwLock::new(hook_manager_inner)),
        skills_manager: Arc::new(RwLock::new(SkillsManager::with_defaults(&working_dir))),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::PUT,
            Method::DELETE,
        ])
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .nest(
            "/api",
            routes::api_router().layer(middleware::from_fn_with_state(
                state.clone(),
                auth::auth_middleware,
            )),
        )
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{port}").parse()?;
    tracing::info!("Starting krusty-server on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

async fn root() -> &'static str {
    "Krusty Server"
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        features: HashMap::from([("chat".to_string(), true), ("tools".to_string(), true)]),
    })
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    features: HashMap<String, bool>,
}
