# AGENTS Guide: /apps/desktop/shell/src-tauri

## Purpose
Rust/Tauri app bootstrap, bundling, and platform config.

## Guardrails
- Keep `tauri.conf.json`, `Cargo.toml`, and runtime bootstrap aligned.
- Treat permissions, deep links, and updater config as security-sensitive.
- Minimize platform-specific branches; document any unavoidable divergence.

## Validation
- `cd apps/desktop/shell && cargo check --manifest-path src-tauri/Cargo.toml`
