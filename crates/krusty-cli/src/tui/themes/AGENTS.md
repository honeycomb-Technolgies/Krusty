# AGENTS Guide: /crates/krusty-cli/src/tui/themes

## Purpose
Theme registry and style primitives for terminal rendering.

## Guardrails
- Keep contrast/readability strong in both dense and sparse views.
- Theme additions must update registry wiring and defaults intentionally.
- Avoid hardcoding colors outside theme primitives.

## Validation
- `cargo check -p krusty`
