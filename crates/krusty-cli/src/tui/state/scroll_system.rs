//! Scroll system state management
//!
//! Groups all scroll, layout, selection, and hover state into one struct.

use super::{EdgeScrollState, HoverState, LayoutState, ScrollState, SelectionState};

/// Scroll and layout system state
///
/// Groups fields related to scrolling, layout, selection, and hover tracking.
#[derive(Default)]
pub struct ScrollSystem {
    /// Main scroll state
    pub scroll: ScrollState,
    /// Layout areas cache
    pub layout: LayoutState,
    /// Text selection state
    pub selection: SelectionState,
    /// Hover state (links, blocks)
    pub hover: HoverState,
    /// Edge scrolling during selection
    pub edge_scroll: EdgeScrollState,
}

impl ScrollSystem {
    pub fn new() -> Self {
        Self {
            scroll: ScrollState::new(),
            layout: LayoutState::new(),
            selection: SelectionState::default(),
            hover: HoverState::default(),
            edge_scroll: EdgeScrollState::default(),
        }
    }
}
