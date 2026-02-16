# AGENTS Guide: /apps/desktop

## Purpose
Desktop delivery layer for Krusty.

## Guardrails
- Desktop shell is a host for the PWA, not a second product surface.
- Keep desktop-specific code focused on windowing, permissions, and packaging.
- Avoid introducing runtime behavior that diverges from server/PWA contracts.

## Validation
- `cd apps/desktop/shell && cargo check --manifest-path src-tauri/Cargo.toml`
