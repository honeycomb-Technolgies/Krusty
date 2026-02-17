//! Installable plugins popup keyboard handler

use crossterm::event::KeyCode;

use crate::tui::app::{App, Popup};

impl App {
    pub fn handle_plugins_popup_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.ui.popup = Popup::None;
            }
            KeyCode::Up | KeyCode::Char('k') => self.ui.popups.plugins.prev(),
            KeyCode::Down | KeyCode::Char('j') => self.ui.popups.plugins.next(),
            KeyCode::Enter | KeyCode::Char('e') => self.toggle_selected_plugin_from_popup(),
            KeyCode::Char('r') => {
                self.refresh_plugins_browser();
            }
            _ => {}
        }
    }
}
