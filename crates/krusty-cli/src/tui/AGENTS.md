# AGENTS Guide: /crates/krusty-cli/src/tui

## Purpose
Interactive terminal UI runtime.

## Guardrails
- Protect frame-time performance and input responsiveness.
- Avoid heavy allocations in render/event hot paths.
- Keep streaming updates idempotent and visually stable.

## Key Subsystems
- `handlers/`: event and streaming orchestration.
- `themes/`: theme model and registry.
- other subdirs inherit these rules unless they define stricter local guidance.

## Validation
- `cargo check -p krusty`
