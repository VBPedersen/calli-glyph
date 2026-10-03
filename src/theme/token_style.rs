//! A single named style: a color plus modifiers. Used both for syntax
//! token styles (`[syntax.tokens]`) and could be reused for UI component
//! overrides that need more than a flat color later.

use super::color::parse_color;
use ratatui::style::{Modifier, Style};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub struct TokenStyle {
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
}

impl TokenStyle {
    pub fn to_style(&self) -> Style {
        let mut style = Style::default();
        if let Some(fg) = &self.fg {
            style = style.fg(parse_color(fg));
        }
        if let Some(bg) = &self.bg {
            style = style.bg(parse_color(bg));
        }
        if self.bold == Some(true) {
            style = style.add_modifier(Modifier::BOLD);
        }
        if self.italic == Some(true) {
            style = style.add_modifier(Modifier::ITALIC);
        }
        if self.underline == Some(true) {
            style = style.add_modifier(Modifier::UNDERLINED);
        }
        style
    }
}

#[cfg(test)]
mod unit_token_style_tests {
    use super::*;

    #[test]
    fn empty_style_produces_default_style() {
        let ts = TokenStyle::default();
        assert_eq!(ts.to_style(), Style::default());
    }

    #[test]
    fn applies_fg_and_bold() {
        let ts = TokenStyle {
            fg: Some("#ff0000".to_string()),
            bold: Some(true),
            ..Default::default()
        };
        let style = ts.to_style();
        assert_eq!(style.fg, Some(ratatui::style::Color::Rgb(255, 0, 0)));
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn false_modifiers_are_not_applied() {
        let ts = TokenStyle {
            bold: Some(false),
            italic: Some(false),
            ..Default::default()
        };
        let style = ts.to_style();
        assert!(!style.add_modifier.contains(Modifier::BOLD));
        assert!(!style.add_modifier.contains(Modifier::ITALIC));
    }

    #[test]
    fn none_modifiers_are_not_applied() {
        // Distinguishes "not set" (None) from "explicitly false" — both
        // should leave the modifier off, but they're different states in
        // the underlying Option, worth covering separately.
        let ts = TokenStyle::default();
        let style = ts.to_style();
        assert!(!style.add_modifier.contains(Modifier::BOLD));
        assert!(!style.add_modifier.contains(Modifier::ITALIC));
        assert!(!style.add_modifier.contains(Modifier::UNDERLINED));
    }

    #[test]
    fn applies_bg_independent_of_fg() {
        let ts = TokenStyle {
            bg: Some("#0000ff".to_string()),
            ..Default::default()
        };
        let style = ts.to_style();
        assert_eq!(style.bg, Some(ratatui::style::Color::Rgb(0, 0, 255)));
        assert_eq!(style.fg, None);
    }

    #[test]
    fn applies_underline() {
        let ts = TokenStyle {
            underline: Some(true),
            ..Default::default()
        };
        assert!(ts.to_style().add_modifier.contains(Modifier::UNDERLINED));
    }

    #[test]
    fn applies_all_fields_simultaneously() {
        let ts = TokenStyle {
            fg: Some("#ff0000".to_string()),
            bg: Some("#0000ff".to_string()),
            bold: Some(true),
            italic: Some(true),
            underline: Some(true),
        };
        let style = ts.to_style();
        assert_eq!(style.fg, Some(ratatui::style::Color::Rgb(255, 0, 0)));
        assert_eq!(style.bg, Some(ratatui::style::Color::Rgb(0, 0, 255)));
        assert!(style.add_modifier.contains(Modifier::BOLD));
        assert!(style.add_modifier.contains(Modifier::ITALIC));
        assert!(style.add_modifier.contains(Modifier::UNDERLINED));
    }

    #[test]
    fn invalid_color_string_degrades_to_reset_rather_than_panicking() {
        let ts = TokenStyle {
            fg: Some("not-a-real-color".to_string()),
            ..Default::default()
        };
        assert_eq!(ts.to_style().fg, Some(ratatui::style::Color::Reset));
    }

    #[test]
    fn deserializes_from_partial_toml() {
        let ts: TokenStyle = toml::from_str("fg = \"#ff0000\"\nbold = true").unwrap();
        assert_eq!(ts.fg.as_deref(), Some("#ff0000"));
        assert_eq!(ts.bold, Some(true));
        assert_eq!(ts.bg, None);
        assert_eq!(ts.italic, None);
    }

    #[test]
    fn deserializes_from_empty_toml_table() {
        let ts: TokenStyle = toml::from_str("").unwrap();
        assert_eq!(ts, TokenStyle::default());
    }

    #[test]
    fn serialize_then_deserialize_round_trips() {
        let original = TokenStyle {
            fg: Some("#abcdef".to_string()),
            bg: Some("#123456".to_string()),
            bold: Some(true),
            italic: Some(false),
            underline: None,
        };
        let toml_str = toml::to_string(&original).unwrap();
        let round_tripped: TokenStyle = toml::from_str(&toml_str).unwrap();
        assert_eq!(original, round_tripped);
    }
}
