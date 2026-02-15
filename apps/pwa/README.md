# PWA App

This directory is the future open-source mobile/desktop web app shell for chat.

## Active App
- `app/` is the active PWA target.

## Scope
- Chat UI and session management
- PWA install/offline behavior
- API client for `krusty-server` (`/api/*`, `/ws/*` when enabled)

## Rules
- Do not add billing, account, or marketing pages here.
- Keep all auth optional for self-host use.
- Use environment-driven API base URL (no Cloudflare-only assumptions).

## Local Run

```bash
cd apps/pwa/app
npm ci
npm run dev
```

Build:

```bash
npm run build
```
