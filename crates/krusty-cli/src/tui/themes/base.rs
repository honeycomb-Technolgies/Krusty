//! Theme builder with intelligent defaults

use super::Theme;
use ratatui::style::Color;

/// Builder pattern for creating themes with sensible defaults
pub struct ThemeBuilder {
    theme: Theme,
}

impl ThemeBuilder {
    /// Create a new theme builder with the given name and base colors
    pub fn new(name: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            theme: Theme {
                name: name.into(),
                display_name: display_name.into(),
                // Initialize with black - will be overridden
                bg_color: Color::Rgb(0, 0, 0),
                border_color: Color::Rgb(0, 0, 0),
                title_color: Color::Rgb(0, 0, 0),
                accent_color: Color::Rgb(0, 0, 0),
                text_color: Color::Rgb(0, 0, 0),
                success_color: Color::Rgb(0, 0, 0),
                dim_color: Color::Rgb(0, 0, 0),
                mode_view_color: Color::Rgb(0, 0, 0),
                mode_chat_color: Color::Rgb(0, 0, 0),
                mode_plan_color: Color::Rgb(0, 0, 0),
                mode_bash_color: Color::Rgb(0, 0, 0),
                mode_leader_color: Color::Rgb(0, 0, 0),
                warning_color: Color::Rgb(0, 0, 0),
                error_color: Color::Rgb(0, 0, 0),
                code_bg_color: Color::Rgb(0, 0, 0),
                cursor_color: Color::Rgb(0, 0, 0),
                selection_bg_color: Color::Rgb(0, 0, 0),
                selection_fg_color: Color::Rgb(0, 0, 0),
                user_msg_color: Color::Rgb(0, 0, 0),
                assistant_msg_color: Color::Rgb(0, 0, 0),
                system_msg_color: Color::Rgb(0, 0, 0),
                tool_msg_color: Color::Rgb(0, 0, 0),
                info_color: Color::Rgb(0, 0, 0),
                progress_color: Color::Rgb(0, 0, 0),
                input_bg_color: Color::Rgb(0, 0, 0),
                input_placeholder_color: Color::Rgb(0, 0, 0),
                input_border_color: Color::Rgb(0, 0, 0),
                user_msg_bg_color: Color::Rgb(0, 0, 0),
                assistant_msg_bg_color: Color::Rgb(0, 0, 0),
                system_msg_bg_color: Color::Rgb(0, 0, 0),
                tool_msg_bg_color: Color::Rgb(0, 0, 0),
                status_bar_bg_color: Color::Rgb(0, 0, 0),
                scrollbar_bg_color: Color::Rgb(0, 0, 0),
                scrollbar_fg_color: Color::Rgb(0, 0, 0),
                scrollbar_hover_color: Color::Rgb(0, 0, 0),
                logo_primary_color: Color::Rgb(0, 0, 0),
                logo_secondary_color: Color::Rgb(0, 0, 0),
                animation_color: Color::Rgb(0, 0, 0),
                processing_color: Color::Rgb(0, 0, 0),
                highlight_color: Color::Rgb(0, 0, 0),
                bubble_color: Color::Rgb(0, 0, 0),
                token_low_color: Color::Rgb(0, 0, 0),
                token_medium_color: Color::Rgb(0, 0, 0),
                token_high_color: Color::Rgb(0, 0, 0),
                token_critical_color: Color::Rgb(0, 0, 0),
                syntax_keyword_color: Color::Rgb(0, 0, 0),
                syntax_function_color: Color::Rgb(0, 0, 0),
                syntax_string_color: Color::Rgb(0, 0, 0),
                syntax_number_color: Color::Rgb(0, 0, 0),
                syntax_comment_color: Color::Rgb(0, 0, 0),
                syntax_type_color: Color::Rgb(0, 0, 0),
                syntax_variable_color: Color::Rgb(0, 0, 0),
                syntax_operator_color: Color::Rgb(0, 0, 0),
                syntax_punctuation_color: Color::Rgb(0, 0, 0),
                // Diff & code display colors
                diff_add_color: Color::Rgb(0, 0, 0),
                diff_add_bg_color: Color::Rgb(0, 0, 0),
                diff_remove_color: Color::Rgb(0, 0, 0),
                diff_remove_bg_color: Color::Rgb(0, 0, 0),
                diff_context_color: Color::Rgb(0, 0, 0),
                line_number_color: Color::Rgb(0, 0, 0),
                link_color: Color::Rgb(0, 0, 0),
                running_color: Color::Rgb(0, 0, 0),
            },
        }
    }

    /// Set core colors - these are required for every theme
    pub fn core_colors(
        mut self,
        bg: Color,
        border: Color,
        title: Color,
        accent: Color,
        text: Color,
        success: Color,
        dim: Color,
    ) -> Self {
        self.theme.bg_color = bg;
        self.theme.border_color = border;
        self.theme.title_color = title;
        self.theme.accent_color = accent;
        self.theme.text_color = text;
        self.theme.success_color = success;
        self.theme.dim_color = dim;
        self
    }

    /// Set mode colors (used as accent variants)
    pub fn mode_colors(
        mut self,
        view: Color,
        chat: Color,
        plan: Color,
        bash: Color,
        leader: Color,
    ) -> Self {
        self.theme.mode_view_color = view;
        self.theme.mode_chat_color = chat;
        self.theme.mode_plan_color = plan;
        self.theme.mode_bash_color = bash;
        self.theme.mode_leader_color = leader;
        self
    }

    /// Set special colors for warnings, errors, and code
    pub fn special_colors(mut self, warning: Color, error: Color, code_bg: Color) -> Self {
        self.theme.warning_color = warning;
        self.theme.error_color = error;
        self.theme.code_bg_color = code_bg;
        self
    }

    /// Set UI element colors
    pub fn ui_colors(mut self, cursor: Color, selection_bg: Color, selection_fg: Color) -> Self {
        self.theme.cursor_color = cursor;
        self.theme.selection_bg_color = selection_bg;
        self.theme.selection_fg_color = selection_fg;
        self
    }

    /// Set message role colors
    pub fn message_colors(
        mut self,
        user: Color,
        assistant: Color,
        system: Color,
        tool: Color,
    ) -> Self {
        self.theme.user_msg_color = user;
        self.theme.assistant_msg_color = assistant;
        self.theme.system_msg_color = system;
        self.theme.tool_msg_color = tool;
        self
    }

    /// Set status colors
    pub fn status_colors(mut self, info: Color, progress: Color) -> Self {
        self.theme.info_color = info;
        self.theme.progress_color = progress;
        self
    }

    /// Set all extended colors manually
    pub fn extended_colors(mut self, f: impl FnOnce(&mut Theme)) -> Self {
        f(&mut self.theme);
        self
    }

    /// Build the theme with intelligent defaults for any unset fields
    pub fn build(mut self) -> Theme {
        // Apply intelligent defaults for extended fields if not set
        if matches!(self.theme.input_bg_color, Color::Rgb(0, 0, 0)) {
            self.theme.input_bg_color = self.theme.code_bg_color;
        }
        if matches!(self.theme.input_placeholder_color, Color::Rgb(0, 0, 0)) {
            self.theme.input_placeholder_color = self.theme.dim_color;
        }
        if matches!(self.theme.input_border_color, Color::Rgb(0, 0, 0)) {
            self.theme.input_border_color = self.theme.border_color;
        }

        // Message backgrounds default to code_bg
        if matches!(self.theme.user_msg_bg_color, Color::Rgb(0, 0, 0)) {
            self.theme.user_msg_bg_color = self.theme.code_bg_color;
        }
        if matches!(self.theme.assistant_msg_bg_color, Color::Rgb(0, 0, 0)) {
            self.theme.assistant_msg_bg_color = self.theme.code_bg_color;
        }
        if matches!(self.theme.system_msg_bg_color, Color::Rgb(0, 0, 0)) {
            self.theme.system_msg_bg_color = self.theme.code_bg_color;
        }
        if matches!(self.theme.tool_msg_bg_color, Color::Rgb(0, 0, 0)) {
            self.theme.tool_msg_bg_color = self.theme.code_bg_color;
        }

        // Status bar and scrollbar
        if matches!(self.theme.status_bar_bg_color, Color::Rgb(0, 0, 0)) {
            self.theme.status_bar_bg_color = self.theme.code_bg_color;
        }
        if matches!(self.theme.scrollbar_bg_color, Color::Rgb(0, 0, 0)) {
            self.theme.scrollbar_bg_color = self.theme.code_bg_color;
        }
        if matches!(self.theme.scrollbar_fg_color, Color::Rgb(0, 0, 0)) {
            self.theme.scrollbar_fg_color = self.theme.border_color;
        }
        if matches!(self.theme.scrollbar_hover_color, Color::Rgb(0, 0, 0)) {
            self.theme.scrollbar_hover_color = self.theme.accent_color;
        }

        // Logo colors
        if matches!(self.theme.logo_primary_color, Color::Rgb(0, 0, 0)) {
            self.theme.logo_primary_color = self.theme.title_color;
        }
        if matches!(self.theme.logo_secondary_color, Color::Rgb(0, 0, 0)) {
            self.theme.logo_secondary_color = self.theme.accent_color;
        }

        // Animation colors
        if matches!(self.theme.animation_color, Color::Rgb(0, 0, 0)) {
            self.theme.animation_color = self.theme.title_color;
        }
        if matches!(self.theme.processing_color, Color::Rgb(0, 0, 0)) {
            self.theme.processing_color = self.theme.warning_color;
        }
        if matches!(self.theme.highlight_color, Color::Rgb(0, 0, 0)) {
            self.theme.highlight_color = self.theme.warning_color;
        }
        if matches!(self.theme.bubble_color, Color::Rgb(0, 0, 0)) {
            self.theme.bubble_color = self.theme.title_color;
        }

        // Token usage colors
        if matches!(self.theme.token_low_color, Color::Rgb(0, 0, 0)) {
            self.theme.token_low_color = self.theme.success_color;
        }
        if matches!(self.theme.token_medium_color, Color::Rgb(0, 0, 0)) {
            self.theme.token_medium_color = self.theme.warning_color;
        }
        if matches!(self.theme.token_high_color, Color::Rgb(0, 0, 0)) {
            self.theme.token_high_color = self.theme.mode_chat_color;
        }
        if matches!(self.theme.token_critical_color, Color::Rgb(0, 0, 0)) {
            self.theme.token_critical_color = self.theme.error_color;
        }

        // Syntax highlighting colors (with sensible defaults)
        if matches!(self.theme.syntax_keyword_color, Color::Rgb(0, 0, 0)) {
            self.theme.syntax_keyword_color = self.theme.mode_chat_color;
        }
        if matches!(self.theme.syntax_function_color, Color::Rgb(0, 0, 0)) {
            self.theme.syntax_function_color = self.theme.title_color;
        }
        if matches!(self.theme.syntax_string_color, Color::Rgb(0, 0, 0)) {
            self.theme.syntax_string_color = self.theme.success_color;
        }
        if matches!(self.theme.syntax_number_color, Color::Rgb(0, 0, 0)) {
            self.theme.syntax_number_color = self.theme.warning_color;
        }
        if matches!(self.theme.syntax_comment_color, Color::Rgb(0, 0, 0)) {
            self.theme.syntax_comment_color = self.theme.dim_color;
        }
        if matches!(self.theme.syntax_type_color, Color::Rgb(0, 0, 0)) {
            self.theme.syntax_type_color = self.theme.accent_color;
        }
        if matches!(self.theme.syntax_variable_color, Color::Rgb(0, 0, 0)) {
            self.theme.syntax_variable_color = self.theme.text_color;
        }
        if matches!(self.theme.syntax_operator_color, Color::Rgb(0, 0, 0)) {
            self.theme.syntax_operator_color = self.theme.text_color;
        }
        if matches!(self.theme.syntax_punctuation_color, Color::Rgb(0, 0, 0)) {
            self.theme.syntax_punctuation_color = self.theme.dim_color;
        }

        // Diff & code display colors (with sensible defaults)
        if matches!(self.theme.diff_add_color, Color::Rgb(0, 0, 0)) {
            self.theme.diff_add_color = self.theme.success_color;
        }
        if matches!(self.theme.diff_add_bg_color, Color::Rgb(0, 0, 0)) {
            // Derive a subtle background from success color
            if let Color::Rgb(r, g, b) = self.theme.success_color {
                self.theme.diff_add_bg_color = Color::Rgb(r / 6, g / 6, b / 6);
            } else {
                self.theme.diff_add_bg_color = Color::Rgb(20, 40, 20);
            }
        }
        if matches!(self.theme.diff_remove_color, Color::Rgb(0, 0, 0)) {
            self.theme.diff_remove_color = self.theme.error_color;
        }
        if matches!(self.theme.diff_remove_bg_color, Color::Rgb(0, 0, 0)) {
            // Derive a subtle background from error color
            if let Color::Rgb(r, g, b) = self.theme.error_color {
                self.theme.diff_remove_bg_color = Color::Rgb(r / 6, g / 6, b / 6);
            } else {
                self.theme.diff_remove_bg_color = Color::Rgb(40, 20, 20);
            }
        }
        if matches!(self.theme.diff_context_color, Color::Rgb(0, 0, 0)) {
            self.theme.diff_context_color = self.theme.dim_color;
        }
        if matches!(self.theme.line_number_color, Color::Rgb(0, 0, 0)) {
            self.theme.line_number_color = self.theme.dim_color;
        }
        if matches!(self.theme.link_color, Color::Rgb(0, 0, 0)) {
            self.theme.link_color = self.theme.accent_color;
        }
        if matches!(self.theme.running_color, Color::Rgb(0, 0, 0)) {
            self.theme.running_color = self.theme.accent_color;
        }

        balance_theme_contrast(&mut self.theme);
        self.theme
    }
}

fn balance_theme_contrast(theme: &mut Theme) {
    let is_high_contrast = theme.name == "high-contrast";

    if !is_high_contrast {
        theme.code_bg_color = normalize_surface(theme.code_bg_color, theme.bg_color, 1.05, 1.35);
        theme.input_bg_color = normalize_surface(theme.input_bg_color, theme.bg_color, 1.05, 1.35);
        theme.user_msg_bg_color =
            normalize_surface(theme.user_msg_bg_color, theme.bg_color, 1.02, 1.28);
        theme.assistant_msg_bg_color =
            normalize_surface(theme.assistant_msg_bg_color, theme.bg_color, 1.02, 1.28);
        theme.system_msg_bg_color =
            normalize_surface(theme.system_msg_bg_color, theme.bg_color, 1.02, 1.28);
        theme.tool_msg_bg_color =
            normalize_surface(theme.tool_msg_bg_color, theme.bg_color, 1.02, 1.28);
        theme.status_bar_bg_color =
            normalize_surface(theme.status_bar_bg_color, theme.bg_color, 1.03, 1.32);
        theme.scrollbar_bg_color =
            normalize_surface(theme.scrollbar_bg_color, theme.bg_color, 1.03, 1.32);
        theme.selection_bg_color =
            normalize_surface(theme.selection_bg_color, theme.bg_color, 1.08, 1.9);
    }

    theme.text_color = ensure_min_contrast(theme.text_color, theme.bg_color, 4.5);
    theme.dim_color = ensure_min_contrast(theme.dim_color, theme.bg_color, 2.4);
    theme.border_color = ensure_min_contrast(theme.border_color, theme.bg_color, 1.35);

    theme.title_color = ensure_min_contrast(theme.title_color, theme.bg_color, 2.5);
    theme.accent_color = ensure_min_contrast(theme.accent_color, theme.bg_color, 2.5);
    theme.success_color = ensure_min_contrast(theme.success_color, theme.bg_color, 2.5);
    theme.warning_color = ensure_min_contrast(theme.warning_color, theme.bg_color, 2.5);
    theme.error_color = ensure_min_contrast(theme.error_color, theme.bg_color, 2.5);

    theme.mode_view_color = ensure_min_contrast(theme.mode_view_color, theme.bg_color, 2.5);
    theme.mode_chat_color = ensure_min_contrast(theme.mode_chat_color, theme.bg_color, 2.5);
    theme.mode_plan_color = ensure_min_contrast(theme.mode_plan_color, theme.bg_color, 2.5);
    theme.mode_bash_color = ensure_min_contrast(theme.mode_bash_color, theme.bg_color, 2.5);
    theme.mode_leader_color = ensure_min_contrast(theme.mode_leader_color, theme.bg_color, 2.5);

    theme.input_placeholder_color =
        ensure_min_contrast(theme.input_placeholder_color, theme.input_bg_color, 2.5);
    theme.selection_fg_color =
        ensure_min_contrast(theme.selection_fg_color, theme.selection_bg_color, 4.5);

    theme.user_msg_color = ensure_min_contrast(theme.user_msg_color, theme.user_msg_bg_color, 3.0);
    theme.assistant_msg_color =
        ensure_min_contrast(theme.assistant_msg_color, theme.assistant_msg_bg_color, 3.0);
    theme.system_msg_color =
        ensure_min_contrast(theme.system_msg_color, theme.system_msg_bg_color, 3.0);
    theme.tool_msg_color = ensure_min_contrast(theme.tool_msg_color, theme.tool_msg_bg_color, 3.0);

    theme.info_color = ensure_min_contrast(theme.info_color, theme.bg_color, 2.5);
    theme.progress_color = ensure_min_contrast(theme.progress_color, theme.bg_color, 2.5);
    theme.link_color = ensure_min_contrast(theme.link_color, theme.bg_color, 2.5);
    theme.running_color = ensure_min_contrast(theme.running_color, theme.bg_color, 2.5);

    theme.scrollbar_fg_color =
        ensure_min_contrast(theme.scrollbar_fg_color, theme.scrollbar_bg_color, 1.6);
    theme.scrollbar_hover_color =
        ensure_min_contrast(theme.scrollbar_hover_color, theme.scrollbar_bg_color, 2.3);

    theme.syntax_keyword_color =
        ensure_min_contrast(theme.syntax_keyword_color, theme.code_bg_color, 2.4);
    theme.syntax_function_color =
        ensure_min_contrast(theme.syntax_function_color, theme.code_bg_color, 2.4);
    theme.syntax_string_color =
        ensure_min_contrast(theme.syntax_string_color, theme.code_bg_color, 2.4);
    theme.syntax_number_color =
        ensure_min_contrast(theme.syntax_number_color, theme.code_bg_color, 2.4);
    theme.syntax_comment_color =
        ensure_min_contrast(theme.syntax_comment_color, theme.code_bg_color, 2.0);
    theme.syntax_type_color =
        ensure_min_contrast(theme.syntax_type_color, theme.code_bg_color, 2.4);
    theme.syntax_variable_color =
        ensure_min_contrast(theme.syntax_variable_color, theme.code_bg_color, 2.4);
    theme.syntax_operator_color =
        ensure_min_contrast(theme.syntax_operator_color, theme.code_bg_color, 2.4);
    theme.syntax_punctuation_color =
        ensure_min_contrast(theme.syntax_punctuation_color, theme.code_bg_color, 2.0);

    theme.diff_add_color = ensure_min_contrast(theme.diff_add_color, theme.diff_add_bg_color, 2.4);
    theme.diff_remove_color =
        ensure_min_contrast(theme.diff_remove_color, theme.diff_remove_bg_color, 2.4);
    theme.diff_context_color =
        ensure_min_contrast(theme.diff_context_color, theme.code_bg_color, 1.8);
    theme.line_number_color =
        ensure_min_contrast(theme.line_number_color, theme.code_bg_color, 1.8);
}

fn normalize_surface(surface: Color, bg: Color, min_ratio: f32, max_ratio: f32) -> Color {
    let Some(ratio) = contrast_ratio(surface, bg) else {
        return surface;
    };

    if ratio < min_ratio {
        return ensure_min_contrast(surface, bg, min_ratio);
    }
    if ratio > max_ratio {
        return reduce_max_contrast(surface, bg, max_ratio);
    }
    surface
}

fn ensure_min_contrast(fg: Color, bg: Color, min_ratio: f32) -> Color {
    let Some(current) = contrast_ratio(fg, bg) else {
        return fg;
    };
    if current >= min_ratio {
        return fg;
    }

    let white = Color::Rgb(255, 255, 255);
    let black = Color::Rgb(0, 0, 0);
    let white_ratio = contrast_ratio(white, bg).unwrap_or(current);
    let black_ratio = contrast_ratio(black, bg).unwrap_or(current);
    let target = if white_ratio >= black_ratio {
        white
    } else {
        black
    };

    let mut low = 0.0_f32;
    let mut high = 1.0_f32;
    for _ in 0..12 {
        let mid = (low + high) / 2.0;
        let candidate = mix_colors(fg, target, mid);
        if contrast_ratio(candidate, bg).unwrap_or(0.0) >= min_ratio {
            high = mid;
        } else {
            low = mid;
        }
    }
    mix_colors(fg, target, high)
}

fn reduce_max_contrast(surface: Color, bg: Color, max_ratio: f32) -> Color {
    let Some(current) = contrast_ratio(surface, bg) else {
        return surface;
    };
    if current <= max_ratio {
        return surface;
    }

    let mut low = 0.0_f32;
    let mut high = 1.0_f32;
    for _ in 0..12 {
        let mid = (low + high) / 2.0;
        let candidate = mix_colors(surface, bg, mid);
        if contrast_ratio(candidate, bg).unwrap_or(current) > max_ratio {
            low = mid;
        } else {
            high = mid;
        }
    }
    mix_colors(surface, bg, high)
}

fn mix_colors(from: Color, to: Color, amount: f32) -> Color {
    let (fr, fg, fb) = match from {
        Color::Rgb(r, g, b) => (r as f32, g as f32, b as f32),
        _ => return from,
    };
    let (tr, tg, tb) = match to {
        Color::Rgb(r, g, b) => (r as f32, g as f32, b as f32),
        _ => return from,
    };

    let t = amount.clamp(0.0, 1.0);
    let mix = |a: f32, b: f32| ((a + (b - a) * t).round() as i32).clamp(0, 255) as u8;
    Color::Rgb(mix(fr, tr), mix(fg, tg), mix(fb, tb))
}

fn contrast_ratio(a: Color, b: Color) -> Option<f32> {
    let la = relative_luminance(a)?;
    let lb = relative_luminance(b)?;
    let (light, dark) = if la >= lb { (la, lb) } else { (lb, la) };
    Some((light + 0.05) / (dark + 0.05))
}

fn relative_luminance(color: Color) -> Option<f32> {
    let (r, g, b) = match color {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => return None,
    };
    let linear = |channel: u8| {
        let value = channel as f32 / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };

    Some(0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b))
}
