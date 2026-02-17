# AGENTS Guide: /crates/krusty-server/src

## Purpose
Server source modules, route wiring, auth, and push dispatch.

## Guardrails
- Keep route handlers thin; push shared logic into core services where appropriate.
- Treat push dispatch as reliability-critical.
- Keep auth checks explicit at route boundaries.

## Key Local Guide
- `routes/AGENTS.md`
