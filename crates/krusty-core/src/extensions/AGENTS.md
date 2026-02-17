# AGENTS Guide: /crates/krusty-core/src/extensions

## Purpose
Extension runtime and WIT integration.

## Guardrails
- Treat WIT and extension host changes as ABI-sensitive.
- Keep manifest parsing strict and error messages actionable.
- Preserve compatibility rules across extension API versions.

## Validation
- `cargo check -p krusty-core`
- run extension host tests for host/runtime contract changes.
