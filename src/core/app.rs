use std::collections::VecDeque;
use super::command_line::{command, command_executor, CommandLine};
use super::editor::Editor;
use crate::app_config::AppLaunchConfig;
use crate::config::Config;
use crate::core::debug::DebugState;
use crate::errors::error::AppError;
use crate::errors::error::AppError::EditorFailure;
use crate::input::input::handle_input;
use crate::input::input_action::InputAction;
use crate::ui::debug::DebugView;
use crate::ui::popups::error_popup::ErrorPopup;
use crate::ui::popups::popup::{Popup, PopupResult, PopupType};
use crate::ui::ui::ui;
use color_eyre::Result;
use crossterm::event;
use ratatui::DefaultTerminal;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct App {
    /// Is the application running?
    running: bool,
    pub config: Config,
    pub active_area: ActiveArea,
    pub editor: Editor,
    pub command_line: CommandLine,
    pub(crate) cursor_visible: bool,
    pub(crate) terminal_height: i16,
    pub file_path: Option<PathBuf>,
    pub popup: Option<Box<dyn Popup>>,
    pub popup_result: PopupResult,
    pub pending_states: VecDeque<PendingState>,
    pub debug_state: DebugState,
    pub debug_view: DebugView,
    pub content_modified: bool,
}

pub type OpCallback = Box<dyn FnOnce(&mut App)>;

pub enum PendingState {
    None,
    Saving(PathBuf),
    Quitting,
    ConfigEdit {
        on_confirm: OpCallback,
    },
}

#[derive(PartialEq, Debug, Default, Copy, Clone)]
pub enum ActiveArea {
    #[default]
    Editor,
    CommandLine,
    Popup,
    DebugConsole,
}

impl Default for App {
    fn default() -> Self {
        let config = Config::default();
        let temp_config = Arc::new(config.editor.clone());
        Self {
            running: Default::default(),
            config,
            active_area: Default::default(),
            editor: Editor::new(temp_config),
            command_line: CommandLine::new(),
            cursor_visible: true,
            terminal_height: 0,
            file_path: None,
            popup: None,
            popup_result: PopupResult::None,
            pending_states: VecDeque::new(),
            debug_state: DebugState::new(),
            debug_view: DebugView::new(),
            content_modified: false,
        }
    }
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new(config: Config, launch_config: AppLaunchConfig) -> Self {
        let editor_config_arc = Arc::new(config.editor.clone());
        Self {
            running: Default::default(),
            config,
            active_area: Default::default(),
            editor: Editor::new(editor_config_arc),
            command_line: CommandLine::new(),
            cursor_visible: true,
            terminal_height: 0,
            file_path: launch_config.file_path,
            popup: None,
            popup_result: PopupResult::None,
            pending_states: VecDeque::new(),
            debug_state: DebugState::new(),
            debug_view: DebugView::new(),
            content_modified: false,
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        //SETUP
        self.running = true;
        self.active_area = ActiveArea::Editor;
        let mut last_auto_save = Instant::now();
        let mut needs_redraw = true;

        // Read file contents if a file path is provided
        self.read_file_to_editor_if_path_provided();

        let tick_rate = Duration::from_millis(self.config.performance.tick_rate_ms); // use tick rate from config
        let cursor_blink_rate = Duration::from_millis(self.config.performance.cursor_blink_rate_ms); // use blink rate from config

        let mut last_tick = Instant::now();
        let mut last_cursor_toggle = Instant::now();

        while self.running {
            // Only draw if needed (lazy redraw)
            if !self.config.performance.lazy_redraw || needs_redraw {
                terminal.draw(|frame| ui(frame, &mut self))?;
                needs_redraw = false;
            }

            // Auto-save check
            if self.config.editor.auto_save
                && self.content_modified
                && last_auto_save.elapsed()
                    >= Duration::from_millis(self.config.editor.auto_save_delay_ms)
            {
                if let Some(path) = &self.file_path.clone() {
                    let _ = self.save_to_path(&*path.clone());
                    self.content_modified = false;
                    log_info!("Auto-saved file: {:?}", path);
                }
                last_auto_save = Instant::now();
            }

            // Calculate timeout until next cursor blink or tick
            let time_until_cursor = cursor_blink_rate.saturating_sub(last_cursor_toggle.elapsed());
            let time_until_tick = tick_rate.saturating_sub(last_tick.elapsed());

            let timeout = time_until_cursor.min(time_until_tick);

            // Poll for input with calculated timeout
            if event::poll(timeout)? {
                handle_input(&mut self)?;
                needs_redraw = true; // Redraw after input
            }

            // Handle cursor blinking
            if last_cursor_toggle.elapsed() >= cursor_blink_rate {
                self.cursor_visible = !self.cursor_visible;
                last_cursor_toggle = Instant::now();
                needs_redraw = true; // Redraw on blink
            }

            // Handle periodic tick (for debug metrics)
            if last_tick.elapsed() >= tick_rate {
                if self.debug_state.enabled {
                    self.debug_state.tick_frame();
                }
                last_tick = Instant::now();
            }
        }
        Ok(())
    }

    /// Function to read a file to the editor if file path is some.
    fn read_file_to_editor_if_path_provided(&mut self) {
        self.editor.editor_content = if let Some(ref path) = self.file_path {
            log_info!("Attempting to load file : {}", path.display());

            match File::open(path) {
                Ok(f) => {
                    let mut buff_read_file = BufReader::new(f);
                    let mut contents = String::new();
                    match buff_read_file.read_to_string(&mut contents) {
                        Ok(_size) => contents.lines().map(String::from).collect(),
                        Err(err) => {
                            //if file not found create new
                            self.running = false;
                            panic!("Failed to create file '{}': {}", path.display(), err);
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
                            panic!("Failed to create file '{}': {}", path.display(), create_err);
                        }
                    }
                }
            }
        } else {
            vec![String::new()] // Start with an empty editor if no file is provided
        };
    }

    ///function to process input action, responsible for calling the related active area,
    /// with the gotten input action.
    pub fn process_input_action(&mut self, action: InputAction) {
        //if debug state is enabled, record input event and only if debug console is not active,
        // Makes no sense to debug actions on debugger console to log
        if self.debug_state.enabled && self.active_area != ActiveArea::DebugConsole {
            self.debug_state.metrics.record_event();
            log_info!("Action: {:?}", action);
        }
        self.check_for_app_related_input_actions(action.clone());
        match self.active_area {
            ActiveArea::Editor => {
                if let Err(e) = self.editor.handle_input_action(action) {
                    let popup = Box::new(ErrorPopup::new("Editor Error", EditorFailure(e)));
                    self.open_popup(popup);
                }

                // else is successful, so set content modified true
                self.content_modified = self.editor.undo_redo_manager.is_dirty();
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
                        PopupType::Info => self.handle_info_popup_response(),
                        _ => {}
                    }
                }
            }
            ActiveArea::DebugConsole => {
                self.handle_debug_input_action(action);
            }
        }
    }

    ///function to check for app related input actions,
    /// i.e. input action that should result in app related functionality,
    /// like quitting should call method quit in app.rs
    fn check_for_app_related_input_actions(&mut self, action: InputAction) {
        match action {
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
        //split commandline input to command and arguments
        //if successful parse to command and use the executor to execute commands
        //open popup for error if execution unsuccessful
        match self.command_line.split_command_bind_and_args() {
            Ok((bind, args)) => {
                let command = command::parse_command(bind, args);
                if let Err(e) = command_executor::execute_command(self, command) {
                    let popup = Box::new(ErrorPopup::new(
                        "Command Failed",
                        AppError::CommandFailure(e),
                    ));
                    self.open_popup(popup);
                }
            }
            Err(error) => {
                let popup = Box::new(ErrorPopup::new(
                    "Command Parse Failed",
                    AppError::InternalError(error.to_string()),
                ));
                self.open_popup(popup);
            }
        }
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
    pub fn handle_confirmation_popup_response(&mut self) { //TODO maybe change pending_states to be VecDeque and pop front
        let Some(pending) = self.pending_states.front() else { return };

        // check for absolutes, here quitting should quit regardless anything
        if matches!(pending, PendingState::Quitting) {
            self.pending_states.pop_front();
            self.quit();
            return;
        }

        // since method is for confirmation popup responses, only bool is relevant
        let PopupResult::Bool(confirmed) = self.popup_result else {
            return;
        };

        // Consume, and since we already checked if front is some, we can safely unwrap
        let state = self.pending_states.pop_front().unwrap();
        
        // Only check state if confirmed, since these functionalities should only execute when confirmed
        if confirmed {
            match state {
                PendingState::Saving(path) => if confirmed { //Should only execute if confirmed (clicked yes)
                    match self.save_to_path(&*path.clone()) {
                        Ok(()) => {
                            self.pending_states.remove(0);
                            self.close_popup();
                        }
                        Err(e) => {
                            let popup = Box::new(ErrorPopup::new(
                                "Failed to save file",
                                AppError::InternalError(e.to_string()),
                            ));
                            self.open_popup(popup);
                        }
                    }
                }
                PendingState::ConfigEdit { on_confirm } => if confirmed { //Should only execute if confirmed (clicked yes)
                    on_confirm(self);
                    self.close_popup();
                }
                _ => {}
            }
        }
        
        self.popup_result = PopupResult::None;
        self.close_popup();

        // Check again if there's more to do (like Quitting after Saving)
        if !self.pending_states.is_empty() {
            self.handle_confirmation_popup_response();
       }
    }

    ///handles response from error popup, should only close popup
    pub fn handle_error_popup_response(&mut self) {
        if self.popup_result == PopupResult::Affirmed {
            self.close_popup();
        }
    }

    ///handles response from info popup, should only close popup
    pub fn handle_info_popup_response(&mut self) {
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

    ///saves contents to file at path
    pub fn save_to_path(&mut self, path: &Path) -> Result<(), AppError> {
        let new_content = self.editor.editor_content.join("\n");

        let path_ref = Path::new(&path);
        if let Some(parent) = path_ref.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(new_content.as_bytes())?;
        writer.flush()?;

        self.file_path = Some(path.to_path_buf()); // optionally update file_path

        // mark saved index on undo tree
        self.editor.undo_redo_manager.mark_saved();
        Ok(())
    }

    ///checks if file has changes and returns boolean
    pub(crate) fn file_has_changes(
        &self,
        editor_content: String,
        file_path: &Path,
    ) -> Result<bool> {
        let file = File::open(file_path)?;
        let mut buff_read_file = BufReader::new(file);
        let mut read_file_contents = String::new();

        buff_read_file
            .read_to_string(&mut read_file_contents)
            .expect("TODO: panic message");
        // If file has changes, return true else return false
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
        let app = App::default();
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
        let mut app = App::default();
        app.command_line.input = s;
        app
    }

    #[test]
    fn test_valid_command_with_args() {
        let mut app = create_app(":command arg1 arg2".to_string());

        let result = app.command_line.split_command_bind_and_args();
        assert!(result.is_ok());
        let (cmd, args) = result.unwrap();
        assert_eq!(cmd, "command");
        assert_eq!(args, vec!["arg1", "arg2"]);
    }

    #[test]
    fn test_valid_command_no_args() {
        let mut app = create_app(":hello".to_string());

        let result = app.command_line.split_command_bind_and_args();
        assert!(result.is_ok());
        let (cmd, args) = result.unwrap();
        assert_eq!(cmd, "hello");
        assert!(args.is_empty());
    }

    #[test]
    fn test_missing_command() {
        let mut app = create_app("not_a_command arg1".to_string());

        let result = app.command_line.split_command_bind_and_args();
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "No valid command found");
    }

    #[test]
    fn test_empty_input() {
        let mut app = create_app("".to_string());

        let result = app.command_line.split_command_bind_and_args();
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "No valid command found");
    }
}
