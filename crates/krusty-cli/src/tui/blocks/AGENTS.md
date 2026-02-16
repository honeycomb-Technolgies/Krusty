# AGENTS Guide: /crates/krusty-cli/src/tui/blocks

## Scope
- Applies to `/crates/krusty-cli/src/tui/blocks` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Renderable stream block implementations for chat/tool output.

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
- `edit/`: Specialized edit block state and rendering internals. See `crates/krusty-cli/src/tui/blocks/edit/AGENTS.md` for local detail.

### Files
- `bash.rs`: TUI stream block module responsible for bash rendering and interaction.
- `build.rs`: Cargo build script executed before compilation.
- `explore.rs`: TUI stream block module responsible for explore rendering and interaction.
- `mod.rs`: Module root that wires child modules and shared exports.
- `read.rs`: TUI stream block module responsible for read rendering and interaction.
- `terminal_pane.rs`: TUI stream block module responsible for terminal pane rendering and interaction.
- `thinking.rs`: TUI stream block module responsible for thinking rendering and interaction.
- `tool_result.rs`: TUI stream block module responsible for tool result rendering and interaction.
- `web_search.rs`: TUI stream block module responsible for web search rendering and interaction.
- `write.rs`: TUI stream block module responsible for write rendering and interaction.
