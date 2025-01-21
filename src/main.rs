pub use app::App;

pub mod app;
mod ui;

use ratatui::crossterm::event::EnableMouseCapture;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode};
use std::{env, io};
use crossterm::event::DisableMouseCapture;
use crossterm::terminal::LeaveAlternateScreen;

fn main() -> color_eyre::Result<()> {
    let args: Vec<String> = env::args().collect();

    let file_path = if args.len() > 1 {
        Some(args[1].clone())
    } else {
        None
    };


    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal, file_path);
    //let result = ui::ui(&mut terminal, &app);
    ratatui::restore();

    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;

    result
}
