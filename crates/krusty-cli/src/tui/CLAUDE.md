# TUI Module

Ratatui-based terminal interface.

## Architecture (Post-Refactor)
- `app.rs` - Main App struct (~800 lines)
- `state/` - All state structs (scroll, blocks, popups, selection)
- `handlers/` - Event handlers (keyboard, mouse, streaming, rendering)
- `blocks/` - Stream blocks (thinking, bash, read, edit, write, etc.)
- `components/` - Reusable widgets (toolbar, status_bar, scrollbars)
- `popups/` - Modal dialogs

## Block System
All stream blocks implement `StreamBlock` trait:
- `height(width, theme) -> u16`
- `render(area, buf, theme, focused, clip)`
- `handle_event(event, area, clip) -> EventResult`

## State Ownership
State extracted to `state/` module:
- `BlockManager` - All block vectors
- `ScrollState`, `ScrollSystem` - Scroll position and caching
- `PopupState` - Active popup tracking
