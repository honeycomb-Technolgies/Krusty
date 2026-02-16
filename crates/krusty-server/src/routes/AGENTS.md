# AGENTS Guide: /crates/krusty-server/src/routes

## Scope
- Applies to `/crates/krusty-server/src/routes` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
HTTP route handlers for chat, sessions, tools, files, credentials, and push APIs.

## Local Standards
- Deliver best-in-class quality: elegant, modular, organized, and performant code.
- Keep code self-explanatory; add comments only for non-obvious constraints or tradeoffs.
- Avoid over-engineering; add abstractions only when they buy clear maintainability.
- Keep boundaries explicit between CLI, core runtime, server, desktop shell, and PWA surfaces.
- Prefer safe implementations; justify `unsafe` usage explicitly if ever required.
- For Rust changes, use idiomatic patterns (`Result`/`Option`, iterators, trait-based composition) and keep `anyhow::Context` on fallible IO/network boundaries.
- Maintain self-host assumptions and avoid platform-specific deployment coupling.
- Keep `/push/*` route contracts aligned with `apps/pwa/app/src/lib/api/client.ts` because Settings diagnostics and test-send UX depend on stable field names.

## Quality Gates
- Rust workspace: `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --all -- --check`
- Server local run: `cargo run -p krusty-server`
- PWA: `cd apps/pwa/app && bun run check && bun run build`
- Desktop shell: `cd apps/desktop/shell && cargo check --manifest-path src-tauri/Cargo.toml`

## Structure Map
### Subdirectories
- _(none)_

### Files
- `chat.rs`: HTTP route handler module for chat endpoints.
- `credentials.rs`: HTTP route handler module for credentials endpoints.
- `files.rs`: HTTP route handler module for files endpoints.
- `git.rs`: HTTP route handler module for git endpoints.
- `hooks.rs`: HTTP route handler module for hooks endpoints.
- `mcp.rs`: HTTP route handler module for mcp endpoints.
- `mod.rs`: Module root that wires child modules and shared exports.
- `models.rs`: HTTP route handler module for models endpoints.
- `processes.rs`: HTTP route handler module for processes endpoints.
- `push.rs`: HTTP route handler module for push subscription, diagnostics (`/status`), and test-send (`/test`) endpoints.
- `sessions.rs`: HTTP route handler module for sessions endpoints.
- `tools.rs`: HTTP route handler module for tools endpoints.
