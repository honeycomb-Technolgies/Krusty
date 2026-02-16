# AGENTS Guide: /apps/desktop/shell

## Scope
- Applies to `/apps/desktop/shell` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Tauri wrapper that hosts the PWA as a desktop application.

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
- `src-tauri/`: Rust/Tauri configuration and bootstrap code for the desktop shell. See `apps/desktop/shell/src-tauri/AGENTS.md` for local detail.

### Files
- `.gitignore`: Ignore rules for generated files and local-only artifacts.
- `README.md`: Human-facing documentation for this directory's purpose and workflows.
- `bun.lock`: Bun lockfile ensuring deterministic JavaScript dependency installs.
- `package.json`: Node/Bun manifest with scripts, dependencies, and package metadata.
