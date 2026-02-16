# AGENTS Guide: /apps/desktop/shell/src-tauri

## Scope
- Applies to `/apps/desktop/shell/src-tauri` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Rust/Tauri configuration and bootstrap code for the desktop shell.

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
- `icons/`: Desktop application icon assets. See `apps/desktop/shell/src-tauri/icons/AGENTS.md` for local detail.
- `src/`: Rust source for desktop shell startup and bindings. See `apps/desktop/shell/src-tauri/src/AGENTS.md` for local detail.

### Files
- `Cargo.lock`: Pinned Rust dependency lockfile for reproducible builds.
- `Cargo.toml`: Crate manifest declaring package metadata, dependencies, and build settings.
- `build.rs`: Build script configuring desktop shell build-time behavior.
- `tauri.conf.json`: Tauri app configuration (windows, bundling, permissions, metadata).
