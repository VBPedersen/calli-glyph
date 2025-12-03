pub use core::app::App;

pub mod app_config;
mod args;
mod config;
pub mod core; //expose app module
pub mod errors;
pub mod input; //expose input module
pub mod ui;

use crate::app_config::AppLaunchConfig;
use crate::args::AppLaunchArgs;
use crate::config::Config;
use clap::Parser;
use crossterm::terminal::LeaveAlternateScreen;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen};
use std::{env, io};

fn main() -> color_eyre::Result<()> {
    env::set_var("RUST_BACKTRACE", "1"); //more verbose error codes

    let args = AppLaunchArgs::parse();
    let app_launch_config = AppLaunchConfig::from_args(args);
    /* let args: Vec<String> = env::args().collect();

    let file_path = if args.len() > 1 {
        Some(args[1].clone())
    } else {
        None
    };*/

    enable_raw_mode().expect("Failed to enable raw mode");
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    color_eyre::install()?;
    let terminal = ratatui::init();

    // Load config
    let config = Config::load();
    let result = App::new(config, app_launch_config?).run(terminal);
    //let result = ui::ui(&mut terminal, &app);
    ratatui::restore();

    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;

    result
}
