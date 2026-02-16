# AGENTS Guide: /crates/krusty-server/src

## Scope
- Applies to `/crates/krusty-server/src` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Server source modules and API wiring, including push notification dispatch.

## Local Standards
- Deliver best-in-class quality: elegant, modular, organized, and performant code.
- Keep code self-explanatory; add comments only for non-obvious constraints or tradeoffs.
- Avoid over-engineering; add abstractions only when they buy clear maintainability.
- Keep boundaries explicit between CLI, core runtime, server, desktop shell, and PWA surfaces.
- Prefer safe implementations; justify `unsafe` usage explicitly if ever required.
- For Rust changes, use idiomatic patterns (`Result`/`Option`, iterators, trait-based composition) and keep `anyhow::Context` on fallible IO/network boundaries.
- Maintain self-host assumptions and avoid platform-specific deployment coupling.
- Treat `push.rs` as reliability-critical: preserve retry behavior, stale endpoint cleanup, and outcome recording when modifying notification flow.

## Quality Gates
- Rust workspace: `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --all -- --check`
- Server local run: `cargo run -p krusty-server`
- PWA: `cd apps/pwa/app && bun run check && bun run build`
- Desktop shell: `cd apps/desktop/shell && cargo check --manifest-path src-tauri/Cargo.toml`

## Structure Map
### Subdirectories
- `routes/`: HTTP route handlers for chat, sessions, tools, files, and credentials. See `crates/krusty-server/src/routes/AGENTS.md` for local detail.
- `utils/`: Server utility helpers for paths/providers and shared concerns. See `crates/krusty-server/src/utils/AGENTS.md` for local detail.
- `ws/`: WebSocket transport handlers including terminal streams. See `crates/krusty-server/src/ws/AGENTS.md` for local detail.

### Files
- `auth.rs`: Rust source module implementing auth behavior.
- `error.rs`: Rust source module implementing error behavior.
- `lib.rs`: Library root that declares modules and public exports.
- `push.rs`: Push notification service with VAPID handling, retry policy, stale-prune logic, and delivery outcome stats.
- `types.rs`: Rust source module implementing types behavior.
