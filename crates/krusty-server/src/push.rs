//! Web Push notification service
//!
//! Handles VAPID key management and sending push notifications
//! via the Web Push protocol (RFC 8030).

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::http;
use base64ct::{Base64UrlUnpadded, Encoding};
use serde::Serialize;
use web_push_native::jwt_simple::algorithms::{ECDSAP256PublicKeyLike, ES256KeyPair};
use web_push_native::p256::PublicKey;
use web_push_native::{Auth, WebPushBuilder};

use krusty_core::storage::{Database, PushSubscriptionStore};

/// Payload sent inside a push notification.
#[derive(Debug, Clone, Serialize)]
pub struct PushPayload {
    pub title: String,
    pub body: String,
    pub session_id: Option<String>,
    pub tag: Option<String>,
}

pub struct PushService {
    keypair: ES256KeyPair,
    public_key_base64url: String,
    contact: String,
    db_path: Arc<PathBuf>,
    http_client: reqwest::Client,
}

impl PushService {
    /// Load or generate a VAPID keypair and create the service.
    pub fn init(vapid_key_path: &std::path::Path, db_path: Arc<PathBuf>) -> Result<Self> {
        if let Some(parent) = vapid_key_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let keypair = if vapid_key_path.exists() {
            let pem =
                std::fs::read_to_string(vapid_key_path).context("Failed to read VAPID key file")?;
            ES256KeyPair::from_pem(&pem).context("Failed to parse VAPID PEM")?
        } else {
            let kp = ES256KeyPair::generate();
            let pem = kp.to_pem().context("Failed to serialize VAPID key")?;
            std::fs::write(vapid_key_path, &pem).context("Failed to write VAPID key file")?;
            tracing::info!(
                "Generated new VAPID keypair at {}",
                vapid_key_path.display()
            );
            kp
        };

        let public_key_bytes = keypair.public_key().public_key().to_bytes_uncompressed();
        let public_key_base64url = Base64UrlUnpadded::encode_string(&public_key_bytes);

        Ok(Self {
            keypair,
            public_key_base64url,
            contact: "mailto:krusty@localhost".to_string(),
            db_path,
            http_client: reqwest::Client::new(),
        })
    }

    /// The VAPID public key encoded as base64url (no padding).
    /// Clients need this to create a PushSubscription.
    pub fn vapid_public_key_base64url(&self) -> &str {
        &self.public_key_base64url
    }

    /// Send a push notification to a single subscription endpoint.
    /// Returns Ok(true) if sent, Ok(false) if subscription was stale (deleted).
    async fn send(
        &self,
        endpoint: &str,
        p256dh: &str,
        auth: &str,
        payload: &PushPayload,
    ) -> Result<bool> {
        let endpoint_uri: http::Uri = endpoint
            .parse()
            .context("Invalid push subscription endpoint")?;

        let ua_public_bytes =
            Base64UrlUnpadded::decode_vec(p256dh).context("Invalid p256dh key")?;
        let ua_public =
            PublicKey::from_sec1_bytes(&ua_public_bytes).context("Invalid p256dh public key")?;

        let ua_auth_bytes = Base64UrlUnpadded::decode_vec(auth).context("Invalid auth secret")?;
        let ua_auth_arr: [u8; 16] = ua_auth_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Auth secret must be 16 bytes"))?;
        let ua_auth: Auth = ua_auth_arr.into();

        let body = serde_json::to_vec(payload)?;

        let http_request = WebPushBuilder::new(endpoint_uri, ua_public, ua_auth)
            .with_vapid(&self.keypair, &self.contact)
            .build(body)
            .context("Failed to build push request")?;

        let (parts, body_bytes) = http_request.into_parts();
        let url = parts.uri.to_string();

        let mut req_builder = self.http_client.request(
            reqwest::Method::from_bytes(parts.method.as_str().as_bytes())
                .unwrap_or(reqwest::Method::POST),
            &url,
        );
        for (name, value) in &parts.headers {
            if let Ok(v) = value.to_str() {
                req_builder = req_builder.header(name.as_str(), v);
            }
        }

        let response = req_builder
            .body(body_bytes)
            .send()
            .await
            .context("Failed to send push notification")?;

        let status = response.status().as_u16();

        // 404 or 410 = subscription expired/invalid → clean up
        if status == 404 || status == 410 {
            tracing::info!(endpoint, "Push subscription expired, removing");
            if let Ok(db) = Database::new(&self.db_path) {
                let store = PushSubscriptionStore::new(&db);
                let _ = store.remove_by_endpoint(endpoint);
            }
            return Ok(false);
        }

        if !(200..300).contains(&(status as usize)) {
            let body = response.text().await.unwrap_or_default();
            tracing::warn!(status, endpoint, body, "Push delivery failed");
            return Err(anyhow::anyhow!("Push failed with status {}", status));
        }

        // Update last_used_at
        if let Ok(db) = Database::new(&self.db_path) {
            let store = PushSubscriptionStore::new(&db);
            let _ = store.touch(endpoint);
        }

        Ok(true)
    }

    /// Send a notification to all subscriptions for a user.
    /// In single-tenant mode (user_id = None), sends to all subscriptions.
    pub async fn notify_user(&self, user_id: Option<&str>, payload: PushPayload) {
        let subscriptions = match Database::new(&self.db_path) {
            Ok(db) => {
                let store = PushSubscriptionStore::new(&db);
                match user_id {
                    Some(uid) => store.get_for_user(uid).unwrap_or_default(),
                    None => store.get_all().unwrap_or_default(),
                }
            }
            Err(e) => {
                tracing::error!("Failed to open DB for push: {}", e);
                return;
            }
        };

        if subscriptions.is_empty() {
            return;
        }

        tracing::debug!(count = subscriptions.len(), "Sending push notifications");

        for sub in subscriptions {
            match self
                .send(&sub.endpoint, &sub.p256dh, &sub.auth, &payload)
                .await
            {
                Ok(true) => tracing::debug!(endpoint = %sub.endpoint, "Push sent"),
                Ok(false) => {
                    tracing::debug!(endpoint = %sub.endpoint, "Stale subscription removed")
                }
                Err(e) => tracing::warn!(endpoint = %sub.endpoint, "Push failed: {}", e),
            }
        }
    }
}
