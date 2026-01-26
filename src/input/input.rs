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
        Event::Key(key) if key.kind == KeyEventKind::Press => on_key_event(app, key),
        Event::Resize(_, _) => {}
        Event::Mouse(mouse_event) => on_scroll_events(app, mouse_event),
        _ => {}
    }
    Ok(())
}

/// Handles the key events
fn on_key_event(app: &mut App, key: KeyEvent) {
    let config = &app.config;
    let keymaps = config.runtime_keymaps();
    /*app.debug_state
    .log(LogLevel::Trace, format!("keyevent: {:?}", key));*/
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
            //TODO Figure out if popup keybinds should be configurable,
            // for now they are standard figure this is probably best behaviour
            (KeyModifiers::NONE, KeyCode::Up) => Some(InputAction::MoveCursor(Direction::Up)),
            (KeyModifiers::NONE, KeyCode::Down) => Some(InputAction::MoveCursor(Direction::Down)),
            (KeyModifiers::NONE, KeyCode::Left) => Some(InputAction::MoveCursor(Direction::Left)),
            (KeyModifiers::NONE, KeyCode::Right) => Some(InputAction::MoveCursor(Direction::Right)),
            (KeyModifiers::NONE, KeyCode::Enter) => Some(InputAction::ENTER),
            (KeyModifiers::NONE, KeyCode::Esc) => Some(InputAction::ENTER),

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
