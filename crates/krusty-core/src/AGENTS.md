# AGENTS Guide: /crates/krusty-core/src

## Scope
- Applies to `/crates/krusty-core/src` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Core crate source modules exported by krusty-core.

## Local Standards
- Deliver best-in-class quality: elegant, modular, organized, and performant code.
- Keep code self-explanatory; add comments only for non-obvious constraints or tradeoffs.
- Avoid over-engineering; add abstractions only when they buy clear maintainability.
- Keep boundaries explicit between CLI, core runtime, server, desktop shell, and PWA surfaces.
- Prefer safe implementations; justify `unsafe` usage explicitly if ever required.
- For Rust changes, use idiomatic patterns (`Result`/`Option`, iterators, trait-based composition) and keep `anyhow::Context` on fallible IO/network boundaries.
- Keep provider/tool differences behind stable abstractions so user-facing behavior remains consistent across integrations.

## Quality Gates
- Rust workspace: `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --all -- --check`
- Server local run: `cargo run -p krusty-server`
- PWA: `cd apps/pwa/app && bun run check && bun run build`
- Desktop shell: `cd apps/desktop/shell && cargo check --manifest-path src-tauri/Cargo.toml`

## Structure Map
### Subdirectories
- `acp/`: Agent Client Protocol server/bridge implementation for editor integrations. See `crates/krusty-core/src/acp/AGENTS.md` for local detail.
- `agent/`: Agent orchestration, hooks, events, context building, and lifecycle state. See `crates/krusty-core/src/agent/AGENTS.md` for local detail.
- `ai/`: Provider abstraction, streaming, parsing, and AI response normalization. See `crates/krusty-core/src/ai/AGENTS.md` for local detail.
- `auth/`: Authentication flows, provider adapters, and credential handling. See `crates/krusty-core/src/auth/AGENTS.md` for local detail.
- `extensions/`: Extension runtime support, manifests, types, and host integration. See `crates/krusty-core/src/extensions/AGENTS.md` for local detail.
- `mcp/`: Model Context Protocol client, transport, and tool bridge. See `crates/krusty-core/src/mcp/AGENTS.md` for local detail.
- `plan/`: Plan mode parsing and manager utilities. See `crates/krusty-core/src/plan/AGENTS.md` for local detail.
- `process/`: Process orchestration utilities shared by tools/runtime. See `crates/krusty-core/src/process/AGENTS.md` for local detail.
- `skills/`: Filesystem skill discovery/loading and runtime skill metadata. See `crates/krusty-core/src/skills/AGENTS.md` for local detail.
- `storage/`: SQLite persistence layer for sessions, credentials, plans, and UI state. See `crates/krusty-core/src/storage/AGENTS.md` for local detail.
- `tools/`: Tool trait abstractions, registry, and shared path/tool utilities. See `crates/krusty-core/src/tools/AGENTS.md` for local detail.
- `updater/`: Update checking logic for release/version notifications. See `crates/krusty-core/src/updater/AGENTS.md` for local detail.

### Files
- `constants.rs`: Rust source module implementing constants behavior.
- `git.rs`: Rust source module implementing git behavior.
- `lib.rs`: Library root that declares modules and public exports.
- `paths.rs`: Rust source module implementing paths behavior.
- `server_instance.rs`: Rust source module implementing server instance behavior.
- `tailscale.rs`: Rust source module implementing tailscale behavior.
