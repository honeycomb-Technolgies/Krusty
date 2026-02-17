//! Push notification subscription endpoints

use axum::{
    extract::State,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use krusty_core::storage::{Database, PushDeliveryAttemptStore, PushSubscriptionStore};

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::push::{PushEventType, PushPayload};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/vapid-public-key", get(vapid_public_key))
        .route("/status", get(status))
        .route("/test", post(send_test_notification))
        .route("/subscribe", post(subscribe))
        .route("/subscribe", delete(unsubscribe))
}

#[derive(Serialize)]
struct VapidKeyResponse {
    public_key: String,
}

async fn vapid_public_key(
    State(state): State<AppState>,
) -> Result<Json<VapidKeyResponse>, AppError> {
    let push_service = state
        .push_service
        .as_ref()
        .ok_or_else(|| AppError::Internal("Push notifications not configured".into()))?;

    Ok(Json(VapidKeyResponse {
        public_key: push_service.vapid_public_key_base64url().to_string(),
    }))
}

#[derive(Serialize)]
struct PushStatusResponse {
    push_configured: bool,
    subscription_count: usize,
    last_attempt_at: Option<String>,
    last_success_at: Option<String>,
    last_failure_at: Option<String>,
    last_failure_reason: Option<String>,
    recent_failures_24h: usize,
}

async fn status(
    State(state): State<AppState>,
    user: Option<CurrentUser>,
) -> Result<Json<PushStatusResponse>, AppError> {
    let user_id = user.and_then(|u| u.0.user_id);
    let db = Database::new(&state.db_path)?;
    let sub_store = PushSubscriptionStore::new(&db);
    let attempts = PushDeliveryAttemptStore::new(&db);
    let summary = attempts.summary_for_user(user_id.as_deref())?;
    let subscription_count = sub_store.count_for_user(user_id.as_deref())?;

    Ok(Json(PushStatusResponse {
        push_configured: state.push_service.is_some(),
        subscription_count,
        last_attempt_at: summary.last_attempt_at,
        last_success_at: summary.last_success_at,
        last_failure_at: summary.last_failure_at,
        last_failure_reason: summary.last_failure_reason,
        recent_failures_24h: summary.recent_failures_24h,
    }))
}

#[derive(Deserialize, Default)]
struct PushTestRequest {
    session_id: Option<String>,
    title: Option<String>,
    body: Option<String>,
}

#[derive(Serialize)]
struct PushTestResponse {
    accepted: bool,
    attempted: usize,
    sent: usize,
    stale_removed: usize,
    failed: usize,
}

async fn send_test_notification(
    State(state): State<AppState>,
    user: Option<CurrentUser>,
    Json(req): Json<PushTestRequest>,
) -> Result<Json<PushTestResponse>, AppError> {
    let push_service = state
        .push_service
        .as_ref()
        .cloned()
        .ok_or_else(|| AppError::Internal("Push notifications not configured".into()))?;

    let user_id = user.and_then(|u| u.0.user_id);
    let session_id = req.session_id;
    let title = req
        .title
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "Krusty".to_string());
    let body = req
        .body
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "Test notification from Krusty".to_string());

    let stats = push_service
        .notify_user(
            user_id.as_deref(),
            PushPayload {
                title,
                body,
                session_id: session_id.clone(),
                tag: Some(
                    session_id
                        .map(|id| format!("session-{id}"))
                        .unwrap_or_else(|| "push-test".to_string()),
                ),
            },
            PushEventType::Test,
        )
        .await;

    Ok(Json(PushTestResponse {
        accepted: stats.attempted > 0,
        attempted: stats.attempted,
        sent: stats.sent,
        stale_removed: stats.stale_removed,
        failed: stats.failed,
    }))
}

#[derive(Deserialize)]
struct SubscribeRequest {
    endpoint: String,
    p256dh: String,
    auth: String,
}

#[derive(Serialize)]
struct SubscribeResponse {
    id: String,
}

async fn subscribe(
    State(state): State<AppState>,
    user: Option<CurrentUser>,
    Json(req): Json<SubscribeRequest>,
) -> Result<Json<SubscribeResponse>, AppError> {
    let user_id = user.and_then(|u| u.0.user_id);
    let db = Database::new(&state.db_path)?;
    let store = PushSubscriptionStore::new(&db);
    let id = store.upsert(user_id.as_deref(), &req.endpoint, &req.p256dh, &req.auth)?;
    Ok(Json(SubscribeResponse { id }))
}

#[derive(Deserialize)]
struct UnsubscribeRequest {
    endpoint: String,
}

async fn unsubscribe(
    State(state): State<AppState>,
    Json(req): Json<UnsubscribeRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let db = Database::new(&state.db_path)?;
    let store = PushSubscriptionStore::new(&db);
    let removed = store.remove_by_endpoint(&req.endpoint)?;
    Ok(Json(serde_json::json!({ "removed": removed })))
}
