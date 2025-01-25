use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind};
use crate::App;
use crate::app::ActiveArea;

//COMMAND BINDS
const COMMAND_EXIT_DONT_SAVE:&str = ":q";

/// Reads the crossterm events and updates the state of [`App`].
///
/// If your application needs to perform work in between handling events, you can use the
/// [`event::poll`] function to check if there are any events available with a timeout.
pub(crate) fn handle_input(app: &mut App) -> color_eyre::Result<()> {
    match event::read()? {
        // it's important to check KeyEventKind::Press to avoid handling key release events
        Event::Key(key) if key.kind == KeyEventKind::Press => on_key_event(app,key),
        Event::Mouse(mouse) if (mouse.kind == MouseEventKind::ScrollDown) |
            (mouse.kind == MouseEventKind::ScrollUp) => {on_scroll_events(app,mouse)}
        Event::Resize(_, _) => {}
        _ => {}
    }
    Ok(())
}

fn on_scroll_events(app: &mut App, mouse: MouseEvent) {
    match app.active_area {
        ActiveArea::Editor => {
            match mouse.kind {
                MouseEventKind::ScrollDown => { app.move_scroll_offset(1) },
                MouseEventKind::ScrollUp => { app.move_scroll_offset(-1) },
                _ => {}
            }
        },
        _ => {}
    }
}

/// Handles the key events and updates the state of [`App`].
fn on_key_event(app: &mut App, key: KeyEvent) {
    match app.active_area {
        ActiveArea::Editor => match (key.modifiers, key.code) {
            (_, KeyCode::Up) => app.move_cursor_in_editor(0, -1),
            (_, KeyCode::Down) => app.move_cursor_in_editor(0, 1),
            (_, KeyCode::Left) => app.move_cursor_in_editor(-1, 0),
            (_, KeyCode::Right) => app.move_cursor_in_editor(1, 0),
            (_, KeyCode::Esc) | (KeyModifiers::SHIFT, KeyCode::Char(':')) => app.toggle_active_area(),
            (_, KeyCode::Char(c)) =>  app.write_char_to_editor(c) ,
            (_, KeyCode::Backspace) => { app.backpace_on_editor() },
            _ => {}
        },
        ActiveArea::CommandLine => match (key.modifiers, key.code) {
            (_, KeyCode::Left) => app.move_cursor_in_command_line(-1),
            (_, KeyCode::Right) => app.move_cursor_in_command_line(1),
            (_, KeyCode::Tab | KeyCode::Esc) => app.toggle_active_area(),
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => app.quit(),
            (_, KeyCode::Char(c)) => { app.write_char_to_command_line(c) },
            (_, KeyCode::Backspace) => { app.backpace_on_command_line() },
            (_, KeyCode::Enter) => { on_command_enter(app); },
            _ => {}
        },
    }
}

fn on_command_enter(app: &mut App) {
    match app.command_input.as_str(){
        COMMAND_EXIT_DONT_SAVE => {app.quit()},
        _ => {}
    }
}