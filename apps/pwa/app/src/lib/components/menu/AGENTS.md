# AGENTS Guide: /apps/pwa/app/src/lib/components/menu

## Scope
- Applies to `/apps/pwa/app/src/lib/components/menu` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Settings and control panel components, including push notification controls and diagnostics.

## Local Standards
- Deliver best-in-class quality: elegant, modular, organized, and performant code.
- Keep code self-explanatory; add comments only for non-obvious constraints or tradeoffs.
- Avoid over-engineering; add abstractions only when they buy clear maintainability.
- Keep boundaries explicit between CLI, core runtime, server, desktop shell, and PWA surfaces.
- Prefer safe implementations; justify `unsafe` usage explicitly if ever required.
- Keep Svelte components focused and colocate cross-view state in `src/lib/stores`.
- Do not introduce billing/account coupling into the PWA runtime surface.
- Keep Settings push state authoritative to browser/service-worker subscription state, and surface server diagnostics without blocking core menu flows.

## Quality Gates
- Rust workspace: `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --all -- --check`
- Server local run: `cargo run -p krusty-server`
- PWA: `cd apps/pwa/app && bun run check && bun run build`
- Desktop shell: `cd apps/desktop/shell && cargo check --manifest-path src-tauri/Cargo.toml`

## Structure Map
### Subdirectories
- _(none)_

### Files
- `McpView.svelte`: Reusable Svelte component implementing Mcp View UI behavior.
- `MenuView.svelte`: Reusable Svelte component implementing Menu View UI behavior.
- `ProcessesView.svelte`: Reusable Svelte component implementing Processes View UI behavior.
- `ProvidersView.svelte`: Reusable Svelte component implementing Providers View UI behavior.
- `SessionList.svelte`: Reusable Svelte component implementing Session List UI behavior.
- `Settings.svelte`: Reusable Svelte component for notification toggles, push diagnostics, and test-notification actions.
