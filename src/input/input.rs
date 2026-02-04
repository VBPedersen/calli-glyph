use super::input_action::*;
use crate::core::app::ActiveArea;
use crate::core::app::App;
use color_eyre::eyre::Result;
use crossterm::event;
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind,
};

/// Reads the crossterm events and updates the state of [`App`].
///
/// If your application needs to perform work in between handling events, you can use the
/// [`event::read`] function to read a event.
pub(crate) fn handle_input(app: &mut App) -> Result<()> {
    match event::read()? {
        // it's important to check KeyEventKind::Press to avoid handling key release events
        Event::Key(key) if key.kind == KeyEventKind::Press => {
            // Check if active plugin wants to handle this
            if let Some(plugin_name) = app.plugins.active_plugin_name() {
                // To handle borrow for now i remove and add back plugin from list
                // TODO find better solution if possible
                if let Some(mut plugin) = app.plugins.plugins.remove(&plugin_name) {
                    let consumed = plugin.handle_key_event(app, key);

                    if consumed {
                        // Plugin consumed input
                        if key.code == KeyCode::Esc {
                            app.plugins.deactivate_plugin();
                        }
                    }

                    // Put back plugin
                    app.plugins.plugins.insert(plugin_name, plugin);
                    return Ok(());
                }
            }

            // Try and see if keybind match plugin keybind
            if try_activate_plugin_with_key(app, key) {
                return Ok(());
            }

            // Otherwise, normal app input handling
            on_key_event(app, key)
        }
        Event::Resize(_, _) => {}
        Event::Mouse(mouse_event) => on_scroll_events(app, mouse_event),
        _ => {}
    }
    Ok(())
}

/// Check if any plugin keybinding matches this key
fn try_activate_plugin_with_key(app: &mut App, key: KeyEvent) -> bool {
    // Convert raw KeyEvent to keybinding string (e.g., "Ctrl+t")
    let key_str = key_event_to_keybinding_string(key);
    // Check each plugin's keybindings
    if let Some(plugin_name) = app.plugins.find_plugin_by_keybinding(&key_str) {
        log_info!(
            "Activating plugin '{}' with keybinding '{}'",
            plugin_name,
            key_str
        );
        app.plugins.activate_plugin(&plugin_name);

        // Execute the plugin's command
        if let Some(command_name) = app.plugins.get_keybinding_command(&plugin_name, &key_str) {
            let _ = app.execute_plugin_command(&command_name, vec![]);
        }

        return true;
    }

    false
}

/// Convert KeyEvent to keybinding string format
fn key_event_to_keybinding_string(key: KeyEvent) -> String {
    let mut result = String::new();

    if key.modifiers.contains(KeyModifiers::CONTROL) {
        result.push_str("Ctrl+");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        result.push_str("Alt+");
    }
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        result.push_str("Shift+");
    }

    let key_str = match key.code {
        KeyCode::Char(c) => c.to_uppercase().to_string(),
        KeyCode::F(n) => format!("F{}", n),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        _ => return result,
    };

    result.push_str(&key_str);
    result
}
/// Handles the key events
fn on_key_event(app: &mut App, key: KeyEvent) {
    let config = &app.config;
    let keymaps = config.runtime_keymaps();

    let action = match app.active_area {
        ActiveArea::Editor => {
            if key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT {
                if let KeyCode::Char(c) = key.code {
                    Some(InputAction::WriteChar(c))
                } else {
                    keymaps.get_editor_action(key.modifiers, key.code).cloned()
                }
            } else {
                keymaps.get_editor_action(key.modifiers, key.code).cloned()
            }
        }
        ActiveArea::CommandLine => {
            if key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT {
                if let KeyCode::Char(c) = key.code {
                    Some(InputAction::WriteChar(c))
                } else {
                    keymaps
                        .get_command_line_action(key.modifiers, key.code)
                        .cloned()
                }
            } else {
                keymaps
                    .get_command_line_action(key.modifiers, key.code)
                    .cloned()
            }
        }
        ActiveArea::DebugConsole => keymaps.get_debug_action(key.modifiers, key.code).cloned(),
        ActiveArea::Popup => match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Up) => Some(InputAction::MoveCursor(Direction::Up)),
            (KeyModifiers::NONE, KeyCode::Down) => Some(InputAction::MoveCursor(Direction::Down)),
            (KeyModifiers::NONE, KeyCode::Left) => Some(InputAction::MoveCursor(Direction::Left)),
            (KeyModifiers::NONE, KeyCode::Right) => Some(InputAction::MoveCursor(Direction::Right)),
            (KeyModifiers::NONE, KeyCode::Enter) => Some(InputAction::ENTER),
            (KeyModifiers::NONE, KeyCode::Esc) => Some(InputAction::ToggleActiveArea),

            _ => Some(InputAction::NoOp),
        },
    };
    /* app.debug_state
    .log(LogLevel::Trace, format!("Action: {:?}", action));*/
    if let Some(action) = action {
        app.process_input_action(action);
    }
}

/// Handles the scroll events
fn on_scroll_events(app: &mut App, mouse_event: MouseEvent) {
    // For now if not editor don't scroll
    if app.active_area != ActiveArea::Editor {
        return;
    }

    match mouse_event.kind {
        MouseEventKind::ScrollUp => app.editor.move_scroll_offset(-1),
        MouseEventKind::ScrollDown => app.editor.move_scroll_offset(1),
        _ => {}
    }
}
