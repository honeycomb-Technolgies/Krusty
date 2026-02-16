# AGENTS Guide: /crates/krusty-cli/src

## Scope
- Applies to `/crates/krusty-cli/src` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Source modules for CLI command parsing and runtime setup.

## Local Standards
- Deliver best-in-class quality: elegant, modular, organized, and performant code.
- Keep code self-explanatory; add comments only for non-obvious constraints or tradeoffs.
- Avoid over-engineering; add abstractions only when they buy clear maintainability.
- Keep boundaries explicit between CLI, core runtime, server, desktop shell, and PWA surfaces.
- Prefer safe implementations; justify `unsafe` usage explicitly if ever required.
- For Rust changes, use idiomatic patterns (`Result`/`Option`, iterators, trait-based composition) and keep `anyhow::Context` on fallible IO/network boundaries.
- Preserve TUI responsiveness: avoid render-loop allocations, cache expensive calculations, and keep scrolling/streaming smooth.

## Quality Gates
- Rust workspace: `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --all -- --check`
- Server local run: `cargo run -p krusty-server`
- PWA: `cd apps/pwa/app && bun run check && bun run build`
- Desktop shell: `cd apps/desktop/shell && cargo check --manifest-path src-tauri/Cargo.toml`

## Structure Map
### Subdirectories
- `tui/`: Ratatui interface implementation with state, handlers, and renderers. See `crates/krusty-cli/src/tui/AGENTS.md` for local detail.

### Files
- `main.rs`: Binary entrypoint that boots this crate or application runtime.
- `serve.rs`: Rust source module implementing serve behavior.
