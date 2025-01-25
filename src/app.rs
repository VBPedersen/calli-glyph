use std::{fs};
use std::fs::File;
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
            match fs::read_to_string(&path) {
                Ok(contents) => vec!(contents),
                Err(_err) => { //if file not found create new
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

    ///writes char to y position line, with x position
    pub(crate) fn write_char_to_editor(&mut self, c: char) {
        //creating lines until y position of cursor
        while self.editor_content.len() <= self.cursor_y as usize {
            self.editor_content.push(String::new());
        }

        let line = &mut self.editor_content[self.cursor_y as usize];

        //position cursor to line end
        if line.len() < self.cursor_x as usize {
            self.cursor_x = line.len() as i16;
        }

        line.insert(self.cursor_x as usize, c);
        self.move_cursor_in_editor(1, 0);
    }

    pub(crate) fn write_char_to_command_line(&mut self, c: char) {
        let line = &mut self.command_input;
        if line.len() < self.cursor_x as usize {
            self.cursor_x = line.len() as i16;
        }
        line.insert(self.cursor_x as usize, c);
        self.move_cursor_in_command_line(1);
    }

    pub(crate) fn backpace_on_editor(&mut self) {
        if self.cursor_x > 0 && self.cursor_x <= self.editor_content[self.cursor_y as usize].len() as i16 {
            let line = &mut self.editor_content[self.cursor_y as usize];
            line.remove(self.cursor_x as usize -1);
            self.move_cursor_in_editor(-1, 0);
        } else if self.cursor_y > 0 {
            let line = &mut self.editor_content.remove(self.cursor_y as usize);
            let new_x_value = self.editor_content[(self.cursor_y -1) as usize].len() as i16;
            self.move_cursor_in_editor(new_x_value, -1);
            self.editor_content[self.cursor_y as usize].push_str(&line);
        }
    }

    pub(crate) fn backpace_on_command_line(&mut self) {
        let line = &mut self.command_input;
        if self.cursor_x > 0 && self.cursor_x <= line.len() as i16 {
            line.remove(self.cursor_x as usize -1);
            self.move_cursor_in_command_line(-1);
        }
    }



    //CURSOR
    ///moves cursor by x and y amounts in editor
    pub(crate) fn move_cursor_in_editor(&mut self, x: i16, y: i16) {
        //make more lines if less lines than cursor future y
        while self.editor_content.len() <= (self.cursor_y + y) as usize {
            self.editor_content.push(String::new());
        }

        let max_x_pos:i16 = self.editor_content[(self.cursor_y + y) as usize].len() as i16;

        self.cursor_x = (self.cursor_x + x).clamp(0, max_x_pos);
        self.cursor_y = (self.cursor_y + y).clamp(0, i16::MAX);

        let (top, bottom) = self.is_cursor_top_or_bottom();

        //if on way down and at bottom, move scroll
        if (y == 1 && bottom) || (y == -1 && top) {
            self.scroll_offset = (self.scroll_offset + y).clamp(0, i16::MAX);
            return;
        }
    }

    ///moves cursor by x and y amounts in commandline
    pub(crate) fn move_cursor_in_command_line(&mut self, x: i16) {

        let max_x_pos:i16 = self.command_input.len() as i16;
        self.cursor_x = (self.cursor_x + x).clamp(0, max_x_pos);

    }


    fn is_cursor_top_or_bottom(&self) -> (bool,bool) {
        let top = self.cursor_y == self.scroll_offset;
        let bottom =  self.cursor_y == self.scroll_offset + (self.terminal_height -2);
        (top,bottom)
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
                self.active_area = ActiveArea::CommandLine;
                self.cursor_x = 0;
                self.cursor_y = 0;
            },
            ActiveArea::CommandLine => {
                self.active_area = ActiveArea::Editor;
                self.cursor_x = 0;
                self.cursor_y = 0;
            },

        }
    }


    //Basic Commands

    /// Set running to false, to quit the application.
    pub(crate) fn quit(&mut self) {
        self.running = false;
    }
}
