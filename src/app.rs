use std::{fs};
use std::fs::File;
use std::time::{Instant, Duration};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::{

    DefaultTerminal
};

use crate::ui::{ui};

#[derive(Debug)]
pub struct App {
    /// Is the application running?
    running: bool,
    pub(crate) active_area: ActiveArea,
    pub(crate) editor_content: Vec<String>,
    pub(crate) command_input: String,
    file_path: Option<String>,
    pub(crate) cursor_x: u16,
    pub(crate) cursor_y: u16,
    pub(crate) cursor_visible: bool,
    last_tick: Instant,
    pub(crate) scroll_offset: u16,
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
                Err(err) => { //if file not found create new
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
            self.command_input.push('x');
        }


        while self.running {
            terminal.draw(|frame| ui(frame, &self))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }



    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(mouse) if (mouse.kind == MouseEventKind::ScrollDown) |
                (mouse.kind == MouseEventKind::ScrollUp) => {self.on_scroll_events(mouse)}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    fn on_scroll_events(&mut self, mouse: MouseEvent) {
        match self.active_area {
            ActiveArea::Editor => {
                match mouse.kind {
                    MouseEventKind::ScrollDown => { self.scroll_offset += 1},
                    MouseEventKind::ScrollUp => {
                        if self.scroll_offset != 0 {
                            self.scroll_offset -= 1;
                        }
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match self.active_area {
            ActiveArea::Editor => match (key.modifiers, key.code) {
                (_, KeyCode::Esc) | (KeyModifiers::SHIFT, KeyCode::Char(':')) => self.toggle_active_area(),
                (_, KeyCode::Char(c)) => {
                    self.editor_content.push(String::from(c));
                    self.cursor_x += 1;
                },
                (_, KeyCode::Backspace) => {
                    self.editor_content.pop();
                    self.cursor_x -= 1;
                },
                _ => {}
            },
            ActiveArea::CommandLine => match (key.modifiers, key.code) {
                (_, KeyCode::Tab | KeyCode::Esc) => self.toggle_active_area(),
                (KeyModifiers::CONTROL, KeyCode::Char('c')) => self.quit(),
                (_, KeyCode::Char(c)) => {
                    self.command_input.push(c);
                    self.cursor_x += 1;
                },
                (_, KeyCode::Backspace) => {
                    self.command_input.pop();
                    self.cursor_x -= 1;
                },
                _ => {}
            },
        }

    }

    fn toggle_active_area(&mut self) {
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

    /// Set running to false, to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}
