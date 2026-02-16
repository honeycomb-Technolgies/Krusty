# AGENTS Guide: /crates/krusty-cli/src/tui/handlers

## Scope
- Applies to `/crates/krusty-cli/src/tui/handlers` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Event handlers orchestrating keyboard, mouse, render, and streams.

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
- `popup_keys/`: Popup-specific keybinding handlers. See `crates/krusty-cli/src/tui/handlers/popup_keys/AGENTS.md` for local detail.
- `rendering/`: Render pipeline orchestration and viewport calculations. See `crates/krusty-cli/src/tui/handlers/rendering/AGENTS.md` for local detail.
- `streaming/`: Streaming event parsing and tool execution integration. See `crates/krusty-cli/src/tui/handlers/streaming/AGENTS.md` for local detail.

### Files
- `commands.rs`: Event-handling module implementing commands behavior.
- `event_loop.rs`: Event-handling module implementing event loop behavior.
- `hit_test.rs`: Event-handling module implementing hit test behavior.
- `keyboard.rs`: Event-handling module implementing keyboard behavior.
- `mod.rs`: Module root that wires child modules and shared exports.
- `models.rs`: Event-handling module implementing models behavior.
- `mouse.rs`: Event-handling module implementing mouse behavior.
- `pinch.rs`: Event-handling module implementing pinch behavior.
- `provider.rs`: Event-handling module implementing provider behavior.
- `scrollbar.rs`: Event-handling module implementing scrollbar behavior.
- `selection.rs`: Event-handling module implementing selection behavior.
- `sessions.rs`: Event-handling module implementing sessions behavior.
- `stream_events.rs`: Event-handling module implementing stream events behavior.
- `terminal.rs`: Event-handling module implementing terminal behavior.
- `themes.rs`: Event-handling module implementing themes behavior.
- `update.rs`: Event-handling module implementing update behavior.
