use crate::errors::config_errors::ConfigError;
use crate::input::input_action::{DebugAction, InputAction};
use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeymapConfig {
    pub editor: HashMap<String, String>,
    pub command_line: HashMap<String, String>,
    pub debug: HashMap<String, String>,
}

impl KeymapConfig {
    /// Build runtime lookup tables from string config
    pub fn build_runtime_maps(&self) -> Result<RuntimeKeymaps, ConfigError> {
        let mut editor = HashMap::new();
        for (key_str, action_str) in &self.editor {
            let key = Self::parse_key(key_str)?;
            let action = Self::parse_action(action_str)?;
            editor.insert(key, action);
        }

        let mut command_line = HashMap::new();
        for (key_str, action_str) in &self.command_line {
            let key = Self::parse_key(key_str)?;
            let action = Self::parse_action(action_str)?;
            command_line.insert(key, action);
        }

        let mut debug = HashMap::new();
        for (key_str, action_str) in &self.debug {
            let key = Self::parse_key(key_str)?;
            let action = Self::parse_action(action_str)?;
            debug.insert(key, action);
        }

        Ok(RuntimeKeymaps {
            editor,
            command_line,
            debug,
        })
    }

    /// Parse a key string like "Ctrl+s" into KeyCode and KeyModifiers
    pub fn parse_key(key_str: &str) -> Result<(KeyModifiers, KeyCode), ConfigError> {
        let parts: Vec<&str> = key_str.split('+').collect(); //this also means configs should use + to separate keys in bind

        let mut modifiers = KeyModifiers::empty();
        let key_part = parts
            .last()
            .ok_or_else(|| ConfigError::InvalidKeymap(key_str.to_string()))?;

        for part in &parts[..parts.len() - 1] {
            match part.to_lowercase().as_str() {
                "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
                "alt" => modifiers |= KeyModifiers::ALT,
                "shift" => modifiers |= KeyModifiers::SHIFT,
                //TODO consider adding right shift,
                // this will need me to change from using KeyModifiers to KeyCode::Modifier(ModifierKeyCode)
                _ => {
                    return Err(ConfigError::InvalidKeymap(format!(
                        "Unknown modifier: {}",
                        part
                    )))
                }
            }
        }

        let keycode = match key_part.to_lowercase().as_str() {
            "enter" => KeyCode::Enter,
            "tab" => KeyCode::Tab,
            "backtab" => KeyCode::BackTab,
            "backspace" => KeyCode::Backspace,
            "delete" => KeyCode::Delete,
            "esc" | "escape" => KeyCode::Esc,
            "up" => KeyCode::Up,
            "down" => KeyCode::Down,
            "left" => KeyCode::Left,
            "right" => KeyCode::Right,
            "pageup" => KeyCode::PageUp,
            "pagedown" => KeyCode::PageDown,
            "home" => KeyCode::Home,
            "end" => KeyCode::End,
            "f1" => KeyCode::F(1),
            "f2" => KeyCode::F(2),
            "f3" => KeyCode::F(3),
            "f4" => KeyCode::F(4),
            "f5" => KeyCode::F(5),
            "f6" => KeyCode::F(6),
            "f7" => KeyCode::F(7),
            "f8" => KeyCode::F(8),
            "f9" => KeyCode::F(9),
            "f10" => KeyCode::F(10),
            "f11" => KeyCode::F(11),
            "f12" => KeyCode::F(12),
            s if s.len() == 1 => KeyCode::Char(
                s.chars()
                    .next()
                    .ok_or_else(|| ConfigError::InvalidKeymap(key_str.to_string()))?,
            ),
            _ => {
                return Err(ConfigError::InvalidKeymap(format!(
                    "Unknown key: {}",
                    key_part
                )))
            }
        };

        Ok((modifiers, keycode))
    }

    /// Parse action string like "copy" into InputAction
    pub fn parse_action(action_str: &str) -> Result<InputAction, ConfigError> {
        use crate::input::input_action::Direction;

        match action_str.to_lowercase().as_str() {
            // Editor actions
            "save" => Ok(InputAction::SAVE),
            "copy" => Ok(InputAction::COPY),
            "paste" => Ok(InputAction::PASTE),
            "cut" => Ok(InputAction::CUT),
            "undo" => Ok(InputAction::UNDO),
            "redo" => Ok(InputAction::REDO),
            "backspace" => Ok(InputAction::BACKSPACE),
            "delete" => Ok(InputAction::DELETE),
            "enter" => Ok(InputAction::ENTER),
            "tab" => Ok(InputAction::TAB),
            "toggle_area" => Ok(InputAction::ToggleActiveArea),

            // Movement
            "move_up" => Ok(InputAction::MoveCursor(Direction::Up)),
            "move_down" => Ok(InputAction::MoveCursor(Direction::Down)),
            "move_left" => Ok(InputAction::MoveCursor(Direction::Left)),
            "move_right" => Ok(InputAction::MoveCursor(Direction::Right)),

            // Selection
            "select_up" => Ok(InputAction::MoveSelectionCursor(Direction::Up)),
            "select_down" => Ok(InputAction::MoveSelectionCursor(Direction::Down)),
            "select_left" => Ok(InputAction::MoveSelectionCursor(Direction::Left)),
            "select_right" => Ok(InputAction::MoveSelectionCursor(Direction::Right)),

            // Debug actions
            "exit_debug" => Ok(InputAction::Debug(DebugAction::ExitDebug)),
            "next_tab" => Ok(InputAction::Debug(DebugAction::DebugNextTab)),
            "prev_tab" => Ok(InputAction::Debug(DebugAction::DebugPrevTab)),
            "scroll_up" => Ok(InputAction::Debug(DebugAction::DebugScrollUp)),
            "scroll_down" => Ok(InputAction::Debug(DebugAction::DebugScrollDown)),
            "clear_logs" => Ok(InputAction::Debug(DebugAction::DebugClearLogs)),
            "clear_snapshots" => Ok(InputAction::Debug(DebugAction::DebugClearSnapshots)),
            "manual_snapshot" => Ok(InputAction::Debug(DebugAction::DebugManualSnapshot)),
            "cycle_mode" => Ok(InputAction::Debug(DebugAction::DebugCycleMode)),
            "reset_metrics" => Ok(InputAction::Debug(DebugAction::DebugResetMetrics)),
            "view_snapshot" => Ok(InputAction::Debug(DebugAction::DebugViewSnapshot)),
            "close_viewer" => Ok(InputAction::Debug(DebugAction::DebugCloseSnapshotViewer)),

            _ => Err(ConfigError::InvalidKeymap(format!(
                "Unknown action: {}",
                action_str
            ))),
        }
    }

    /// Get action for a key combination
    pub fn get_editor_action(&self, key: &str) -> Option<&String> {
        self.editor.get(key)
    }
}

///Default keybinds for app, maps keybind String to action String,
/// all actions can be seen in parse_action
impl Default for KeymapConfig {
    fn default() -> Self {
        let mut editor = HashMap::new();
        editor.insert("Ctrl+s".to_string(), "save".to_string());
        editor.insert("Ctrl+c".to_string(), "copy".to_string());
        editor.insert("Ctrl+v".to_string(), "paste".to_string());
        editor.insert("Ctrl+x".to_string(), "cut".to_string());
        editor.insert("Ctrl+z".to_string(), "undo".to_string());
        editor.insert("Ctrl+y".to_string(), "redo".to_string());
        editor.insert("Backspace".to_string(), "backspace".to_string());
        editor.insert("Delete".to_string(), "delete".to_string());
        editor.insert("Up".to_string(), "move_up".to_string());
        editor.insert("Down".to_string(), "move_down".to_string());
        editor.insert("Left".to_string(), "move_left".to_string());
        editor.insert("Right".to_string(), "move_right".to_string());
        editor.insert("Shift+Up".to_string(), "select_up".to_string());
        editor.insert("Shift+Down".to_string(), "select_down".to_string());
        editor.insert("Shift+Left".to_string(), "select_left".to_string());
        editor.insert("Shift+Right".to_string(), "select_right".to_string());
        editor.insert("Esc".to_string(), "toggle_area".to_string());
        editor.insert("Enter".to_string(), "enter".to_string());
        editor.insert("Tab".to_string(), "tab".to_string());

        let mut command_line = HashMap::new();
        command_line.insert("Enter".to_string(), "enter".to_string());
        command_line.insert("Esc".to_string(), "toggle_area".to_string());
        command_line.insert("Backspace".to_string(), "backspace".to_string());
        command_line.insert("Delete".to_string(), "delete".to_string());
        command_line.insert("Left".to_string(), "move_left".to_string());
        command_line.insert("Right".to_string(), "move_right".to_string());

        let mut debug = HashMap::new();
        debug.insert("q".to_string(), "exit_debug".to_string());
        debug.insert("Esc".to_string(), "exit_debug".to_string());
        debug.insert("Tab".to_string(), "next_tab".to_string());
        debug.insert("Shift+BackTab".to_string(), "prev_tab".to_string());
        debug.insert("Up".to_string(), "scroll_up".to_string());
        debug.insert("Down".to_string(), "scroll_down".to_string());
        debug.insert("h".to_string(), "prev_tab".to_string());
        debug.insert("k".to_string(), "scroll_up".to_string());
        debug.insert("j".to_string(), "scroll_down".to_string());
        debug.insert("l".to_string(), "next_tab".to_string());
        debug.insert("s".to_string(), "manual_snapshot".to_string());
        debug.insert("r".to_string(), "reset_metrics".to_string());
        debug.insert("c".to_string(), "clear_logs".to_string());
        debug.insert("C".to_string(), "clear_snapshots".to_string());
        debug.insert("m".to_string(), "cycle_mode".to_string());
        debug.insert("Enter".to_string(), "view_snapshot".to_string());

        Self {
            editor,
            command_line,
            debug,
        }
    }
}

/// Runtime keymaps with parsed enums
#[derive(Debug, Clone)]
pub struct RuntimeKeymaps {
    pub editor: HashMap<(KeyModifiers, KeyCode), InputAction>,
    pub command_line: HashMap<(KeyModifiers, KeyCode), InputAction>,
    pub debug: HashMap<(KeyModifiers, KeyCode), InputAction>,
}

impl RuntimeKeymaps {
    //RuntimeKeymaps functions that get InputAction related to specified hashmap with mods and key.

    pub fn get_editor_action(&self, mods: KeyModifiers, key: KeyCode) -> Option<&InputAction> {
        self.editor.get(&(mods, key))
    }

    pub fn get_command_line_action(
        &self,
        mods: KeyModifiers,
        key: KeyCode,
    ) -> Option<&InputAction> {
        self.command_line.get(&(mods, key))
    }

    pub fn get_debug_action(&self, mods: KeyModifiers, key: KeyCode) -> Option<&InputAction> {
        self.debug.get(&(mods, key))
    }
}
