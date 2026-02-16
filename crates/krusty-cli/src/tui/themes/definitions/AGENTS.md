# AGENTS Guide: /crates/krusty-cli/src/tui/themes/definitions

## Scope
- Applies to `/crates/krusty-cli/src/tui/themes/definitions` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Concrete TUI color theme definitions.

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
- _(none)_

### Files
- `aura.rs`: Terminal theme palette definition for aura.
- `ayu_dark.rs`: Terminal theme palette definition for ayu dark.
- `catppuccin_mocha.rs`: Terminal theme palette definition for catppuccin mocha.
- `cobalt2.rs`: Terminal theme palette definition for cobalt2.
- `cyberpunk.rs`: Terminal theme palette definition for cyberpunk.
- `dracula.rs`: Terminal theme palette definition for dracula.
- `everforest.rs`: Terminal theme palette definition for everforest.
- `forest_night.rs`: Terminal theme palette definition for forest night.
- `github_dark.rs`: Terminal theme palette definition for github dark.
- `gruvbox_dark.rs`: Terminal theme palette definition for gruvbox dark.
- `high_contrast.rs`: Terminal theme palette definition for high contrast.
- `kanagawa.rs`: Terminal theme palette definition for kanagawa.
- `krusty.rs`: Terminal theme palette definition for krusty.
- `material_ocean.rs`: Terminal theme palette definition for material ocean.
- `matrix.rs`: Terminal theme palette definition for matrix.
- `mod.rs`: Module root that wires child modules and shared exports.
- `monokai.rs`: Terminal theme palette definition for monokai.
- `moonlight.rs`: Terminal theme palette definition for moonlight.
- `night_owl.rs`: Terminal theme palette definition for night owl.
- `nord.rs`: Terminal theme palette definition for nord.
- `one_dark.rs`: Terminal theme palette definition for one dark.
- `palenight.rs`: Terminal theme palette definition for palenight.
- `retro_wave.rs`: Terminal theme palette definition for retro wave.
- `rosepine.rs`: Terminal theme palette definition for rosepine.
- `serenity.rs`: Terminal theme palette definition for serenity.
- `sith_lord.rs`: Terminal theme palette definition for sith lord.
- `solarized_dark.rs`: Terminal theme palette definition for solarized dark.
- `synthwave_84.rs`: Terminal theme palette definition for synthwave 84.
- `terminal.rs`: Terminal theme palette definition for terminal.
- `tokyo_night.rs`: Terminal theme palette definition for tokyo night.
- `vesper.rs`: Terminal theme palette definition for vesper.
- `zenburn.rs`: Terminal theme palette definition for zenburn.
