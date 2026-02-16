# AGENTS Guide: /crates/krusty-cli/src/tui

## Scope
- Applies to `/crates/krusty-cli/src/tui` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Ratatui interface implementation with state, handlers, and renderers.

## Local Standards
- Deliver best-in-class quality: elegant, modular, organized, and performant code.
- Keep code self-explanatory; add comments only for non-obvious constraints or tradeoffs.
- Avoid over-engineering; add abstractions only when they buy clear maintainability.
- Keep boundaries explicit between CLI, core runtime, server, desktop shell, and PWA surfaces.
- Prefer safe implementations; justify `unsafe` usage explicitly if ever required.
- For Rust changes, use idiomatic patterns (`Result`/`Option`, iterators, trait-based composition) and keep `anyhow::Context` on fallible IO/network boundaries.
- Preserve TUI responsiveness: avoid render-loop allocations, cache expensive calculations, and keep scrolling/streaming smooth.

## Quality Gates
- Rust workspace: `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --all -- --check`
- Server local run: `cargo run -p krusty-server`
- PWA: `cd apps/pwa/app && bun run check && bun run build`
- Desktop shell: `cd apps/desktop/shell && cargo check --manifest-path src-tauri/Cargo.toml`

## Structure Map
### Subdirectories
- `animation/`: Animation systems used by menu and visual polish effects. See `crates/krusty-cli/src/tui/animation/AGENTS.md` for local detail.
- `auth/`: OAuth callback and auth-flow assets for the terminal UI. See `crates/krusty-cli/src/tui/auth/AGENTS.md` for local detail.
- `blocks/`: Renderable stream block implementations for chat/tool output. See `crates/krusty-cli/src/tui/blocks/AGENTS.md` for local detail.
- `components/`: Reusable terminal widgets (toolbar, status, prompts, sidebars). See `crates/krusty-cli/src/tui/components/AGENTS.md` for local detail.
- `graphics/`: Terminal graphics protocol abstractions and adapters. See `crates/krusty-cli/src/tui/graphics/AGENTS.md` for local detail.
- `handlers/`: Event handlers orchestrating keyboard, mouse, render, and streams. See `crates/krusty-cli/src/tui/handlers/AGENTS.md` for local detail.
- `input/`: Input subsystem including autocomplete, parsing, and multi-line editing. See `crates/krusty-cli/src/tui/input/AGENTS.md` for local detail.
- `markdown/`: Markdown parser, cache, and renderer for terminal message content. See `crates/krusty-cli/src/tui/markdown/AGENTS.md` for local detail.
- `plugins/`: Terminal plugin integrations (graphics, gamepad, retro components). See `crates/krusty-cli/src/tui/plugins/AGENTS.md` for local detail.
- `polling/`: Polling loops that sync external process/auth/tool states. See `crates/krusty-cli/src/tui/polling/AGENTS.md` for local detail.
- `popups/`: Modal popup implementations and shared popup utilities. See `crates/krusty-cli/src/tui/popups/AGENTS.md` for local detail.
- `state/`: Centralized UI state models and scroll/selection coordination. See `crates/krusty-cli/src/tui/state/AGENTS.md` for local detail.
- `streaming/`: TUI streaming state types and adapters. See `crates/krusty-cli/src/tui/streaming/AGENTS.md` for local detail.
- `themes/`: Theme model, registry, and theme wiring logic. See `crates/krusty-cli/src/tui/themes/AGENTS.md` for local detail.
- `utils/`: Shared terminal utility helpers for text, syntax, channels, and worktrees. See `crates/krusty-cli/src/tui/utils/AGENTS.md` for local detail.

### Files
- `app.rs`: Rust source module implementing app behavior.
- `app_builder.rs`: Rust source module implementing app builder behavior.
- `auth.rs`: Rust source module implementing auth behavior.
- `mod.rs`: Module root that wires child modules and shared exports.
