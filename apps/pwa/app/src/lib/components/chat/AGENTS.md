# AGENTS Guide: /apps/pwa/app/src/lib/components/chat

## Scope
- Applies to `/apps/pwa/app/src/lib/components/chat` and its direct contents.
- If a deeper directory has its own `AGENTS.md`, that file takes precedence for its subtree.

## Purpose
Chat interface widgets and streaming message presentation.

## Local Standards
- Deliver best-in-class quality: elegant, modular, organized, and performant code.
- Keep code self-explanatory; add comments only for non-obvious constraints or tradeoffs.
- Avoid over-engineering; add abstractions only when they buy clear maintainability.
- Keep boundaries explicit between CLI, core runtime, server, desktop shell, and PWA surfaces.
- Prefer safe implementations; justify `unsafe` usage explicitly if ever required.
- Keep Svelte components focused and colocate cross-view state in `src/lib/stores`.
- Do not introduce billing/account coupling into the PWA runtime surface.

## Quality Gates
- Rust workspace: `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --all -- --check`
- Server local run: `cargo run -p krusty-server`
- PWA: `cd apps/pwa/app && bun run check && bun run build`
- Desktop shell: `cd apps/desktop/shell && cargo check --manifest-path src-tauri/Cargo.toml`

## Structure Map
### Subdirectories
- _(none)_

### Files
- `AsciiTitle.svelte`: Reusable Svelte component implementing Ascii Title UI behavior.
- `AskUserQuestionWidget.svelte`: Reusable Svelte component implementing Ask User Question Widget UI behavior.
- `ChatHeader.svelte`: Reusable Svelte component implementing Chat Header UI behavior.
- `ChatView.svelte`: Reusable Svelte component implementing Chat View UI behavior.
- `Message.svelte`: Reusable Svelte component implementing Message UI behavior.
- `ModelSelector.svelte`: Reusable Svelte component implementing Model Selector UI behavior.
- `PlanConfirmWidget.svelte`: Reusable Svelte component implementing Plan Confirm Widget UI behavior.
- `PlanTracker.svelte`: Reusable Svelte component implementing Plan Tracker UI behavior.
- `PlasmaBackground.svelte`: Reusable Svelte component implementing Plasma Background UI behavior.
- `SessionSidebar.svelte`: Reusable Svelte component implementing Session Sidebar UI behavior.
- `ThinkingBlock.svelte`: Reusable Svelte component implementing Thinking Block UI behavior.
- `ToolApprovalWidget.svelte`: Reusable Svelte component implementing Tool Approval Widget UI behavior.
- `ToolWidget.svelte`: Reusable Svelte component implementing Tool Widget UI behavior.
