# AGENTS Guide: /apps/pwa/app/src/lib/components/ide

## Scope
- Applies to `/apps/pwa/app/src/lib/components/ide` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
IDE-style workspace components (editor, tree, symbol navigation).

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
- `DirectoryPicker.svelte`: Reusable Svelte component implementing Directory Picker UI behavior.
- `Editor.svelte`: Reusable Svelte component implementing Editor UI behavior.
- `FileTree.svelte`: Reusable Svelte component implementing File Tree UI behavior.
- `IDEView.svelte`: Reusable Svelte component implementing IDEView UI behavior.
- `SymbolBar.svelte`: Reusable Svelte component implementing Symbol Bar UI behavior.
