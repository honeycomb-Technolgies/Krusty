# AGENTS Guide: /apps/pwa/app/src

## Scope
- Applies to `/apps/pwa/app/src` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Primary source tree for SvelteKit application logic and UI.

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
- `lib/`: Shared frontend modules, stores, APIs, and reusable UI components. See `apps/pwa/app/src/lib/AGENTS.md` for local detail.
- `routes/`: SvelteKit route entrypoints for top-level pages. See `apps/pwa/app/src/routes/AGENTS.md` for local detail.

### Files
- `app.css`: Global stylesheet and theme tokens for the PWA interface.
- `app.d.ts`: Ambient TypeScript declarations for the Svelte app.
- `app.html`: Root HTML shell template for SvelteKit rendering.
- `service-worker.ts`: Service worker entry handling offline caching and update behavior.
