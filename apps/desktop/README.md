# Desktop App

This directory is reserved for desktop packaging (Tauri/Electron) around the same app experience as PWA.

## Scope
- Native wrapper and packaging
- OS integration (notifications, file dialogs, autostart)
- Local connection config to self-hosted `krusty-server`

## Active Scaffold
- `shell/` contains a Tauri wrapper targeting `apps/pwa/app`.

## Local Run

```bash
cd apps/desktop/shell
npm ci
npm run dev
```

Build:

```bash
npm run build
```
