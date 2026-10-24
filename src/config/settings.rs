//! Curated, user-editable settings: definitions plus reading/writing config.toml.

use crate::config::Config;
use crate::errors::error::AppError;
use std::path::Path;
use std::{fs, io};
use toml_edit::{DocumentMut, TableLike};

pub enum SettingKind {
    Bool,
    Int { min: i64, max: i64, step: i64 },
    Choice(&'static [&'static str]),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Apply {
    Live,
    Restart,
}

pub struct SettingDef {
    pub category: &'static str,
    /// Dotted path into config.toml, e.g. "editor.tab_width"
    pub key: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub kind: SettingKind,
    pub apply: Apply,
}

const fn int(min: i64, max: i64, step: i64) -> SettingKind {
    SettingKind::Int { min, max, step }
}

pub const SETTINGS: &[SettingDef] = &[
    SettingDef { category: "Editor", key: "editor.tab_width", label: "Tab width",
        description: "Columns per tab / spaces per Tab press", kind: int(1, 16, 1), apply: Apply::Live },
    SettingDef { category: "Editor", key: "editor.use_spaces", label: "Use spaces",
        description: "Insert spaces instead of a tab character", kind: SettingKind::Bool, apply: Apply::Live },
    SettingDef { category: "Editor", key: "editor.line_numbers", label: "Use line numbers",
        description: "Show line numbers inside the editor", kind: SettingKind::Bool, apply: Apply::Live },
    SettingDef { category: "Editor", key: "editor.relative_line_numbers", label: "Use relative line numbers",
        description: "Show line numbers as relative distance from current line instead of absolute count", kind: SettingKind::Bool, apply: Apply::Live },
    SettingDef { category: "Editor", key: "editor.wrap_lines", label: "Use line wrapping",
        description: "Wrap lines when they get bigger then editor width instead of just showing as long line with horizontal scroll", kind: SettingKind::Bool, apply: Apply::Live },
    SettingDef { category: "Editor", key: "editor.show_whitespace", label: "Show whitespace",
        description: "Shows whitespaces as · characters", kind: SettingKind::Bool, apply: Apply::Live },
    SettingDef { category: "Editor", key: "editor.highlight_current_line", label: "Highlight current line",
        description: "Highlights the current line in editor with a defined theme color", kind: SettingKind::Bool, apply: Apply::Live },
    SettingDef { category: "Editor", key: "editor.scrolloff", label: "Scrolloff",
        description: "Lines kept visible around the cursor", kind: int(0, 20, 1), apply: Apply::Live },
    SettingDef { category: "Editor", key: "editor.scroll_lines", label: "Scroll lines",
        description: "Lines moved per mouse wheel notch", kind: int(1, 20, 1), apply: Apply::Live },
    SettingDef { category: "Editor", key: "editor.scroll_margin_bottom", label: "Bottom margin",
        description: "Extra scroll space past the last line", kind: int(0, 20, 1), apply: Apply::Live },
    SettingDef { category: "Editor", key: "editor.undo_history_limit", label: "Undo history",
        description: "Max undo steps kept", kind: int(10, 10_000, 50), apply: Apply::Restart },
    SettingDef { category: "Saving", key: "editor.auto_save", label: "Auto-save",
        description: "Save automatically after edits", kind: SettingKind::Bool, apply: Apply::Live },
    SettingDef { category: "Saving", key: "editor.auto_save_delay_ms", label: "Auto-save delay (ms)",
        description: "Wait this long after an edit before saving", kind: int(500, 60_000, 500), apply: Apply::Live },
    SettingDef { category: "Performance", key: "performance.lazy_redraw", label: "Lazy redraw",
        description: "Only redraw when something changed", kind: SettingKind::Bool, apply: Apply::Live },
    SettingDef { category: "Performance", key: "performance.tick_rate_ms", label: "Tick rate (ms)",
        description: "Main loop tick interval", kind: int(10, 500, 10), apply: Apply::Restart },
    SettingDef { category: "Performance", key: "performance.cursor_blink_rate_ms", label: "Cursor blink (ms)",
        description: "Cursor blink interval", kind: int(100, 2000, 50), apply: Apply::Restart },
    SettingDef { category: "Project", key: "project.file_picker.mode", label: "File picker",
        description: "Built-in browser or an external tool such as yazi",
        kind: SettingKind::Choice(&["builtin", "external"]), apply: Apply::Live },
];

pub fn categories() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for d in SETTINGS {
        if !out.contains(&d.category) {
            out.push(d.category);
        }
    }
    out
}

// Reading

/// Effective config as a toml value tree (user overrides + defaults).
pub fn snapshot(config: &Config) -> Option<toml::Value> {
    toml::Value::try_from(config).ok()
}

pub fn lookup<'a>(root: &'a toml::Value, key: &str) -> Option<&'a toml::Value> {
    key.split('.').try_fold(root, |cur, part| cur.get(part))
}

pub fn display(v: &toml::Value) -> String {
    match v {
        toml::Value::Boolean(b) => (if *b { "on" } else { "off" }).to_string(),
        toml::Value::Integer(i) => i.to_string(),
        toml::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// Next value for a setting: toggles bools, steps ints (clamped), cycles choices.
pub fn step_value(kind: &SettingKind, current: &toml::Value, dir: i64) -> Option<toml_edit::Value> {
    match (kind, current) {
        (SettingKind::Bool, toml::Value::Boolean(b)) => Some((!*b).into()),
        (SettingKind::Int { min, max, step }, toml::Value::Integer(i)) => {
            Some((*i + dir.signum() * step).clamp(*min, *max).into())
        }
        (SettingKind::Choice(opts), toml::Value::String(s)) => {
            if opts.is_empty() {
                return None;
            }
            let idx = opts.iter().position(|o| *o == s.as_str()).unwrap_or(0) as i64;
            let next = (idx + dir.signum()).rem_euclid(opts.len() as i64) as usize;
            Some(opts[next].into())
        }
        _ => None,
    }
}

// Writing

/// Sets (`Some`) or removes (`None`, i.e. reset to default) `key` in the toml
/// file at `path`, preserving comments and formatting. Refuses to touch a file
/// that doesn't parse, so a user's hand edit is never overwritten, deleted or destroyed.
pub fn write_setting(
    path: &Path,
    key: &str,
    value: Option<toml_edit::Value>,
) -> Result<(), AppError> {
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e.into()),
    };
    let mut doc: DocumentMut = text.parse().map_err(|e| {
        AppError::InternalError(format!("config.toml has a syntax error, fix it first: {e}"))
    })?;

    let mut parts: Vec<&str> = key.split('.').collect();
    let leaf = parts
        .pop()
        .ok_or_else(|| AppError::InternalError("empty setting key".into()))?;

    let mut table: &mut dyn TableLike = doc.as_table_mut();
    for part in parts {
        let item = table.entry(part).or_insert(toml_edit::table());
        table = item.as_table_like_mut().ok_or_else(|| {
            AppError::InternalError(format!("'{part}' in config.toml is not a table"))
        })?;
    }
    match value {
        Some(v) => {
            table.insert(leaf, toml_edit::Item::Value(v));
        }
        None => {
            table.remove(leaf);
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    // temp + rename so a crash mid-write can't leave a truncated config
    let tmp = path.with_extension("toml.tmp");
    fs::write(&tmp, doc.to_string())?;
    fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod unit_settings_tests {
    use super::*;

    #[test]
    fn bool_toggles() {
        let v = step_value(&SettingKind::Bool, &toml::Value::Boolean(true), 1).unwrap();
        assert_eq!(v.as_bool(), Some(false));
    }

    #[test]
    fn int_steps_and_clamps() {
        let k = int(1, 16, 1);
        let up = step_value(&k, &toml::Value::Integer(16), 1).unwrap();
        assert_eq!(up.as_integer(), Some(16));
        let down = step_value(&k, &toml::Value::Integer(1), -1).unwrap();
        assert_eq!(down.as_integer(), Some(1));
    }

    #[test]
    fn choice_wraps_both_ways() {
        let k = SettingKind::Choice(&["a", "b"]);
        let next = step_value(&k, &toml::Value::String("b".into()), 1).unwrap();
        assert_eq!(next.as_str(), Some("a"));
        let prev = step_value(&k, &toml::Value::String("a".into()), -1).unwrap();
        assert_eq!(prev.as_str(), Some("b"));
    }

    #[test]
    fn write_preserves_comments_and_creates_nested_tables() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "# my notes\n[editor]\ntab_width = 2 # narrow\n").unwrap();

        write_setting(&path, "editor.tab_width", Some(4i64.into())).unwrap();
        write_setting(&path, "project.file_picker.mode", Some("external".into())).unwrap();

        let out = std::fs::read_to_string(&path).unwrap();
        assert!(out.contains("# my notes"));
        assert!(out.contains("tab_width = 4"));
        assert!(out.contains("mode = \"external\""));
    }

    #[test]
    fn write_none_removes_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "[editor]\ntab_width = 2\n").unwrap();
        write_setting(&path, "editor.tab_width", None).unwrap();
        assert!(!std::fs::read_to_string(&path)
            .unwrap()
            .contains("tab_width"));
    }

    #[test]
    fn syntax_error_is_not_clobbered() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "[editor\nbroken").unwrap();
        assert!(write_setting(&path, "editor.tab_width", Some(4i64.into())).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "[editor\nbroken");
    }
}
