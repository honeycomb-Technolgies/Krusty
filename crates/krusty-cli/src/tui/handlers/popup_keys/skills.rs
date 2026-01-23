//! Skills browser popup keyboard handler

use crossterm::event::KeyCode;

use crate::tui::app::{App, Popup};

impl App {
    /// Handle skills browser popup keyboard events
    pub fn handle_skills_popup_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                if self.popups.skills.search_active {
                    self.popups.skills.toggle_search();
                } else {
                    self.popup = Popup::None;
                }
            }
            KeyCode::Up => self.popups.skills.prev(),
            KeyCode::Down => self.popups.skills.next(),
            KeyCode::Char('k') if !self.popups.skills.search_active => {
                self.popups.skills.prev();
            }
            KeyCode::Char('j') if !self.popups.skills.search_active => {
                self.popups.skills.next();
            }
            KeyCode::Char('/') if !self.popups.skills.search_active => {
                self.popups.skills.toggle_search();
            }
            KeyCode::Char('r') if !self.popups.skills.search_active => {
                self.refresh_skills_browser();
            }
            KeyCode::Backspace if self.popups.skills.search_active => {
                self.popups.skills.backspace_search();
            }
            KeyCode::Char(c) if self.popups.skills.search_active => {
                self.popups.skills.add_search_char(c);
            }
            _ => {}
        }
    }
}
