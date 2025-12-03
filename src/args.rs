use clap::{Args, Parser};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(
    author,
    version,
    about = "A lightweight terminal text editor built in Rust",
    long_about = "calli-glyph (from calligraphy + glyph) is a simple, minimalistic terminal-based text editor written in Rust."
)]
pub struct AppLaunchArgs {
    /// The file path to open or create
    pub file_path: Option<PathBuf>,
    /// Deletes the application's user configuration file before starting.
    #[arg(long)]
    pub reset_config: bool,
}
