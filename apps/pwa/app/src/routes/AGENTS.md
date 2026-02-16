# AGENTS Guide: /apps/pwa/app/src/routes

## Scope
- Applies to `/apps/pwa/app/src/routes` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
SvelteKit route entrypoints for top-level pages.

## Local Standards
- Deliver best-in-class quality: elegant, modular, organized, and performant code.
- Keep code self-explanatory; add comments only for non-obvious constraints or tradeoffs.
- Avoid over-engineering; add abstractions only when they buy clear maintainability.
- Keep boundaries explicit between CLI, core runtime, server, desktop shell, and PWA surfaces.
- Prefer safe implementations; justify `unsafe` usage explicitly if ever required.
- Keep Svelte components focused and colocate cross-view state in `src/lib/stores`.
- Do not introduce billing/account coupling into the PWA runtime surface.
- Preserve startup orchestration in `+layout.svelte`: service worker registration, notification click handling, and push subscription reconciliation.

## Quality Gates
- Rust workspace: `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --all -- --check`
- Server local run: `cargo run -p krusty-server`
- PWA: `cd apps/pwa/app && bun run check && bun run build`
- Desktop shell: `cd apps/desktop/shell && cargo check --manifest-path src-tauri/Cargo.toml`

## Structure Map
### Subdirectories
- `app/`: Route for the primary chat application view. See `apps/pwa/app/src/routes/app/AGENTS.md` for local detail.
- `ide/`: Route for IDE view mode. See `apps/pwa/app/src/routes/ide/AGENTS.md` for local detail.
- `menu/`: Route for settings/menu view mode. See `apps/pwa/app/src/routes/menu/AGENTS.md` for local detail.
- `terminal/`: Route for terminal-centric view mode. See `apps/pwa/app/src/routes/terminal/AGENTS.md` for local detail.
- `workspace/`: Route for workspace-focused view mode. See `apps/pwa/app/src/routes/workspace/AGENTS.md` for local detail.

### Files
- `+layout.svelte`: Route layout that initializes workspace/session wiring and service worker + push reconciliation behavior.
- `+page.svelte`: SvelteKit route component for page view rendering.
