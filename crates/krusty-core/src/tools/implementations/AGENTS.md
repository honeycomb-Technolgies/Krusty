# AGENTS Guide: /crates/krusty-core/src/tools/implementations

## Scope
- Applies to `/crates/krusty-core/src/tools/implementations` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Concrete tool implementations used by the agent runtime.

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
- _(none)_

### Files
- `add_subtask.rs`: Rust source module implementing add subtask behavior.
- `ask_user.rs`: Rust source module implementing ask user behavior.
- `bash.rs`: Rust source module implementing bash behavior.
- `build.rs`: Cargo build script executed before compilation.
- `edit.rs`: Rust source module implementing edit behavior.
- `explore.rs`: Rust source module implementing explore behavior.
- `glob.rs`: Rust source module implementing glob behavior.
- `grep.rs`: Rust source module implementing grep behavior.
- `mod.rs`: Module root that wires child modules and shared exports.
- `plan_mode.rs`: Rust source module implementing plan mode behavior.
- `processes.rs`: Rust source module implementing processes behavior.
- `read.rs`: Rust source module implementing read behavior.
- `set_dependency.rs`: Rust source module implementing set dependency behavior.
- `set_work_mode.rs`: Rust source module implementing work mode switching behavior.
- `skill.rs`: Rust source module implementing skill behavior.
- `task_complete.rs`: Rust source module implementing task complete behavior.
- `task_start.rs`: Rust source module implementing task start behavior.
- `write.rs`: Rust source module implementing write behavior.
