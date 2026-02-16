# AGENTS Guide: /apps/pwa/app/src/lib/components/terminal

## Scope
- Applies to `/apps/pwa/app/src/lib/components/terminal` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Terminal emulator components and command UI controls.

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
- `QuickActions.svelte`: Reusable Svelte component implementing Quick Actions UI behavior.
- `TerminalInstance.svelte`: Reusable Svelte component implementing Terminal Instance UI behavior.
- `TerminalTabs.svelte`: Reusable Svelte component implementing Terminal Tabs UI behavior.
- `TerminalView.svelte`: Reusable Svelte component implementing Terminal View UI behavior.
