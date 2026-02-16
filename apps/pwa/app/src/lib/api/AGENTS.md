# AGENTS Guide: /apps/pwa/app/src/lib/api

## Purpose
Typed browser client for server HTTP/SSE endpoints.

## Guardrails
- Treat interface types here as API contracts.
- Keep endpoint names and response shapes aligned with `crates/krusty-server/src/routes`.
- Preserve backwards compatibility for active clients when possible.

## Validation
- `cd apps/pwa/app && bun run check`
