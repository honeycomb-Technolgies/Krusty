# AGENTS Guide: /apps/pwa/app/src/lib

## Scope
- Applies to `/apps/pwa/app/src/lib` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Shared frontend modules, stores, APIs, and reusable UI components.

## Local Standards
- Deliver best-in-class quality: elegant, modular, organized, and performant code.
- Keep code self-explanatory; add comments only for non-obvious constraints or tradeoffs.
- Avoid over-engineering; add abstractions only when they buy clear maintainability.
- Keep boundaries explicit between CLI, core runtime, server, desktop shell, and PWA surfaces.
- Prefer safe implementations; justify `unsafe` usage explicitly if ever required.
- Keep Svelte components focused and colocate cross-view state in `src/lib/stores`.
- Do not introduce billing/account coupling into the PWA runtime surface.
- Keep push subscription truth sourced from service worker state (`push.ts` reconciliation) rather than localStorage-only assumptions.

## Quality Gates
- Rust workspace: `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --all -- --check`
- Server local run: `cargo run -p krusty-server`
- PWA: `cd apps/pwa/app && bun run check && bun run build`
- Desktop shell: `cd apps/desktop/shell && cargo check --manifest-path src-tauri/Cargo.toml`

## Structure Map
### Subdirectories
- `api/`: Browser API client wrappers for backend communication. See `apps/pwa/app/src/lib/api/AGENTS.md` for local detail.
- `components/`: Reusable Svelte components grouped by feature area. See `apps/pwa/app/src/lib/components/AGENTS.md` for local detail.
- `stores/`: Svelte stores for app state, sessions, plans, git, and terminal data. See `apps/pwa/app/src/lib/stores/AGENTS.md` for local detail.

### Files
- `push.ts`: Push subscription lifecycle logic, startup reconciliation, and server synchronization helpers.
