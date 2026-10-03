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

    #[test]
    fn token_lookup_is_case_sensitive() {
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

        // "Keyword" (capitalized) is a different key entirely — tree-sitter
        // node kind strings are always lowercase, so this should NOT match
        // and should fall back to defaults instead.
        assert_eq!(theme.style_for("Keyword"), theme.style_for("unrelated"));
    }

    #[test]
    fn empty_string_token_type_falls_back_to_defaults() {
        let theme = SyntaxTheme {
            tokens: HashMap::new(),
            defaults: TokenStyle {
                fg: Some("#123456".to_string()),
                ..Default::default()
            },
        };
        assert_eq!(
            theme.style_for(""),
            theme.style_for("anything_else_unmapped")
        );
    }

    #[test]
    fn defaults_with_no_fields_set_produces_plain_style() {
        let theme = SyntaxTheme::default();
        let style = theme.style_for("whatever");
        assert_eq!(style.fg, None);
        assert_eq!(style.bg, None);
    }

    #[test]
    fn clone_is_independent_of_original() {
        let mut tokens = HashMap::new();
        tokens.insert("keyword".to_string(), TokenStyle::default());
        let original = SyntaxTheme {
            tokens,
            defaults: TokenStyle::default(),
        };
        let mut cloned = original.clone();
        cloned.tokens.insert(
            "string".to_string(),
            TokenStyle {
                fg: Some("#ff0000".to_string()),
                ..Default::default()
            },
        );

        assert!(!original.tokens.contains_key("string"));
        assert!(cloned.tokens.contains_key("string"));
    }

    #[test]
    fn deserializes_from_toml_with_multiple_tokens() {
        let toml_str = r##"
            [defaults]
            fg = "#ffffff"

            [tokens.keyword]
            fg = "#ff0000"
            bold = true

            [tokens.string]
            fg = "#00ff00"
        "##;
        let theme: SyntaxTheme = toml::from_str(toml_str).unwrap();
        assert_eq!(theme.tokens.len(), 2);
        assert_eq!(theme.defaults.fg.as_deref(), Some("#ffffff"));
        assert_eq!(
            theme.style_for("keyword").fg,
            Some(ratatui::style::Color::Rgb(255, 0, 0))
        );
        assert_eq!(
            theme.style_for("string").fg,
            Some(ratatui::style::Color::Rgb(0, 255, 0))
        );
    }

    #[test]
    fn deserializes_from_empty_toml_with_all_defaults() {
        let theme: SyntaxTheme = toml::from_str("").unwrap();
        assert!(theme.tokens.is_empty());
        assert_eq!(theme.defaults, TokenStyle::default());
    }
}
