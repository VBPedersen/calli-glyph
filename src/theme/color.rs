//! Color parsing shared by every part of the theme system. Both
//! `SyntaxTheme` and `UiTheme` need "#RRGGBB or a named color" parsing, so
//! it lives here once rather than being duplicated per-section the way the
//! old `language/theme.rs` had it private to itself.

use ratatui::style::Color;

/// Parses a color from a hex code (`"#RRGGBB"`) or a handful of common
/// color names. Unrecognized input falls back to `Color::Reset` rather
/// than erroring, a typoed color in a theme file should degrade
/// gracefully (terminal default), not break loading the whole theme.
pub fn parse_color(s: &str) -> Color {
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return Color::Rgb(r, g, b);
            }
        }
        return Color::Reset;
    }

    match s.to_lowercase().as_str() {
        "red" => Color::Red,
        "blue" => Color::Blue,
        "yellow" => Color::Yellow,
        "green" => Color::Green,
        "cyan" => Color::Cyan,
        "black" => Color::Black,
        "gray" | "grey" => Color::Gray,
        "darkgray" | "darkgrey" | "dark_gray" | "dark_grey" => Color::DarkGray,
        "magenta" => Color::Magenta,
        "white" => Color::White,
        "lightred" | "light_red" => Color::LightRed,
        "lightblue" | "light_blue" => Color::LightBlue,
        "lightyellow" | "light_yellow" => Color::LightYellow,
        "lightgreen" | "light_green" => Color::LightGreen,
        "lightcyan" | "light_cyan" => Color::LightCyan,
        "lightmagenta" | "light_magenta" => Color::LightMagenta,
        _ => Color::Reset,
    }
}

#[cfg(test)]
mod unit_color_tests {
    use super::*;

    #[test]
    fn parses_hex_colors() {
        assert_eq!(parse_color("#ff8800"), Color::Rgb(0xff, 0x88, 0x00));
        assert_eq!(parse_color("#000000"), Color::Rgb(0, 0, 0));
        assert_eq!(parse_color("#FFFFFF"), Color::Rgb(255, 255, 255));
    }

    #[test]
    fn parses_named_colors_case_insensitively() {
        assert_eq!(parse_color("red"), Color::Red);
        assert_eq!(parse_color("RED"), Color::Red);
        assert_eq!(parse_color("Yellow"), Color::Yellow);
        assert_eq!(parse_color("grey"), Color::Gray);
        assert_eq!(parse_color("gray"), Color::Gray);
    }

    #[test]
    fn unrecognized_input_falls_back_to_reset() {
        assert_eq!(parse_color("not-a-color"), Color::Reset);
        assert_eq!(parse_color("#zzzzzz"), Color::Reset);
        assert_eq!(parse_color("#fff"), Color::Reset); // 3-digit hex not supported
    }
}
