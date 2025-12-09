//TODO make command for loading, and reloading and related commands for config

use crate::core::app::App;
use crate::core::command_line::command::CommandFlag;
use crate::errors::command_errors::CommandError;
use std::collections::HashSet;
use crate::config::Config;

enum ConfigSubcommand {
    Reload,
    Reset,
    Edit,
    Show,
    Validate,
    Set { key: String, value: String },
    InvalidCommandArgument { name: String, args: Vec<String> },
}

///Parses argument strings to sub command enum
fn parse_to_subcommand(args: Vec<String>) -> ConfigSubcommand {
    if args.is_empty() {
        return ConfigSubcommand::Show;
    }

    match args[0].as_str() {
        "reload" => ConfigSubcommand::Reload,
        "reset" => ConfigSubcommand::Reset,
        "edit" => ConfigSubcommand::Edit,
        "show" => ConfigSubcommand::Show,
        "validate" => ConfigSubcommand::Validate,
        "set" => {
            //since set requires key and value (2 args), just check args length,
            // if not long enough = invalid
            if args.len() >= 3 {
                ConfigSubcommand::Set {
                    key: String::from(args[1].clone()),
                    value: String::from(args[2].clone()),
                }
            } else {
                ConfigSubcommand::InvalidCommandArgument {
                    name: "Arguments for Set command invalid".to_string(),
                    args,
                }
            }
        }
        _ => ConfigSubcommand::InvalidCommandArgument {
            name: "Argument for base config command unrecognized".to_string(),
            args,
        },
    }
}

pub fn config_command(
    app: &mut App,
    args: Vec<String>,
    _flags: HashSet<CommandFlag>,
) -> Result<(), CommandError> {
    let sub_command = parse_to_subcommand(args);

    match sub_command {
        ConfigSubcommand::Reload => reload_config_command(app),
        ConfigSubcommand::Reset => reset_config_command(app),
        ConfigSubcommand::Edit => edit_config_command(app),
        ConfigSubcommand::Show => show_config_command(app),
        ConfigSubcommand::Validate => validate_config_command(app),
        ConfigSubcommand::Set { key, value } => set_config_command(app, key, value),
        ConfigSubcommand::InvalidCommandArgument { name, args } => {
            Err(CommandError::InvalidArguments {
                command: format!("InvalidArguments: {}", name),
                reason: format!("Args: {:?}", args),
            })
        }
    }
}

///Reloads config, rebuilds keymaps and applies to running app
pub fn reload_config_command(app: &mut App) -> Result<(), CommandError> {
    match app.config.reload() {
        Ok(config) => {
            //set in memory config to new config
            app.config = config;
            Ok(())
        }
        Err(e) => Err(CommandError::ExecutionFailed(format!(
            "Failed to reload config: {}",
            e
        ))),
    }
}

/// Reset config to defaults
pub fn reset_config_command(app: &mut App) -> Result<(), CommandError> {
   app.config = Config::default();

    match app.config.save() {
        Ok(_) => {
            // Rebuild runtime keymaps
            app.config.runtime_keymaps = Some(
                app.config.keymaps.build_runtime_maps()
                    .map_err(|e| CommandError::ExecutionFailed(format!("Failed to build keymaps: {}", e)))?
            );
            Ok(())
        }
        Err(e) => Err(CommandError::ExecutionFailed(format!("Failed to save config: {}", e)))
    }
}

///Open config file inside the editor
pub fn edit_config_command(app: &mut App) -> Result<(), CommandError> {
    let config_path = Config::get_config_path()
        .map_err(|e| CommandError::ExecutionFailed(format!("Failed to get config path: {}", e)))?;
    
    // Ensure config exists
    if !config_path.exists() {
        app.config.save()
            .map_err(|e| CommandError::ExecutionFailed(format!("Failed to create config: {}", e)))?;
    }
    
    // Set the file path to config file
    app.file_path = Some(config_path.clone());
    
    // Load config content into editor
    match std::fs::read_to_string(&config_path) {
        Ok(content) => {
            app.editor.editor_content = content.lines().map(String::from).collect();
            if app.editor.editor_content.is_empty() {
                app.editor.editor_content.push(String::new());
            }
            app.editor.cursor.x = 0;
            app.editor.cursor.y = 0;
            Ok(())
        }
        Err(e) => Err(CommandError::ExecutionFailed(format!("Failed to read config: {}", e)))
    }
}

///Show config.toml as scrollable popup window
pub fn show_config_command(app: &mut App) -> Result<(), CommandError> {
    match app.config.reload() {
        Ok(_) => {
            //TODO
            // show config possibly in previewer popup or something
            Ok(())
        }
        Err(e) => Err(CommandError::ExecutionFailed(format!(
            "Failed to show config: {}",
            e
        ))),
    }
}

pub fn validate_config_command(app: &mut App) -> Result<(), CommandError> {
    let result = app.config.validate();
    // Display detailed report in editor or status
    let report = result.detailed_report();

    // Only open popup if any errors or warns, else just status msg TODO when made
    //for now just print
    eprintln!("\n{}", report);

    if result.is_valid() {
        Ok(())
    } else {
        Err(CommandError::ExecutionFailed("Config validation failed. See errors above.".to_string()))
    }
}

pub fn set_config_command(app: &mut App, key: String, value: String) -> Result<(), CommandError> {
    match app.config.reload() {
        Ok(_) => {
            //TODO
            // set specific key of config temporarily in app memory to value
            // Consider only specific fields not keymaps, like tick rate, tab width so on
            Ok(())
        }
        Err(e) => Err(CommandError::ExecutionFailed(format!(
            "Failed to set config key value: {}",
            e
        ))),
    }
}

// --- Unit Tests ---
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, KeymapConfig};
    use crate::errors::config_errors::ConfigError;
    use crate::input::input_action::{DebugAction, Direction, InputAction};
    use crossterm::event::{KeyCode, KeyModifiers};

    // Helper to easily create KeyModifiers with Control, Alt, and Shift bits set
    fn ca_mods() -> KeyModifiers {
        let mut m = KeyModifiers::empty();
        m |= KeyModifiers::CONTROL;
        m |= KeyModifiers::ALT;
        m
    }

    // --- Key Parsing Tests ---

    #[test]
    fn test_parse_simple_key() {
        let result = KeymapConfig::parse_key("Enter").unwrap();
        assert_eq!(result, (KeyModifiers::empty(), KeyCode::Enter));
    }

    #[test]
    fn test_parse_char_key() {
        let result = KeymapConfig::parse_key("x").unwrap();
        assert_eq!(result, (KeyModifiers::empty(), KeyCode::Char('x')));
    }

    #[test]
    fn test_parse_ctrl_key() {
        let result = KeymapConfig::parse_key("Ctrl+c").unwrap();
        let mut expected_mods = KeyModifiers::empty();
        expected_mods |= KeyModifiers::CONTROL;
        assert_eq!(result, (expected_mods, KeyCode::Char('c')));
    }

    #[test]
    fn test_parse_multiple_modifiers() {
        let result = KeymapConfig::parse_key("Ctrl+Alt+Delete").unwrap();
        let expected_mods = ca_mods(); // Control + Alt
        assert_eq!(result, (expected_mods, KeyCode::Delete));
    }

    #[test]
    fn test_parse_unknown_modifier_fails() {
        let result = KeymapConfig::parse_key("Super+s");
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidKeymap(s) => assert!(s.contains("Unknown modifier")),
            _ => panic!("Expected InvalidKeymap error"),
        }
    }

    #[test]
    fn test_parse_unknown_key_fails() {
        let result = KeymapConfig::parse_key("F20");
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidKeymap(s) => assert!(s.contains("Unknown key")),
            _ => panic!("Expected InvalidKeymap error"),
        }
    }

    // --- Action Parsing Tests ---

    #[test]
    fn test_parse_simple_action() {
        let result = KeymapConfig::parse_action("save").unwrap();
        assert_eq!(result, InputAction::SAVE);
    }

    #[test]
    fn test_parse_directional_action() {
        let result = KeymapConfig::parse_action("move_up").unwrap();
        assert_eq!(result, InputAction::MoveCursor(Direction::Up));
    }

    #[test]
    fn test_parse_debug_action() {
        let result = KeymapConfig::parse_action("cycle_mode").unwrap();
        assert_eq!(result, InputAction::Debug(DebugAction::DebugCycleMode));
    }

    #[test]
    fn test_parse_unknown_action_fails() {
        let result = KeymapConfig::parse_action("teleport");
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidKeymap(s) => assert!(s.contains("Unknown action")),
            _ => panic!("Expected InvalidKeymap error"),
        }
    }

    // --- Config Integrity Tests ---

    #[test]
    fn test_default_config_is_valid() {
        // This test ensures that the hardcoded defaults can be parsed successfully
        // and do not trigger the panic! in Config::default().
        let config = Config::default();

        // Assert that the runtime keymaps were successfully built
        assert!(
            config.runtime_keymaps.is_some(),
            "Default configuration failed to build runtime keymaps."
        );

        // Check a specific keymap entry to ensure it was correctly parsed
        let runtime_maps = config.runtime_keymaps.as_ref().unwrap();

        // Check Ctrl+s -> SAVE in editor map
        let mut ctrl_mods = KeyModifiers::empty();
        ctrl_mods |= KeyModifiers::CONTROL;
        let ctrl_s = (ctrl_mods, KeyCode::Char('s'));
        assert_eq!(runtime_maps.editor.get(&ctrl_s), Some(&InputAction::SAVE));

        // Check Up -> move_up in editor map
        let up = (KeyModifiers::empty(), KeyCode::Up);
        assert_eq!(
            runtime_maps.editor.get(&up),
            Some(&InputAction::MoveCursor(Direction::Up))
        );

        // Check q -> exit_debug in debug map
        let q = (KeyModifiers::empty(), KeyCode::Char('q'));
        assert_eq!(
            runtime_maps.debug.get(&q),
            Some(&InputAction::Debug(DebugAction::ExitDebug))
        );
    }

    #[test]
    fn test_keymap_config_default_is_populated() {
        // Test that the string maps (the serializable part) are populated
        let keymaps = KeymapConfig::default();
        assert!(!keymaps.editor.is_empty());
        assert!(!keymaps.command_line.is_empty());
        assert!(!keymaps.debug.is_empty());
        assert_eq!(keymaps.editor.get("Ctrl+s"), Some(&"save".to_string()));
    }
}
