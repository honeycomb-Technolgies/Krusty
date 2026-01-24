//! LSP browser and install popup keyboard handlers

use crossterm::event::KeyCode;

use crate::lsp::manager::LspSuggestion;
use crate::tui::app::{App, Popup};

impl App {
    /// Handle LSP browser popup keyboard events
    pub fn handle_lsp_popup_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                if self.popups.lsp.search_active {
                    self.popups.lsp.toggle_search();
                } else {
                    self.ui.popup = Popup::None;
                }
            }
            KeyCode::Up => self.popups.lsp.prev(),
            KeyCode::Down => self.popups.lsp.next(),
            KeyCode::Char('k') if !self.popups.lsp.search_active => {
                self.popups.lsp.prev();
            }
            KeyCode::Char('j') if !self.popups.lsp.search_active => {
                self.popups.lsp.next();
            }
            KeyCode::Char('/') if !self.popups.lsp.search_active => {
                self.popups.lsp.toggle_search();
            }
            KeyCode::Char('r') if !self.popups.lsp.search_active => {
                self.popups.lsp.refresh();
                self.start_extensions_fetch();
            }
            KeyCode::Enter => {
                if self.popups.lsp.needs_fetch() {
                    self.start_extensions_fetch();
                } else if !self.popups.lsp.is_loading() {
                    self.start_lsp_install();
                }
            }
            KeyCode::Backspace if self.popups.lsp.search_active => {
                self.popups.lsp.backspace_search();
            }
            KeyCode::Char(c) if self.popups.lsp.search_active => {
                self.popups.lsp.add_search_char(c);
            }
            _ => {}
        }
    }

    /// Handle file preview popup keyboard events
    pub fn handle_file_preview_popup_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.popups.file_preview.reset();
                self.ui.popup = Popup::None;
            }
            KeyCode::Char('o') | KeyCode::Char('O') => {
                self.popups.file_preview.open_external();
            }
            KeyCode::Left => {
                self.popups.file_preview.prev_page();
            }
            KeyCode::Right => {
                self.popups.file_preview.next_page();
            }
            _ => {}
        }
    }

    /// Handle LSP install popup keyboard events
    pub fn handle_lsp_install_popup_key(&mut self, code: KeyCode) {
        // Don't handle keys while installing
        if self.popups.lsp_install.installing {
            return;
        }

        match code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                let suggestion = self
                    .popups
                    .lsp_install
                    .get_info()
                    .map(|i| i.suggested.clone());

                match suggestion {
                    Some(LspSuggestion::Builtin(builtin)) => {
                        self.popups.lsp_install.start_install();
                        self.start_builtin_lsp_install(builtin);
                    }
                    Some(LspSuggestion::Extension(name)) => {
                        self.popups.lsp_install.start_install();
                        self.start_extension_lsp_install(name);
                    }
                    Some(LspSuggestion::None) | None => {
                        self.popups.lsp_install.clear();
                        self.ui.popup = Popup::None;
                    }
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                // Skip for this session
                if let Some(ext) = self
                    .popups
                    .lsp_install
                    .get_info()
                    .map(|i| i.extension.clone())
                {
                    self.services.lsp_skip_list.insert(ext);
                }
                self.popups.lsp_install.clear();
                self.ui.popup = Popup::None;
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                // Always skip
                if let Some(ext) = self
                    .popups
                    .lsp_install
                    .get_info()
                    .map(|i| i.extension.clone())
                {
                    self.services.lsp_skip_list.insert(ext);
                }
                self.popups.lsp_install.clear();
                self.ui.popup = Popup::None;
            }
            _ => {}
        }
    }
}
