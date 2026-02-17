# AGENTS Guide: /apps/pwa/app/src/lib

## Purpose
Shared frontend logic: API access, stores, reusable components, and push lifecycle.

## Guardrails
- Keep push subscription truth based on browser/service-worker state, not local hints alone.
- Keep API request/response types synchronized with server route contracts.
- Avoid cross-component hidden state; use stores for shared state.

## Validation
- `cd apps/pwa/app && bun run check`
