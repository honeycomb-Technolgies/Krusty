# AGENTS Guide: /crates/krusty-core/src/tools

## Purpose
Tool registry, interfaces, and implementations used by agent runtime.

## Guardrails
- Tool argument parsing and error surfaces are user-facing contracts.
- Keep permission/approval semantics explicit and conservative.
- Avoid hidden filesystem/network side effects in tool implementations.

## Validation
- `cargo check -p krusty-core`
- run targeted tool registry/implementation tests when changing tool behavior.
