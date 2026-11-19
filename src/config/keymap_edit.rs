//! Pure logic behind the controls editor: action catalogs, key formatting,
//! validation and map edits.

use super::keymaps::normalize_key;
use super::KeymapConfig;
use crossterm::event::{KeyCode, KeyModifiers};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Context {
    Editor,
    CommandLine,
    Debug,
}

impl Context {
    pub fn label(self) -> &'static str {
        match self {
            Context::Editor => "Editor",
            Context::CommandLine => "Command line",
            Context::Debug => "Debug",
        }
    }
    /// Table name under `[keymaps]` in config.toml.
    pub fn section(self) -> &'static str {
        match self {
            Context::Editor => "editor",
            Context::CommandLine => "command_line",
            Context::Debug => "debug",
        }
    }
}

const EDITOR_ACTIONS: &[(&str, &str)] = &[
    ("save", "Save"),
    ("copy", "Copy"),
    ("cut", "Cut"),
    ("paste", "Paste"),
    ("undo", "Undo"),
    ("redo", "Redo"),
    ("backspace", "Backspace"),
    ("delete", "Delete"),
    ("enter", "New line"),
    ("tab", "Tab / indent"),
    ("toggle_area", "Switch to command line"),
    ("move_up", "Cursor up"),
    ("move_down", "Cursor down"),
    ("move_left", "Cursor left"),
    ("move_right", "Cursor right"),
    ("select_up", "Select up"),
    ("select_down", "Select down"),
    ("select_left", "Select left"),
    ("select_right", "Select right"),
];
const COMMAND_LINE_ACTIONS: &[(&str, &str)] = &[
    ("enter", "Run command"),
    ("toggle_area", "Back to editor"),
    ("backspace", "Backspace"),
    ("delete", "Delete"),
    ("tab", "Tab"),
    ("move_up", "Cursor up"),
    ("move_down", "Cursor down"),
    ("move_left", "Cursor left"),
    ("move_right", "Cursor right"),
];
const DEBUG_ACTIONS: &[(&str, &str)] = &[
    ("exit_debug", "Close debug console"),
    ("next_tab", "Next tab"),
    ("prev_tab", "Previous tab"),
    ("scroll_up", "Scroll up"),
    ("scroll_down", "Scroll down"),
    ("manual_snapshot", "Take snapshot"),
    ("clear_logs", "Clear logs"),
    ("clear_snapshots", "Clear snapshots"),
    ("reset_metrics", "Reset metrics"),
    ("cycle_mode", "Cycle mode"),
    ("debug_interact", "Interact"),
    ("enter", "Enter"),
    ("tab", "Tab"),
    ("toggle_area", "Switch area"),
];

pub fn actions(ctx: Context) -> &'static [(&'static str, &'static str)] {
    match ctx {
        Context::Editor => EDITOR_ACTIONS,
        Context::CommandLine => COMMAND_LINE_ACTIONS,
        Context::Debug => DEBUG_ACTIONS,
    }
}

pub fn map_of(km: &KeymapConfig, ctx: Context) -> &HashMap<String, String> {
    match ctx {
        Context::Editor => &km.editor,
        Context::CommandLine => &km.command_line,
        Context::Debug => &km.debug,
    }
}

pub fn set_map(km: &mut KeymapConfig, ctx: Context, map: HashMap<String, String>) {
    match ctx {
        Context::Editor => km.editor = map,
        Context::CommandLine => km.command_line = map,
        Context::Debug => km.debug = map,
    }
}

/// Actions that must keep a key, or the user could lock themselves out.
pub fn is_essential(ctx: Context, action: &str) -> bool {
    matches!(
        (ctx, action),
        (Context::Editor, "toggle_area")
            | (Context::CommandLine, "toggle_area")
            | (Context::Debug, "exit_debug")
    )
}

// Key formatting & validation

/// Inverse of `KeymapConfig::parse_key`. None for keys the config can't express.
pub fn format_key(mods: KeyModifiers, code: KeyCode) -> Option<String> {
    let (mods, code) = normalize_key(mods, code);
    let allowed = KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT;
    if !allowed.contains(mods) {
        return None;
    }
    let name = match code {
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::BackTab => "BackTab".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::F(n) if (1..=12).contains(&n) => format!("F{n}"),
        KeyCode::Char(' ') => "Space".to_string(),
        KeyCode::Char('+') => "Plus".to_string(),
        KeyCode::Char(c) if c.is_ascii_graphic() => c.to_string(),
        _ => return None,
    };
    let mut s = String::new();
    if mods.contains(KeyModifiers::CONTROL) {
        s.push_str("Ctrl+");
    }
    if mods.contains(KeyModifiers::ALT) {
        s.push_str("Alt+");
    }
    if mods.contains(KeyModifiers::SHIFT) {
        s.push_str("Shift+");
    }
    s.push_str(&name);
    Some(s)
}

fn alias_of(c: char) -> Option<&'static str> {
    match c {
        'm' | 'j' => Some("Enter"),
        'i' => Some("Tab"),
        'h' => Some("Backspace"),
        '[' => Some("Esc"),
        _ => None,
    }
}

fn check_alias(mods: KeyModifiers, code: KeyCode) -> Result<(), String> {
    if let (true, KeyCode::Char(c)) = (mods.contains(KeyModifiers::CONTROL), code) {
        if let Some(alias) = alias_of(c) {
            return Err(format!(
                "Ctrl+{c} is sent as {alias} by most terminals, so it can't be told apart"
            ));
        }
    }
    Ok(())
}

/// Whether a key can work as a binding in `ctx`.
pub fn validate(ctx: Context, mods: KeyModifiers, code: KeyCode) -> Result<(), String> {
    let (mods, code) = normalize_key(mods, code);
    check_alias(mods, code)?;
    if let KeyCode::Char(_) = code {
        match ctx {
            Context::Editor => {
                if !mods.contains(KeyModifiers::CONTROL) || mods.contains(KeyModifiers::ALT) {
                    return Err("in the editor, character keys need Ctrl (without Alt), otherwise they type the character".into());
                }
            }
            Context::CommandLine => {
                if mods.difference(KeyModifiers::SHIFT).is_empty() {
                    return Err("in the command line, character keys need Ctrl or Alt, otherwise they type the character".into());
                }
            }
            Context::Debug => {}
        }
    }
    Ok(())
}

/// Plugin shortcuts are checked before everything else, so a bare character
/// would hijack typing.
pub fn validate_plugin(mods: KeyModifiers, code: KeyCode) -> Result<(), String> {
    let (mods, code) = normalize_key(mods, code);
    check_alias(mods, code)?;
    if let KeyCode::Char(_) = code {
        if !mods.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) {
            return Err("plugin shortcuts on character keys need Ctrl or Alt".into());
        }
    }
    Ok(())
}

// Map edits

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BindMode {
    /// The action ends up with exactly this key.
    Replace,
    /// The key is added next to the action's existing keys.
    Add,
}

/// Keys currently bound to `action`, sorted.
pub fn keys_for(map: &HashMap<String, String>, action: &str) -> Vec<String> {
    let mut keys: Vec<String> = map
        .iter()
        .filter(|(_, v)| v.as_str() == action)
        .map(|(k, _)| k.clone())
        .collect();
    keys.sort();
    keys
}

/// Existing (key, action) bound to the same physical key, compared after
/// normalization ("Ctrl+S" == "Ctrl+s").
pub fn find_conflict(
    map: &HashMap<String, String>,
    mods: KeyModifiers,
    code: KeyCode,
) -> Option<(String, String)> {
    let target = normalize_key(mods, code);
    map.iter()
        .find(|(k, _)| KeymapConfig::parse_key(k).ok() == Some(target))
        .map(|(k, v)| (k.clone(), v.clone()))
}

/// New map with `action` bound to `new_key`. Any other binding on the same
/// physical key is removed.
pub fn with_binding(
    map: &HashMap<String, String>,
    action: &str,
    new_key: &str,
    mode: BindMode,
) -> HashMap<String, String> {
    let target = KeymapConfig::parse_key(new_key).ok();
    let mut out: HashMap<String, String> = map
        .iter()
        .filter(|(k, v)| {
            let same_key = target.is_some() && KeymapConfig::parse_key(k).ok() == target;
            let replaced = mode == BindMode::Replace && v.as_str() == action;
            !(same_key || replaced)
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    out.insert(new_key.to_string(), action.to_string());
    out
}

pub fn without_action(map: &HashMap<String, String>, action: &str) -> HashMap<String, String> {
    map.iter()
        .filter(|(_, v)| v.as_str() != action)
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

/// Restores `action`'s default keys (defaults win over conflicting bindings).
pub fn reset_action(
    map: &HashMap<String, String>,
    defaults: &HashMap<String, String>,
    action: &str,
) -> HashMap<String, String> {
    let mut out = without_action(map, action);
    for key in keys_for(defaults, action) {
        out = with_binding(&out, action, &key, BindMode::Add);
    }
    out
}

#[cfg(test)]
mod unit_keymap_edit_tests {
    use super::*;

    const C: KeyModifiers = KeyModifiers::CONTROL;
    const N: KeyModifiers = KeyModifiers::NONE;

    fn map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn every_listed_action_is_accepted_by_the_parser() {
        for (name, _) in EDITOR_ACTIONS {
            assert!(
                KeymapConfig::parse_editor_action(name).is_ok(),
                "editor: {name}"
            );
        }
        for (name, _) in COMMAND_LINE_ACTIONS {
            assert!(
                KeymapConfig::parse_command_line_action(name).is_ok(),
                "command_line: {name}"
            );
        }
        for (name, _) in DEBUG_ACTIONS {
            assert!(
                KeymapConfig::parse_debug_action(name).is_ok(),
                "debug: {name}"
            );
        }
    }

    #[test]
    fn every_default_binding_targets_a_listed_action() {
        let d = KeymapConfig::default();
        for ctx in [Context::Editor, Context::CommandLine, Context::Debug] {
            let names: Vec<&str> = actions(ctx).iter().map(|(n, _)| *n).collect();
            for action in map_of(&d, ctx).values() {
                assert!(
                    names.contains(&action.as_str()),
                    "{ctx:?}: {action} not in catalog"
                );
            }
        }
    }

    #[test]
    fn format_then_parse_round_trips() {
        let cases = [
            (C, KeyCode::Char('s')),
            (N, KeyCode::F(5)),
            (KeyModifiers::SHIFT, KeyCode::BackTab),
            (KeyModifiers::ALT, KeyCode::Left),
            (N, KeyCode::Char(' ')),
            (C, KeyCode::Char('+')),
            (N, KeyCode::Char('C')),
            (C | KeyModifiers::ALT, KeyCode::Char('x')),
        ];
        for (m, c) in cases {
            let s = format_key(m, c).unwrap();
            assert_eq!(
                KeymapConfig::parse_key(&s).unwrap(),
                normalize_key(m, c),
                "{s}"
            );
        }
    }

    #[test]
    fn format_rejects_unrepresentable_keys() {
        assert_eq!(format_key(N, KeyCode::Char('å')), None);
        assert_eq!(format_key(KeyModifiers::SUPER, KeyCode::Char('a')), None);
        assert_eq!(format_key(N, KeyCode::F(13)), None);
    }

    #[test]
    fn validate_rules_per_context() {
        assert!(validate(Context::Editor, C, KeyCode::Char('a')).is_ok());
        assert!(validate(Context::Editor, N, KeyCode::Char('a')).is_err());
        assert!(validate(Context::Editor, N, KeyCode::Up).is_ok());
        assert!(validate(Context::CommandLine, N, KeyCode::Char('a')).is_err());
        assert!(validate(Context::CommandLine, KeyModifiers::ALT, KeyCode::Char('a')).is_ok());
        assert!(validate(Context::Debug, N, KeyCode::Char('x')).is_ok());
    }

    #[test]
    fn terminal_aliases_are_rejected_everywhere() {
        for c in ['m', 'j', 'i', 'h', '['] {
            assert!(
                validate(Context::Debug, C, KeyCode::Char(c)).is_err(),
                "{c}"
            );
            assert!(validate_plugin(C, KeyCode::Char(c)).is_err(), "{c}");
        }
    }

    #[test]
    fn plugin_char_keys_need_a_modifier() {
        assert!(validate_plugin(N, KeyCode::Char('p')).is_err());
        assert!(validate_plugin(C, KeyCode::Char('p')).is_ok());
        assert!(validate_plugin(N, KeyCode::F(2)).is_ok());
    }

    #[test]
    fn conflicts_compare_physical_keys_not_spellings() {
        let m = map(&[("Ctrl+S", "save")]);
        assert_eq!(
            find_conflict(&m, C, KeyCode::Char('s')),
            Some(("Ctrl+S".into(), "save".into()))
        );
        assert_eq!(find_conflict(&m, C, KeyCode::Char('z')), None);
    }

    #[test]
    fn replace_drops_old_keys_and_conflicting_bindings() {
        let m = map(&[("Ctrl+s", "save"), ("Ctrl+w", "save"), ("Ctrl+k", "cut")]);
        let out = with_binding(&m, "save", "Ctrl+k", BindMode::Replace);
        assert_eq!(keys_for(&out, "save"), vec!["Ctrl+k"]);
        assert!(keys_for(&out, "cut").is_empty()); // displaced
    }

    #[test]
    fn add_keeps_existing_keys() {
        let m = map(&[("Ctrl+s", "save")]);
        let out = with_binding(&m, "save", "Ctrl+w", BindMode::Add);
        assert_eq!(keys_for(&out, "save"), vec!["Ctrl+s", "Ctrl+w"]);
    }

    #[test]
    fn reset_action_restores_default_keys() {
        let d = KeymapConfig::default();
        let broken = without_action(&d.editor, "undo");
        assert!(keys_for(&broken, "undo").is_empty());
        let fixed = reset_action(&broken, &d.editor, "undo");
        assert_eq!(keys_for(&fixed, "undo"), vec!["Ctrl+z"]);
    }

    #[test]
    fn essential_actions_are_flagged() {
        assert!(is_essential(Context::Editor, "toggle_area"));
        assert!(is_essential(Context::Debug, "exit_debug"));
        assert!(!is_essential(Context::Editor, "save"));
    }
}
