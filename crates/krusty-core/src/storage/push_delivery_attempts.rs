//! Push delivery attempt storage
//!
//! Tracks notification delivery outcomes for diagnostics and reliability.

use anyhow::Result;
use chrono::{Duration, Utc};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

use super::database::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushDeliveryAttempt {
    pub id: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub endpoint_hash: String,
    pub provider_host: String,
    pub event_type: String,
    pub outcome: String,
    pub http_status: Option<i64>,
    pub error_message: Option<String>,
    pub latency_ms: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PushDeliverySummary {
    pub last_attempt_at: Option<String>,
    pub last_success_at: Option<String>,
    pub last_failure_at: Option<String>,
    pub last_failure_reason: Option<String>,
    pub recent_failures_24h: usize,
}

pub struct PushDeliveryAttemptStore<'a> {
    db: &'a Database,
}

impl<'a> PushDeliveryAttemptStore<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn record_attempt(
        &self,
        user_id: Option<&str>,
        session_id: Option<&str>,
        endpoint: &str,
        event_type: &str,
        outcome: &str,
        http_status: Option<u16>,
        error_message: Option<&str>,
        latency_ms: Option<u64>,
    ) -> Result<()> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let endpoint_hash = Self::endpoint_hash(endpoint);
        let provider_host = Self::provider_host(endpoint);
        let http_status = http_status.map(i64::from);
        let latency_ms = latency_ms.map(|v| v as i64);

        self.db.conn().execute(
            "INSERT INTO push_delivery_attempts (
                id, user_id, session_id, endpoint_hash, provider_host, event_type,
                outcome, http_status, error_message, latency_ms, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                id,
                user_id,
                session_id,
                endpoint_hash,
                provider_host,
                event_type,
                outcome,
                http_status,
                error_message,
                latency_ms,
                now
            ],
        )?;

        Ok(())
    }

    pub fn latest_for_user(&self, user_id: Option<&str>) -> Result<Option<PushDeliveryAttempt>> {
        let sql = match user_id {
            Some(_) => {
                "SELECT id, user_id, session_id, endpoint_hash, provider_host, event_type,
                        outcome, http_status, error_message, latency_ms, created_at
                 FROM push_delivery_attempts
                 WHERE user_id = ?1
                 ORDER BY created_at DESC
                 LIMIT 1"
            }
            None => {
                "SELECT id, user_id, session_id, endpoint_hash, provider_host, event_type,
                        outcome, http_status, error_message, latency_ms, created_at
                 FROM push_delivery_attempts
                 ORDER BY created_at DESC
                 LIMIT 1"
            }
        };

        let mut stmt = self.db.conn().prepare(sql)?;
        let mut rows = match user_id {
            Some(uid) => stmt.query([uid])?,
            None => stmt.query([])?,
        };

        if let Some(row) = rows.next()? {
            Ok(Some(PushDeliveryAttempt {
                id: row.get(0)?,
                user_id: row.get(1)?,
                session_id: row.get(2)?,
                endpoint_hash: row.get(3)?,
                provider_host: row.get(4)?,
                event_type: row.get(5)?,
                outcome: row.get(6)?,
                http_status: row.get(7)?,
                error_message: row.get(8)?,
                latency_ms: row.get(9)?,
                created_at: row.get(10)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn summary_for_user(&self, user_id: Option<&str>) -> Result<PushDeliverySummary> {
        let last_attempt_at = self.latest_timestamp(user_id, None);
        let last_success_at = self.latest_timestamp(user_id, Some("outcome = 'success'"));
        let last_failure_at = self.latest_timestamp(user_id, Some("outcome = 'failure'"));
        let last_failure_reason = self.latest_failure_reason(user_id);

        let threshold = (Utc::now() - Duration::hours(24)).to_rfc3339();
        let recent_failures_24h: i64 = match user_id {
            Some(uid) => self.db.conn().query_row(
                "SELECT COUNT(*)
                 FROM push_delivery_attempts
                 WHERE user_id = ?1 AND outcome = 'failure' AND created_at >= ?2",
                params![uid, threshold],
                |row| row.get(0),
            )?,
            None => self.db.conn().query_row(
                "SELECT COUNT(*)
                 FROM push_delivery_attempts
                 WHERE outcome = 'failure' AND created_at >= ?1",
                params![threshold],
                |row| row.get(0),
            )?,
        };

        Ok(PushDeliverySummary {
            last_attempt_at,
            last_success_at,
            last_failure_at,
            last_failure_reason,
            recent_failures_24h: recent_failures_24h as usize,
        })
    }

    fn latest_timestamp(
        &self,
        user_id: Option<&str>,
        extra_filter: Option<&str>,
    ) -> Option<String> {
        let mut sql = String::from("SELECT created_at FROM push_delivery_attempts");
        let mut has_where = false;

        if user_id.is_some() {
            sql.push_str(" WHERE user_id = ?1");
            has_where = true;
        }

        if let Some(filter) = extra_filter {
            if has_where {
                sql.push_str(" AND ");
            } else {
                sql.push_str(" WHERE ");
            }
            sql.push_str(filter);
        }

        sql.push_str(" ORDER BY created_at DESC LIMIT 1");

        let result = match user_id {
            Some(uid) => self.db.conn().query_row(&sql, [uid], |row| row.get(0)),
            None => self.db.conn().query_row(&sql, [], |row| row.get(0)),
        };

        result.ok()
    }

    fn latest_failure_reason(&self, user_id: Option<&str>) -> Option<String> {
        let sql = match user_id {
            Some(_) => {
                "SELECT error_message
                 FROM push_delivery_attempts
                 WHERE user_id = ?1 AND outcome = 'failure'
                 ORDER BY created_at DESC
                 LIMIT 1"
            }
            None => {
                "SELECT error_message
                 FROM push_delivery_attempts
                 WHERE outcome = 'failure'
                 ORDER BY created_at DESC
                 LIMIT 1"
            }
        };

        let result = match user_id {
            Some(uid) => self.db.conn().query_row(sql, [uid], |row| row.get(0)),
            None => self.db.conn().query_row(sql, [], |row| row.get(0)),
        };

        result.ok()
    }

    fn endpoint_hash(endpoint: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(endpoint.as_bytes());
        let digest = hasher.finalize();
        hex_encode(digest.as_slice())
    }

    fn provider_host(endpoint: &str) -> String {
        Url::parse(endpoint)
            .ok()
            .and_then(|u| u.host_str().map(str::to_string))
            .unwrap_or_else(|| "unknown".to_string())
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{:02x}", b));
    }
    out
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::storage::{Database, PushDeliveryAttemptStore};

    fn create_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(&db_path).expect("Failed to create database");
        (db, temp_dir)
    }

    #[test]
    fn test_record_and_summarize_attempts() {
        let (db, _temp) = create_test_db();
        let store = PushDeliveryAttemptStore::new(&db);

        store
            .record_attempt(
                None,
                Some("session-1"),
                "https://web.push.apple.com/test-endpoint",
                "completion",
                "success",
                Some(201),
                None,
                Some(30),
            )
            .expect("Failed to record success attempt");

        store
            .record_attempt(
                None,
                Some("session-1"),
                "https://web.push.apple.com/test-endpoint",
                "completion",
                "failure",
                Some(500),
                Some("temporary failure"),
                Some(45),
            )
            .expect("Failed to record failure attempt");

        let latest = store
            .latest_for_user(None)
            .expect("Failed to fetch latest attempt")
            .expect("Expected latest attempt");
        assert_eq!(latest.outcome, "failure");
        assert_eq!(latest.provider_host, "web.push.apple.com");
        assert_eq!(latest.endpoint_hash.len(), 64);

        let summary = store
            .summary_for_user(None)
            .expect("Failed to summarize attempts");
        assert!(summary.last_attempt_at.is_some());
        assert!(summary.last_success_at.is_some());
        assert!(summary.last_failure_at.is_some());
        assert_eq!(
            summary.last_failure_reason.as_deref(),
            Some("temporary failure")
        );
        assert_eq!(summary.recent_failures_24h, 1);
    }
}
