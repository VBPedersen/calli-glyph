use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::time::{Instant, Duration};
use color_eyre::Result;
use ratatui::{

    DefaultTerminal
};
use crate::clipboard::Clipboard;
use crate::command_line::CommandLine;
use crate::ui::{ui};
use crate::input::{handle_input};
use crate::config::editor_settings;
use crate::confirmation_popup::ConfirmationPopup;
use crate::cursor::CursorPosition;
use crate::editor::Editor;
use crate::popup::{Popup, PopupResult};

#[derive(Debug)]
pub struct App {
    /// Is the application running?
    running: bool,
    pub(crate) active_area: ActiveArea,
    pub editor: Editor,
    pub command_line: CommandLine,
    pub(crate) cursor_visible: bool,
    last_tick: Instant,
    pub(crate) scroll_offset: i16,
    pub(crate) terminal_height: i16,
    pub clipboard: Clipboard,
    pub file_path: Option<String>,
    pub popup: Option<Box<dyn Popup>>,
    pub popup_result: PopupResult,
    pub pending_states: Vec<PendingState>
}

#[derive(Debug,PartialEq)]
pub enum PendingState{
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
            scroll_offset: 0,
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
                        Err(err) => { //if file not found create new
                            self.running = false;
                            panic!(
                                "Failed to create file '{}': {}",
                                path, err
                            );
                        }
                    }
                },
                Err(_err) => {
                    match File::create(path) { //create file, if ok then return else quit and panic
                        Ok(_) => {
                            vec!(String::new()) // Return an empty string as the content
                        }
                        Err(create_err) => {
                            self.running = false;
                            panic!(
                                "Failed to create file '{}': {}",
                                path, create_err
                            );
                        }
                    }
                }
            }
        } else {
            vec!(String::new()) // Start with an empty editor if no file is provided
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
        if self.editor.text_selection_start.is_some() && self.editor.text_selection_end.is_some() {
            true
        } else {
            false
        }
    }


        //IN EDITOR
    ///wrapper function to either call write char with selected text or function write char,
    /// where text isn't selected
    pub(crate) fn write_all_char_in_editor(&mut self, c: char){
        if self.is_text_selected() {
            self.write_char_in_editor_text_is_selected(c)
        } else {
            self.write_char_in_editor(c)
        }
    }

    ///replaces all selected text with char to y position line, with x position
    fn write_char_in_editor_text_is_selected(&mut self, c: char){
        let start =self.editor.text_selection_start.clone().unwrap();
        let end =self.editor.text_selection_end.clone().unwrap();
        let lines = &mut self.editor.editor_content[start.y..=end.y];
        let lines_length = lines.len().clone();
        if lines_length > 1 {
            for (y,line) in lines.iter_mut().enumerate() {
                let mut line_chars_vec:Vec<char> = line.chars().collect();

                //last line selected
                if y == lines_length -1 {
                    line_chars_vec.drain(0..end.x);
                } else {
                    line_chars_vec.drain(start.x..line.chars().count());
                }

                //start line selected
                if y == 0 {
                    line_chars_vec.insert(start.x,c);
                }

                *line = line_chars_vec.into_iter().collect();
            }
        } else {
            let line = &mut self.editor.editor_content[start.y];
            let mut line_chars_vec:Vec<char> = line.chars().collect();
            line_chars_vec.drain(start.x..end.x);
            line_chars_vec.insert(start.x,c);
            *line = line_chars_vec.into_iter().collect();
        }
        self.editor.cursor.x = self.editor.text_selection_start.unwrap().x as i16;
        self.editor.cursor.y = self.editor.text_selection_start.unwrap().y as i16;
        self.editor.text_selection_start = None;
        self.editor.text_selection_end = None;
        self.move_cursor_in_editor(1, 0);
    }

    ///writes char to y position line, with x position
    pub(crate) fn write_char_in_editor(&mut self, c: char) {
        //creating lines until y position of cursor
        while self.editor.editor_content.len() <= self.editor.cursor.y as usize {
            self.editor.editor_content.push(String::new());
        }

        let line = &mut self.editor.editor_content[self.editor.cursor.y as usize];

        let char_count = line.chars().count();
        //position cursor to line end in chars count
        if char_count < self.editor.cursor.x as usize {
            self.editor.cursor.x = char_count as i16;
        }

        let mut line_chars_vec:Vec<char> = line.chars().collect();

        line_chars_vec.insert(self.editor.cursor.x as usize, c);

        *line = line_chars_vec.into_iter().collect();

        self.move_cursor_in_editor(1, 0);
    }

    ///wrapper function to either call backspace in editor with selected text or function backspace_in_editor,
    /// where text isn't selected
    pub(crate) fn backspace_all_in_editor(&mut self){
        if self.is_text_selected() {
            self.backspace_in_editor_text_is_selected();
        } else {
            self.backspace_in_editor();
        }
    }

    ///handles backspace in editor, removes char at y line x position and sets new cursor position
    pub(crate) fn backspace_in_editor_text_is_selected(&mut self) {
        let start =self.editor.text_selection_start.clone().unwrap();
        let end =self.editor.text_selection_end.clone().unwrap();
        let lines = &mut self.editor.editor_content[start.y..=end.y];
        let lines_length = lines.len().clone();
        if lines_length > 1 {
            for (y,line) in lines.iter_mut().enumerate() {
                let mut line_chars_vec:Vec<char> = line.chars().collect();
                //last line selected
                if y == lines_length -1 {
                    line_chars_vec.drain(0..end.x);
                } else {
                    line_chars_vec.drain(start.x..line.chars().count());
                }

                *line = line_chars_vec.into_iter().collect();
            }
        } else {
            let line = &mut self.editor.editor_content[start.y];
            let mut line_chars_vec:Vec<char> = line.chars().collect();
            line_chars_vec.drain(start.x..end.x);
            *line = line_chars_vec.into_iter().collect();
        }
        self.editor.cursor.x = self.editor.text_selection_start.unwrap().x as i16;
        self.editor.cursor.y = self.editor.text_selection_start.unwrap().y as i16;
        self.editor.text_selection_start = None;
        self.editor.text_selection_end = None;
        //replace visual cursor
        self.editor.visual_cursor_x = self.calculate_visual_x() as i16;
    }

    ///handles backspace in editor, removes char at y line x position and sets new cursor position
    pub(crate) fn backspace_in_editor(&mut self) {
        let line_char_count = self.editor.editor_content[self.editor.cursor.y as usize].chars().count() as i16;
        if self.editor.cursor.x > 0 && self.editor.cursor.x <= line_char_count {
            let line = &mut self.editor.editor_content[self.editor.cursor.y as usize];
            let mut line_chars_vec:Vec<char> = line.chars().collect();

            line_chars_vec.remove(self.editor.cursor.x as usize -1);

            *line = line_chars_vec.into_iter().collect();
            //line.remove(self.editor.cursor.x as usize -1);
            self.move_cursor_in_editor(-1, 0);
        } else if self.editor.cursor.y > 0 {
            let line = &mut self.editor.editor_content.remove(self.editor.cursor.y as usize);
            let new_x_value = self.editor.editor_content[(self.editor.cursor.y -1) as usize].chars().count() as i16;
            self.editor.cursor.y -= 1;
            self.editor.cursor.x = new_x_value;
            self.editor.editor_content[self.editor.cursor.y as usize].push_str(&line);
        }
    }


    ///wrapper function to either call backspace in editor with selected text or function backspace_in_editor,
    /// where text isn't selected
    pub(crate) fn delete_all_in_editor(&mut self){
        if self.is_text_selected() {
            self.delete_in_editor_text_is_selected();
        } else {
            self.delete_in_editor();
        }
    }

    ///handles delete in editor, removes char at y line x position and sets new cursor position
    pub(crate) fn delete_in_editor_text_is_selected(&mut self) {
        let start =self.editor.text_selection_start.clone().unwrap();
        let end =self.editor.text_selection_end.clone().unwrap();
        let lines = &mut self.editor.editor_content[start.y..=end.y];
        let lines_length = lines.len().clone();
        if lines_length > 1 {
            for (y,line) in lines.iter_mut().enumerate() {
                let mut line_chars_vec:Vec<char> = line.chars().collect();

                //last line selected
                if y == lines_length -1 {
                    //line_chars_vec.drain(0..end.x);
                    line_chars_vec[0..end.x].fill(' ');
                } else {
                    //line_chars_vec[start.x..line.chars().count()].fill(' ');
                    line_chars_vec.drain(start.x..line.chars().count());
                }

                *line = line_chars_vec.into_iter().collect();
            }
        } else {
            let line = &mut self.editor.editor_content[start.y];
            let mut line_chars_vec:Vec<char> = line.chars().collect();
            line_chars_vec[start.x..end.x].fill(' ');
            //line_chars_vec.drain(start.x..end.x);
            *line = line_chars_vec.into_iter().collect();
        }
        self.editor.cursor.x = self.editor.text_selection_end.unwrap().x as i16;
        self.editor.cursor.y = self.editor.text_selection_end.unwrap().y as i16;
        self.editor.text_selection_start = None;
        self.editor.text_selection_end = None;
        //replace visual cursor
        self.editor.visual_cursor_x = self.calculate_visual_x() as i16;
    }

    ///handles DELETE action, of deleting char in editor at x +1 position
    pub(crate) fn delete_in_editor(&mut self) {
        let current_line_len = self.editor.editor_content[self.editor.cursor.y as usize].chars().count() as i16;

        if current_line_len == 0 { return; }
        //if at line end, move line below up,  else if current line length is bigger than current cursor x pos, remove char
        if self.editor.cursor.x >= current_line_len -1 && self.editor.editor_content.len() > (self.editor.cursor.y +1) as usize {
            let line = &mut self.editor.editor_content.remove((self.editor.cursor.y +1) as usize);
            self.editor.editor_content[self.editor.cursor.y as usize].push_str(&line);
        } else if current_line_len > (self.editor.cursor.x+1) {
            let line = &mut self.editor.editor_content[self.editor.cursor.y as usize];
            let mut line_chars_vec:Vec<char> = line.chars().collect();

            line_chars_vec.remove(self.editor.cursor.x as usize +1);

            *line = line_chars_vec.into_iter().collect();
            //line.remove((self.editor.cursor.x+1) as usize);
        }
    }

    ///handles TAB action in editor, by writing \t to editor content.
    pub(crate) fn tab_in_editor(&mut self) {

        let line = &mut self.editor.editor_content[self.editor.cursor.y as usize];

        let mut line_chars_vec:Vec<char> = line.chars().collect();

        line_chars_vec.insert(self.editor.cursor.x as usize, '\t');

        *line = line_chars_vec.into_iter().collect();

        self.move_cursor_in_editor(1,0)
    }

    ///handles enter new line, with possible move of text
    pub(crate) fn enter_in_editor(&mut self) {
        let line = &mut self.editor.editor_content[self.editor.cursor.y as usize];
        //if at end of line len, then just move cursor and make new line, else move text too
        if self.editor.cursor.x >= line.chars().count() as i16 {
            self.editor.editor_content.insert(self.editor.cursor.y as usize +1,String::new());
            self.move_cursor_in_editor(0,1);
        } else {
            //split current line and remove split part
            let mut line_chars_vec:Vec<char> = line.chars().collect();
            let line_end = line_chars_vec.split_off(self.editor.cursor.x as usize);
            *line = line_chars_vec.into_iter().collect();

            //move down and insert split line to next line
            self.move_cursor_in_editor(0,1);
            self.editor.editor_content.insert(self.editor.cursor.y as usize,String::new());
            self.editor.editor_content[self.editor.cursor.y as usize] = line_end.into_iter().collect();
            self.editor.cursor.x = 0;

        }
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
            line.remove(self.command_line.cursor.x as usize -1);
            self.move_cursor_in_command_line(-1);
        }
    }


    //CURSOR
        //IN EDITOR
    ///calculates the visual position of the cursor
    fn calculate_visual_x(&mut self) -> usize {
        let line = &self.editor.editor_content[self.editor.cursor.y as usize];
        let cursor_x = self.editor.cursor.x as usize;
        let tab_width = editor_settings::TAB_WIDTH as usize;
        let mut visual_x = 0;
        for (i, c) in line.chars().enumerate() {
            if i == cursor_x {
                break;
            }

            if c == '\t' {
                visual_x += tab_width - (visual_x % tab_width);
            } else {
                visual_x += 1;
            }

        }


        visual_x
    }

    ///wrapper function to either call move text selection cursor in editor or call to move cursor in editor,
    pub(crate) fn move_all_cursor_editor(&mut self, x: i16, y: i16, shift_held:bool) {

        if shift_held {
            self.move_selection_cursor(x,y);
        }else {
            self.move_cursor_in_editor(x,y);
            self.editor.text_selection_start = None;
            self.editor.text_selection_end = None;
        }

    }


    ///moves logical cursor by x and y, under conditions. and recalculates the visual cursor position
    pub(crate) fn move_cursor_in_editor(&mut self, x: i16, y: i16) {
        if self.editor.cursor.y == 0 && y == -1 {
            return;
        }
        //if wanting to go beyond current length of editor
        while self.editor.editor_content.len() <= (self.editor.cursor.y + y) as usize {
            self.editor.editor_content.push(String::new());
        }

        let max_x_pos = self.editor.editor_content[(self.editor.cursor.y + y) as usize].chars().count() as i16;
        //let current_line = &self.editor.editor_content[self.editor.cursor.y as usize];

        // Moving Right →
        if x > 0 && self.editor.cursor.x < max_x_pos {
            self.editor.cursor.x += x;
        }else if  x == 1 && self.editor.cursor.x >= self.editor.editor_content[self.editor.cursor.y as usize].chars().count() as i16
            && self.editor.editor_content.len() > self.editor.cursor.y as usize +1{ //else if end of line and more lines
            self.editor.cursor.y += 1;
            self.editor.cursor.x = 0;
            self.editor.visual_cursor_x = self.calculate_visual_x() as i16;
            return;
        }

        // Moving Left ←
        if x < 0 && self.editor.cursor.x > 0 {
            self.editor.cursor.x += x;
        } else if self.editor.cursor.x == 0 && x == -1 && self.editor.cursor.y != 0 { //else if start of line and more lines
            self.editor.cursor.y -= 1;
            self.editor.cursor.x = self.editor.editor_content[self.editor.cursor.y as usize].chars().count() as i16;
            self.editor.visual_cursor_x = self.calculate_visual_x() as i16;
            return;
        }


        let (top, bottom) = self.is_cursor_top_or_bottom();
        //to offset scroll
        if (y == 1 && bottom) || (y == -1 && top) {
            self.scroll_offset = (self.scroll_offset + y).clamp(0, i16::MAX);
            return;
        }

        self.editor.cursor.x = self.editor.cursor.x.clamp(0, max_x_pos);
        self.editor.cursor.y = (self.editor.cursor.y + y).clamp(0, i16::MAX);
        self.editor.visual_cursor_x = self.calculate_visual_x() as i16;
    }


    ///checks if cursor is at top or bottom of the screen
    fn is_cursor_top_or_bottom(&self) -> (bool,bool) {
        let top = self.editor.cursor.y == self.scroll_offset;
        let bottom =  self.editor.cursor.y == self.scroll_offset + (self.terminal_height -2);
        (top,bottom)
    }

    ///moves selection cursor
    pub(crate) fn move_selection_cursor(&mut self, x: i16, y: i16) {
        let old_x = self.editor.cursor.x.clone();
        let old_y = self.editor.cursor.y.clone();
        self.move_cursor_in_editor(x,y);
        let new_x = self.editor.cursor.x.clone();
        let new_y = self.editor.cursor.y.clone();

        let mut start_cp = CursorPosition::default();
        let mut end_cp = CursorPosition::default();
        if x > 0 || y > 0 {
            start_cp = CursorPosition{ x: old_x as usize, y: old_y as usize };
            end_cp = CursorPosition{ x: new_x as usize, y: new_y as usize };

            if self.editor.text_selection_start.is_none() {
                self.editor.text_selection_start = Option::from(start_cp);
            }
            self.editor.text_selection_end = Option::from(end_cp);
        }

        if x < 0 || y < 0 {
            start_cp = CursorPosition{ x: new_x as usize, y: new_y as usize };
            end_cp = CursorPosition{ x: old_x as usize, y: old_y as usize };
            self.editor.text_selection_start = Option::from(start_cp);
            if self.editor.text_selection_end.is_none() {
                self.editor.text_selection_end = Option::from(end_cp);
            }
        }

    }

        //IN COMMAND LINE
    ///moves cursor by x and y amounts in commandline
    pub(crate) fn move_cursor_in_command_line(&mut self, x: i16) {
        let max_x_pos:i16 = self.command_line.input.len() as i16;
        self.command_line.cursor.x = (self.command_line.cursor.x + x).clamp(0, max_x_pos);

    }


    //SCROLL
    ///moves the scroll offset
    pub(crate) fn move_scroll_offset(&mut self, offset: i16) {
        let (top, bottom) = self.is_cursor_top_or_bottom();

        //if on way down and at bottom, move scroll
        if (offset == 1 && bottom) || (offset == -1 && top) {
            self.scroll_offset = (self.scroll_offset + offset).clamp(0, i16::MAX);
            return;
        }

        self.move_cursor_in_editor(0, offset);
    }





    //PANEL HANDLING
    ///toggles the active area of the app, between editor and command line
    pub(crate) fn toggle_active_area(&mut self) {
        match self.active_area {
            ActiveArea::Editor =>  {
                self.active_area = ActiveArea::CommandLine;
            },
            ActiveArea::CommandLine => {
                self.active_area = ActiveArea::Editor;
            },

            _ => {}
        }
    }

    ///handles creating popup to confirm if file should be overridden
    pub fn handle_confirmation_popup_response(&mut self) {
        //get first state in vec, match the state and if needed checks next state after that
        if self.pending_states.is_empty(){
            return;
        }

        let state = self.pending_states.first().unwrap();
        match state {
            PendingState::Saving(save_path) => {
                if self.popup_result == PopupResult::Bool(true) {
                    self.save(vec![save_path.clone()]);
                    self.popup_result = PopupResult::None;
                    self.close_popup();
                    self.pending_states.remove(0);
                    //if next state is quit, then quit
                    if !self.pending_states.is_empty() &&
                        self.pending_states[0] == PendingState::Quitting {
                        self.quit()
                    }

                } else if self.popup_result == PopupResult::Bool(false) {
                    self.popup_result = PopupResult::None;
                    self.close_popup();
                }
            },
            PendingState::Quitting => self.quit(),
            _ => {}
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
    pub(crate) fn save(&mut self, args:Vec<String>) -> Result<()> {

        let path;
        let mut path_is_current_file:bool = false;
        let has_changes:bool;
        let mut force_flag:bool = false;

        let new_content = self.editor.editor_content.join("\n");

        //if file path to save on is set in command args
        if !args.is_empty() {
            path = args.get(0).unwrap().clone();
            force_flag = args.contains(&"--force".to_string());
        } else if self.file_path.is_some(){
            path = self.file_path.clone().unwrap();
            path_is_current_file = true;
        } else {
            path = "untitled".to_string();

        }

        let path_ref = Path::new(&path);

        // Check if file exists
        if path_ref.exists() {
            has_changes = self.file_has_changes(new_content.clone(),path.clone())?;
            //if path is the current file, has changes and force is false
            // and no confirmation has been asked, then make user confirm
            if !path_is_current_file && has_changes  && !force_flag &&  self.popup_result == PopupResult::None{
                let popup = Box::new(ConfirmationPopup::new("Confirm Overwrite of file"));
                self.open_popup(popup);
                self.pending_states.push(PendingState::Saving(path));
                return Ok(());
            }

        } else {
            has_changes = new_content.len() > 0;
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
        }else {
            Ok(())
        }
    }


    ///saves file and exits window
    pub(crate) fn save_and_exit(&mut self, args:Vec<String>) -> Result<()> {
        match self.save(args) {
            Ok(_) => {
                // If a save confirmation is needed, push Quit AFTER Saving
                if self.pending_states.iter().any(|s| matches!(s, PendingState::Saving(_))) {

                    self.pending_states.push(PendingState::Quitting);  // Add Quit to the queue
                    return Ok(());
                }
                self.quit();

                Ok(())
            },
            Err(e) => Err(e),
        }
    }

    ///checks if file has changes and returns boolean
    pub(crate) fn file_has_changes(&self,editor_content:String,file_path:String) -> Result<bool> {

        let file = File::open(file_path)?;
        let mut buff_read_file = BufReader::new(file);
        let mut read_file_contents = String::new();

        buff_read_file.read_to_string(&mut read_file_contents).expect("TODO: panic message");
        //if has changes, return true else return false
        if !read_file_contents.eq(&editor_content) {
            Ok(true)
        } else {
            Ok(false)
        }
    }
    

    ///copies text within bound of text selected to copied_text
    pub(crate) fn copy_selected_text(&mut self) -> Result<()> {
        if let (Some(start), Some(end)) = (self.editor.text_selection_start.clone(), self.editor.text_selection_end.clone()) {
            let mut selected_text: Vec<String> = Vec::new();
            let lines = &self.editor.editor_content[start.y..=end.y];

            if lines.len() > 1 {
                for (y, line) in lines.iter().enumerate() {
                    let mut line_chars: Vec<char> = line.chars().collect();
                    let extracted_text: String;
                    
                    //if first line, else if last line, else 
                    if y == 0 {
                        extracted_text = line_chars.drain(start.x..).collect();
                    } else if y == lines.len() - 1 {
                        extracted_text = line_chars.drain(..end.x).collect();
                    } else {
                        extracted_text = line_chars.into_iter().collect();
                    }

                    selected_text.push(extracted_text);
                }
            } else {
                let mut line_chars: Vec<char> = self.editor.editor_content[start.y].chars().collect();
                let extracted_text: String = line_chars.drain(start.x..end.x).collect();
                selected_text.push(extracted_text);
            }

            self.clipboard.copy(&selected_text.clone());
            Ok(())
        } else {
            Ok(())
        }
    }

    ///pastes text from copied text to editor content
    pub(crate) fn paste_selected_text(&mut self) -> Result<()> {
        //if no text in copied text
        if self.clipboard.copied_text.is_empty() {
            return Ok(());
        }

        let insert_y = self.editor.cursor.y as usize;
        let insert_x = self.editor.cursor.x as usize;


        while self.editor.editor_content.len() < insert_y + self.clipboard.copied_text.len() -1 {
            self.editor.editor_content.push(String::new());
        }

        let current_line = &self.editor.editor_content[insert_y];

        // Convert the line into a Vec<char> to handle multi-byte characters correctly
        let chars: Vec<char> = current_line.chars().collect();
        let (before_cursor, after_cursor) = chars.split_at(insert_x.min(chars.len()));

        if self.clipboard.copied_text.len() == 1 {
            // Single-line paste: correctly insert into character-safe split
            let new_line = format!(
                "{}{}{}",
                before_cursor.iter().collect::<String>(),
                self.clipboard.copied_text[0],
                after_cursor.iter().collect::<String>()
            );
            self.editor.editor_content[insert_y] = new_line;
        } else {
            // Multi-line paste
            let mut new_lines = Vec::new();

            // First line: insert copied text at cursor position
            new_lines.push(format!(
                "{}{}",
                before_cursor.iter().collect::<String>(),
                self.clipboard.copied_text[0]
            ));

            // Middle lines: insert as separate lines
            for line in &self.clipboard.copied_text[1..self.clipboard.copied_text.len() - 1] {
                new_lines.push(line.clone());
            }

            // Last copied line + remainder of the original line
            let last_copied_line = &self.clipboard.copied_text[self.clipboard.copied_text.len() - 1];
            new_lines.push(format!(
                "{}{}",
                last_copied_line,
                after_cursor.iter().collect::<String>()
            ));

            // Replace the current line and insert new lines
            self.editor.editor_content.splice(insert_y..=insert_y, new_lines);
        }

        // Clear copied text after pasting
        //self.copied_text.clear();
        Ok(())

    }




    //HELPER FUNCTIONS FOR BASIC COMMANDS


}
