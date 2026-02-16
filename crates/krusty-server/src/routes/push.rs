//! Push notification subscription endpoints

use axum::{
    extract::State,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use krusty_core::storage::{Database, PushSubscriptionStore};

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/vapid-public-key", get(vapid_public_key))
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
