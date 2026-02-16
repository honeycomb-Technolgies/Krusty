# Desktop Shell (Tauri)

This wraps the PWA app surface as a native desktop app.

## Dev Flow
1. Run desktop shell (`bun run dev`) in this folder.
2. Tauri will automatically run the PWA dev server from `apps/pwa/app`.

The shell loads `http://localhost:5173` during development.

## Build Flow
1. Build desktop binary: `bun run build` in this folder.
2. Tauri will automatically run the PWA build first.

`tauri.conf.json` points `frontendDist` at `../../pwa/app/build`.

## Bundle Notes
- `bun run build` creates Linux `.deb` and `.rpm` bundles by default.
- `bun run build:all` attempts all bundle formats.
- `bun run build:appimage` builds AppImage only (requires host `linuxdeploy` support).
