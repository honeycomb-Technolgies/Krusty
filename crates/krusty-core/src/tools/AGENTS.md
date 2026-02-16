# AGENTS Guide: /crates/krusty-core/src/tools

## Scope
- Applies to `/crates/krusty-core/src/tools` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Tool trait abstractions, registry, and shared path/tool utilities.

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
- `implementations/`: Concrete tool implementations used by the agent runtime. See `crates/krusty-core/src/tools/implementations/AGENTS.md` for local detail.

### Files
- `git_identity.rs`: Rust source module implementing git identity behavior.
- `image.rs`: Rust source module implementing image behavior.
- `mod.rs`: Module root that wires child modules and shared exports.
- `path_utils.rs`: Rust source module implementing path utils behavior.
- `registry.rs`: Rust source module implementing registry behavior.
