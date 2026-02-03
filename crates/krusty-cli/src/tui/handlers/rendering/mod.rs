//! UI rendering coordinator
//!
//! Main entry point that dispatches to specialized render modules.

mod messages;
mod scroll_calc;
mod views;

use ratatui::{style::Style, widgets::Block, Frame};

use crate::tui::app::{App, Popup};
use crate::tui::components::render_toasts;

impl App {
    /// Main UI rendering dispatcher
    pub fn ui(&mut self, f: &mut Frame) {
        // Render background
        let bg = Block::default().style(Style::default().bg(self.ui.ui.theme.bg_color));
        f.render_widget(bg, f.area());

        // Render main view - direct match avoids borrow conflicts
        match self.ui.ui.view {
            crate::tui::app::View::StartMenu => self.render_start_menu(f),
            crate::tui::app::View::Chat => self.render_chat(f),
        }

        // Render popup on top - use reference matching for short-lived borrows
        match &self.ui.ui.popup {
            Popup::None => {}
            Popup::Help => self.ui.popups.help.render(f, &self.ui.ui.theme),
            Popup::ThemeSelect => {
                let theme_name = self.ui.ui.theme_name.clone();
                self.ui
                    .popups
                    .theme
                    .render(f, &self.ui.ui.theme, &theme_name)
            }
            Popup::ModelSelect => self.ui.popups.model.render(
                f,
                &self.ui.ui.theme,
                &self.runtime.current_model,
                self.runtime.context_tokens_used,
            ),
            Popup::SessionList => self.ui.popups.session.render(f, &self.ui.ui.theme),
            Popup::Auth => self.ui.popups.auth.render(f, &self.ui.ui.theme),
            Popup::ProcessList => self.ui.popups.process.render(f, &self.ui.ui.theme),
            Popup::Pinch => self.ui.popups.pinch.render(f, &self.ui.ui.theme),
            Popup::FilePreview => self.ui.popups.file_preview.render(f, &self.ui.ui.theme),
            Popup::SkillsBrowser => self.ui.popups.skills.render(f, &self.ui.ui.theme),
            Popup::McpBrowser => self.ui.popups.mcp.render(f, &self.ui.ui.theme),
            Popup::Hooks => self.ui.popups.hooks.render(f, &self.ui.ui.theme),
        }

        // Render toasts on top of everything
        let area = f.area();
        render_toasts(f.buffer_mut(), area, &self.ui.toasts, &self.ui.ui.theme);
    }
}
