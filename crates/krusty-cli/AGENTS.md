# AGENTS Guide: /crates/krusty-cli

## Purpose
Terminal client crate.

## Guardrails
- Maintain responsive TUI behavior under streaming load.
- Keep terminal UI concerns in CLI crate; do not re-implement core runtime logic.
- Preserve compatibility with server/core protocol changes.

## Validation
- `cargo check -p krusty`
