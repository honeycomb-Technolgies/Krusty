# AGENTS Guide: /crates/krusty-cli/src/tui/handlers/popup_keys

## Scope
- Applies to `/crates/krusty-cli/src/tui/handlers/popup_keys` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Popup-specific keybinding handlers.

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
- `auth.rs`: Event-handling module implementing auth behavior.
- `file_preview.rs`: Event-handling module implementing file preview behavior.
- `hooks.rs`: Event-handling module implementing hooks behavior.
- `mcp.rs`: Event-handling module implementing mcp behavior.
- `mod.rs`: Module root that wires child modules and shared exports.
- `pinch.rs`: Event-handling module implementing pinch behavior.
- `process.rs`: Event-handling module implementing process behavior.
- `skills.rs`: Event-handling module implementing skills behavior.
