# AGENTS Guide: /crates

## Purpose
Rust workspace crates for CLI, core runtime, and server.

## Guardrails
- Preserve crate boundaries:
  - `krusty-cli`: terminal UX
  - `krusty-core`: shared runtime
  - `krusty-server`: HTTP API
- Move shared logic to `krusty-core`; avoid duplication across CLI/server.
- Keep public APIs deliberate and documented.

## Validation
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo fmt --all -- --check`
