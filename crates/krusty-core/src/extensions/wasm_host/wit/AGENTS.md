# AGENTS Guide: /crates/krusty-core/src/extensions/wasm_host/wit

## Scope
- Applies to `/crates/krusty-core/src/extensions/wasm_host/wit` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Generated and maintained Rust bindings for WIT extension interfaces.

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
- `mod.rs`: Module root that wires child modules and shared exports.
- `since_v0_0_1.rs`: Rust source module implementing since v0 0 1 behavior.
- `since_v0_0_4.rs`: Rust source module implementing since v0 0 4 behavior.
- `since_v0_0_6.rs`: Rust source module implementing since v0 0 6 behavior.
- `since_v0_1_0.rs`: Rust source module implementing since v0 1 0 behavior.
- `since_v0_2_0.rs`: Rust source module implementing since v0 2 0 behavior.
- `since_v0_3_0.rs`: Rust source module implementing since v0 3 0 behavior.
- `since_v0_4_0.rs`: Rust source module implementing since v0 4 0 behavior.
- `since_v0_5_0.rs`: Rust source module implementing since v0 5 0 behavior.
- `since_v0_6_0.rs`: Rust source module implementing since v0 6 0 behavior.
- `since_v0_8_0.rs`: Rust source module implementing since v0 8 0 behavior.
