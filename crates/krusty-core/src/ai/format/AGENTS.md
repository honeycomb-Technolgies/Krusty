# AGENTS Guide: /crates/krusty-core/src/ai/format

## Scope
- Applies to `/crates/krusty-core/src/ai/format` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Provider payload formatting and normalized response conversion.

## Local Standards
- Deliver best-in-class quality: elegant, modular, organized, and performant code.
- Keep code self-explanatory; add comments only for non-obvious constraints or tradeoffs.
- Avoid over-engineering; add abstractions only when they buy clear maintainability.
- Keep boundaries explicit between CLI, core runtime, server, desktop shell, and PWA surfaces.
- Prefer safe implementations; justify `unsafe` usage explicitly if ever required.
- For Rust changes, use idiomatic patterns (`Result`/`Option`, iterators, trait-based composition) and keep `anyhow::Context` on fallible IO/network boundaries.
- Keep provider/tool differences behind stable abstractions so user-facing behavior remains consistent across integrations.

## Quality Gates
- Rust workspace: `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --all -- --check`
- Server local run: `cargo run -p krusty-server`
- PWA: `cd apps/pwa/app && bun run check && bun run build`
- Desktop shell: `cd apps/desktop/shell && cargo check --manifest-path src-tauri/Cargo.toml`

## Structure Map
### Subdirectories
- _(none)_

### Files
- `anthropic.rs`: Rust source module implementing anthropic behavior.
- `google.rs`: Rust source module implementing google behavior.
- `mod.rs`: Module root that wires child modules and shared exports.
- `openai.rs`: Rust source module implementing openai behavior.
- `response.rs`: Rust source module implementing response behavior.
