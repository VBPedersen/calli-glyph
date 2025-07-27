use super::input_action::*;
use crate::config::key_binds;
use crate::core::app::ActiveArea;
use crate::core::app::App;
use crossterm::event;
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind,
};

/// Reads the crossterm events and updates the state of [`App`].
///
/// If your application needs to perform work in between handling events, you can use the
/// [`event::poll`] function to check if there are any events available with a timeout.
pub(crate) fn handle_input(app: &mut App) -> color_eyre::Result<()> {
    match event::read()? {
        // it's important to check KeyEventKind::Press to avoid handling key release events
        Event::Key(key) if key.kind == KeyEventKind::Press => on_key_event(app, key),
        Event::Mouse(mouse)
            if (mouse.kind == MouseEventKind::ScrollDown)
                | (mouse.kind == MouseEventKind::ScrollUp) =>
        {
            on_scroll_events(app, mouse)
        }
        Event::Resize(_, _) => {}
        _ => {}
    }
    Ok(())
}

fn on_scroll_events(app: &mut App, mouse: MouseEvent) {
    match app.active_area {
        ActiveArea::Editor => match mouse.kind {
            MouseEventKind::ScrollDown => app.move_scroll_offset(1),
            MouseEventKind::ScrollUp => app.move_scroll_offset(-1),
            _ => {}
        },
        _ => {}
    }
}

/// Handles the key events and updates the state of [`App`].
fn on_key_event(app: &mut App, key: KeyEvent) {
    //println!("Detected key: {:?}, modifiers: {:?}", key.code, key.modifiers);
    let input_action: InputAction = map_key_to_action(app, key);
    app.process_input_action(input_action);
    /*
    match app.active_area {
        ActiveArea::Editor => match (key.modifiers, key.code) {
            key_binds::KEYBIND_UP => app.move_all_cursor_editor(0, -1, false),
            key_binds::KEYBIND_DOWN => app.move_all_cursor_editor(0, 1, false),
            key_binds::KEYBIND_LEFT => app.move_all_cursor_editor(-1, 0, false),
            key_binds::KEYBIND_RIGHT => app.move_all_cursor_editor(1, 0, false),
            key_binds::KEYBIND_TOGGLE_AREA => app.toggle_active_area(),
            key_binds::KEYBIND_BACKSPACE => app.backspace_all_in_editor(),
            key_binds::KEYBIND_TAB => app.tab_in_editor(),
            key_binds::KEYBIND_ENTER => {
                app.enter_in_editor();
            }
            key_binds::KEYBIND_DELETE => {
                app.delete_all_in_editor();
            }
            key_binds::KEYBIND_SELECTION_UP => {
                app.move_all_cursor_editor(0, -1, true);
            }
            key_binds::KEYBIND_SELECTION_DOWN => {
                app.move_all_cursor_editor(0, 1, true);
            }
            key_binds::KEYBIND_SELECTION_LEFT => {
                app.move_all_cursor_editor(-1, 0, true);
            }
            key_binds::KEYBIND_SELECTION_RIGHT => {
                app.move_all_cursor_editor(1, 0, true);
            }
            key_binds::KEYBIND_SAVE => {
                if let Err(e) = app.save(vec![]) {
                    let popup = Box::new(ErrorPopup::new(
                        "Failed to Save File",
                        AppError::InternalError(e.to_string()),
                    ));
                    app.open_popup(popup);
                }
            }
            key_binds::KEYBIND_COPY => {
                if let Err(e) = app.copy_selected_text() {
                    let popup = Box::new(ErrorPopup::new("Failed to copy selected text", e));
                    app.open_popup(popup);
                }
            }
            key_binds::KEYBIND_CUT => {
                if let Err(e) = app.cut_selected_text() {
                    let popup = Box::new(ErrorPopup::new("Failed to cut selected text", e));
                    app.open_popup(popup);
                }
            }
            key_binds::KEYBIND_PASTE => {
                if let Err(e) = app.paste_selected_text() {
                    let popup = Box::new(ErrorPopup::new("Failed to paste selected text", e));
                    app.open_popup(popup);
                }
            }
            key_binds::KEYBIND_UNDO => {
                if let Err(e) = app.undo_in_editor() {
                    let popup = Box::new(ErrorPopup::new("Failed to UNDO", e));
                    app.open_popup(popup);
                }
            }
            key_binds::KEYBIND_REDO => {
                if let Err(e) = app.redo_in_editor() {
                    let popup = Box::new(ErrorPopup::new("Failed to REDO", e));
                    app.open_popup(popup);
                }
            }
            (_, KeyCode::Char(c)) => app.write_all_char_in_editor(c), // HAS TO BE LAST
            _ => {}
        },
        ActiveArea::CommandLine => match (key.modifiers, key.code) {
            key_binds::KEYBIND_LEFT => app.move_cursor_in_command_line(-1),
            key_binds::KEYBIND_RIGHT => app.move_cursor_in_command_line(1),
            (_, KeyCode::Tab | KeyCode::Esc) => app.toggle_active_area(),
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => app.quit(),
            (_, KeyCode::Char(c)) => app.write_char_to_command_line(c),
            key_binds::KEYBIND_BACKSPACE => app.backspace_on_command_line(),
            key_binds::KEYBIND_ENTER => {
                on_command_enter(app);
            }
            _ => {}
        },
        ActiveArea::Popup => {
            if let Some(popup) = app.popup.as_mut() {
                let res = popup.handle_key_input(key);
                app.popup_result = res;

                match popup.get_popup_type() {
                    PopupType::Confirmation => app.handle_confirmation_popup_response(),
                    PopupType::Error => app.handle_error_popup_response(),
                    _ => {}
                }
            }
        }
    }*/
}

pub fn map_key_to_action(app: &App, key: KeyEvent) -> InputAction {
    use key_binds::*;

    match app.active_area {
        ActiveArea::Editor => match (key.modifiers, key.code) {
            KEYBIND_UP => InputAction::MoveCursor(Direction::Up),
            KEYBIND_DOWN => InputAction::MoveCursor(Direction::Down),
            KEYBIND_LEFT => InputAction::MoveCursor(Direction::Left),
            KEYBIND_RIGHT => InputAction::MoveCursor(Direction::Right),
            KEYBIND_SELECTION_UP => InputAction::MoveSelectionCursor(Direction::Up),
            KEYBIND_SELECTION_DOWN => InputAction::MoveSelectionCursor(Direction::Down),
            KEYBIND_SELECTION_LEFT => InputAction::MoveSelectionCursor(Direction::Left),
            KEYBIND_SELECTION_RIGHT => InputAction::MoveSelectionCursor(Direction::Right),
            KEYBIND_TAB => InputAction::TAB,
            KEYBIND_ENTER => InputAction::ENTER,
            KEYBIND_BACKSPACE => InputAction::BACKSPACE,
            KEYBIND_DELETE => InputAction::DELETE,
            KEYBIND_SAVE => InputAction::SAVE,
            KEYBIND_COPY => InputAction::COPY,
            KEYBIND_CUT => InputAction::CUT,
            KEYBIND_PASTE => InputAction::PASTE,
            KEYBIND_UNDO => InputAction::UNDO,
            KEYBIND_REDO => InputAction::REDO,
            KEYBIND_TOGGLE_AREA => InputAction::ToggleActiveArea,
            (_, KeyCode::Char(c)) => InputAction::WriteChar(c),
            _ => InputAction::NoOp,
        },
        ActiveArea::CommandLine => match (key.modifiers, key.code) {
            KEYBIND_LEFT => InputAction::MoveCursor(Direction::Left),
            KEYBIND_RIGHT => InputAction::MoveCursor(Direction::Right),
            KEYBIND_BACKSPACE => InputAction::BACKSPACE,
            KEYBIND_DELETE => InputAction::DELETE,
            KEYBIND_ENTER => InputAction::ENTER,
            (_, KeyCode::Tab | KeyCode::Esc) => InputAction::ToggleActiveArea,
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => InputAction::QUIT,
            (_, KeyCode::Char(c)) => InputAction::WriteChar(c),
            _ => InputAction::NoOp,
        },
        ActiveArea::Popup => match (key.modifiers, key.code) {
            KEYBIND_UP => InputAction::MoveCursor(Direction::Up),
            KEYBIND_DOWN => InputAction::MoveCursor(Direction::Down),
            KEYBIND_LEFT => InputAction::MoveCursor(Direction::Left),
            KEYBIND_RIGHT => InputAction::MoveCursor(Direction::Right),
            KEYBIND_ENTER => InputAction::ENTER,
            _ => InputAction::NoOp,
        },
    }
}
