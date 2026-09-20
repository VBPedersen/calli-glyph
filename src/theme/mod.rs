//! The unified theme system. One theme = one `.toml` file with four
//! sections (`[meta]`, `[syntax]`, `[ui]`, `[layout]`), loaded through
//! `ThemeManager` and exposed to the rest of the app as `Theme`.
//!
//! Each section is its own type on purpose: `LanguageManager` should only
//! ever see `Theme.syntax` (a `SyntaxTheme`), render code should only ever
//! see `Theme.ui`/`Theme.layout`, nobody has to reach through a concern
//! that isn't theirs to get what they need.

pub mod color;
pub mod layout;
pub mod manager;
pub mod meta;
pub mod syntax_theme;
pub mod token_style;
pub mod ui_theme;

pub use layout::{BorderStyle, LayoutOptions, LineNumberStyle, StatusBarPosition};
pub use manager::ThemeManager;
pub use meta::ThemeMeta;
pub use syntax_theme::SyntaxTheme;
pub use token_style::TokenStyle;
pub use ui_theme::{UiColors, UiComponentOverrides, UiTheme};

use serde::{Deserialize, Serialize};

/// Compiled-in fallback themes — always available even if `themes_dir`
/// doesn't exist yet or is unreadable, mirroring the same
/// "never fail to render, degrade instead" philosophy as
/// `HelpRegistry::empty()`. `pub(crate)` so `ThemeManager` can write the
/// raw text out to disk on first launch (preserving the hand-authored
/// formatting/comments, rather than round-tripping through a parsed
/// `Theme` and re-serializing).
pub(crate) const EMBEDDED_DARK: &str = include_str!("defaults/dark.toml");
pub(crate) const EMBEDDED_LIGHT: &str = include_str!("defaults/light.toml");
pub(crate) const EMBEDDED_DRACULA: &str = include_str!("defaults/dracula.toml");

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct Theme {
    pub meta: ThemeMeta,
    pub syntax: SyntaxTheme,
    pub ui: UiTheme,
    pub layout: LayoutOptions,
}

impl Theme {
    pub fn from_toml_str(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    pub fn from_file(path: &std::path::Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        Self::from_toml_str(&content)
            .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
    }

    /// The embedded "dark" theme — used as the manager's last-resort
    /// fallback, and as the source written to disk on first launch.
    pub fn embedded_dark() -> Self {
        Self::from_toml_str(EMBEDDED_DARK).expect("embedded dark.toml must always parse")
    }

    /// The embedded "light" theme.
    pub fn embedded_light() -> Self {
        Self::from_toml_str(EMBEDDED_LIGHT).expect("embedded light.toml must always parse")
    }

    /// The embedded "dracula" theme.
    pub fn embedded_dracula() -> Self {
        Self::from_toml_str(EMBEDDED_DRACULA).expect("embedded dracula.toml must always parse")
    }
}

#[cfg(test)]
mod unit_theme_tests {
    use super::*;

    #[test]
    fn embedded_dark_parses_and_has_a_name() {
        let theme = Theme::embedded_dark();
        assert_eq!(theme.meta.name, "Dark");
        assert!(theme.meta.dark);
    }

    #[test]
    fn embedded_light_parses_and_has_a_name() {
        let theme = Theme::embedded_light();
        assert_eq!(theme.meta.name, "Light");
        assert!(!theme.meta.dark);
    }

    #[test]
    fn embedded_dracula_parses_and_has_a_name() {
        let theme = Theme::embedded_dracula();
        assert_eq!(theme.meta.name, "Dracula");
        assert!(theme.meta.dark);
    }

    #[test]
    fn embedded_dark_has_syntax_tokens() {
        let theme = Theme::embedded_dark();
        assert!(theme.syntax.tokens.contains_key("keyword"));
        assert!(theme.syntax.tokens.contains_key("string"));
        assert!(theme.syntax.tokens.contains_key("comment"));
    }

    #[test]
    fn from_toml_str_rejects_garbage() {
        assert!(Theme::from_toml_str("this is not valid toml {{{").is_err());
    }

    #[test]
    fn from_toml_str_accepts_partial_theme_and_fills_defaults() {
        // A minimal theme file, just a name, should still parse, with
        // every other section falling back to its own Default.
        let theme = Theme::from_toml_str("[meta]\nname = \"Minimal\"\n").unwrap();
        assert_eq!(theme.meta.name, "Minimal");
        assert_eq!(theme.layout.border_style, BorderStyle::Plain);
    }

    #[test]
    fn from_file_reports_missing_file_clearly() {
        let result = Theme::from_file(std::path::Path::new("/definitely/not/a/real/path.toml"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read"));
    }
}
