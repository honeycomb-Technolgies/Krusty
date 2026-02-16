# AGENTS Guide: /crates/krusty-cli/src/tui/components

## Scope
- Applies to `/crates/krusty-cli/src/tui/components` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Reusable terminal widgets (toolbar, status, prompts, sidebars).

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
- `decision_prompt.rs`: Rust source module implementing decision prompt behavior.
- `mod.rs`: Module root that wires child modules and shared exports.
- `plan_sidebar.rs`: Rust source module implementing plan sidebar behavior.
- `plugin_window.rs`: Rust source module implementing plugin window behavior.
- `scrollbars.rs`: Rust source module implementing scrollbars behavior.
- `status_bar.rs`: Rust source module implementing status bar behavior.
- `toast.rs`: Rust source module implementing toast behavior.
- `toolbar.rs`: Rust source module implementing toolbar behavior.
