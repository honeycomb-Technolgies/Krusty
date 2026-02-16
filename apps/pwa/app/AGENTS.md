# AGENTS Guide: /apps/pwa/app

## Scope
- Applies to `/apps/pwa/app` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Installable PWA chat/workspace client (active web surface).

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
- `src/`: Primary source tree for SvelteKit application logic and UI. See `apps/pwa/app/src/AGENTS.md` for local detail.
- `static/`: Static PWA assets (icons, manifest, favicon). See `apps/pwa/app/static/AGENTS.md` for local detail.

### Files
- `.env.example`: Example environment variables for local PWA development.
- `.gitignore`: Ignore rules for generated files and local-only artifacts.
- `STYLE_GUIDE.md`: UI style and interaction standards for the PWA surface.
- `bun.lock`: Bun lockfile ensuring deterministic JavaScript dependency installs.
- `package.json`: Node/Bun manifest with scripts, dependencies, and package metadata.
- `postcss.config.js`: PostCSS plugin pipeline configuration for frontend styles.
- `svelte.config.js`: SvelteKit configuration for adapter and compiler behavior.
- `tailwind.config.js`: Tailwind configuration for content scanning and design tokens.
- `tsconfig.json`: TypeScript compiler options and path/type settings.
- `vite.config.ts`: Vite build and dev-server configuration for the frontend app.
