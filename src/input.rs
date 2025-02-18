use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind};
use crate::App;
use crate::app::ActiveArea;
use crate::config::{command_binds, key_binds};


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
    //println!("Detected key: {:?}, modifiers: {:?}", key.code, key.modifiers);
    match app.active_area {
        ActiveArea::Editor => match (key.modifiers, key.code) {
            //no modifiers
            key_binds::KEYBIND_UP => app.move_all_cursor_editor(0, -1,false),
            key_binds::KEYBIND_DOWN => app.move_all_cursor_editor(0, 1,false),
            key_binds::KEYBIND_LEFT => app.move_all_cursor_editor(-1, 0,false),
            key_binds::KEYBIND_RIGHT => app.move_all_cursor_editor(1, 0,false),
            key_binds::KEYBIND_TOGGLE_AREA => app.toggle_active_area(),
            (KeyModifiers::NONE, KeyCode::Char(c)) =>  app.write_all_char_in_editor(c) ,
            key_binds::KEYBIND_BACKSPACE => { app.backspace_all_in_editor() },
            key_binds::KEYBIND_TAB => { app.tab_in_editor() },
            key_binds::KEYBIND_ENTER => { app.enter_in_editor(); },
            key_binds::KEYBIND_DELETE => { app.delete_all_in_editor(); },
            //with modifiers
            key_binds::KEYBIND_SELECTION_UP => { app.move_all_cursor_editor(0,-1,true); },
            key_binds::KEYBIND_SELECTION_DOWN => { app.move_all_cursor_editor(0,1,true); },
            key_binds::KEYBIND_SELECTION_LEFT => { app.move_all_cursor_editor(-1,0,true); },
            key_binds::KEYBIND_SELECTION_RIGHT => { app.move_all_cursor_editor(1,0,true); },
            key_binds::KEYBIND_COPY =>  { app.copy_selected_text(); } ,
            key_binds::KEYBIND_PASTE =>  { app.paste_selected_text(); } ,
            _ => {}
        },
        ActiveArea::CommandLine => match (key.modifiers, key.code) {
            key_binds::KEYBIND_LEFT => app.move_cursor_in_command_line(-1),
            key_binds::KEYBIND_RIGHT => app.move_cursor_in_command_line(1),
            (_, KeyCode::Tab | KeyCode::Esc) => app.toggle_active_area(),
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => app.quit(),
            (_, KeyCode::Char(c)) => { app.write_char_to_command_line(c) },
            key_binds::KEYBIND_BACKSPACE => { app.backspace_on_command_line() },
            key_binds::KEYBIND_ENTER => { on_command_enter(app); },
            _ => {}
        },
    }
}

fn on_command_enter(app: &mut App) {
    match app.command_input.as_str(){
        command_binds::COMMAND_EXIT_DONT_SAVE => {app.quit()},
        command_binds::COMMAND_SAVE_DONT_EXIT => { app.save().expect("TODO: panic message");},
        command_binds::COMMAND_SAVE_AND_EXIT => { app.save_and_exit().expect("TODO: panic message");},
        _ => {}
    }
}