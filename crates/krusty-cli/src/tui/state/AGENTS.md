# AGENTS Guide: /crates/krusty-cli/src/tui/state

## Scope
- Applies to `/crates/krusty-cli/src/tui/state` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Centralized UI state models and scroll/selection coordination.

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
- _(none)_

### Files
- `blocks.rs`: Rust source module implementing blocks behavior.
- `chat.rs`: Rust source module implementing chat behavior.
- `hover.rs`: Rust source module implementing hover behavior.
- `indices.rs`: Rust source module implementing indices behavior.
- `layout.rs`: Rust source module implementing layout behavior.
- `mod.rs`: Module root that wires child modules and shared exports.
- `popups.rs`: Rust source module implementing popups behavior.
- `scroll.rs`: Rust source module implementing scroll behavior.
- `scroll_system.rs`: Rust source module implementing scroll system behavior.
- `selection.rs`: Rust source module implementing selection behavior.
- `ui_state.rs`: Rust source module implementing ui state behavior.
