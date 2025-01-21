use std::{fs, process};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{

    DefaultTerminal
};

use crate::ui;
use crate::ui::{ui};

#[derive(Debug, Default)]
pub struct App {
    /// Is the application running?
    running: bool,
    active_area: ActiveArea,
    pub(crate) editor_content: String,
    command_input: String,
    file_path: Option<String>

}

#[derive(PartialEq, Debug, Default)]
enum ActiveArea {
    #[default]
    Editor,
    CommandLine,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal, file_path: Option<String>) -> Result<()> {
        self.running = true;
        self.active_area = ActiveArea::Editor;
        self.file_path = file_path;



        // Read file contents if a file path is provided
        self.editor_content = if let Some(ref path) = self.file_path {
            match fs::read_to_string(&path) {
                Ok(contents) => contents,
                Err(err) => {
                    eprintln!("Failed to read file '{}': {}", path, err);
                    process::exit(1); // Exit with an error if the file can't be read
                }
            }
        } else {
            String::new() // Start with an empty editor if no file is provided
        };


        while self.running {
            terminal.draw(|frame| ui(frame, &self))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }



    /*
    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/master/examples>
    fn draw(&mut self, frame: &mut Frame) {
        /*let title = Line::from("Ratatui Simple Template")
            .bold()
            .blue()
            .centered();*/
        /*let text = "Hello, Ratatui!\n\n\
            Created using https://github.com/ratatui/templates\n\
            Press `Esc`, `Ctrl-C` or `q` to stop running.";*/
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(90),
                Constraint::Percentage(10),
            ])
            .split(frame.area());

        frame.render_widget(
            Block::new().title("TextEditor").borders(Borders::ALL),
            layout[0],
        );
        frame.render_widget(
            Block::new().title("CommandLine").borders(Borders::ALL),
            layout[1],
        );
    }*/

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (KeyModifiers::SHIFT, KeyCode::Char(':')) =>  self.toggle_active_area(),
            _ => {}
        }
    }

    fn toggle_active_area(&mut self) {
        match self.active_area {
            ActiveArea::Editor =>  self.active_area = ActiveArea::CommandLine,
            ActiveArea::CommandLine => self.active_area = ActiveArea::Editor,

        }
        println!("test");
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}
