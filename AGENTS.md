# AGENTS Guide: /

## Scope
- Applies to `/` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Workspace root coordinating crates, apps, packaging, and governance docs.

## Local Standards
- Deliver best-in-class quality: elegant, modular, organized, and performant code.
- Keep code self-explanatory; add comments only for non-obvious constraints or tradeoffs.
- Avoid over-engineering; add abstractions only when they buy clear maintainability.
- Keep boundaries explicit between CLI, core runtime, server, desktop shell, and PWA surfaces.
- Prefer safe implementations; justify `unsafe` usage explicitly if ever required.
- Keep AGENTS guidance current: when a notable change impacts structure, file responsibilities, workflows, or standards, stop and update the relevant `AGENTS.md` files in the same change.

## Quality Gates
- Quality gates are blocking: all listed checks must pass with no warnings/errors before considering work complete.
- Rust workspace: `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --all -- --check`
- Server local run: `cargo run -p krusty-server`
- PWA: `cd apps/pwa/app && bun run check && bun run build`
- Desktop shell: `cd apps/desktop/shell && cargo check --manifest-path src-tauri/Cargo.toml`

## Structure Map
### Subdirectories
- `.githooks/`: Git hook scripts that enforce local quality checks before commits. See `.githooks/AGENTS.md` for local detail.
- `.github/`: GitHub automation and release infrastructure configuration. See `.github/AGENTS.md` for local detail.
- `apps/`: Application surfaces built on top of the shared Rust runtime. See `apps/AGENTS.md` for local detail.
- `aur/`: Arch Linux packaging metadata for AUR distribution. See `aur/AGENTS.md` for local detail.
- `crates/`: Rust workspace crates implementing CLI, core runtime, and server. See `crates/AGENTS.md` for local detail.
- `wit/`: Top-level WIT contracts shared across extension tooling. See `wit/AGENTS.md` for local detail.

### Files
- `.gitignore`: Ignore rules for generated files and local-only artifacts.
- `AGENTS.md`: Repository-level agent instructions and standards for all contributors.
- `CLAUDE.md`: Claude guidance document describing values, architecture, and coding standards for this scope.
- `Cargo.lock`: Pinned Rust dependency lockfile for reproducible builds.
- `Cargo.toml`: Workspace manifest declaring crate members and shared dependency settings.
- `Cross.toml`: Cross-compilation configuration for multi-target Rust builds.
- `KRAB.md`: Project internal reference/spec document used by maintainers.
- `LICENSE`: Project license terms and usage permissions.
- `README.md`: Human-facing documentation for this directory's purpose and workflows.
- `RESTRUCTURE_ROADMAP.md`: Roadmap detailing current restructuring phases and migration targets.
- `install.sh`: Installer script for provisioning krusty binaries and assets.
