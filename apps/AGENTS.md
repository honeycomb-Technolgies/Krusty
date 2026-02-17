# AGENTS Guide: /apps

## Purpose
All user-facing application surfaces.

## Guardrails
- Preserve strict separation between app surfaces and core runtime internals.
- Do not duplicate business logic that already exists in `krusty-core` or `krusty-server`.
- Keep desktop and PWA behavior aligned where features overlap.

## Directory Notes
- `desktop/`: Tauri shell for desktop distribution.
- `pwa/`: installable web client.
- `marketing/`: static pages only.
