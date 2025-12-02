use calliglyph::config::{self, Config, EditorConfig, KeymapConfig, PerformanceConfig, UIConfig};
use calliglyph::errors::config_errors::ConfigError;
use calliglyph::input::input_action::{DebugAction, Direction, InputAction};
use crossterm::event::{KeyCode, KeyModifiers};
use std::{fs, path::PathBuf};

// Helper function to create the Control + Alt modifiers for tests
fn ca_mods() -> KeyModifiers {
    let mut m = KeyModifiers::empty();
    m |= KeyModifiers::CONTROL;
    m |= KeyModifiers::ALT;
    m
}

// --- Keymap Parsing Integration Tests (via public interface) ---

// We test parsing indirectly through build_runtime_maps since parse_key/parse_action are now private.

#[test]
fn test_runtime_maps_integrity() {
    let config = KeymapConfig::default();
    let runtime_maps = config
        .build_runtime_maps()
        .expect("Failed to build runtime maps from default config");

    // Check a simple key: Enter
    let enter = (KeyModifiers::empty(), KeyCode::Enter);
    assert_eq!(
        runtime_maps.editor.get(&enter),
        Some(&InputAction::ENTER),
        "Enter key binding failed."
    );

    // Check a complex key: Ctrl+s
    let mut ctrl_mods = KeyModifiers::empty();
    ctrl_mods |= KeyModifiers::CONTROL;
    let ctrl_s = (ctrl_mods, KeyCode::Char('s'));
    assert_eq!(
        runtime_maps.editor.get(&ctrl_s),
        Some(&InputAction::SAVE),
        "Ctrl+s binding failed."
    );

    // Check a directional key: Up -> move_up
    let up = (KeyModifiers::empty(), KeyCode::Up);
    assert_eq!(
        runtime_maps.editor.get(&up),
        Some(&InputAction::MoveCursor(Direction::Up)),
        "Up key binding failed."
    );

    // Check a debug key: q -> exit_debug
    let q = (KeyModifiers::empty(), KeyCode::Char('q'));
    assert_eq!(
        runtime_maps.debug.get(&q),
        Some(&InputAction::Debug(DebugAction::ExitDebug)),
        "Debug 'q' binding failed."
    );
}

#[test]
fn test_parsing_invalid_key_fails() {
    let mut bad_config = KeymapConfig::default();
    bad_config
        .editor
        .insert("Super+x".to_string(), "save".to_string());

    let result = bad_config.build_runtime_maps();
    assert!(result.is_err());

    // Verify it's an InvalidKeymap error related to the bad key
    match result.unwrap_err() {
        ConfigError::InvalidKeymap(s) => assert!(s.contains("Unknown modifier: Super")),
        _ => panic!("Expected InvalidKeymap error"),
    }
}

#[test]
fn test_parsing_invalid_action_fails() {
    let mut bad_config = KeymapConfig::default();
    bad_config
        .editor
        .insert("Ctrl+k".to_string(), "teleport".to_string());

    let result = bad_config.build_runtime_maps();
    assert!(result.is_err());

    // Verify it's an InvalidKeymap error related to the bad action
    match result.unwrap_err() {
        ConfigError::InvalidKeymap(s) => assert!(s.contains("Unknown action: teleport")),
        _ => panic!("Expected InvalidKeymap error"),
    }
}

// --- Config Loading and Saving Tests (Updated for nested structs) ---

// Helper to clean up after test
fn cleanup_test_path(path: &PathBuf) {
    if path.exists() {
        let _ = fs::remove_file(path);
    }
    Config::set_test_config_path(None);
}

#[test]
fn test_config_load_fallback_on_no_file() {
    // Setup: Point to a non-existent path
    let temp_path = PathBuf::from("temp_nonexistent_config.toml");
    Config::set_test_config_path(Some(temp_path.clone()));

    // Ensure the file does not exist before loading
    cleanup_test_path(&temp_path);

    // Action: Load config
    let loaded_config = Config::load();

    // Assertions (Fallback): Check default settings from nested structs
    assert_eq!(
        loaded_config.editor.tab_width, 4,
        "Editor default tab_size failed."
    );

    // It should not save the default config
    assert!(!temp_path.exists());

    // Cleanup
    cleanup_test_path(&temp_path);
}

#[test]
fn test_config_load_and_save_success() {
    // Setup: Create a temporary file path
    let temp_path = PathBuf::from("temp_test_config_success.toml");
    Config::set_test_config_path(Some(temp_path.clone()));

    // Action: Create a custom config and save it
    let mut custom_config = Config::default();
    // Set custom values on nested structs
    custom_config.editor.tab_width = 8;
    custom_config.performance.undo_history_limit = 500;

    custom_config
        .keymaps
        .editor
        .insert("F1".to_string(), "save".to_string());

    assert!(custom_config.save().is_ok());
    assert!(temp_path.exists());

    // Action: Load the custom config
    let loaded_config = Config::load();

    // Assertions (Load): Check custom values
    assert_eq!(loaded_config.editor.tab_width, 8);
    assert_eq!(loaded_config.performance.undo_history_limit, 500);

    // Check if the custom keymap was loaded and mapped
    let runtime_maps = loaded_config.runtime_keymaps.as_ref().unwrap();
    // Since F1 is not defined in mock, checking the string map is more robust:
    assert_eq!(
        loaded_config.keymaps.editor.get("F1"),
        Some(&"save".to_string())
    );

    // Cleanup
    cleanup_test_path(&temp_path);
}

#[test]
fn test_config_load_fallback_on_invalid_content() {
    // Setup: Create a temporary file path with invalid TOML content
    let temp_path = PathBuf::from("temp_invalid_config.toml");
    Config::set_test_config_path(Some(temp_path.clone()));

    // Invalid TOML structure (e.g., passing a string where an integer is expected)
    let invalid_content = r#"
    [editor]
    tab_width = "oops_not_an_int"

    [keymaps]
    editor = {}
    "#;
    fs::write(&temp_path, invalid_content).expect("Failed to write bad content");

    // Action: Load config - this should fail and fall back to default
    let loaded_config = Config::load();

    // Assertions (Fallback):
    // It should have fallen back to default settings (tab_size: 4)
    assert_eq!(
        loaded_config.editor.tab_width, 4,
        "Fallback tab_size check failed."
    );

    //Should not overwrite error, since want users to fix their own instead
    let new_content = fs::read_to_string(&temp_path).expect("Failed to read back overwritten file");
    assert!(
        new_content.contains("oops_not_an_int"),
        "Invalid file content was not overwritten."
    );

    // Cleanup
    cleanup_test_path(&temp_path);
}

#[test]
fn test_config_reload_success() {
    // Setup: Initial config file
    let temp_path = PathBuf::from("temp_reload_config.toml");
    Config::set_test_config_path(Some(temp_path.clone()));

    // Use default config for initial save
    let initial_config = Config::default();
    initial_config
        .save()
        .expect("Failed to save initial config");

    // Action: Load the initial config into a mutable variable
    let mut app_config = Config::load();

    // Action: Manually change the config file content
    let new_content = r#"
        [editor]
        tab_width = 2
        use_spaces = true
        line_numbers = true
        relative_line_numbers = true
        wrap_lines = false
        auto_save = false
        auto_save_delay_ms = 1000
        show_whitespace = false
        highlight_current_line = false

        [keymaps.editor]

        [keymaps.command_line]

        [keymaps.debug]

        [ui]
        theme = "default"
        show_status_bar = true
        show_tab_bar = false
        cursor_style = "block"
        cursor_blink = true
        scrolloff = 3

        [performance]
        tick_rate_ms = 50
        cursor_blink_rate_ms = 500
        undo_history_limit = 500
        clipboard_history_limit = 100
        lazy_redraw = false
    "#;
    fs::write(&temp_path, new_content).expect("Failed to write updated content");

    // Action: Reload the config
    assert!(app_config.reload().is_ok());

    // Assertions: Check if settings were updated
    assert_eq!(
        app_config.editor.tab_width, 2,
        "Reloaded editor setting failed."
    );
    assert_eq!(
        app_config.performance.undo_history_limit, 500,
        "Reloaded performance setting failed."
    );
    eprintln!("{}", format!("{}", app_config.runtime_keymaps.is_some()));
    // Check keymaps were reparsed (they should be empty based on the new content)
    assert!(app_config
        .runtime_keymaps
        .as_ref()
        .unwrap()
        .editor
        .is_empty());

    // Cleanup
    cleanup_test_path(&temp_path);
}
