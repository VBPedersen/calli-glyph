use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    pub tab_width: u16,
    pub use_spaces: bool,
    pub line_numbers: bool,
    pub relative_line_numbers: bool,
    pub wrap_lines: bool,
    pub auto_save: bool,
    pub auto_save_delay_ms: u64,
    pub show_whitespace: bool,
    pub highlight_current_line: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_width: 4,
            use_spaces: false,
            line_numbers: true,
            relative_line_numbers: true,
            wrap_lines: false,
            auto_save: false,
            auto_save_delay_ms: 1000,
            show_whitespace: false,
            highlight_current_line: false,
        }
    }
}
