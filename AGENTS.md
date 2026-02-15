# Repository Guidelines

## Project Structure & Module Organization
- `crates/krusty-cli`: terminal client (`krusty`) and TUI handlers.
- `crates/krusty-core`: shared runtime (AI providers, tools, storage, planning, MCP/ACP).
- `crates/krusty-server`: self-host API used by app clients.
- `apps/pwa/app`: active installable PWA chat/workspace client.
- `apps/desktop/shell`: Tauri wrapper around the PWA surface.
- `apps/marketing/site`: static marketing/legal pages only.

## Build, Test, and Development Commands
- Rust workspace:
  `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --all -- --check`
- Server local run:
  `cargo run -p krusty-server` (defaults to `http://localhost:3000`)
- PWA:
  `cd apps/pwa/app && npm ci && npm run check && npm run build`
- Desktop shell:
  `cd apps/desktop/shell && npm ci && cargo check --manifest-path src-tauri/Cargo.toml`
- Marketing site smoke:
  verify files in `apps/marketing/site` and serve with `python3 -m http.server 8080`.

## Coding Style & Naming Conventions
- Rust: 2021 edition, `rustfmt` defaults, `snake_case` functions/modules, `CamelCase` types.
- Svelte/TypeScript: keep components focused; colocate state in `src/lib/stores`.
- Prefer explicit boundaries: marketing code must not import app runtime; PWA must not include billing/account coupling.

## Testing Guidelines
- Keep Rust unit tests near implementation (`#[cfg(test)]`).
- Use `#[tokio::test]` for async behavior.
- For frontend changes, run `npm run check` and `npm run build` in `apps/pwa/app`.
- Treat warnings as cleanup candidates even if builds pass.

## Commit & Pull Request Guidelines
- Use Conventional Commit prefixes (`feat:`, `fix:`, `refactor:`, `docs:`, `chore:`).
- Keep PRs scoped by boundary (`server`, `pwa`, `desktop`, `marketing`).
- Include: problem statement, change summary, test evidence (commands run), and screenshots for UI updates.

## Security & Configuration Tips
- Never commit provider keys or local credential files.
- Use env vars (`PORT`, `KRUSTY_PROVIDER`, `KRUSTY_MODEL`, provider API keys) or `/api/credentials/*`.
- Self-host focus: avoid introducing Kubernetes-only assumptions in runtime paths.
