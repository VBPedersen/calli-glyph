//! Non-color, structural preferences — "how things are laid out" as
//! opposed to "what color things are". Bundled into the same theme file
//! since a look and a layout (e.g. "Dracula colors + rounded borders")
//! are naturally chosen together, not as two separate settings to keep in
//! sync.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(default)]
pub struct LayoutOptions {
    pub border_style: BorderStyle,
    pub line_number_style: LineNumberStyle,
    pub status_bar_position: StatusBarPosition,
    pub popup_padding: u16,
    pub scrollbar: bool,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            border_style: BorderStyle::Plain,
            line_number_style: LineNumberStyle::Absolute,
            status_bar_position: StatusBarPosition::Bottom,
            popup_padding: 1,
            scrollbar: true,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum BorderStyle {
    #[default]
    Plain,
    Rounded,
    Double,
    Thick,
}

impl BorderStyle {
    /// Maps to ratatui's own border-set type at the one place that needs
    /// to know about ratatui's naming for this.
    pub fn to_ratatui(self) -> ratatui::symbols::border::Set {
        use ratatui::symbols::border;
        match self {
            BorderStyle::Plain => border::PLAIN,
            BorderStyle::Rounded => border::ROUNDED,
            BorderStyle::Double => border::DOUBLE,
            BorderStyle::Thick => border::THICK,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LineNumberStyle {
    #[default]
    Absolute,
    Relative,
    Both,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum StatusBarPosition {
    Top,
    #[default]
    Bottom,
}

#[cfg(test)]
mod unit_layout_tests {
    use super::*;

    #[test]
    fn default_matches_documented_defaults() {
        let layout = LayoutOptions::default();
        assert_eq!(layout.border_style, BorderStyle::Plain);
        assert_eq!(layout.line_number_style, LineNumberStyle::Absolute);
        assert_eq!(layout.status_bar_position, StatusBarPosition::Bottom);
        assert_eq!(layout.popup_padding, 1);
        assert!(layout.scrollbar);
    }
}
