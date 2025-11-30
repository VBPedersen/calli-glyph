use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    pub theme: String,
    pub show_status_bar: bool,
    pub show_tab_bar: bool,
    pub cursor_style: CursorStyle,
    pub cursor_blink: bool,
    pub scrolloff: u16, // Lines to keep visible above/below cursor
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CursorStyle {
    Block,
    Line,
    Underline,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            show_status_bar: true,
            show_tab_bar: false,
            cursor_style: CursorStyle::Block,
            cursor_blink: true,
            scrolloff: 3,
        }
    }
}
