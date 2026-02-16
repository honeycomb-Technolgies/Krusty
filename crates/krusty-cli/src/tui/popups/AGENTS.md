# AGENTS Guide: /crates/krusty-cli/src/tui/popups

## Scope
- Applies to `/crates/krusty-cli/src/tui/popups` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Modal popup implementations and shared popup utilities.

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
- `auth.rs`: Rust source module implementing auth behavior.
- `common.rs`: Rust source module implementing common behavior.
- `file_preview.rs`: Rust source module implementing file preview behavior.
- `help.rs`: Rust source module implementing help behavior.
- `hooks.rs`: Rust source module implementing hooks behavior.
- `mcp_browser.rs`: Rust source module implementing mcp browser behavior.
- `mod.rs`: Module root that wires child modules and shared exports.
- `model_select.rs`: Rust source module implementing model select behavior.
- `pinch.rs`: Rust source module implementing pinch behavior.
- `process_list.rs`: Rust source module implementing process list behavior.
- `scroll.rs`: Rust source module implementing scroll behavior.
- `session_list.rs`: Rust source module implementing session list behavior.
- `skills_browser.rs`: Rust source module implementing skills browser behavior.
- `theme_select.rs`: Rust source module implementing theme select behavior.
