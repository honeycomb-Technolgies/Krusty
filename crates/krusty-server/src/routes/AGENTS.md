# AGENTS Guide: /crates/krusty-server/src/routes

## Purpose
HTTP route handlers and endpoint contracts.

## Guardrails
- Keep request/response shapes synchronized with CLI/PWA clients.
- Validate and sanitize all user inputs before side effects.
- Preserve streaming route stability and backpressure behavior.
- Push endpoints (`/push/*`) must stay aligned with PWA diagnostics and test-send flows.

## Validation
- `cargo check -p krusty-server`
- test affected endpoints from client code paths when contracts change.
