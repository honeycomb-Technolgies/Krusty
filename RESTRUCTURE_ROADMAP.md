# Reorganization Roadmap

This roadmap keeps the product split clean for self-hosting and future SaaS additions.

## Step 1: Server Baseline in `krusty-public` (Completed)
- Added `crates/krusty-server` and wired it into the workspace.
- Kept only self-host relevant APIs: sessions, chat, tools, files, models, credentials, hooks, processes, MCP.
- Removed Kubernetes/SaaS-only server modules and non-compiling legacy imports.
- Verified with `cargo check --offline --workspace`.

## Step 2: App Boundaries (Completed)
- Added explicit targets under `apps/`: `pwa`, `desktop`, `marketing`.
- Added app boundary docs in each `apps/*/README.md`.

## Step 3: Frontend Split Execution (Completed)
- Imported the active chat runtime into `apps/pwa/app`.
- Removed billing/account/server-auth coupling from active PWA surface.
- Switched PWA to static adapter SPA fallback and added service worker registration.
- Extracted landing/legal pages into `apps/marketing/site` (static-only).
- Removed legacy snapshot directories from the public repo tree.

## Step 4: Desktop Wrapper (Completed)
- Added Tauri desktop shell scaffold in `apps/desktop/shell`.
- Configured shell to use PWA dev URL in development and `apps/pwa/app/build` for packaged builds.
- Wired Tauri `beforeDevCommand`/`beforeBuildCommand` to run PWA automatically.
- Isolated shell Rust manifest from workspace resolution conflicts.

## Step 5: Cleanup and Cutover (In Progress)
- Added CI split jobs for Rust workspace, PWA check/build, and marketing static smoke checks.
- Documented self-host run path and boundaries in root `README.md` and `crates/krusty-server/README.md`.
- Remaining: final release packaging + distribution checks for desktop binaries.
