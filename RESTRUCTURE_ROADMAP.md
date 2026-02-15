# Reorganization Roadmap

This roadmap tracks the product restructure into a single-binary, self-hosted architecture.

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

## Step 5: Unified Architecture (Completed)
- Converted `krusty-server` from standalone binary to library crate.
- PWA frontend embedded in server binary via `rust-embed` with SPA fallback.
- Added `krusty serve` CLI subcommand with first-run setup wizard.
- Desktop app starts server internally or reuses existing instance via PID file detection.
- Added Tailscale module for automatic HTTPS on tailnet.
- Switched frontend tooling from npm to bun.
- Marketing site updated: removed "Future Managed" tier, positioned as distribution + docs only.

## Step 6: CI and Distribution (In Progress)
- CI split jobs for Rust workspace, PWA check/build, and marketing static smoke checks.
- Documented self-host run path and boundaries in root `README.md` and `crates/krusty-server/README.md`.
- Remaining: final release packaging + distribution checks for desktop binaries.

## Architecture Summary

```
krusty              CLI binary (TUI, serve, acp)
├── krusty serve    Starts API server with embedded PWA
├── krusty acp      Editor integration via ACP protocol
└── (default)       Terminal UI

krusty-server       Library crate (no binary)
├── Axum routes     Sessions, chat, tools, files, models
├── PWA assets      Embedded via rust-embed at compile time
└── SPA fallback    index.html for client-side routing

krusty-core         Shared library
├── AI providers    MiniMax, OpenAI, Z.AI, OpenRouter
├── Tools           File ops, shell, search, etc.
├── Tailscale       Device detection, serve, URL resolution
├── Instance mgmt   PID file + health check
└── Storage         SQLite at ~/.krusty/krusty.db

Desktop app         Tauri wrapper
└── Starts server   Or reuses running instance
```

No cloud tier. No managed hosting. Single binary, bring your own API key.
