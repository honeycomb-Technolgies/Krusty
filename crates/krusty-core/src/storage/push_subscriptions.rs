//! Push subscription storage
//!
//! CRUD operations for Web Push notification subscriptions.

use anyhow::Result;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::database::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushSubscription {
    pub id: String,
    pub user_id: Option<String>,
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

pub struct PushSubscriptionStore<'a> {
    db: &'a Database,
}

impl<'a> PushSubscriptionStore<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Insert or update a push subscription (upsert on endpoint).
    pub fn upsert(
        &self,
        user_id: Option<&str>,
        endpoint: &str,
        p256dh: &str,
        auth: &str,
    ) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        self.db.conn().execute(
            "INSERT INTO push_subscriptions (id, user_id, endpoint, p256dh, auth, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(endpoint) DO UPDATE SET
                user_id = excluded.user_id,
                p256dh = excluded.p256dh,
                auth = excluded.auth",
            params![id, user_id, endpoint, p256dh, auth, now],
        )?;

        Ok(id)
    }

    /// Remove a subscription by endpoint.
    pub fn remove_by_endpoint(&self, endpoint: &str) -> Result<bool> {
        let rows = self.db.conn().execute(
            "DELETE FROM push_subscriptions WHERE endpoint = ?1",
            [endpoint],
        )?;
        Ok(rows > 0)
    }

    /// Get all subscriptions for a specific user.
    pub fn get_for_user(&self, user_id: &str) -> Result<Vec<PushSubscription>> {
        let mut stmt = self.db.conn().prepare(
            "SELECT id, user_id, endpoint, p256dh, auth, created_at, last_used_at
             FROM push_subscriptions WHERE user_id = ?1",
        )?;

        let subs = stmt.query_map([user_id], |row| {
            Ok(PushSubscription {
                id: row.get(0)?,
                user_id: row.get(1)?,
                endpoint: row.get(2)?,
                p256dh: row.get(3)?,
                auth: row.get(4)?,
                created_at: row.get(5)?,
                last_used_at: row.get(6)?,
            })
        })?;

        subs.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Get all subscriptions (for single-tenant mode).
    pub fn get_all(&self) -> Result<Vec<PushSubscription>> {
        let mut stmt = self.db.conn().prepare(
            "SELECT id, user_id, endpoint, p256dh, auth, created_at, last_used_at
             FROM push_subscriptions",
        )?;

        let subs = stmt.query_map([], |row| {
            Ok(PushSubscription {
                id: row.get(0)?,
                user_id: row.get(1)?,
                endpoint: row.get(2)?,
                p256dh: row.get(3)?,
                auth: row.get(4)?,
                created_at: row.get(5)?,
                last_used_at: row.get(6)?,
            })
        })?;

        subs.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Update last_used_at timestamp for a subscription.
    pub fn touch(&self, endpoint: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.db.conn().execute(
            "UPDATE push_subscriptions SET last_used_at = ?1 WHERE endpoint = ?2",
            params![now, endpoint],
        )?;
        Ok(())
    }
}
