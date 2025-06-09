use crate::clipboard::Clipboard;
use crate::command_line::CommandLine;
use crate::confirmation_popup::ConfirmationPopup;
use crate::editor::Editor;
use crate::error_popup::ErrorPopup;
use crate::input::handle_input;
use crate::popup::{Popup, PopupResult};
use crate::ui::ui;
use color_eyre::Result;
use ratatui::DefaultTerminal;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::time::{Duration, Instant};
use crate::errors::{AppError};
use crate::errors::AppError::EditorFailure;

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
            command_line: CommandLine::default(),
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

    //TEXT OPERATIONS

    fn is_text_selected(&self) -> bool {
        self.editor.text_selection_start.is_some() && self.editor.text_selection_end.is_some()
    }

    //IN EDITOR

    ///wrapper function to either call write char with selected text or function write char,
    /// where text isn't selected
    pub(crate) fn write_all_char_in_editor(&mut self, c: char) {
        if self.is_text_selected() {
            self.write_char_in_editor_text_is_selected(c)
        } else {
            self.write_char_in_editor(c)
        }
    }

    ///replaces all selected text with char to y position line, with x position
    fn write_char_in_editor_text_is_selected(&mut self, c: char) {
        self.editor.write_char_text_is_selected(c);
    }

    ///writes char to y position line, with x position
    pub(crate) fn write_char_in_editor(&mut self, c: char) {
        self.editor.write_char(c);
    }

    ///wrapper function to either call backspace in editor with selected text or function backspace_in_editor,
    /// where text isn't selected
    pub(crate) fn backspace_all_in_editor(&mut self) {
        if self.is_text_selected() {
            self.backspace_in_editor_text_is_selected();
        } else {
            self.backspace_in_editor();
        }
    }

    ///handles backspace in editor, removes char at y line x position and sets new cursor position
    pub(crate) fn backspace_in_editor_text_is_selected(&mut self) {
        self.editor.backspace_text_is_selected();
    }

    ///handles backspace in editor, removes char at y line x position and sets new cursor position
    pub(crate) fn backspace_in_editor(&mut self) {
        self.editor.backspace_in_editor();
    }

    ///wrapper function to either call backspace in editor with selected text or function backspace_in_editor,
    /// where text isn't selected
    pub(crate) fn delete_all_in_editor(&mut self) {
        if self.is_text_selected() {
            self.delete_in_editor_text_is_selected();
        } else {
            self.delete_in_editor();
        }
    }

    ///handles delete in editor, removes char at y line x position and sets new cursor position
    pub(crate) fn delete_in_editor_text_is_selected(&mut self) {
        self.editor.delete_text_is_selected()
    }

    ///handles DELETE action, of deleting char in editor at x +1 position
    pub(crate) fn delete_in_editor(&mut self) {
        self.editor.delete_in_editor();
    }

    ///handles TAB action in editor, by writing \t to editor content.
    pub(crate) fn tab_in_editor(&mut self) {
        self.editor.tab();
    }

    ///handles enter new line, with possible move of text
    pub(crate) fn enter_in_editor(&mut self) {
        self.editor.enter();
    }

    //IN COMMANDLINE

    /// writes char to the commandline content at x position, and moves cursor
    pub(crate) fn write_char_to_command_line(&mut self, c: char) {
        let line = &mut self.command_line.input;
        if line.len() < self.command_line.cursor.x as usize {
            self.command_line.cursor.x = line.len() as i16;
        }
        line.insert(self.command_line.cursor.x as usize, c);
        self.move_cursor_in_command_line(1);
    }

    pub(crate) fn backspace_on_command_line(&mut self) {
        let line = &mut self.command_line.input;
        if self.command_line.cursor.x > 0 && self.command_line.cursor.x <= line.len() as i16 {
            line.remove(self.command_line.cursor.x as usize - 1);
            self.move_cursor_in_command_line(-1);
        }
    }

    //CURSOR

    ///wrapper function to either call move text selection cursor in editor or call to move cursor in editor,
    pub(crate) fn move_all_cursor_editor(&mut self, x: i16, y: i16, shift_held: bool) {
        if shift_held {
            self.move_selection_cursor(x, y);
        } else {
            self.move_cursor_in_editor(x, y);
            self.editor.text_selection_start = None;
            self.editor.text_selection_end = None;
        }
    }

    ///moves logical cursor by x and y, under conditions. and recalculates the visual cursor position
    pub(crate) fn move_cursor_in_editor(&mut self, x: i16, y: i16) {
        self.editor.move_cursor(x, y);

    }

    ///moves selection cursor
    pub(crate) fn move_selection_cursor(&mut self, x: i16, y: i16) {
        self.editor.move_selection_cursor(x, y);
    }


    //IN COMMAND LINE
    ///moves cursor by x and y amounts in commandline
    pub(crate) fn move_cursor_in_command_line(&mut self, x: i16) {
        let max_x_pos: i16 = self.command_line.input.len() as i16;
        self.command_line.cursor.x = (self.command_line.cursor.x + x).clamp(0, max_x_pos);
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
                        let popup = Box::new(ErrorPopup::new("Failed to save file", AppError::InternalError(e.to_string())));
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

    ///copies text within bound of text selected to copied_text
    pub(crate) fn copy_selected_text(&mut self) -> Result<(),AppError> {
        match self.editor.copy_selected_text(){
            Ok(selected_text) => {
                //copy to clipboard
                self.clipboard.copy(&*selected_text);
                //reset text selection
                self.editor.text_selection_start = None;
                self.editor.text_selection_end = None;
                Ok(())
            },
            Err(e) => {
                Err(EditorFailure(e))
            }
        }
    }

    ///cuts text within bound of text selected to copied_text
    pub(crate) fn cut_selected_text(&mut self) -> Result<(),AppError> {
        match self.editor.cut_selected_text(){
            Ok(selected_text) => {
                //copy to clipboard
                self.clipboard.copy(&*selected_text);
                //reset text selection
                self.editor.text_selection_start = None;
                self.editor.text_selection_end = None;
                Ok(())
            },
            Err(e) => {
                Err(EditorFailure(e))
            }
        }

    }

    ///pastes text from copied text to editor content
    pub(crate) fn paste_selected_text(&mut self) -> Result<(), AppError> {
        match self.editor.paste_selected_text(self.clipboard.copied_text.clone()){
            Ok(()) => {
                Ok(())
            },
            Err(e) => {
                Err(EditorFailure(e))
            }
        }
    }

    ///undos last edit action
    pub(crate) fn undo_in_editor(&mut self) -> Result<(),AppError> {
        match self.editor.undo(){
            Ok(()) => {
                Ok(())
            },
            Err(e) => {
                Err(EditorFailure(e))
            }
        }
    }

    ///redos last edit action
    pub(crate) fn redo_in_editor(&mut self) -> Result<(),AppError> {
        match self.editor.redo(){
            Ok(()) => {
                Ok(())
            },
            Err(e) => {
                Err(EditorFailure(e))
            }
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
    use crate::app::*;

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
mod unit_app_command_line_tests {
    use crate::app::App;

    

    fn create_app_with_command_input(s: String) -> App {
        let mut app = App::new();
        app.command_line.input = s;
        app
    }

    //writing chars to command line
    #[test]
    fn test_write_char_to_command_line() {
        let mut app = create_app_with_command_input("".to_string());
        app.write_char_to_command_line('A');

        assert_eq!(app.command_line.input, "A");
        assert_eq!(app.command_line.cursor.x, 1);
    }

    #[test]
    fn test_write_char_to_command_line_mid_input() {
        let mut app = create_app_with_command_input("Test".to_string());
        app.command_line.cursor.x = 2;
        app.write_char_to_command_line('X');

        assert_eq!(app.command_line.input, "TeXst");
        assert_eq!(app.command_line.cursor.x, 3);
    }

    //BACKSPACE in commandline

    #[test]
    fn test_backspace_at_start() {
        let mut app = create_app_with_command_input("".to_string());
        app.command_line.cursor.x = 0;
        app.backspace_on_command_line();

        assert_eq!(app.command_line.input, "");
        assert_eq!(app.command_line.cursor.x, 0);
    }

    #[test]
    fn test_backspace_in_middle() {
        let mut app = create_app_with_command_input("Test".to_string());
        app.command_line.cursor.x = 3;
        app.backspace_on_command_line();

        assert_eq!(app.command_line.input, "Tet");
        assert_eq!(app.command_line.cursor.x, 2);
    }
    
}
#[cfg(test)]
mod unit_app_cutcopy_tests{
    use crate::cursor::CursorPosition;
    use crate::app::*;

    fn create_app_with_editor_content(vec: Vec<String>) -> App {
        let mut app = App::new();
        app.editor.editor_content = vec;
        app.editor.editor_height = 10; //since testing doesnt start ui.rs, height isnt set
        app
    }


    //copy selected text
    #[test]
    fn test_copy_single_line_selection() {
        let mut app = create_app_with_editor_content(vec!["Hello, world!".to_string()]);
        app.editor.text_selection_start = Some(CursorPosition { x: 7, y: 0 });
        app.editor.text_selection_end = Some(CursorPosition { x: 12, y: 0 });

        let result = app.copy_selected_text();

        assert!(result.is_ok());
        assert_eq!(app.clipboard.copied_text, vec!["world".to_string()]);
    }

    #[test]
    fn test_copy_multi_line_selection() {
        let mut app = create_app_with_editor_content(vec![
            "Hello,".to_string(),
            " world!".to_string(),
            " Rust".to_string(),
        ]);
        app.editor.text_selection_start = Some(CursorPosition { x: 4, y: 0 });
        app.editor.text_selection_end = Some(CursorPosition { x: 3, y: 2 });

        let result = app.copy_selected_text();

        assert!(result.is_ok());
        assert_eq!(
            app.clipboard.copied_text,
            vec!["o,", " world!", " Ru"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<String>>()
        );
    }

    #[test]
    fn test_copy_no_selection() {
        let mut app = create_app_with_editor_content(vec!["Hello, world!".to_string()]);
        app.editor.text_selection_start = None;
        app.editor.text_selection_end = None;

        let result = app.copy_selected_text();

        assert!(result.is_err());
        assert!(app.clipboard.copied_text.is_empty());
    }

    //cut selected text
    #[test]
    fn test_cut_single_line_selection() {
        let mut app = create_app_with_editor_content(vec!["Hello, world!".to_string()]);
        app.editor.text_selection_start = Some(CursorPosition { x: 7, y: 0 });
        app.editor.text_selection_end = Some(CursorPosition { x: 12, y: 0 });

        let result = app.cut_selected_text();

        assert!(result.is_ok());
        assert_eq!(app.clipboard.copied_text, vec!["world".to_string()]);
        assert!(app.editor.text_selection_start.is_none());
        assert!(app.editor.text_selection_end.is_none());
    }

    #[test]
    fn test_cut_multi_line_selection() {
        let mut app = create_app_with_editor_content(vec![
            "Hello,".to_string(),
            " world!".to_string(),
            " Rust".to_string(),
        ]);
        app.editor.text_selection_start = Some(CursorPosition { x: 4, y: 0 });
        app.editor.text_selection_end = Some(CursorPosition { x: 3, y: 2 });

        let result = app.cut_selected_text();

        assert!(result.is_ok());
        assert_eq!(
            app.clipboard.copied_text,
            vec!["o,", " world!", " Ru"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<String>>()
        );
        assert!(app.editor.text_selection_start.is_none());
        assert!(app.editor.text_selection_end.is_none());
    }

    #[test]
    fn test_cut_no_selection() {
        let mut app = create_app_with_editor_content(vec!["Hello, world!".to_string()]);
        app.editor.text_selection_start = None;
        app.editor.text_selection_end = None;

        let result = app.cut_selected_text();

        assert!(result.is_err());
        assert!(app.clipboard.copied_text.is_empty());
        assert!(app.editor.text_selection_start.is_none());
        assert!(app.editor.text_selection_end.is_none());
    }

    //paste selected text
    #[test]
    fn test_paste_single_line() {
        let mut app = create_app_with_editor_content(vec![
            "Hello, world!".to_string(),
            "This is a test.".to_string(),
            "Another line.".to_string(),
        ]);
        app.clipboard.copy(&vec!["PASTED".to_string()]);
        app.editor.cursor.x = 8;
        app.editor.cursor.y = 0;

        app.paste_selected_text().unwrap();

        assert_eq!(
            app.editor.editor_content,
            vec![
                "Hello, wPASTEDorld!".to_string(),
                "This is a test.".to_string(),
                "Another line.".to_string(),
            ]
        );
    }

    #[test]
    fn test_paste_multiline() {
        let mut app = create_app_with_editor_content(vec![
            "Hello, world!".to_string(),
            "This is a test.".to_string(),
            "Another line.".to_string(),
        ]);
        app.clipboard
            .copy(&vec!["First".to_string(), "Second ".to_string()]);
        app.editor.cursor.x = 5;
        app.editor.cursor.y = 1;

        app.paste_selected_text().unwrap();

        assert_eq!(
            app.editor.editor_content,
            vec![
                "Hello, world!".to_string(),
                "This First".to_string(),
                "Second is a test.".to_string(),
                "Another line.".to_string(),
            ]
        );
    }

    #[test]
    fn test_paste_single_line_special_characters() {
        let mut app = create_app_with_editor_content(vec![
            "Hello, wᚠᚠᚠᚠorld!".to_string(),
            "This is a test.".to_string(),
            "Another line.".to_string(),
        ]);
        app.clipboard.copy(&vec!["PASTED".to_string()]);
        app.editor.cursor.x = 10;
        app.editor.cursor.y = 0;

        app.paste_selected_text().unwrap();

        assert_eq!(
            app.editor.editor_content,
            vec![
                "Hello, wᚠᚠPASTEDᚠᚠorld!".to_string(),
                "This is a test.".to_string(),
                "Another line.".to_string(),
            ]
        );
    }

    #[test]
    fn test_paste_multiline_special_charaters() {
        let mut app = create_app_with_editor_content(vec![
            "Hello, world!".to_string(),
            "This ᚠᚠᚠᚠis a test.".to_string(),
            "Another line.".to_string(),
        ]);
        app.clipboard
            .copy(&vec!["First".to_string(), "Second ".to_string()]);
        app.editor.cursor.x = 7;
        app.editor.cursor.y = 1;

        app.paste_selected_text().unwrap();

        assert_eq!(
            app.editor.editor_content,
            vec![
                "Hello, world!".to_string(),
                "This ᚠᚠFirst".to_string(),
                "Second ᚠᚠis a test.".to_string(),
                "Another line.".to_string(),
            ]
        );
    }

    #[test]
    fn test_paste_at_start_of_line() {
        let mut app = create_app_with_editor_content(vec![
            "Hello, world!".to_string(),
            "This is a test.".to_string(),
            "Another line.".to_string(),
        ]);
        app.clipboard.copy(&vec!["NewStart".to_string()]);
        app.editor.cursor.x = 0;
        app.editor.cursor.y = 2;

        app.paste_selected_text().unwrap();

        assert_eq!(
            app.editor.editor_content,
            vec![
                "Hello, world!".to_string(),
                "This is a test.".to_string(),
                "NewStartAnother line.".to_string(),
            ]
        );
    }

    #[test]
    fn test_paste_at_end_of_line() {
        let mut app = create_app_with_editor_content(vec![
            "Hello, world!".to_string(),
            "This is a test.".to_string(),
            "Another line.".to_string(),
        ]);
        app.clipboard.copy(&vec!["END".to_string()]);
        app.editor.cursor.x = 13;
        app.editor.cursor.y = 0;

        app.paste_selected_text().unwrap();

        assert_eq!(
            app.editor.editor_content,
            vec![
                "Hello, world!END".to_string(),
                "This is a test.".to_string(),
                "Another line.".to_string(),
            ]
        );
    }

    #[test]
    fn test_paste_with_empty_copied_text() {
        let mut app = create_app_with_editor_content(vec![
            "Hello, world!".to_string(),
            "This is a test.".to_string(),
            "Another line.".to_string(),
        ]);
        app.clipboard.copy(&vec![]);
        app.editor.cursor.x = 5;
        app.editor.cursor.y = 1;


        assert!(app.paste_selected_text().is_err());
        assert_eq!(
            app.editor.editor_content,
            vec![
                "Hello, world!".to_string(),
                "This is a test.".to_string(),
                "Another line.".to_string(),
            ]
        );
    }

    #[test]
    fn test_paste_into_empty_editor() {
        let mut app = create_app_with_editor_content(vec![]);
        app.clipboard
            .copy(&vec!["Hello".to_string(), "World".to_string()]);

        app.paste_selected_text().unwrap();

        assert_eq!(
            app.editor.editor_content,
            vec!["Hello".to_string(), "World".to_string()]
        );
    }
}
