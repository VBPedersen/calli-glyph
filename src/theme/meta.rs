use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct ThemeMeta {
    pub name: String,
    pub author: Option<String>,
    /// Hints things like which icon/glyph set looks better against this
    /// theme's background. Purely informational for now.
    pub dark: bool,
}