use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind};
use crate::App;
use crate::app::ActiveArea;
use crate::config::{command_binds, key_binds};
use crate::error_popup::ErrorPopup;
use crate::popup::{PopupType};

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

            key_binds::KEYBIND_UP => app.move_all_cursor_editor(0, -1, false),
            key_binds::KEYBIND_DOWN => app.move_all_cursor_editor(0, 1, false),
            key_binds::KEYBIND_LEFT => app.move_all_cursor_editor(-1, 0, false),
            key_binds::KEYBIND_RIGHT => app.move_all_cursor_editor(1, 0, false),
            key_binds::KEYBIND_TOGGLE_AREA => app.toggle_active_area(),
            key_binds::KEYBIND_BACKSPACE => { app.backspace_all_in_editor() },
            key_binds::KEYBIND_TAB => { app.tab_in_editor() },
            key_binds::KEYBIND_ENTER => { app.enter_in_editor(); },
            key_binds::KEYBIND_DELETE => { app.delete_all_in_editor(); },
            key_binds::KEYBIND_SELECTION_UP => { app.move_all_cursor_editor(0, -1, true); },
            key_binds::KEYBIND_SELECTION_DOWN => { app.move_all_cursor_editor(0, 1, true); },
            key_binds::KEYBIND_SELECTION_LEFT => { app.move_all_cursor_editor(-1, 0, true); },
            key_binds::KEYBIND_SELECTION_RIGHT => { app.move_all_cursor_editor(1, 0, true); },
            key_binds::KEYBIND_COPY => {
                if let Err(e) = app.copy_selected_text() {
                    let popup = Box::new(ErrorPopup::new("Failed to copy selected text",e));
                    app.open_popup(popup);
                }
            },
            key_binds::KEYBIND_PASTE => {
                if let Err(e) = app.paste_selected_text() {
                    let popup = Box::new(ErrorPopup::new("Failed to paste selected text",e));
                    app.open_popup(popup);
                }
            },
            (_, KeyCode::Char(c)) => app.write_all_char_in_editor(c), // HAS TO BE LAST
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
        ActiveArea::Popup => if let Some(popup) = app.popup.as_mut() {
            let res = popup.handle_key_input(key);
            app.popup_result = res;

            match popup.get_popup_type() { 
                PopupType::Confirmation => {
                    app.handle_confirmation_popup_response()
                },
                PopupType::Error => {
                    app.handle_error_popup_response()
                }
                _ => {}
            }
        }
    }
}


///handles checking command and executing said command with given args
fn on_command_enter(app: &mut App) {
    match split_command_bind_and_args(app) {
        Ok((command_bind, command_args)) => {
            match command_bind.as_ref() {
                command_binds::COMMAND_EXIT_DONT_SAVE => {app.quit()},
                command_binds::COMMAND_SAVE_DONT_EXIT => { app.save(command_args).expect("TODO: panic message");},
                command_binds::COMMAND_SAVE_AND_EXIT => { app.save_and_exit(command_args).expect("TODO: panic message");},
                command_binds::COMMAND_HELP => { },
                _ => {}
            }
        }
        Err(error) => {
            println!("Error: {}", error);
        }
    }
}

///to split command line text into a command and arguments
fn split_command_bind_and_args(app: &mut App) -> Result<(String, Vec<String>), String> {
    let mut command_bind:Option<String> = None;
    let mut command_args = vec![];
    let mut parts = app.command_line.input.split_whitespace();

    if let Some(first) = parts.next() {
        if let Some(':') = first.chars().next() {
            command_bind = Some(first.chars().skip(1).collect());
        }
    }

    if let Some(ref cmd) = command_bind {
        command_args.extend(parts.map(String::from));
        return Ok((cmd.clone(), command_args));
    }

    Err("No valid command found".to_string())
}

//



#[cfg(test)]
mod tests {
    use super::*;
    fn create_app_with_command_input(s: String) -> App {
        let mut app = App::new();
        app.command_line.input = s;
        app

    }
    #[test]
    fn test_valid_command_with_args() {
        let mut app = create_app_with_command_input(":command arg1 arg2".to_string());

        let result = split_command_bind_and_args(&mut app);
        assert!(result.is_ok());
        let (cmd, args) = result.unwrap();
        assert_eq!(cmd, "command");
        assert_eq!(args, vec!["arg1", "arg2"]);
    }

    #[test]
    fn test_valid_command_no_args() {
        let mut app = create_app_with_command_input(":hello".to_string());

        let result = split_command_bind_and_args(&mut app);
        assert!(result.is_ok());
        let (cmd, args) = result.unwrap();
        assert_eq!(cmd, "hello");
        assert!(args.is_empty());
    }

    #[test]
    fn test_missing_command() {
        let mut app = create_app_with_command_input("not_a_command arg1".to_string());

        let result = split_command_bind_and_args(&mut app);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "No valid command found");
    }

    #[test]
    fn test_empty_input() {
        let mut app = create_app_with_command_input("".to_string());

        let result = split_command_bind_and_args(&mut app);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "No valid command found");
    }
}