# AGENTS Guide: /crates/krusty-core/src/auth

## Scope
- Applies to `/crates/krusty-core/src/auth` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Authentication flows, provider adapters, and credential handling.

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
- `providers/`: Provider-specific authentication implementations. See `crates/krusty-core/src/auth/providers/AGENTS.md` for local detail.

### Files
- `browser_flow.rs`: Rust source module implementing browser flow behavior.
- `device_flow.rs`: Rust source module implementing device flow behavior.
- `mod.rs`: Module root that wires child modules and shared exports.
- `pkce.rs`: Rust source module implementing pkce behavior.
- `storage.rs`: Rust source module implementing storage behavior.
- `types.rs`: Rust source module implementing types behavior.
