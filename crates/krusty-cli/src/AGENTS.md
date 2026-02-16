# AGENTS Guide: /crates/krusty-cli/src

## Purpose
CLI source modules and command/TUI wiring.

## Guardrails
- Keep command-layer orchestration readable and explicit.
- Isolate terminal rendering concerns from command execution logic.
- Prefer small modules with clear ownership.

## Validation
- `cargo check -p krusty`
