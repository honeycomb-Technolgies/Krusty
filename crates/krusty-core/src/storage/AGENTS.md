# AGENTS Guide: /crates/krusty-core/src/storage

## Scope
- Applies to `/crates/krusty-core/src/storage` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
SQLite persistence layer for sessions, credentials, plans, UI state, and push delivery observability.

## Local Standards
- Deliver best-in-class quality: elegant, modular, organized, and performant code.
- Keep code self-explanatory; add comments only for non-obvious constraints or tradeoffs.
- Avoid over-engineering; add abstractions only when they buy clear maintainability.
- Keep boundaries explicit between CLI, core runtime, server, desktop shell, and PWA surfaces.
- Prefer safe implementations; justify `unsafe` usage explicitly if ever required.
- For Rust changes, use idiomatic patterns (`Result`/`Option`, iterators, trait-based composition) and keep `anyhow::Context` on fallible IO/network boundaries.
- Keep provider/tool differences behind stable abstractions so user-facing behavior remains consistent across integrations.
- Keep push persistence changes coordinated across `database.rs`, `push_subscriptions.rs`, `push_delivery_attempts.rs`, and migration tests so delivery diagnostics remain trustworthy.

## Quality Gates
- Rust workspace: `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --all -- --check`
- Server local run: `cargo run -p krusty-server`
- PWA: `cd apps/pwa/app && bun run check && bun run build`
- Desktop shell: `cd apps/desktop/shell && cargo check --manifest-path src-tauri/Cargo.toml`

## Structure Map
### Subdirectories
- _(none)_

### Files
- `agent_state.rs`: Rust source module implementing agent state behavior.
- `block_ui.rs`: Rust source module implementing block ui behavior.
- `credentials.rs`: Rust source module implementing credentials behavior.
- `database.rs`: Rust source module implementing database behavior.
- `database_tests.rs`: Rust source module implementing database tests behavior.
- `file_activity.rs`: Rust source module implementing file activity behavior.
- `messages.rs`: Rust source module implementing messages behavior.
- `mod.rs`: Module root that wires child modules and shared exports.
- `plans.rs`: Rust source module implementing plans behavior.
- `preferences.rs`: Rust source module implementing preferences behavior.
- `push_delivery_attempts.rs`: Rust source module for recording push delivery attempts and diagnostics summaries.
- `push_subscriptions.rs`: Rust source module implementing push subscriptions behavior.
- `sessions.rs`: Rust source module implementing sessions behavior.
