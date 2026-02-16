# AGENTS Guide: /crates/krusty-core/src/storage

## Purpose
SQLite persistence for sessions, plans, credentials, and push observability.

## Guardrails
- Migration safety first: schema changes must be forward-only and tested.
- Keep read/write behavior explicit and transaction-aware.
- For push reliability changes, keep `database.rs`, `push_subscriptions.rs`, and `push_delivery_attempts.rs` aligned.
- Never log sensitive credentials.

## Validation
- `cargo check -p krusty-core`
- run targeted storage migration tests for schema changes.
