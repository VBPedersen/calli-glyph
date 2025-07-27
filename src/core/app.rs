use super::clipboard::Clipboard;
use super::command_line::CommandLine;
use super::editor::Editor;
use super::errors::AppError;
use super::errors::AppError::EditorFailure;
use crate::config::command_binds;
use crate::input::input::handle_input;
use crate::input::input_action::InputAction;
use crate::ui::popups::confirmation_popup::ConfirmationPopup;
use crate::ui::popups::error_popup::ErrorPopup;
use crate::ui::popups::popup::{Popup, PopupResult, PopupType};
use crate::ui::ui::ui;
use color_eyre::Result;
use ratatui::DefaultTerminal;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::time::{Duration, Instant};
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    running: bool,
    pub(crate) active_area: ActiveArea,
    pub editor: Editor,
    pub command_line: CommandLine,
    pub(crate) cursor_visible: bool,
    last_tick: Instant,
    pub(crate) terminal_height: i16,
    pub clipboard: Clipboard,
    pub file_path: Option<String>,
    pub popup: Option<Box<dyn Popup>>,
    pub popup_result: PopupResult,
    pub pending_states: Vec<PendingState>,
}

#[derive(Debug, PartialEq)]
pub enum PendingState {
    None,
    Saving(String),
    Quitting,
}

#[derive(PartialEq, Debug, Default)]
pub(crate) enum ActiveArea {
    #[default]
    Editor,
    CommandLine,
    Popup,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: Default::default(),
            active_area: Default::default(),
            editor: Editor::new(),
            command_line: CommandLine::new(),
            last_tick: Instant::now(),
            cursor_visible: true,
            terminal_height: 0,
            clipboard: Clipboard::new(),
            file_path: None,
            popup: None,
            popup_result: PopupResult::None,
            pending_states: vec![],
        }
    }
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal, file_path: Option<String>) -> Result<()> {
        //SETUP

        self.running = true;
        self.active_area = ActiveArea::Editor;
        self.file_path = file_path;

        // Read file contents if a file path is provided
        self.editor.editor_content = if let Some(ref path) = self.file_path {
            match File::open(path) {
                Ok(f) => {
                    let mut buff_read_file = BufReader::new(f);
                    let mut contents = String::new();
                    match buff_read_file.read_to_string(&mut contents) {
                        Ok(_size) => contents.lines().map(String::from).collect(),
                        Err(err) => {
                            //if file not found create new
                            self.running = false;
                            panic!("Failed to create file '{}': {}", path, err);
                        }
                    }
                }
                Err(_err) => {
                    match File::create(path) {
                        //create file, if ok then return else quit and panic
                        Ok(_) => {
                            vec![String::new()] // Return an empty string as the content
                        }
                        Err(create_err) => {
                            self.running = false;
                            panic!("Failed to create file '{}': {}", path, create_err);
                        }
                    }
                }
            }
        } else {
            vec![String::new()] // Start with an empty editor if no file is provided
        };

        //LOGIC

        // Handle cursor blinking (toggle cursor visibility every 500ms)
        if self.last_tick.elapsed() >= Duration::from_millis(500) {
            self.cursor_visible = !self.cursor_visible;
            self.last_tick = Instant::now();
        }

        while self.running {
            terminal.draw(|frame| ui(frame, &mut self))?;
            handle_input(&mut self)?;
        }
        Ok(())
    }

    ///function to process input action, responsible for calling the related active area,
    /// with the gotten input action.
    pub fn process_input_action(&mut self, action: InputAction) {
        self.check_for_app_related_input_actions(action.clone());
        match self.active_area {
            ActiveArea::Editor => {
                if let Err(e) = self.editor.handle_input_action(action) {
                    let popup = Box::new(ErrorPopup::new("Editor Error", EditorFailure(e)));
                    self.open_popup(popup);
                }
            }
            ActiveArea::CommandLine => {
                //check for ENTER on commandline, to execute commands,
                // since entering on a command needs app related function execution,
                // else handle input on commandline, like editing command line content.
                if action == InputAction::ENTER {
                    self.on_command_enter()
                } else {
                    self.command_line.handle_input_action(action)
                }
            }
            ActiveArea::Popup => {
                if let Some(popup) = self.popup.as_mut() {
                    let res = popup.handle_input_action(action);
                    self.popup_result = res;

                    match popup.get_popup_type() {
                        PopupType::Confirmation => self.handle_confirmation_popup_response(),
                        PopupType::Error => self.handle_error_popup_response(),
                        _ => {}
                    }
                }
            }
        }
    }

    ///function to check for app related input actions,
    /// i.e. input action that should result in app related functionality,
    /// like quitting should call method quit in app.rs
    fn check_for_app_related_input_actions(&mut self, action: InputAction) {
        match action {
            //check for Save,
            //because saving should be handled by the app centrally.
            InputAction::SAVE => {
                if let Err(e) = self.save(vec![]) {
                    let popup = Box::new(ErrorPopup::new(
                        "Failed to Save File",
                        AppError::InternalError(e.to_string()),
                    ));
                    self.open_popup(popup);
                }
            }
            //check for active area toggling,
            //because toggle active area should be handled by the app centrally.
            InputAction::ToggleActiveArea => self.toggle_active_area(),
            //check for quitting,
            //because quitting should be handled by the app centrally
            InputAction::QUIT => self.quit(),
            InputAction::NoOp => {}
            _ => {}
        }
    }

    //command line command execution
    ///handles checking command and executing said command with given args
    fn on_command_enter(&mut self) {
        match self.split_command_bind_and_args() {
            Ok((command_bind, command_args)) => match command_bind.as_ref() {
                command_binds::COMMAND_EXIT_DONT_SAVE => self.quit(),
                command_binds::COMMAND_SAVE_DONT_EXIT => {
                    self.save(command_args).expect("TODO: panic message");
                }
                command_binds::COMMAND_SAVE_AND_EXIT => {
                    self.save_and_exit(command_args)
                        .expect("TODO: panic message");
                }
                command_binds::COMMAND_HELP => {}
                _ => {}
            },
            Err(error) => {
                println!("Error: {}", error);
            }
        }
    }

    ///to split command line text into a command and arguments
    fn split_command_bind_and_args(&mut self) -> Result<(String, Vec<String>), String> {
        let mut command_bind: Option<String> = None;
        let mut command_args = vec![];
        let mut parts = self.command_line.input.split_whitespace();

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

    //SCROLL
    ///moves the scroll offset
    pub(crate) fn move_scroll_offset(&mut self, offset: i16) {
        self.editor.move_scroll_offset(offset);
    }

    //PANEL HANDLING
    ///toggles the active area of the app, between editor and command line
    pub(crate) fn toggle_active_area(&mut self) {
        match self.active_area {
            ActiveArea::Editor => {
                self.active_area = ActiveArea::CommandLine;
            }
            ActiveArea::CommandLine => {
                self.active_area = ActiveArea::Editor;
            }

            _ => {}
        }
    }

    ///handles creating popup to confirm if file should be overridden
    pub fn handle_confirmation_popup_response(&mut self) {
        //get first state in vec, match the state and if needed checks next state after that
        if self.pending_states.is_empty() {
            return;
        }

        let state = self.pending_states.first().unwrap();
        match state {
            PendingState::Saving(save_path) => {
                if self.popup_result == PopupResult::Bool(true) {
                    if let Err(e) = self.save(vec![save_path.clone()]) {
                        let popup = Box::new(ErrorPopup::new(
                            "Failed to save file",
                            AppError::InternalError(e.to_string()),
                        ));
                        self.open_popup(popup);
                    }

                    self.popup_result = PopupResult::None;
                    self.close_popup();
                    self.pending_states.remove(0);
                    //if next state is quit, then quit
                    if !self.pending_states.is_empty()
                        && self.pending_states[0] == PendingState::Quitting
                    {
                        self.pending_states.clear();
                        self.quit()
                    }
                } else if self.popup_result == PopupResult::Bool(false) {
                    self.popup_result = PopupResult::None;
                    self.close_popup();
                }
            }
            PendingState::Quitting => {
                self.pending_states.clear();
                self.quit()
            }
            _ => {}
        }
    }

    ///handles response from error popup, should only close popup
    pub fn handle_error_popup_response(&mut self) {
        if self.popup_result == PopupResult::Affirmed {
            self.close_popup();
        }
    }

    ///handles setting popup with defined popup object
    pub fn open_popup(&mut self, popup: Box<dyn Popup>) {
        self.popup = Some(popup);
        self.active_area = ActiveArea::Popup;
    }

    pub fn close_popup(&mut self) {
        self.popup = None;
        self.active_area = ActiveArea::Editor; // Go back to editor
    }

    //Basic Commands

    /// Set running == false, to quit the application.
    pub(crate) fn quit(&mut self) {
        self.running = false;
    }

    ///saves contents to file, if any file path specified in args then saves to that file,
    /// if not and file path is existing then saves to that, else saves to untitled
    /// command_bind <file_path> --flags
    pub fn save(&mut self, args: Vec<String>) -> Result<()> {
        let path;
        let mut path_is_current_file: bool = false;
        let has_changes: bool;
        let mut force_flag: bool = false;

        let new_content = self.editor.editor_content.join("\n");

        //if file path to save on is set in command args
        if !args.is_empty() {
            path = args.first().unwrap().clone();
            force_flag = args.contains(&"--force".to_string());
        } else if self.file_path.is_some() {
            path = self.file_path.clone().unwrap();
            path_is_current_file = true;
        } else {
            path = "untitled".to_string();
        }

        let path_ref = Path::new(&path);

        // Check if file exists
        if path_ref.exists() {
            has_changes = self.file_has_changes(new_content.clone(), path.clone())?;
            //if path is the current file, has changes and force is false
            // and no confirmation has been asked, then make user confirm
            if !path_is_current_file
                && has_changes
                && !force_flag
                && self.popup_result == PopupResult::None
            {
                let popup = Box::new(ConfirmationPopup::new("Confirm Overwrite of file"));
                self.open_popup(popup);
                self.pending_states.push(PendingState::Saving(path));
                return Ok(());
            }
        } else {
            has_changes = !new_content.is_empty();
            // If file doesn't exist, ensure the parent directory exists
            if let Some(parent) = path_ref.parent() {
                fs::create_dir_all(parent)?;
            }
        }

        //if file has changes write these to file
        if has_changes {
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)?;
            let mut buff_write_file = BufWriter::new(file);
            buff_write_file.write_all(new_content.as_bytes())?;
            buff_write_file.flush()?;
            Ok(())
        } else {
            Ok(())
        }
    }

    ///saves file and exits window
    pub(crate) fn save_and_exit(&mut self, args: Vec<String>) -> Result<()> {
        match self.save(args) {
            Ok(_) => {
                // If a save confirmation is needed, push Quit AFTER Saving
                if self
                    .pending_states
                    .iter()
                    .any(|s| matches!(s, PendingState::Saving(_)))
                {
                    self.pending_states.push(PendingState::Quitting); // Add Quit to the queue
                    return Ok(());
                }
                self.quit();

                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    ///checks if file has changes and returns boolean
    pub(crate) fn file_has_changes(
        &self,
        editor_content: String,
        file_path: String,
    ) -> Result<bool> {
        let file = File::open(file_path)?;
        let mut buff_read_file = BufReader::new(file);
        let mut read_file_contents = String::new();

        buff_read_file
            .read_to_string(&mut read_file_contents)
            .expect("TODO: panic message");
        //if has changes, return true else return false
        if !read_file_contents.eq(&editor_content) {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

//████████╗███████╗███████╗████████╗███████╗
//╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//   ██║   █████╗  ███████╗   ██║   ███████╗
//   ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//   ██║   ███████╗███████║   ██║   ███████║
//   ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝

#[cfg(test)]
mod unit_app_tests {
    use super::super::app::*;

    fn create_app() -> App {
        let app = App::new();
        app
    }

    #[test]
    fn test_toggle_to_command_line() {
        let mut app = create_app();
        app.active_area = ActiveArea::Editor;
        app.editor.cursor.x = 5;
        app.editor.cursor.y = 3;

        app.toggle_active_area();
        assert_eq!(app.active_area, ActiveArea::CommandLine);
        assert_eq!(app.command_line.cursor.x, 0);
        assert_eq!(app.command_line.cursor.y, 0);
        assert_eq!(app.editor.cursor.x, 5);
        assert_eq!(app.editor.cursor.y, 3);
    }

    #[test]
    fn test_toggle_to_editor() {
        let mut app = create_app();
        app.active_area = ActiveArea::CommandLine;
        app.editor.cursor.x = 5;
        app.editor.cursor.y = 3;

        app.toggle_active_area();
        assert_eq!(app.active_area, ActiveArea::Editor);
        assert_eq!(app.editor.cursor.x, 5);
        assert_eq!(app.editor.cursor.y, 3);
    }
}
#[cfg(test)]
mod unit_app_command_tests {
    use super::super::app::*;

    fn create_app(s: String) -> App {
        let mut app = App::new();
        app.command_line.input = s;
        app
    }

    #[test]
    fn test_valid_command_with_args() {
        let mut app = create_app(":command arg1 arg2".to_string());

        let result = app.split_command_bind_and_args();
        assert!(result.is_ok());
        let (cmd, args) = result.unwrap();
        assert_eq!(cmd, "command");
        assert_eq!(args, vec!["arg1", "arg2"]);
    }

    #[test]
    fn test_valid_command_no_args() {
        let mut app = create_app(":hello".to_string());

        let result = app.split_command_bind_and_args();
        assert!(result.is_ok());
        let (cmd, args) = result.unwrap();
        assert_eq!(cmd, "hello");
        assert!(args.is_empty());
    }

    #[test]
    fn test_missing_command() {
        let mut app = create_app("not_a_command arg1".to_string());

        let result = app.split_command_bind_and_args();
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "No valid command found");
    }

    #[test]
    fn test_empty_input() {
        let mut app = create_app("".to_string());

        let result = app.split_command_bind_and_args();
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "No valid command found");
    }
}
