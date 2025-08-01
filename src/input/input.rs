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
}

fn map_key_to_action(app: &App, key: KeyEvent) -> InputAction {
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
