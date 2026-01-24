//! UI state management
//!
//! Groups UI-related state: view mode, active popup, theme, work mode.

use std::sync::Arc;

use crate::tui::app::{Popup, View, WorkMode};
use crate::tui::themes::Theme;

/// UI presentation state
///
/// Groups fields related to the UI layer: views, popups, theming, work mode.
pub struct UiState {
    /// Current view (StartMenu, Chat)
    pub view: View,
    /// Current active popup
    pub popup: Popup,
    /// Current work mode (Build, Plan)
    pub work_mode: WorkMode,
    /// Active theme
    pub theme: Arc<Theme>,
    /// Theme name for display/saving
    pub theme_name: String,
}

impl UiState {
    pub fn new(theme: Arc<Theme>, theme_name: String) -> Self {
        Self {
            view: View::StartMenu,
            popup: Popup::None,
            work_mode: WorkMode::Build,
            theme,
            theme_name,
        }
    }
}
