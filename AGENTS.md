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

## Required Validation
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo fmt --all -- --check`
- `cd apps/pwa/app && bun run check && bun run build`
