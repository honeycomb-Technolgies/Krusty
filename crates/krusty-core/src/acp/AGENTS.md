# AGENTS Guide: /crates/krusty-core/src/acp

## Scope
- Applies to `/crates/krusty-core/src/acp` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Agent Client Protocol server/bridge implementation for editor integrations.

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
- _(none)_

### Files
- `agent.rs`: Rust source module implementing agent behavior.
- `bridge.rs`: Rust source module implementing bridge behavior.
- `error.rs`: Rust source module implementing error behavior.
- `mod.rs`: Module root that wires child modules and shared exports.
- `model_manager.rs`: Rust source module implementing model manager behavior.
- `processor.rs`: Rust source module implementing processor behavior.
- `server.rs`: Rust source module implementing server behavior.
- `session.rs`: Rust source module implementing session behavior.
- `tools.rs`: Rust source module implementing tools behavior.
- `updates.rs`: Rust source module implementing updates behavior.
- `workspace_context.rs`: Rust source module implementing workspace context behavior.
