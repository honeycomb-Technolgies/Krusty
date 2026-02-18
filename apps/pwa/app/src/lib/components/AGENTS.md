# AGENTS Guide: /apps/pwa/app/src/lib/components

## Purpose
Reusable UI components.

## Guardrails
- Components should be focused and composable.
- Keep side effects in stores/lib utilities, not deeply inside view code.
- Prefer explicit props/events over implicit global dependencies.
- Keep chat controls truthful to active runtime state (selected model, streaming, transcription).
- Mobile-first interactions should be tap-safe and avoid long-press gesture conflicts with text selection.

## Validation
- `cd apps/pwa/app && bun run check`
