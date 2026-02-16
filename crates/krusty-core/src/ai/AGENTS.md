# AGENTS Guide: /crates/krusty-core/src/ai

## Scope
- Applies to `/crates/krusty-core/src/ai` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Provider abstraction, streaming, parsing, and AI response normalization.

## Local Standards
- Deliver best-in-class quality: elegant, modular, organized, and performant code.
- Keep code self-explanatory; add comments only for non-obvious constraints or tradeoffs.
- Avoid over-engineering; add abstractions only when they buy clear maintainability.
- Keep boundaries explicit between CLI, core runtime, server, desktop shell, and PWA surfaces.
- Prefer safe implementations; justify `unsafe` usage explicitly if ever required.
- For Rust changes, use idiomatic patterns (`Result`/`Option`, iterators, trait-based composition) and keep `anyhow::Context` on fallible IO/network boundaries.
- Keep provider/tool differences behind stable abstractions so user-facing behavior remains consistent across integrations.

## Quality Gates
- Rust workspace: `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --all -- --check`
- Server local run: `cargo run -p krusty-server`
- PWA: `cd apps/pwa/app && bun run check && bun run build`
- Desktop shell: `cd apps/desktop/shell && cargo check --manifest-path src-tauri/Cargo.toml`

## Structure Map
### Subdirectories
- `client/`: AI client request/streaming orchestration and provider-specific adapters. See `crates/krusty-core/src/ai/client/AGENTS.md` for local detail.
- `format/`: Provider payload formatting and normalized response conversion. See `crates/krusty-core/src/ai/format/AGENTS.md` for local detail.
- `parsers/`: Provider output parsers for streamed/non-streamed model responses. See `crates/krusty-core/src/ai/parsers/AGENTS.md` for local detail.
- `retry/`: Retry and backoff logic for resilient model calls. See `crates/krusty-core/src/ai/retry/AGENTS.md` for local detail.

### Files
- `format_detection.rs`: Rust source module implementing format detection behavior.
- `glm.rs`: Rust source module implementing glm behavior.
- `mod.rs`: Module root that wires child modules and shared exports.
- `models.rs`: Rust source module implementing models behavior.
- `openrouter.rs`: Rust source module implementing openrouter behavior.
- `providers.rs`: Rust source module implementing providers behavior.
- `reasoning.rs`: Rust source module implementing reasoning behavior.
- `sse.rs`: Rust source module implementing sse behavior.
- `stream_buffer.rs`: Rust source module implementing stream buffer behavior.
- `streaming.rs`: Rust source module implementing streaming behavior.
- `title.rs`: Rust source module implementing title behavior.
- `transform.rs`: Rust source module implementing transform behavior.
- `types.rs`: Rust source module implementing types behavior.
