# AGENTS Guide: /crates

## Scope
- Applies to `/crates` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Rust workspace crates implementing CLI, core runtime, and server.

## Local Standards
- Deliver best-in-class quality: elegant, modular, organized, and performant code.
- Keep code self-explanatory; add comments only for non-obvious constraints or tradeoffs.
- Avoid over-engineering; add abstractions only when they buy clear maintainability.
- Keep boundaries explicit between CLI, core runtime, server, desktop shell, and PWA surfaces.
- Prefer safe implementations; justify `unsafe` usage explicitly if ever required.

## Quality Gates
- Rust workspace: `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --all -- --check`
- Server local run: `cargo run -p krusty-server`
- PWA: `cd apps/pwa/app && bun run check && bun run build`
- Desktop shell: `cd apps/desktop/shell && cargo check --manifest-path src-tauri/Cargo.toml`

## Structure Map
### Subdirectories
- `krusty-cli/`: Terminal client crate exposing CLI commands and TUI runtime. See `crates/krusty-cli/AGENTS.md` for local detail.
- `krusty-core/`: Core runtime crate providing AI clients, tools, storage, planning, and protocols. See `crates/krusty-core/AGENTS.md` for local detail.
- `krusty-server/`: Self-host API crate used by CLI and app clients. See `crates/krusty-server/AGENTS.md` for local detail.

### Files
- _(none)_
