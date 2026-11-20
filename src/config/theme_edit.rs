use crate::config::settings::{write_setting, SettingKind};
use crate::errors::error::AppError;
use crate::theme::color::parse_color;
use ratatui::style::Color;
use std::{
    fs,
    path::{Path, PathBuf},
};
use toml_edit::DocumentMut;

pub enum RowKind {
    /// `optional` colors have no default: unset is a valid state.
    Color {
        optional: bool,
    },
    Setting(SettingKind),
}

pub struct Row {
    pub group: &'static str,
    pub key: &'static str,
    pub label: &'static str,
    pub kind: RowKind,
}

const fn color(group: &'static str, key: &'static str, label: &'static str) -> Row {
    Row {
        group,
        key,
        label,
        kind: RowKind::Color { optional: false },
    }
}
const fn optional_color(group: &'static str, key: &'static str, label: &'static str) -> Row {
    Row {
        group,
        key,
        label,
        kind: RowKind::Color { optional: true },
    }
}
const fn setting(
    group: &'static str,
    key: &'static str,
    label: &'static str,
    kind: SettingKind,
) -> Row {
    Row {
        group,
        key,
        label,
        kind: RowKind::Setting(kind),
    }
}

pub const ROWS: &[Row] = &[
    color("Base", "ui.colors.background", "Background"),
    color("Base", "ui.colors.foreground", "Foreground"),
    color("Base", "ui.colors.border", "Border"),
    color("Base", "ui.colors.border_focused", "Border (focused)"),
    color("Base", "ui.colors.selection_bg", "Selection background"),
    optional_color("Base", "ui.colors.selection_fg", "Selection text"),
    color("Base", "ui.colors.cursor", "Cursor"),
    color("Bars", "ui.colors.status_bar_bg", "Status bar bg"),
    color("Bars", "ui.colors.status_bar_fg", "Status bar fg"),
    color("Bars", "ui.colors.command_line_bg", "Command line bg"),
    color("Bars", "ui.colors.command_line_fg", "Command line fg"),
    color("Bars", "ui.colors.list_highlight_bg", "List highlight bg"),
    color("Bars", "ui.colors.list_highlight_fg", "List highlight fg"),
    color("Bars", "ui.colors.line_number", "Line number"),
    color(
        "Bars",
        "ui.colors.line_number_current",
        "Current line number",
    ),
    color("Bars", "ui.colors.current_line_bg", "Current line"),
    color("Status", "ui.colors.error", "Error"),
    color("Status", "ui.colors.warning", "Warning"),
    color("Status", "ui.colors.info", "Info"),
    color("Status", "ui.colors.hint", "Hint"),
    color("Status", "ui.colors.success", "Success"),
    color("Accents", "ui.colors.accent_red", "Red"),
    color("Accents", "ui.colors.accent_blue", "Blue"),
    color("Accents", "ui.colors.accent_cyan", "Cyan"),
    color("Accents", "ui.colors.accent_green", "Green"),
    color("Accents", "ui.colors.accent_yellow", "Yellow"),
    color("Accents", "ui.colors.accent_magenta", "Magenta"),
    color("Logs", "ui.colors.log_error", "Error"),
    color("Logs", "ui.colors.log_warning", "Warning"),
    color("Logs", "ui.colors.log_info", "Info"),
    color("Logs", "ui.colors.log_debug", "Debug"),
    color("Logs", "ui.colors.log_trace", "Trace"),
    optional_color("Popups", "ui.components.popup_bg", "Popup background"),
    optional_color("Popups", "ui.components.popup_fg", "Popup text"),
    optional_color("Popups", "ui.components.popup_border", "Popup border"),
    optional_color(
        "Popups",
        "ui.components.debug_console_bg",
        "Debug console bg",
    ),
    color("Syntax", "syntax.defaults.fg", "Default text"),
    color("Syntax", "syntax.tokens.keyword.fg", "Keyword"),
    color("Syntax", "syntax.tokens.string.fg", "String"),
    color("Syntax", "syntax.tokens.comment.fg", "Comment"),
    color("Syntax", "syntax.tokens.number.fg", "Number"),
    color("Syntax", "syntax.tokens.type.fg", "Type"),
    color("Syntax", "syntax.tokens.function.fg", "Function"),
    color("Syntax", "syntax.tokens.boolean.fg", "Boolean"),
    color("Syntax", "syntax.tokens.variable.fg", "Variable"),
    color("Syntax", "syntax.tokens.property.fg", "Property"),
    color("Syntax", "syntax.tokens.parameter.fg", "Parameter"),
    color("Syntax", "syntax.tokens.constant.fg", "Constant"),
    color("Syntax", "syntax.tokens.operator.fg", "Operator"),
    setting(
        "Layout",
        "layout.border_style",
        "Border style",
        SettingKind::Choice(&["plain", "rounded", "double", "thick"]),
    ),
    setting(
        "Layout",
        "layout.line_number_style",
        "Line numbers",
        SettingKind::Choice(&["absolute", "relative", "both"]),
    ),
    setting(
        "Layout",
        "layout.status_bar_position",
        "Status bar",
        SettingKind::Choice(&["top", "bottom"]),
    ),
    setting(
        "Layout",
        "layout.popup_padding",
        "Popup padding",
        SettingKind::Int {
            min: 0,
            max: 4,
            step: 1,
        },
    ),
    setting("Layout", "layout.scrollbar", "Scrollbar", SettingKind::Bool),
    setting(
        "Layout",
        "meta.dark",
        "Dark theme (hint)",
        SettingKind::Bool,
    ),
];

/// Group names in order of first appearance (used as tabs).
pub fn groups() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for r in ROWS {
        if !out.contains(&r.group) {
            out.push(r.group);
        }
    }
    out
}

/// Validates then writes one color into a theme file (comments preserved).
pub fn set_color(theme_path: &Path, key: &str, input: &str) -> Result<(), AppError> {
    let hex = normalize_hex(input).ok_or_else(|| {
        AppError::InternalError(format!("'{input}' is not a hex color (#rrggbb)"))
    })?;
    Ok(write_setting(theme_path, key, Some(hex.into()))
        .map_err(|e| AppError::InternalError(e.to_string()))?)
}

/// "#abc", "abc", "#AABBCC" -> Some([r,g,b]).
pub fn parse_hex(s: &str) -> Option<[u8; 3]> {
    let h = s.trim().trim_start_matches('#');
    let full: String = match h.len() {
        3 => h.chars().flat_map(|c| [c, c]).collect(),
        6 => h.to_string(),
        _ => return None,
    };
    if !full.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let n = u32::from_str_radix(&full, 16).ok()?;
    Some([(n >> 16) as u8, (n >> 8) as u8, n as u8])
}

pub fn normalize_hex(s: &str) -> Option<String> {
    parse_hex(s).map(|[r, g, b]| format!("#{r:02x}{g:02x}{b:02x}"))
}

/// What to store for user input: a normalized `#rrggbb`, or a lowercase color
/// name the theme loader understands. Anything else is rejected, because
/// `parse_color` would silently turn it into `Reset`.
pub fn normalize_color(input: &str) -> Option<String> {
    let t = input.trim();
    if let Some(hex) = normalize_hex(t) {
        return Some(hex);
    }
    let name = t.to_lowercase();
    (!name.is_empty() && !name.starts_with('#') && parse_color(&name) != Color::Reset)
        .then_some(name)
}

pub fn slug(name: &str) -> Option<String> {
    let s: String = name
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let s = s
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    (!s.is_empty()).then_some(s)
}

/// Copies `src` into `dest_dir/<slug>.toml`, sets [meta].name, refuses to overwrite.
pub fn duplicate_theme(src: &Path, dest_dir: &Path, new_name: &str) -> Result<PathBuf, AppError> {
    let slug = slug(new_name)
        .ok_or_else(|| AppError::InternalError("theme name needs letters or digits".into()))?;
    let dest = dest_dir.join(format!("{slug}.toml"));
    if dest.exists() {
        return Err(AppError::InternalError(format!(
            "theme '{slug}' already exists"
        )));
    }
    let mut doc: DocumentMut = fs::read_to_string(src)?
        .parse()
        .map_err(|e| AppError::InternalError(format!("source theme is invalid: {e}")))?;
    doc["meta"]["name"] = toml_edit::value(new_name.trim());
    fs::create_dir_all(dest_dir)?;
    fs::write(&dest, doc.to_string())?;
    Ok(dest)
}

#[cfg(test)]
mod unit_theme_edit_tests {
    use super::*;
    use crate::config::settings::lookup;

    #[test]
    fn every_required_row_resolves_in_the_embedded_dark_theme() {
        // Guards against typos in row keys: a wrong key would show "—" forever.
        let snap = toml::Value::try_from(&crate::theme::Theme::embedded_dark()).unwrap();
        for r in ROWS {
            if !matches!(r.kind, RowKind::Color { optional: true }) {
                assert!(
                    lookup(&snap, r.key).is_some(),
                    "row key not found: {}",
                    r.key
                );
            }
        }
    }

    #[test]
    fn row_keys_are_unique() {
        let mut keys: Vec<_> = ROWS.iter().map(|r| r.key).collect();
        keys.sort_unstable();
        let n = keys.len();
        keys.dedup();
        assert_eq!(keys.len(), n);
    }

    #[test]
    fn normalize_color_expands_short_hex_and_accepts_names() {
        assert_eq!(normalize_color("ABC").as_deref(), Some("#aabbcc"));
        assert_eq!(normalize_color(" #569CD6 ").as_deref(), Some("#569cd6"));
        assert_eq!(normalize_color("Red").as_deref(), Some("red"));
    }

    #[test]
    fn normalize_color_rejects_garbage() {
        for bad in ["", "#", "#12", "#gggggg", "nope", "#red"] {
            assert_eq!(normalize_color(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn duplicate_sets_name_and_refuses_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("dark.toml");
        std::fs::write(&src, "[meta]\nname = \"Dark\"\n").unwrap();
        let out_dir = dir.path().join("user");

        let dest = duplicate_theme(&src, &out_dir, "My Theme!").unwrap();
        assert_eq!(dest.file_name().unwrap(), "my-theme.toml");
        assert!(std::fs::read_to_string(&dest)
            .unwrap()
            .contains("My Theme!"));
        assert!(duplicate_theme(&src, &out_dir, "My Theme!").is_err());
    }

    #[test]
    fn parse_hex_accepts_common_forms() {
        assert_eq!(parse_hex("#1e1e1e"), Some([0x1e, 0x1e, 0x1e]));
        assert_eq!(parse_hex("FFF"), Some([255, 255, 255]));
        assert_eq!(parse_hex(" #569CD6 "), Some([0x56, 0x9c, 0xd6]));
    }

    #[test]
    fn parse_hex_rejects_garbage() {
        assert_eq!(parse_hex("#12"), None);
        assert_eq!(parse_hex("#gggggg"), None);
        assert_eq!(parse_hex(""), None);
    }

    #[test]
    fn normalize_lowercases_and_expands() {
        assert_eq!(normalize_hex("ABC").as_deref(), Some("#aabbcc"));
    }

    #[test]
    fn set_color_writes_inline_syntax_table_and_keeps_other_fields() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("t.toml");
        std::fs::write(
            &p,
            "[syntax.tokens]\nkeyword = { fg = \"#569cd6\", bold = true }\n",
        )
        .unwrap();
        set_color(&p, "syntax.tokens.keyword.fg", "#fff").unwrap();
        let out = std::fs::read_to_string(&p).unwrap();
        assert!(out.contains("#ffffff"));
        assert!(out.contains("bold = true"));
    }

    #[test]
    fn set_color_rejects_invalid_input_without_touching_file() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("t.toml");
        std::fs::write(&p, "[ui.colors]\nbackground = \"#000000\"\n").unwrap();
        assert!(set_color(&p, "ui.colors.background", "nope").is_err());
        assert!(std::fs::read_to_string(&p).unwrap().contains("#000000"));
    }

    #[test]
    fn slug_rules() {
        assert_eq!(slug("  My  Theme! ").as_deref(), Some("my-theme"));
        assert_eq!(slug("!!!"), None);
    }
}
