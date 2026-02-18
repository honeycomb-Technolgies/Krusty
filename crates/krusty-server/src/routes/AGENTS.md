# AGENTS Guide: /crates/krusty-server/src/routes

## Purpose
HTTP route handlers and endpoint contracts.

## Guardrails
- Keep request/response shapes synchronized with CLI/PWA clients.
- Validate and sanitize all user inputs before side effects.
- Preserve streaming route stability and backpressure behavior.
- Chat routes must honor persisted session model unless an explicit per-request override is provided.
- Push endpoints (`/push/*`) must stay aligned with PWA diagnostics and test-send flows.
- Port proxy endpoints (`/ports/*`) must remain localhost-scoped and deny recursive self-proxy loops.

## Validation
- `cargo check -p krusty-server`
- test affected endpoints from client code paths when contracts change.
