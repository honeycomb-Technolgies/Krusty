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
        let bg = Block::default().style(Style::default().bg(self.ui.theme.bg_color));
        f.render_widget(bg, f.area());

        // Render main view - direct match avoids borrow conflicts
        match self.ui.view {
            crate::tui::app::View::StartMenu => self.render_start_menu(f),
            crate::tui::app::View::Chat => self.render_chat(f),
        }

        // Render popup on top - use reference matching for short-lived borrows
        match &self.ui.popup {
            Popup::None => {}
            Popup::Help => self.popups.help.render(f, &self.ui.theme),
            Popup::ThemeSelect => {
                let theme_name = self.ui.theme_name.clone();
                self.popups.theme.render(f, &self.ui.theme, &theme_name)
            }
            Popup::ModelSelect => self.popups.model.render(
                f,
                &self.ui.theme,
                &self.current_model,
                self.context_tokens_used,
            ),
            Popup::SessionList => self.popups.session.render(f, &self.ui.theme),
            Popup::Auth => self.popups.auth.render(f, &self.ui.theme),
            Popup::LspBrowser => self.popups.lsp.render(f, &self.ui.theme),
            Popup::LspInstall => self.popups.lsp_install.render(f, &self.ui.theme),
            Popup::ProcessList => self.popups.process.render(f, &self.ui.theme),
            Popup::Pinch => self.popups.pinch.render(f, &self.ui.theme),
            Popup::FilePreview => self.popups.file_preview.render(f, &self.ui.theme),
            Popup::SkillsBrowser => self.popups.skills.render(f, &self.ui.theme),
            Popup::McpBrowser => self.popups.mcp.render(f, &self.ui.theme),
            Popup::Hooks => self.popups.hooks.render(f, &self.ui.theme),
        }

        // Render toasts on top of everything
        let area = f.area();
        render_toasts(f.buffer_mut(), area, &self.toasts, &self.ui.theme);
    }
}
