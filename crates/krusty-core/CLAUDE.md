# krusty-core

Core library providing AI, storage, tools, extensions, and agent systems.

## Module Overview
- `ai/` - Multi-provider AI clients, streaming, SSE parsing
- `agent/` - Event bus, hooks, sub-agents, build context (Octopod)
- `tools/` - Tool registry, implementations (bash, read, write, edit, etc.)
- `storage/` - SQLite persistence (sessions, plans, credentials)
- `extensions/` - Zed WASM extension host
- `lsp/` - Language server protocol client
- `mcp/` - Model Context Protocol client
- `skills/` - Filesystem-based skill loader

## Patterns
- Trait-based extensibility: `PreToolHook`, `PostToolHook`, `Tool`
- Arc/RwLock for shared state
- anyhow::Result everywhere, tracing for logging
- Async-first: tokio runtime, `#[async_trait]`

## Testing
Tests live inline: `#[cfg(test)] mod tests { }` at file end.
