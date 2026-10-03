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

    #[test]
    fn empty_string_falls_back_to_reset() {
        assert_eq!(parse_color(""), Color::Reset);
    }

    #[test]
    fn hash_alone_falls_back_to_reset() {
        assert_eq!(parse_color("#"), Color::Reset);
    }

    #[test]
    fn hex_too_long_falls_back_to_reset() {
        assert_eq!(parse_color("#ff880000"), Color::Reset);
    }

    #[test]
    fn hex_too_short_falls_back_to_reset() {
        assert_eq!(parse_color("#ff"), Color::Reset);
    }

    #[test]
    fn hex_with_non_hex_chars_falls_back_to_reset() {
        assert_eq!(parse_color("#gg0000"), Color::Reset);
        assert_eq!(parse_color("#12345g"), Color::Reset);
    }

    #[test]
    fn hex_is_case_insensitive() {
        assert_eq!(parse_color("#AABBCC"), parse_color("#aabbcc"));
        assert_eq!(parse_color("#AaBbCc"), Color::Rgb(0xaa, 0xbb, 0xcc));
    }

    #[test]
    fn hex_boundary_values() {
        assert_eq!(parse_color("#000000"), Color::Rgb(0, 0, 0));
        assert_eq!(parse_color("#ffffff"), Color::Rgb(255, 255, 255));
    }

    #[test]
    fn named_color_with_leading_or_trailing_whitespace_is_not_trimmed() {
        // Deliberately documents current (strict) behavior: color names
        // aren't trimmed, so stray whitespace in a theme file's color
        // value degrades to Reset rather than silently "working" — a
        // signal worth surfacing to whoever wrote the theme file, not
        // masking.
        assert_eq!(parse_color(" red"), Color::Reset);
        assert_eq!(parse_color("red "), Color::Reset);
    }

    #[test]
    fn all_documented_named_colors_parse_to_something_other_than_reset() {
        let names = [
            "red",
            "blue",
            "yellow",
            "green",
            "cyan",
            "black",
            "gray",
            "grey",
            "darkgray",
            "darkgrey",
            "magenta",
            "white",
            "lightred",
            "lightblue",
            "lightyellow",
            "lightgreen",
            "lightcyan",
            "lightmagenta",
        ];
        for name in names {
            assert_ne!(
                parse_color(name),
                Color::Reset,
                "expected '{name}' to be a recognized color name"
            );
        }
    }

    #[test]
    fn light_color_variants_are_distinct_from_their_base_color() {
        assert_ne!(parse_color("red"), parse_color("lightred"));
        assert_ne!(parse_color("blue"), parse_color("lightblue"));
        assert_ne!(parse_color("green"), parse_color("lightgreen"));
    }

    #[test]
    fn underscored_and_concatenated_gray_variants_agree() {
        assert_eq!(parse_color("dark_gray"), parse_color("darkgray"));
        assert_eq!(parse_color("dark_grey"), parse_color("darkgrey"));
        assert_eq!(parse_color("darkgray"), parse_color("darkgrey")); // english variants agree
    }
}
