//! Installable plugin browser popup.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::common::{
    center_content, center_rect, popup_block, popup_title, render_popup_background,
    scroll_indicator, PopupSize,
};
use crate::plugins::InstalledPlugin;
use crate::tui::themes::Theme;
use crate::tui::utils::truncate_ellipsis;

pub struct PluginsBrowserPopup {
    pub plugins: Vec<InstalledPlugin>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub status_message: Option<String>,
}

impl Default for PluginsBrowserPopup {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginsBrowserPopup {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            status_message: None,
        }
    }

    pub fn set_plugins(&mut self, plugins: Vec<InstalledPlugin>) {
        self.plugins = plugins;
        if self.plugins.is_empty() {
            self.selected_index = 0;
            self.scroll_offset = 0;
            return;
        }

        self.selected_index = self
            .selected_index
            .min(self.plugins.len().saturating_sub(1));
        self.ensure_visible();
    }

    pub fn set_status_message(&mut self, message: Option<String>) {
        self.status_message = message;
    }

    pub fn selected_plugin_id(&self) -> Option<&str> {
        self.plugins
            .get(self.selected_index)
            .map(|plugin| plugin.id.as_str())
    }

    pub fn next(&mut self) {
        if self.selected_index < self.plugins.len().saturating_sub(1) {
            self.selected_index += 1;
            self.ensure_visible();
        }
    }

    pub fn prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.ensure_visible();
        }
    }

    fn ensure_visible(&mut self) {
        self.ensure_visible_with_height(8);
    }

    fn ensure_visible_with_height(&mut self, visible_height: usize) {
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + visible_height {
            self.scroll_offset = self.selected_index - visible_height + 1;
        }
    }

    pub fn render(&self, f: &mut Frame, theme: &Theme) {
        let (w, h) = PopupSize::Large.dimensions();
        let area = center_rect(w, h, f.area());
        render_popup_background(f, area, theme);

        let block = popup_block(theme);
        let inner = block.inner(area);
        f.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // title
                Constraint::Min(6),    // content
                Constraint::Length(2), // footer
            ])
            .split(inner);

        let title_lines = popup_title(&format!("Plugins ({})", self.plugins.len()), theme);
        f.render_widget(
            Paragraph::new(title_lines).alignment(Alignment::Center),
            chunks[0],
        );

        let visible_height = (chunks[1].height as usize).saturating_sub(3);

        let mut lines = Vec::new();
        if self.plugins.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "  No plugins installed.",
                Style::default().fg(theme.dim_color),
            )]));
            lines.push(Line::from(vec![Span::styled(
                "  Use: /plugins install <manifest-path-or-url>",
                Style::default().fg(theme.text_color),
            )]));
            lines.push(Line::from(vec![Span::styled(
                "  Trust commands: /plugins allow-publisher, /plugins add-key",
                Style::default().fg(theme.dim_color),
            )]));
        } else {
            if self.scroll_offset > 0 {
                lines.push(scroll_indicator("up", self.scroll_offset, theme));
            }

            let visible_end = (self.scroll_offset + visible_height).min(self.plugins.len());
            for (idx, plugin) in self
                .plugins
                .iter()
                .enumerate()
                .skip(self.scroll_offset)
                .take(visible_height)
            {
                let selected = idx == self.selected_index;
                let prefix = if selected { " › " } else { "   " };
                let name_style = if selected {
                    Style::default()
                        .fg(theme.accent_color)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text_color)
                };

                let status = if plugin.enabled {
                    ("enabled", theme.success_color)
                } else {
                    ("disabled", theme.warning_color)
                };

                let mode = if plugin
                    .render_capabilities
                    .iter()
                    .any(|cap| matches!(cap, crate::plugins::PluginRenderCapability::Frame))
                {
                    "frame"
                } else {
                    "text"
                };

                lines.push(Line::from(vec![
                    Span::styled(prefix, name_style),
                    Span::styled(&plugin.name, name_style),
                    Span::styled(
                        format!(" v{}", plugin.version),
                        Style::default().fg(theme.dim_color),
                    ),
                    Span::styled(
                        format!(" [{}]", mode),
                        Style::default().fg(theme.mode_view_color),
                    ),
                    Span::styled(format!(" ({})", status.0), Style::default().fg(status.1)),
                ]));

                let desc = plugin
                    .description
                    .as_ref()
                    .map(|text| truncate_ellipsis(text, 56).into_owned())
                    .unwrap_or_else(|| "No description".to_string());
                lines.push(Line::from(vec![
                    Span::raw("      "),
                    Span::styled(desc, Style::default().fg(theme.dim_color)),
                ]));
            }

            let remaining = self.plugins.len().saturating_sub(visible_end);
            if remaining > 0 {
                lines.push(scroll_indicator("down", remaining, theme));
            }
        }

        if let Some(status) = &self.status_message {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                truncate_ellipsis(status, 70),
                Style::default().fg(theme.accent_color),
            )]));
        }

        let content = Paragraph::new(lines).style(Style::default().bg(theme.bg_color));
        f.render_widget(content, center_content(chunks[1], 2));

        let footer = Paragraph::new(Line::from(vec![
            Span::styled(
                "↑↓",
                Style::default()
                    .fg(theme.accent_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": nav  ", Style::default().fg(theme.text_color)),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(theme.accent_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": toggle  ", Style::default().fg(theme.text_color)),
            Span::styled(
                "r",
                Style::default()
                    .fg(theme.accent_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": refresh  ", Style::default().fg(theme.text_color)),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(theme.accent_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": close", Style::default().fg(theme.text_color)),
        ]))
        .alignment(Alignment::Center);
        f.render_widget(footer, chunks[2]);
    }
}
