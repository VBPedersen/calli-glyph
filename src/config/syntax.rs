use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level language syntax config. Relevant primarily for syntax highlighting.
/// Lives under [syntax] in config.toml.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SyntaxConfig {
    /// To enable lsp function or not: set false to disable all LSP globally
    pub enabled: bool,

    /// Where to look for compiled .so / .dll grammar files.
    /// Defaults to <config_dir>/grammars/
    pub grammar_dir: Option<String>,

    /// Per-language settings, keyed by an arbitrary name (used as grammar filename addition)
    #[serde(default)]
    pub languages: HashMap<String, SyntaxLanguageConfig>,

    /// Per language theme setting
    #[serde(default = "default_theme_name")]
    pub theme: String, // e.g. "dark", "monokai", "catppuccin"
}

/// Default impl for Syntax config, comes with rust, python and TypeScript setup be default
impl Default for SyntaxConfig {
    fn default() -> Self {
        let mut languages = HashMap::new();

        languages.insert(
            "rust".to_string(),
            SyntaxLanguageConfig {
                file_extensions: vec!["rs".to_string(), "toml".to_string()],
                grammar: "rust".to_string(), // looks for tree_sitter_rust.so in grammar_dir
            },
        );

        languages.insert(
            "typescript".to_string(),
            SyntaxLanguageConfig {
                file_extensions: vec!["ts".to_string()],
                grammar: "typescript".to_string(), // looks for tree_sitter_typescript.so in grammar_dir
            },
        );

        languages.insert(
            "python".to_string(),
            SyntaxLanguageConfig {
                file_extensions: vec!["py".to_string()],
                grammar: "python".to_string(), // looks for tree_sitter_python.so in grammar_dir
            },
        );

        Self {
            enabled: true,
            grammar_dir: Config::get_grammar_dir().map(|p| p.to_string_lossy().into_owned()),
            languages,
            theme: default_theme_name(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct SyntaxLanguageConfig {
    pub file_extensions: Vec<String>,
    pub grammar: String, // filename addition of the .so, e.g. "rust" -> "tree_sitter_rust.so"
}

impl SyntaxConfig {
    /// Returns the language name and config for the given extension, if any.
    pub fn language_for_extension(&self, ext: &str) -> Option<(&str, &SyntaxLanguageConfig)> {
        self.languages
            .iter()
            .find(|(_, cfg)| cfg.file_extensions.iter().any(|e| e == ext))
            .map(|(name, cfg)| (name.as_str(), cfg))
    }
}

/// Helper func to use a hardcoded default theme name
fn default_theme_name() -> String {
    "dark".to_string()
}
