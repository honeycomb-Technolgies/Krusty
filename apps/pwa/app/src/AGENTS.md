# AGENTS Guide: /apps/pwa/app/src

## Purpose
Application source tree.

## Guardrails
- `routes/` should compose views, not own shared business rules.
- `lib/` owns reusable logic, API clients, components, and stores.
- Keep service worker behavior explicit and auditable.

## Validation
- `cd apps/pwa/app && bun run check`
