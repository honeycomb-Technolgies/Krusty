# AGENTS Guide: /apps/pwa

## Purpose
Installable web client workspace.

## Guardrails
- PWA is the primary UX surface; prioritize robustness and latency.
- Keep offline/service-worker behavior predictable.
- Do not couple UI state to billing/account internals.

## Validation
- `cd apps/pwa/app && bun run check && bun run build`
