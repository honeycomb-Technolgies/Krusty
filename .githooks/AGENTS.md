# AGENTS Guide: /.githooks

## Purpose
Local git hooks that enforce fast, reliable pre-commit quality gates.

## Guardrails
- Keep hooks deterministic and quick.
- Prefer local checks; avoid network-dependent commands.
- Fail with clear, actionable error messages.
- Any new hook must not duplicate CI logic unnecessarily.

## Validation
- Run hook scripts directly before enabling them in workflows.
