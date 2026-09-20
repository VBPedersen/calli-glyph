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
}