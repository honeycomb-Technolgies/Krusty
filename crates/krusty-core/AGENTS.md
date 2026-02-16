# AGENTS Guide: /crates/krusty-core

## Purpose
Shared runtime crate used by CLI, server, and clients.

## Guardrails
- Keep module APIs stable and provider-agnostic where possible.
- Centralize shared business logic here instead of duplicating elsewhere.
- Treat persistence and protocol changes as compatibility-sensitive.

## Validation
- `cargo check -p krusty-core`
- `cargo test -p krusty-core`
