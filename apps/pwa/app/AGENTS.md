# AGENTS Guide: /apps/pwa/app

## Purpose
SvelteKit app for chat, terminal, IDE, and workspace flows.

## Guardrails
- Keep route-level composition simple; push shared logic into `src/lib`.
- Maintain clear API contracts with `krusty-server`.
- Preserve PWA installability and service worker correctness.

## Validation
- `cd apps/pwa/app && bun run check`
- `cd apps/pwa/app && bun run build`
