//! The general-UI half of a theme — colors for chrome (borders, status
//! bar, popups, selection) as opposed to syntax token colors. Every
//! render-side call site (status bar, borders, popups) should reach into
//! `Theme.ui` through the semantic accessor methods below rather than
//! reading `ui.colors.foo` directly — that keeps "what color is an error"
//! a single decision made here, not re-decided at every call site.

use super::color::parse_color;
use crate::language::lsp::DiagnosticSeverity;
use ratatui::style::{Color, Style};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct UiTheme {
    pub colors: UiColors,
    /// Optional per-component overrides. Anything not set here falls back
    /// to `colors` : e.g. a theme can override just `popup_border` without
    /// having to restate every other color.
    pub components: UiComponentOverrides,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            colors: UiColors::default(),
            components: UiComponentOverrides::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct UiColors {
    pub background: String,
    pub foreground: String,
    pub border: String,
    pub border_focused: String,
    pub selection_bg: String,
    pub selection_fg: Option<String>,
    pub cursor: String,
    pub status_bar_bg: String,
    pub status_bar_fg: String,
    pub error: String,
    pub warning: String,
    pub info: String,
    pub hint: String,
    pub success: String,
    pub line_number: String,
    pub line_number_current: String,
    pub current_line_bg: String,
    pub command_line_bg: String,
    pub command_line_fg: String,
    pub list_highlight_bg: String,
    pub list_highlight_fg: String,

    /// Debug-log severity colors. Deliberately separate from
    /// `error`/`warning`/`info`/`hint` above: those are LSP diagnostic
    /// severities, these are internal log levels, and the original
    /// hardcoded UI already colored them differently (log Info was Blue,
    /// diagnostic Information was Cyan) — collapsing the two would lose a
    /// real, intentional visual distinction.
    pub log_error: String,
    pub log_warning: String,
    pub log_info: String,
    pub log_debug: String,
    pub log_trace: String,

    /// Generic categorical accents — used where a color's only job is to
    /// visually distinguish one category from another (undo vs. redo,
    /// clipboard entries, snapshot triggers), not to carry semantic
    /// weight the way `error`/`warning` do. Themes can still recolor
    /// these; they just aren't tied to a specific meaning.
    pub accent_red: String,
    pub accent_blue: String,
    pub accent_cyan: String,
    pub accent_green: String,
    pub accent_yellow: String,
    pub accent_magenta: String,
}

impl Default for UiColors {
    /// Sane, always-visible colors so a `UiTheme` never renders as
    /// invisible-on-invisible if a theme file omits a field, matches the
    /// same degrade gracefully idea behind `parse_color`'s Reset
    /// fallback.
    fn default() -> Self {
        Self {
            background: "#1e1e1e".to_string(),
            foreground: "#d4d4d4".to_string(),
            border: "#3c3c3c".to_string(),
            border_focused: "#569cd6".to_string(),
            selection_bg: "#264f78".to_string(),
            selection_fg: None,
            cursor: "#d4d4d4".to_string(),
            status_bar_bg: "#007acc".to_string(),
            status_bar_fg: "#ffffff".to_string(),
            error: "#f44747".to_string(),
            warning: "#cca700".to_string(),
            info: "#4ec9b0".to_string(),
            hint: "#808080".to_string(),
            success: "#89d185".to_string(),
            line_number: "#858585".to_string(),
            line_number_current: "#c6c6c6".to_string(),
            current_line_bg: "#4d4d4d".to_string(),
            command_line_bg: "#007acc".to_string(),
            command_line_fg: "#ffffff".to_string(),
            list_highlight_bg: "#3c3c3c".to_string(),
            list_highlight_fg: "#ffffff".to_string(),
            log_error: "#f44747".to_string(),
            log_warning: "#cca700".to_string(),
            log_info: "#3794ff".to_string(),
            log_debug: "#808080".to_string(),
            log_trace: "#5a5a5a".to_string(),
            accent_red: "#f44747".to_string(),
            accent_blue: "#3794ff".to_string(),
            accent_cyan: "#4ec9b0".to_string(),
            accent_green: "#89d185".to_string(),
            accent_yellow: "#cca700".to_string(),
            accent_magenta: "#c586c0".to_string(),
        }
    }
}

/// Per-component color overrides. Every field is optional. unset fields
/// fall back to the matching general `UiColors` field.
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct UiComponentOverrides {
    pub popup_bg: Option<String>,
    pub popup_border: Option<String>,
    pub popup_fg: Option<String>,
    pub debug_console_bg: Option<String>,
}

impl UiTheme {
    pub fn background(&self) -> Color {
        parse_color(&self.colors.background)
    }
    pub fn foreground(&self) -> Color {
        parse_color(&self.colors.foreground)
    }
    pub fn border(&self) -> Color {
        parse_color(&self.colors.border)
    }
    pub fn border_focused(&self) -> Color {
        parse_color(&self.colors.border_focused)
    }
    pub fn cursor(&self) -> Color {
        parse_color(&self.colors.cursor)
    }
    pub fn line_number(&self) -> Color {
        parse_color(&self.colors.line_number)
    }
    pub fn line_number_current(&self) -> Color {
        parse_color(&self.colors.line_number_current)
    }
    pub fn current_line_bg(&self) -> Color {
        parse_color(&self.colors.current_line_bg)
    }
    pub fn list_highlight_style(&self) -> Style {
        Style::default()
            .bg(parse_color(&self.colors.list_highlight_bg))
            .fg(parse_color(&self.colors.list_highlight_fg))
            .add_modifier(ratatui::style::Modifier::BOLD)
    }
    pub fn command_line_style(&self) -> Style {
        Style::default()
            .bg(parse_color(&self.colors.command_line_bg))
            .fg(parse_color(&self.colors.command_line_fg))
    }

    pub fn accent_red(&self) -> Color {
        parse_color(&self.colors.accent_red)
    }
    pub fn accent_blue(&self) -> Color {
        parse_color(&self.colors.accent_blue)
    }
    pub fn accent_cyan(&self) -> Color {
        parse_color(&self.colors.accent_cyan)
    }
    pub fn accent_green(&self) -> Color {
        parse_color(&self.colors.accent_green)
    }
    pub fn accent_yellow(&self) -> Color {
        parse_color(&self.colors.accent_yellow)
    }
    pub fn accent_magenta(&self) -> Color {
        parse_color(&self.colors.accent_magenta)
    }

    /// Color for a debug-log severity level. See the doc comment on
    /// `UiColors::log_error` etc. for why this is distinct from
    /// `severity_color` (LSP diagnostics).
    pub fn log_level_color(&self, level: crate::core::debug::LogLevel) -> Color {
        use crate::core::debug::LogLevel;
        match level {
            LogLevel::Error => parse_color(&self.colors.log_error),
            LogLevel::Warn => parse_color(&self.colors.log_warning),
            LogLevel::Info => parse_color(&self.colors.log_info),
            LogLevel::Debug => parse_color(&self.colors.log_debug),
            LogLevel::Trace => parse_color(&self.colors.log_trace),
        }
    }

    /// Style for the current text selection highlight.
    pub fn selection_style(&self) -> Style {
        let mut style = Style::default().bg(parse_color(&self.colors.selection_bg));
        if let Some(fg) = &self.colors.selection_fg {
            style = style.fg(parse_color(fg));
        }
        style
    }

    pub fn status_bar_style(&self) -> Style {
        Style::default()
            .bg(parse_color(&self.colors.status_bar_bg))
            .fg(parse_color(&self.colors.status_bar_fg))
    }

    /// Resolves a component-level background, falling back to the general
    /// popup default (background color) when the theme doesn't
    /// override it.
    pub fn popup_bg(&self) -> Color {
        self.components
            .popup_bg
            .as_deref()
            .map(parse_color)
            .unwrap_or_else(|| self.background())
    }

    pub fn popup_border(&self) -> Color {
        self.components
            .popup_border
            .as_deref()
            .map(parse_color)
            .unwrap_or_else(|| self.border_focused())
    }

    pub fn popup_fg(&self) -> Color {
        self.components
            .popup_fg
            .as_deref()
            .map(parse_color)
            .unwrap_or_else(|| self.foreground())
    }

    /// One place that decides what color a diagnostic severity renders as
    /// — used by both gutter icons and the diagnostics panel, so they can
    /// never disagree with each other.
    pub fn hint_text(&self) -> Color {
        parse_color(&self.colors.hint)
    }

    pub fn success(&self) -> Color {
        parse_color(&self.colors.success)
    }

    /// One place that decides what color a diagnostic severity renders as
    /// used by both gutter icons and the diagnostics panel, so they can
    /// never disagree with each other.
    pub fn severity_color(&self, severity: DiagnosticSeverity) -> Color {
        match severity {
            DiagnosticSeverity::Error => parse_color(&self.colors.error),
            DiagnosticSeverity::Warning => parse_color(&self.colors.warning),
            DiagnosticSeverity::Information => parse_color(&self.colors.info),
            DiagnosticSeverity::Hint => parse_color(&self.colors.hint),
        }
    }
}

#[cfg(test)]
mod unit_ui_theme_tests {
    use super::*;

    #[test]
    fn default_colors_are_all_valid_non_reset_colors() {
        let ui = UiTheme::default();
        // Every default hex should parse to something other than Reset —
        // if this fails, a hex literal above has a typo.
        assert_ne!(ui.background(), Color::Reset);
        assert_ne!(ui.foreground(), Color::Reset);
        assert_ne!(ui.border(), Color::Reset);
        assert_ne!(ui.border_focused(), Color::Reset);
        assert_ne!(ui.cursor(), Color::Reset);
    }

    #[test]
    fn component_override_falls_back_when_unset() {
        let ui = UiTheme::default();
        assert_eq!(ui.popup_border(), ui.border_focused());
    }

    #[test]
    fn component_override_wins_when_set() {
        let mut ui = UiTheme::default();
        ui.components.popup_border = Some("#ff00ff".to_string());
        assert_eq!(ui.popup_border(), Color::Rgb(255, 0, 255));
    }

    #[test]
    fn selection_style_applies_bg_and_optional_fg() {
        let mut ui = UiTheme::default();
        ui.colors.selection_bg = "#112233".to_string();
        ui.colors.selection_fg = Some("#ffffff".to_string());
        let style = ui.selection_style();
        assert_eq!(style.bg, Some(Color::Rgb(0x11, 0x22, 0x33)));
        assert_eq!(style.fg, Some(Color::Rgb(255, 255, 255)));
    }

    #[test]
    fn selection_style_without_fg_leaves_fg_unset() {
        let mut ui = UiTheme::default();
        ui.colors.selection_fg = None;
        let style = ui.selection_style();
        assert_eq!(style.fg, None);
        assert!(style.bg.is_some());
    }

    #[test]
    fn every_severity_maps_to_a_distinct_color_by_default() {
        use crate::language::lsp::DiagnosticSeverity;
        let ui = UiTheme::default();
        let colors = [
            ui.severity_color(DiagnosticSeverity::Error),
            ui.severity_color(DiagnosticSeverity::Warning),
            ui.severity_color(DiagnosticSeverity::Information),
            ui.severity_color(DiagnosticSeverity::Hint),
        ];
        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                assert_ne!(
                    colors[i], colors[j],
                    "severities {} and {} share a color in the default theme",
                    i, j
                );
            }
        }
    }

    #[test]
    fn every_log_level_maps_to_a_distinct_color_by_default() {
        use crate::core::debug::LogLevel;
        let ui = UiTheme::default();
        let colors = [
            ui.log_level_color(LogLevel::Error),
            ui.log_level_color(LogLevel::Warn),
            ui.log_level_color(LogLevel::Info),
            ui.log_level_color(LogLevel::Debug),
            ui.log_level_color(LogLevel::Trace),
        ];
        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                assert_ne!(
                    colors[i], colors[j],
                    "log levels {} and {} share a color in the default theme",
                    i, j
                );
            }
        }
    }

    #[test]
    fn log_info_and_diagnostic_information_are_intentionally_different() {
        // The whole reason log_* colors exist separately from
        // error/warning/info/hint: they're conceptually different things
        // (internal debug logs vs. LSP diagnostics) and the original UI
        // colored them differently (log Info was Blue, diagnostic
        // Information was Cyan). This pins that distinction so a future
        // "simplification" doesn't quietly collapse it.
        use crate::core::debug::LogLevel;
        use crate::language::lsp::DiagnosticSeverity;
        let ui = UiTheme::default();
        assert_ne!(
            ui.log_level_color(LogLevel::Info),
            ui.severity_color(DiagnosticSeverity::Information)
        );
    }

    #[test]
    fn all_accent_colors_are_distinct_by_default() {
        let ui = UiTheme::default();
        let accents = [
            ui.accent_red(),
            ui.accent_blue(),
            ui.accent_cyan(),
            ui.accent_green(),
            ui.accent_yellow(),
            ui.accent_magenta(),
        ];
        for i in 0..accents.len() {
            for j in (i + 1)..accents.len() {
                assert_ne!(accents[i], accents[j]);
            }
        }
    }

    #[test]
    fn all_component_overrides_fall_back_when_unset() {
        let ui = UiTheme::default();
        assert_eq!(ui.popup_bg(), ui.background());
        assert_eq!(ui.popup_border(), ui.border_focused());
        assert_eq!(ui.popup_fg(), ui.foreground());
    }

    #[test]
    fn all_component_overrides_win_when_set() {
        let mut ui = UiTheme::default();
        ui.components.popup_bg = Some("#111111".to_string());
        ui.components.popup_border = Some("#222222".to_string());
        ui.components.popup_fg = Some("#333333".to_string());

        assert_eq!(ui.popup_bg(), Color::Rgb(0x11, 0x11, 0x11));
        assert_eq!(ui.popup_border(), Color::Rgb(0x22, 0x22, 0x22));
        assert_eq!(ui.popup_fg(), Color::Rgb(0x33, 0x33, 0x33));
    }

    #[test]
    fn status_bar_style_uses_both_bg_and_fg() {
        let mut ui = UiTheme::default();
        ui.colors.status_bar_bg = "#010101".to_string();
        ui.colors.status_bar_fg = "#fefefe".to_string();
        let style = ui.status_bar_style();
        assert_eq!(style.bg, Some(Color::Rgb(1, 1, 1)));
        assert_eq!(style.fg, Some(Color::Rgb(254, 254, 254)));
    }

    #[test]
    fn command_line_style_uses_both_bg_and_fg() {
        let mut ui = UiTheme::default();
        ui.colors.command_line_bg = "#020202".to_string();
        ui.colors.command_line_fg = "#fdfdfd".to_string();
        let style = ui.command_line_style();
        assert_eq!(style.bg, Some(Color::Rgb(2, 2, 2)));
        assert_eq!(style.fg, Some(Color::Rgb(253, 253, 253)));
    }

    #[test]
    fn list_highlight_style_is_bold() {
        let ui = UiTheme::default();
        let style = ui.list_highlight_style();
        assert!(style.add_modifier.contains(ratatui::style::Modifier::BOLD));
        assert!(style.bg.is_some());
        assert!(style.fg.is_some());
    }

    #[test]
    fn simple_color_accessors_all_resolve_to_something_other_than_reset() {
        let ui = UiTheme::default();
        assert_ne!(ui.background(), Color::Reset);
        assert_ne!(ui.foreground(), Color::Reset);
        assert_ne!(ui.border(), Color::Reset);
        assert_ne!(ui.border_focused(), Color::Reset);
        assert_ne!(ui.cursor(), Color::Reset);
        assert_ne!(ui.line_number(), Color::Reset);
        assert_ne!(ui.line_number_current(), Color::Reset);
        assert_ne!(ui.current_line_bg(), Color::Reset);
        assert_ne!(ui.hint_text(), Color::Reset);
        assert_ne!(ui.success(), Color::Reset);
    }

    #[test]
    fn malformed_hex_in_a_color_field_degrades_to_reset_not_a_panic() {
        let mut ui = UiTheme::default();
        ui.colors.foreground = "not-a-hex-value".to_string();
        // The whole point of parse_color's Reset fallback is that a typo'd
        // theme file degrades gracefully instead of crashing the editor.
        assert_eq!(ui.foreground(), Color::Reset);
    }

    #[test]
    fn deserializes_from_partial_toml_filling_remaining_colors_with_defaults() {
        let ui: UiTheme = toml::from_str(
            r##"
            [colors]
            background = "#000000"
            "##,
        )
        .unwrap();
        assert_eq!(ui.background(), Color::Rgb(0, 0, 0));
        // Untouched fields should still carry the UiColors::default() value
        assert_eq!(ui.colors.foreground, UiColors::default().foreground);
    }

    #[test]
    fn deserializes_from_completely_empty_toml() {
        let ui: UiTheme = toml::from_str("").unwrap();
        assert_eq!(ui.colors.background, UiColors::default().background);
    }

    #[test]
    fn clone_is_independent_for_component_overrides() {
        let original = UiTheme::default();
        let mut cloned = original.clone();
        cloned.components.popup_bg = Some("#abcdef".to_string());
        assert!(original.components.popup_bg.is_none());
        assert!(cloned.components.popup_bg.is_some());
    }
}
