# AGENTS Guide: /apps/pwa/app/src/lib/components/menu

## Purpose
Menu/settings surface for operational controls and diagnostics.

## Guardrails
- Keep push notification controls aligned with real browser subscription state.
- Keep diagnostics actionable: include status and clear failure messaging.
- Avoid blocking primary app use if diagnostics endpoints fail.

## Validation
- `cd apps/pwa/app && bun run check`
