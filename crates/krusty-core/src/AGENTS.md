# AGENTS Guide: /crates/krusty-core/src

## Purpose
Core runtime implementation modules.

## Guardrails
- Keep subsystem contracts explicit between AI, tools, storage, plugins, and protocols.
- Prefer typed boundaries over ad-hoc JSON passing.
- Any cross-cutting change should include targeted tests.

## Key Local Guides
- `ai/AGENTS.md`
- `storage/AGENTS.md`
- `tools/AGENTS.md`
- `extensions/AGENTS.md`
- `plugins/AGENTS.md`
