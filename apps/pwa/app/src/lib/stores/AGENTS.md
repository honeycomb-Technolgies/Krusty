# AGENTS Guide: /apps/pwa/app/src/lib/stores

## Scope
- Applies to `/apps/pwa/app/src/lib/stores` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Svelte stores for app state, sessions, plans, git, and terminal data.

## Local Standards
- Deliver best-in-class quality: elegant, modular, organized, and performant code.
- Keep code self-explanatory; add comments only for non-obvious constraints or tradeoffs.
- Avoid over-engineering; add abstractions only when they buy clear maintainability.
- Keep boundaries explicit between CLI, core runtime, server, desktop shell, and PWA surfaces.
- Prefer safe implementations; justify `unsafe` usage explicitly if ever required.
- Keep Svelte components focused and colocate cross-view state in `src/lib/stores`.
- Do not introduce billing/account coupling into the PWA runtime surface.

## Quality Gates
- Rust workspace: `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --all -- --check`
- Server local run: `cargo run -p krusty-server`
- PWA: `cd apps/pwa/app && bun run check && bun run build`
- Desktop shell: `cd apps/desktop/shell && cargo check --manifest-path src-tauri/Cargo.toml`

## Structure Map
### Subdirectories
- _(none)_

### Files
- `git.ts`: Svelte store module managing git state and updates.
- `ide.ts`: Svelte store module managing ide state and updates.
- `plan.ts`: Svelte store module managing plan state and updates.
- `session.ts`: Svelte store module managing session state and updates.
- `sessions.ts`: Svelte store module managing sessions state and updates.
- `terminal.ts`: Svelte store module managing terminal state and updates.
- `workspace.ts`: Svelte store module managing workspace state and updates.
