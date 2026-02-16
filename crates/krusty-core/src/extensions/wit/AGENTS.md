# AGENTS Guide: /crates/krusty-core/src/extensions/wit

## Scope
- Applies to `/crates/krusty-core/src/extensions/wit` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Versioned WIT interface contracts for extension API compatibility.

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
- `since_v0.0.1/`: Versioned WIT API snapshot for extension compatibility and migration stability. See `crates/krusty-core/src/extensions/wit/since_v0.0.1/AGENTS.md` for local detail.
- `since_v0.0.4/`: Versioned WIT API snapshot for extension compatibility and migration stability. See `crates/krusty-core/src/extensions/wit/since_v0.0.4/AGENTS.md` for local detail.
- `since_v0.0.6/`: Versioned WIT API snapshot for extension compatibility and migration stability. See `crates/krusty-core/src/extensions/wit/since_v0.0.6/AGENTS.md` for local detail.
- `since_v0.1.0/`: Versioned WIT API snapshot for extension compatibility and migration stability. See `crates/krusty-core/src/extensions/wit/since_v0.1.0/AGENTS.md` for local detail.
- `since_v0.2.0/`: Versioned WIT API snapshot for extension compatibility and migration stability. See `crates/krusty-core/src/extensions/wit/since_v0.2.0/AGENTS.md` for local detail.
- `since_v0.3.0/`: Versioned WIT API snapshot for extension compatibility and migration stability. See `crates/krusty-core/src/extensions/wit/since_v0.3.0/AGENTS.md` for local detail.
- `since_v0.4.0/`: Versioned WIT API snapshot for extension compatibility and migration stability. See `crates/krusty-core/src/extensions/wit/since_v0.4.0/AGENTS.md` for local detail.
- `since_v0.5.0/`: Versioned WIT API snapshot for extension compatibility and migration stability. See `crates/krusty-core/src/extensions/wit/since_v0.5.0/AGENTS.md` for local detail.
- `since_v0.6.0/`: Versioned WIT API snapshot for extension compatibility and migration stability. See `crates/krusty-core/src/extensions/wit/since_v0.6.0/AGENTS.md` for local detail.
- `since_v0.8.0/`: Versioned WIT API snapshot for extension compatibility and migration stability. See `crates/krusty-core/src/extensions/wit/since_v0.8.0/AGENTS.md` for local detail.

### Files
- _(none)_
