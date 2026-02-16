# AGENTS Guide: /crates/krusty-cli/src/tui/animation

## Scope
- Applies to `/crates/krusty-cli/src/tui/animation` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Animation systems used by menu and visual polish effects.

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
- `menu/`: Menu animation primitives (bubbles, crab, field behavior). See `crates/krusty-cli/src/tui/animation/menu/AGENTS.md` for local detail.

### Files
- `mod.rs`: Module root that wires child modules and shared exports.
