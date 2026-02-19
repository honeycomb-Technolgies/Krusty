# AGENTS Guide: /

## Purpose
Repository-level engineering guardrails for Krusty.

## AGENTS Strategy
- Keep `AGENTS.md` only at architectural boundaries where local rules differ.
- Do not create AGENTS in every leaf folder.
- Add a new AGENTS file only when a directory has unique invariants, workflows, or integration risk.
- When a notable structural or behavioral change lands, update the nearest applicable AGENTS file in the same commit.

## Core Architecture
- `crates/krusty-cli`: terminal client and TUI runtime.
- `crates/krusty-core`: shared runtime (AI, tools, storage, planning, protocols).
- `crates/krusty-server`: self-host API for clients.
- `apps/pwa/app`: primary installable web client.
- `apps/desktop/shell`: Tauri wrapper around the PWA.
- `apps/marketing/site`: static marketing/legal pages only.

## Cross-Cutting Standards
- Prefer clear module boundaries over cross-layer coupling.
- Write code that is composable, testable, and explicit about failure modes.
- Keep changes small and reversible.
- Avoid hidden side effects and global state sprawl.

## Default Dev Workflow
- Build and run current local code only; do not require `git pull` for day-to-day refinement.
- Run backend/API server from repo root: `cargo run -p krusty -- serve` (default `http://localhost:3000`).
- Run PWA hot-reload server in parallel: `cd apps/pwa/app && bun run dev` (default `http://localhost:5173`).
- Do active UI/PWA iteration at `http://localhost:5173` so HMR is enabled, with `/api` and `/ws` proxied to `:3000`.
- Frontend edits hot-reload automatically; Rust backend edits require a backend restart.

## Required Validation
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo fmt --all -- --check`
- `cd apps/pwa/app && bun run check && bun run build`
