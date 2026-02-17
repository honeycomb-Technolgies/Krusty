# AGENTS Guide: /crates/krusty-server

## Purpose
Self-host API crate.

## Guardrails
- Keep HTTP contracts stable and typed.
- Server remains self-host focused; avoid assumptions tied to one deployment platform.
- Reliability and observability changes should include logs and test coverage where practical.

## Validation
- `cargo check -p krusty-server`
- `cargo run -p krusty-server`
