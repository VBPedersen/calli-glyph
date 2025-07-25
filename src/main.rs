pub use core::app::App;

//███╗   ███╗ ██████╗ ██████╗ ██╗   ██╗██╗     ███████╗███████╗
//████╗ ████║██╔═══██╗██╔══██╗██║   ██║██║     ██╔════╝██╔════╝
//██╔████╔██║██║   ██║██║  ██║██║   ██║██║     █████╗  ███████╗
//██║╚██╔╝██║██║   ██║██║  ██║██║   ██║██║     ██╔══╝  ╚════██║
//██║ ╚═╝ ██║╚██████╔╝██████╔╝╚██████╔╝███████╗███████╗███████║
//╚═╝     ╚═╝ ╚═════╝ ╚═════╝  ╚═════╝ ╚══════╝╚══════╝╚══════╝

mod config;
pub mod core; //expose app module
pub mod input; //expose input module

pub mod ui;

//███╗   ███╗ █████╗ ██╗███╗   ██╗
//████╗ ████║██╔══██╗██║████╗  ██║
//██╔████╔██║███████║██║██╔██╗ ██║
//██║╚██╔╝██║██╔══██║██║██║╚██╗██║
//██║ ╚═╝ ██║██║  ██║██║██║ ╚████║
//╚═╝     ╚═╝╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝

use crossterm::event::DisableMouseCapture;
use crossterm::terminal::LeaveAlternateScreen;
use ratatui::crossterm::event::EnableMouseCapture;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen};
use std::{env, io};

fn main() -> color_eyre::Result<()> {
    env::set_var("RUST_BACKTRACE", "1"); //more verbose error codes

    let args: Vec<String> = env::args().collect();

    let file_path = if args.len() > 1 {
        Some(args[1].clone())
    } else {
        None
    };

    enable_raw_mode().expect("Failed to enable raw mode");
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
