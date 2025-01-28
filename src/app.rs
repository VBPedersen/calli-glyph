use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::time::{Instant, Duration};
use color_eyre::Result;
use ratatui::{

    DefaultTerminal
};



use crate::ui::{ui};
use crate::input::{handle_input};


#[derive(Debug)]
pub struct App {
    /// Is the application running?
    running: bool,
    pub(crate) active_area: ActiveArea,
    pub(crate) editor_content: Vec<String>,
    pub(crate) command_input: String,
    pub(crate) file_path: Option<String>,
    pub(crate) cursor_x: i16,
    pub(crate) cursor_y: i16,
    pub(crate) editor_cursor_x: i16, //to save position in editor, when toggling area
    pub(crate) editor_cursor_y: i16, //to save position in editor, when toggling are
    pub(crate) cursor_visible: bool,
    last_tick: Instant,
    pub(crate) scroll_offset: i16,
    pub(crate) terminal_height: i16,
}

#[derive(PartialEq, Debug, Default)]
pub(crate) enum ActiveArea {
    #[default]
    Editor,
    CommandLine,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: Default::default(),
            active_area: Default::default(),
            editor_content: vec!(String::new()),
            command_input: String::new(),
            file_path: None,
            cursor_x: 0,
            cursor_y: 0,
            editor_cursor_x: 0,
            editor_cursor_y: 0,
            last_tick: Instant::now(),
            cursor_visible: true,
            scroll_offset: 0,
            terminal_height: 0,
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
        self.editor_content = if let Some(ref path) = self.file_path {
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

        //IN EDITOR

    ///writes char to y position line, with x position
    pub(crate) fn write_char_in_editor(&mut self, c: char) {
        //creating lines until y position of cursor
        while self.editor_content.len() <= self.cursor_y as usize {
            self.editor_content.push(String::new());
        }

        let line = &mut self.editor_content[self.cursor_y as usize];

        let char_count = line.chars().count();
        //position cursor to line end in chars count
        if char_count < self.cursor_x as usize {
            self.cursor_x = char_count as i16;
        }

        let mut line_chars_vec:Vec<char> = line.chars().collect();

        line_chars_vec.insert(self.cursor_x as usize, c);

        *line = line_chars_vec.into_iter().collect();
        //line.insert(self.cursor_x as usize, c);
        self.move_cursor_in_editor(1, 0);
    }

    ///handles backspace in editor, removes char at y line x position and sets new cursor position
    pub(crate) fn backspace_in_editor(&mut self) {
        let line_char_count = self.editor_content[self.cursor_y as usize].chars().count() as i16;
        if self.cursor_x > 0 && self.cursor_x <= line_char_count {
            let line = &mut self.editor_content[self.cursor_y as usize];
            let mut line_chars_vec:Vec<char> = line.chars().collect();

            line_chars_vec.remove(self.cursor_x as usize -1);

            *line = line_chars_vec.into_iter().collect();
            //line.remove(self.cursor_x as usize -1);
            self.move_cursor_in_editor(-1, 0);
        } else if self.cursor_y > 0 {
            let line = &mut self.editor_content.remove(self.cursor_y as usize);
            let new_x_value = self.editor_content[(self.cursor_y -1) as usize].chars().count() as i16;
            self.move_cursor_in_editor(0, -1);
            self.cursor_x = new_x_value;
            self.editor_content[self.cursor_y as usize].push_str(&line);
        }
    }

    ///handles DELETE action, of deleting char in editor at x +1 position
    pub(crate) fn delete_in_editor(&mut self) {
        let current_line_len = self.editor_content[self.cursor_y as usize].chars().count() as i16;

        if current_line_len == 0 { return; }
        //if at line end, move line below up,  else if current line length is bigger than current cursor x pos, remove char
        if self.cursor_x >= current_line_len -1 && self.editor_content.len() > (self.cursor_y +1) as usize {
            let line = &mut self.editor_content.remove((self.cursor_y +1) as usize);
            self.editor_content[self.cursor_y as usize].push_str(&line);
        } else if current_line_len > (self.cursor_x+1) {
            let line = &mut self.editor_content[self.cursor_y as usize];
            let mut line_chars_vec:Vec<char> = line.chars().collect();

            line_chars_vec.remove(self.cursor_x as usize +1);

            *line = line_chars_vec.into_iter().collect();
            //line.remove((self.cursor_x+1) as usize);
        }
    }

    pub(crate) fn tab_in_editor(&mut self) {
        //TODO
    }

    ///handles enter new line, with possible move of text
    pub(crate) fn enter_in_editor(&mut self) {
        let line = &mut self.editor_content[self.cursor_y as usize];
        //if at end of line len, then just move cursor and make new line, else move text too
        if self.cursor_x >= line.chars().count() as i16 {
            self.editor_content.insert(self.cursor_y as usize +1,String::new());
            self.move_cursor_in_editor(0,1);
        } else {
            //split current line and remove split part
            let mut line_chars_vec:Vec<char> = line.chars().collect();
            let line_end = line_chars_vec.split_off(self.cursor_x as usize);
            *line = line_chars_vec.into_iter().collect();

            //move down and insert split line to next line
            self.move_cursor_in_editor(0,1);
            self.editor_content.insert(self.cursor_y as usize,String::new());
            self.editor_content[self.cursor_y as usize] = line_end.into_iter().collect();
            self.cursor_x = 0;

        }
    }

        //IN COMMANDLINE

    /// writes char to the commandline content at x position, and moves cursor
    pub(crate) fn write_char_to_command_line(&mut self, c: char) {
        let line = &mut self.command_input;
        if line.len() < self.cursor_x as usize {
            self.cursor_x = line.len() as i16;
        }
        line.insert(self.cursor_x as usize, c);
        self.move_cursor_in_command_line(1);
    }

    pub(crate) fn backspace_on_command_line(&mut self) {
        let line = &mut self.command_input;
        if self.cursor_x > 0 && self.cursor_x <= line.len() as i16 {
            line.remove(self.cursor_x as usize -1);
            self.move_cursor_in_command_line(-1);
        }
    }


    //CURSOR

        //IN EDITOR
    ///moves cursor by x and y amounts in editor
    pub(crate) fn move_cursor_in_editor(&mut self, x: i16, y: i16) {
        if self.cursor_y == 0 && y == -1 { return; }
        //make more lines if less lines than cursor future y
        while self.editor_content.len() <= (self.cursor_y + y) as usize {
            self.editor_content.push(String::new());
        }

        //if at end of line x position, and moving right, then move to next line at 0 x
        if  x == 1 && self.cursor_x >= self.editor_content[self.cursor_y as usize].chars().count() as i16
              && self.editor_content.len() > self.cursor_y as usize +1{
            self.cursor_y = (self.cursor_y + 1).clamp(0, i16::MAX);
            self.cursor_x = 0;
            return;
        }
        //if at start of line x position, and moving left, then move to previous line at max x
        if self.cursor_x == 0 && x == -1 && self.cursor_y != 0 {
            self.cursor_y = (self.cursor_y + x).clamp(0, i16::MAX);
            self.cursor_x = self.editor_content[self.cursor_y as usize].chars().count() as i16;
            return;
        }


        let max_x_pos:i16 = self.editor_content[(self.cursor_y + y) as usize].chars().count() as i16;

        self.cursor_x = (self.cursor_x + x).clamp(0, max_x_pos);
        self.cursor_y = (self.cursor_y + y).clamp(0, i16::MAX);

        let (top, bottom) = self.is_cursor_top_or_bottom();

        //if on way down and at bottom, move scroll
        if (y == 1 && bottom) || (y == -1 && top) {
            self.scroll_offset = (self.scroll_offset + y).clamp(0, i16::MAX);
            return;
        }
    }


    fn is_cursor_top_or_bottom(&self) -> (bool,bool) {
        let top = self.cursor_y == self.scroll_offset;
        let bottom =  self.cursor_y == self.scroll_offset + (self.terminal_height -2);
        (top,bottom)
    }

        //IN COMMAND LINE
    ///moves cursor by x and y amounts in commandline
    pub(crate) fn move_cursor_in_command_line(&mut self, x: i16) {

        let max_x_pos:i16 = self.command_input.len() as i16;
        self.cursor_x = (self.cursor_x + x).clamp(0, max_x_pos);

    }




    //SCROLL

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

    pub(crate) fn toggle_active_area(&mut self) {
        match self.active_area {
            ActiveArea::Editor =>  {
                self.editor_cursor_x = self.cursor_x;
                self.editor_cursor_y = self.cursor_y;
                self.active_area = ActiveArea::CommandLine;
                self.cursor_x = 0;
                self.cursor_y = 0;
            },
            ActiveArea::CommandLine => {
                self.active_area = ActiveArea::Editor;
                self.cursor_x = self.editor_cursor_x;
                self.cursor_y = self.editor_cursor_y;
            },

        }
    }


    //Basic Commands

    /// Set running == false, to quit the application.
    pub(crate) fn quit(&mut self) {
        self.running = false;
    }

    ///saves contents to file
    pub(crate) fn save(&self) -> Result<()> {
        let path;
        let has_changes:bool;

        let new_content = self.editor_content.join("\n");
        if self.file_path.is_some() {
            path = self.file_path.clone().unwrap();
            has_changes = self.file_has_changes(new_content.clone(),path.clone())?;
        }else {
            path = "untitled".to_string();
            has_changes = new_content.len() > 0;
        }

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


    //HELPER FUNCTIONS FOR BASIC COMMANDS
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

}
