# AGENTS Guide: /.github

## Purpose
CI/CD and release automation.

## Guardrails
- Treat workflow changes as production-impacting.
- Keep CI reproducible and aligned with local developer commands.
- Avoid secret leakage in logs and artifact names.
- Keep release workflows backward-compatible for existing tags and packaging paths.

## Directory Notes
- `workflows/`: CI, release, and verification pipelines.
- `homebrew/`: packaging metadata and formula automation.

## Validation
- Validate workflow syntax and ensure command parity with repository quality gates.
