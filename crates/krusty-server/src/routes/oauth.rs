//! OAuth authentication endpoints for PWA.

use std::sync::mpsc;
use std::time::Instant;

use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use krusty_core::ai::providers::ProviderId;
use krusty_core::auth::{
    anthropic_oauth_config, openai_oauth_config, run_callback_server, BrowserOAuthFlow,
    CallbackResult, OAuthTokenStore, PasteCodeOAuthFlow, PkceVerifier, DEFAULT_CALLBACK_PORT,
};

use crate::error::AppError;
use crate::AppState;

/// In-flight OAuth flow state stored on the server.
pub struct OAuthFlowState {
    pub verifier_str: String,
    pub started_at: Instant,
    pub provider_id: ProviderId,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/start", post(start_oauth))
        .route("/exchange", post(exchange_code))
        .route("/status/{provider}", get(oauth_status))
        .route("/revoke/{provider}", delete(revoke_oauth))
}

#[derive(Serialize)]
struct OAuthStartResponse {
    auth_url: String,
    provider: String,
    paste_code: bool,
}

#[derive(Deserialize)]
struct OAuthStartRequest {
    provider: String,
}

async fn start_oauth(
    State(state): State<AppState>,
    Json(req): Json<OAuthStartRequest>,
) -> Result<Json<OAuthStartResponse>, AppError> {
    let provider_id = parse_provider(&req.provider)?;

    if !provider_id.supports_oauth() {
        return Err(AppError::BadRequest(format!(
            "Provider {} does not support OAuth",
            req.provider
        )));
    }

    // Check for already-active flow
    {
        let flows = state.oauth_flows.lock().await;
        if let Some(existing) = flows.get(provider_id.storage_key()) {
            if existing.started_at.elapsed().as_secs() < 300 {
                return Err(AppError::Conflict(
                    "OAuth flow already in progress for this provider".to_string(),
                ));
            }
        }
    }

    match provider_id {
        ProviderId::OpenAI => start_openai_oauth(state, provider_id).await,
        ProviderId::Anthropic => start_anthropic_oauth(state, provider_id).await,
        _ => Err(AppError::BadRequest(
            "OAuth not implemented for this provider".to_string(),
        )),
    }
}

async fn start_openai_oauth(
    state: AppState,
    provider_id: ProviderId,
) -> Result<Json<OAuthStartResponse>, AppError> {
    let config = openai_oauth_config();
    let flow = BrowserOAuthFlow::new(config);
    let (auth_url, verifier, expected_state) = flow
        .get_auth_url()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Store flow state
    {
        let mut flows = state.oauth_flows.lock().await;
        flows.insert(
            provider_id.storage_key().to_string(),
            OAuthFlowState {
                verifier_str: verifier.as_str().to_string(),
                started_at: Instant::now(),
                provider_id,
            },
        );
    }

    // Spawn background task: callback server + token exchange
    let oauth_flows = state.oauth_flows.clone();
    tokio::spawn(async move {
        let (tx, rx) = mpsc::channel::<CallbackResult>();

        // Run callback server in a blocking thread
        let state_clone = expected_state.clone();
        let server_handle = tokio::task::spawn_blocking(move || {
            run_callback_server(DEFAULT_CALLBACK_PORT, state_clone, tx);
        });

        // Wait for result
        let result = tokio::task::spawn_blocking(move || {
            rx.recv_timeout(std::time::Duration::from_secs(300))
        })
        .await;

        let _ = server_handle.await;

        match result {
            Ok(Ok(CallbackResult::Success { code })) => {
                let config = openai_oauth_config();
                let exchange_flow = BrowserOAuthFlow::new(config);
                let verifier = PkceVerifier::from_string(
                    oauth_flows
                        .lock()
                        .await
                        .get(provider_id.storage_key())
                        .map(|f| f.verifier_str.clone())
                        .unwrap_or_default(),
                );

                match exchange_flow.exchange_code(&code, &verifier).await {
                    Ok(token_data) => {
                        if let Ok(mut store) = OAuthTokenStore::load() {
                            store.set(provider_id, token_data);
                            if let Err(e) = store.save() {
                                tracing::error!("Failed to save OAuth token: {}", e);
                            } else {
                                tracing::info!("OpenAI OAuth token stored successfully");
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("OpenAI token exchange failed: {}", e);
                    }
                }
            }
            Ok(Ok(CallbackResult::Error { error, description })) => {
                tracing::error!("OpenAI OAuth callback error: {} - {}", error, description);
            }
            _ => {
                tracing::warn!("OpenAI OAuth callback timed out or failed");
            }
        }

        // Clean up flow state
        oauth_flows.lock().await.remove(provider_id.storage_key());
    });

    Ok(Json(OAuthStartResponse {
        auth_url,
        provider: provider_id.storage_key().to_string(),
        paste_code: false,
    }))
}

async fn start_anthropic_oauth(
    state: AppState,
    provider_id: ProviderId,
) -> Result<Json<OAuthStartResponse>, AppError> {
    let config = anthropic_oauth_config();
    let flow = PasteCodeOAuthFlow::new(config);
    let (auth_url, verifier, _state) = flow
        .get_auth_url()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Store verifier for later exchange
    {
        let mut flows = state.oauth_flows.lock().await;
        flows.insert(
            provider_id.storage_key().to_string(),
            OAuthFlowState {
                verifier_str: verifier.as_str().to_string(),
                started_at: Instant::now(),
                provider_id,
            },
        );
    }

    Ok(Json(OAuthStartResponse {
        auth_url,
        provider: provider_id.storage_key().to_string(),
        paste_code: true,
    }))
}

#[derive(Deserialize)]
struct OAuthExchangeRequest {
    provider: String,
    code: String,
}

#[derive(Serialize)]
struct OAuthExchangeResponse {
    success: bool,
}

async fn exchange_code(
    State(state): State<AppState>,
    Json(req): Json<OAuthExchangeRequest>,
) -> Result<Json<OAuthExchangeResponse>, AppError> {
    let provider_id = parse_provider(&req.provider)?;

    let verifier_str = {
        let flows = state.oauth_flows.lock().await;
        flows
            .get(provider_id.storage_key())
            .map(|f| f.verifier_str.clone())
            .ok_or_else(|| {
                AppError::BadRequest("No active OAuth flow for this provider".to_string())
            })?
    };

    let verifier = PkceVerifier::from_string(verifier_str);
    let config = anthropic_oauth_config();
    let flow = PasteCodeOAuthFlow::new(config);

    let token_data = flow
        .exchange_code(&req.code, &verifier)
        .await
        .map_err(|e| {
            tracing::error!("OAuth token exchange failed for {}: {}", provider_id, e);
            AppError::Internal(e.to_string())
        })?;

    let mut store = OAuthTokenStore::load().map_err(|e| {
        tracing::error!("Failed to load OAuth token store: {}", e);
        AppError::Internal(e.to_string())
    })?;
    store.set(provider_id, token_data);
    store.save().map_err(|e| {
        tracing::error!("Failed to save OAuth token: {}", e);
        AppError::Internal(e.to_string())
    })?;

    tracing::info!("OAuth token stored successfully for {}", provider_id);

    // Clean up flow state
    state
        .oauth_flows
        .lock()
        .await
        .remove(provider_id.storage_key());

    Ok(Json(OAuthExchangeResponse { success: true }))
}

#[derive(Serialize)]
struct OAuthStatusResponse {
    has_token: bool,
    flow_active: bool,
}

async fn oauth_status(
    State(state): State<AppState>,
    Path(provider): Path<String>,
) -> Result<Json<OAuthStatusResponse>, AppError> {
    let provider_id = parse_provider(&provider)?;

    let has_token = OAuthTokenStore::load()
        .map(|store| store.has_token(&provider_id))
        .unwrap_or(false);

    let flow_active = state
        .oauth_flows
        .lock()
        .await
        .contains_key(provider_id.storage_key());

    Ok(Json(OAuthStatusResponse {
        has_token,
        flow_active,
    }))
}

async fn revoke_oauth(
    State(state): State<AppState>,
    Path(provider): Path<String>,
) -> Result<Json<OAuthStatusResponse>, AppError> {
    let provider_id = parse_provider(&provider)?;

    let mut store = OAuthTokenStore::load().map_err(|e| AppError::Internal(e.to_string()))?;
    store.remove(&provider_id);
    store
        .save()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Also clean up any active flow
    state
        .oauth_flows
        .lock()
        .await
        .remove(provider_id.storage_key());

    Ok(Json(OAuthStatusResponse {
        has_token: false,
        flow_active: false,
    }))
}

fn parse_provider(s: &str) -> Result<ProviderId, AppError> {
    crate::utils::providers::parse_provider(s)
        .ok_or_else(|| AppError::BadRequest(format!("Unknown provider: {}", s)))
}
