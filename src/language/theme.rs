//! maps token types to Ratatui styles, loaded from a TOML file

use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct ThemeColor {
    pub fg: Option<String>, // e.g. "#FF8800" or "Yellow"
    pub bold: Option<bool>,
    pub italic: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Theme {
    pub name: String,
    pub tokens: HashMap<String, ThemeColor>, // "keyword" -> style
    pub defaults: ThemeColor,
}

impl Theme {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    /// Get the style for token_type
    pub fn style_for(&self, token_type: &str) -> Style {
        let tc = self.tokens.get(token_type).unwrap_or(&self.defaults);
        self.to_ratatui_style(tc)
    }

    /// Maps ThemeColor to specific ratatui style
    fn to_ratatui_style(&self, tc: &ThemeColor) -> Style {
        let mut style = Style::default();
        if let Some(fg) = &tc.fg {
            style = style.fg(parse_color(fg));
        }
        if tc.bold == Some(true) {
            style = style.add_modifier(Modifier::BOLD);
        }
        if tc.italic == Some(true) {
            style = style.add_modifier(Modifier::ITALIC);
        }
        style
    }
}

/// Parses a color from string to Color,
/// either by hex code or standard color names like 'red' or 'yellow'
fn parse_color(s: &str) -> Color {
    if s.starts_with('#') && s.len() == 7 {
        // is hex color code
        // first convert to rgb, then construct RGB color code
        let r = u8::from_str_radix(&s[1..3], 16).unwrap_or(255);
        let g = u8::from_str_radix(&s[3..5], 16).unwrap_or(255);
        let b = u8::from_str_radix(&s[5..7], 16).unwrap_or(255);
        Color::Rgb(r, g, b)
    } else {
        match s.to_lowercase().as_str() {
            "red" => Color::Red,
            "blue" => Color::Blue,
            "yellow" => Color::Yellow,
            "green" => Color::Green,
            "cyan" => Color::Cyan,
            "black" => Color::Black,
            "gray" | "grey" => Color::Gray, // both proper english and fake english
            "magenta" => Color::Magenta,
            "white" => Color::White,
            _ => Color::Reset,
        }
    }
}
