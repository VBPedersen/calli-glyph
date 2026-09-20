//! The syntax-highlighting half of a theme : maps tree-sitter token type
//! names to styles. This is the *only* part of the theme system
//! `LanguageManager` needs, and the only part it holds: it has no business
//! knowing about UI colors or layout preferences.
//!
//! Directly succeeds the old `language::theme::Theme` : `style_for` keeps
//! the same name and signature, so call sites barely change.

use super::token_style::TokenStyle;
use ratatui::style::Style;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct SyntaxTheme {
    /// tree-sitter token type name (e.g. "keyword", "string") -> style
    pub tokens: HashMap<String, TokenStyle>,
    /// Fallback style for any token type not present in `tokens`.
    pub defaults: TokenStyle,
}

impl SyntaxTheme {
    /// Get the resolved style for a token type, falling back to `defaults`
    /// when the type isn't explicitly mapped.
    pub fn style_for(&self, token_type: &str) -> Style {
        self.tokens
            .get(token_type)
            .unwrap_or(&self.defaults)
            .to_style()
    }
}

#[cfg(test)]
mod unit_syntax_theme_tests {
    use super::*;

    #[test]
    fn style_for_known_token_uses_its_mapping() {
        let mut tokens = HashMap::new();
        tokens.insert(
            "keyword".to_string(),
            TokenStyle {
                fg: Some("#ff0000".to_string()),
                ..Default::default()
            },
        );
        let theme = SyntaxTheme {
            tokens,
            defaults: TokenStyle::default(),
        };

        let style = theme.style_for("keyword");
        assert_eq!(style.fg, Some(ratatui::style::Color::Rgb(255, 0, 0)));
    }

    #[test]
    fn style_for_unknown_token_falls_back_to_defaults() {
        let theme = SyntaxTheme {
            tokens: HashMap::new(),
            defaults: TokenStyle {
                fg: Some("#00ff00".to_string()),
                ..Default::default()
            },
        };

        let style = theme.style_for("some_unmapped_node_kind");
        assert_eq!(style.fg, Some(ratatui::style::Color::Rgb(0, 255, 0)));
    }

    #[test]
    fn empty_theme_produces_default_style_for_anything() {
        let theme = SyntaxTheme::default();
        assert_eq!(theme.style_for("keyword"), Style::default());
    }
}