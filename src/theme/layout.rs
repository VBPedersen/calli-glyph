//! Non-color, structural preferences, "how things are laid out" as
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

    #[test]
    fn border_style_deserializes_from_lowercase_toml() {
        // Parse valid TOML document into a generic Value table
        let table: toml::Table = toml::from_str("style = \"rounded\"").unwrap();

        // Extract the field and deserialize directly into BorderStyle
        let parsed: BorderStyle = table["style"].clone().try_into().unwrap();

        assert_eq!(parsed, BorderStyle::Rounded);
    }

    #[test]
    fn all_border_styles_round_trip_through_toml() {
        for style in [
            BorderStyle::Plain,
            BorderStyle::Rounded,
            BorderStyle::Double,
            BorderStyle::Thick,
        ] {
            let value = toml::Value::try_from(&style).unwrap();
            let parsed: BorderStyle = value.try_into().unwrap();
            assert_eq!(style, parsed, "round trip failed for {:?}", style);
        }
    }

    #[test]
    fn all_line_number_styles_round_trip_through_toml() {
        for style in [
            LineNumberStyle::Absolute,
            LineNumberStyle::Relative,
            LineNumberStyle::Both,
        ] {
            let value = toml::Value::try_from(&style).unwrap();
            let parsed: LineNumberStyle = value.try_into().unwrap();
            assert_eq!(style, parsed, "round trip failed for {:?}", style);
        }
    }

    #[test]
    fn all_status_bar_positions_round_trip_through_toml() {
        for pos in [StatusBarPosition::Top, StatusBarPosition::Bottom] {
            let value = toml::Value::try_from(&pos).unwrap();
            let parsed: StatusBarPosition = value.try_into().unwrap();
            assert_eq!(pos, parsed, "round trip failed for {:?}", pos);
        }
    }

    #[test]
    fn each_border_style_maps_to_a_distinct_ratatui_set() {
        // Not a deep check of ratatui's glyph internals — just confirms
        // every enum variant is actually wired to *a* mapping and that
        // they're not all silently aliasing to the same one (e.g. via a
        // copy-paste mistake in `to_ratatui`).
        let plain = BorderStyle::Plain.to_ratatui();
        let rounded = BorderStyle::Rounded.to_ratatui();
        let double = BorderStyle::Double.to_ratatui();
        let thick = BorderStyle::Thick.to_ratatui();

        assert_ne!(plain.top_left, rounded.top_left);
        assert_ne!(plain.top_left, double.top_left);
        assert_ne!(plain.top_left, thick.top_left);
        assert_ne!(rounded.top_left, double.top_left);
    }

    #[test]
    fn unknown_border_style_string_fails_to_deserialize_rather_than_silently_defaulting() {
        // A typo'd border_style in a theme file should surface as a load
        // error (see ThemeManager::scan_dir logging it and skipping the
        // file), not silently become Plain — that would hide the typo.
        let result: Result<BorderStyle, _> = toml::from_str("\"diamond\"");
        assert!(result.is_err());
    }

    #[test]
    fn layout_options_deserializes_from_partial_toml_filling_defaults() {
        let layout: LayoutOptions = toml::from_str("border_style = \"double\"").unwrap();
        assert_eq!(layout.border_style, BorderStyle::Double);
        // Everything else should still be the Default value
        assert_eq!(layout.line_number_style, LineNumberStyle::Absolute);
        assert_eq!(layout.status_bar_position, StatusBarPosition::Bottom);
        assert_eq!(layout.popup_padding, 1);
        assert!(layout.scrollbar);
    }

    #[test]
    fn layout_options_full_round_trip() {
        let original = LayoutOptions {
            border_style: BorderStyle::Thick,
            line_number_style: LineNumberStyle::Both,
            status_bar_position: StatusBarPosition::Top,
            popup_padding: 3,
            scrollbar: false,
        };
        let toml_str = toml::to_string(&original).unwrap();
        let parsed: LayoutOptions = toml::from_str(&toml_str).unwrap();
        assert_eq!(original, parsed);
    }
}
