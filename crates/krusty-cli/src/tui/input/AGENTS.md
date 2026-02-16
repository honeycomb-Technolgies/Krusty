# AGENTS Guide: /crates/krusty-cli/src/tui/input

## Scope
- Applies to `/crates/krusty-cli/src/tui/input` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Input subsystem including autocomplete, parsing, and multi-line editing.

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
- `multi_line/`: Multi-line editor implementation used by the chat composer. See `crates/krusty-cli/src/tui/input/multi_line/AGENTS.md` for local detail.

### Files
- `autocomplete.rs`: Rust source module implementing autocomplete behavior.
- `file_search.rs`: Rust source module implementing file search behavior.
- `image_parser.rs`: Rust source module implementing image parser behavior.
- `mod.rs`: Module root that wires child modules and shared exports.
