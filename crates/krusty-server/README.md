# Krusty Server

Self-hosted API server used by the PWA and desktop shell.

## Scope

- Session lifecycle and message history
- Agentic chat streaming (`/api/chat`, SSE)
- Tool execution, file APIs, process control
- Model listing and provider credential management
- MCP server management and user hooks

No Kubernetes-specific runtime is required for this phase.

## Run Locally

```bash
cargo run -p krusty-server
```

Default bind address: `0.0.0.0:3000`  
Health check: `GET /health`

## Configuration

- `PORT` (optional): server port (default `3000`)
- `KRUSTY_PROVIDER` (optional): `minimax`, `openai`, `openrouter`, `zai`
- `KRUSTY_MODEL` (optional): override default model for selected provider
- Provider API keys (optional): `MINIMAX_API_KEY`, `OPENAI_API_KEY`, `OPENROUTER_API_KEY`, `Z_AI_API_KEY`

Credentials can also be set through:

- `POST /api/credentials/:provider`

## Client Connection

- PWA expects `VITE_API_BASE` (defaults to `/api` with dev proxy to `http://localhost:3000`)
- Desktop shell wraps the same PWA build and therefore uses the same server API

## Optional User Scoping

Requests without auth headers run in single-user local mode.

For scoped mode, clients may send:

- `X-User-Id`
- `X-Workspace-Dir` (optional)

## Main API Groups

- `/api/sessions`
- `/api/chat`
- `/api/models`
- `/api/tools`
- `/api/files`
- `/api/credentials`
- `/api/mcp`
- `/api/processes`
- `/api/hooks`
- `/api/ports` (preview discovery + path proxy)
- `/api/settings/preview` (preview forwarding policy/preferences)
