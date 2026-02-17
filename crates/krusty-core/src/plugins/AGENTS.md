# AGENTS Guide: /crates/krusty-core/src/plugins

## Purpose
Plugin manifests, trust policy, lockfile state, and lifecycle management.

## Guardrails
- Treat plugin install/update flows as security-sensitive.
- Verify trust and signature requirements before writing plugin artifacts.
- Keep lockfile and on-disk state transitions atomic and recoverable.
- Error messages must clearly distinguish trust failures from IO failures.

## Validation
- `cargo check -p krusty-core`
- run targeted plugin manager/signing tests after trust-path changes.
