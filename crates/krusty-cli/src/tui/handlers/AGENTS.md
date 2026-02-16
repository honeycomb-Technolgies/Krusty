# AGENTS Guide: /crates/krusty-cli/src/tui/handlers

## Purpose
TUI event handlers and stream processing.

## Guardrails
- Keep keyboard/mouse/render handling deterministic.
- Handle partial stream events safely; never panic on malformed chunks.
- Keep session/tool side effects explicit and traceable.

## Validation
- `cargo check -p krusty`
