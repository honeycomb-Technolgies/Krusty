# AGENTS Guide: /wit

## Scope
- Applies to `/wit` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Top-level WIT contracts shared across extension tooling.

## Local Standards
- Deliver best-in-class quality: elegant, modular, organized, and performant code.
- Keep code self-explanatory; add comments only for non-obvious constraints or tradeoffs.
- Avoid over-engineering; add abstractions only when they buy clear maintainability.
- Keep boundaries explicit between CLI, core runtime, server, desktop shell, and PWA surfaces.
- Prefer safe implementations; justify `unsafe` usage explicitly if ever required.

## Quality Gates
- Rust workspace: `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --all -- --check`
- Server local run: `cargo run -p krusty-server`
- PWA: `cd apps/pwa/app && bun run check && bun run build`
- Desktop shell: `cd apps/desktop/shell && cargo check --manifest-path src-tauri/Cargo.toml`

## Structure Map
### Subdirectories
- _(none)_

### Files
- `common.wit`: WIT file defining common interface boundaries.
- `extension.wit`: WIT file defining extension interface boundaries.
- `github.wit`: WIT file defining github interface boundaries.
- `http-client.wit`: WIT file defining http client interface boundaries.
- `nodejs.wit`: WIT file defining nodejs interface boundaries.
- `platform.wit`: WIT file defining platform interface boundaries.
- `settings.rs`: Rust source module implementing settings behavior.
- `slash-command.wit`: WIT file defining slash command interface boundaries.
