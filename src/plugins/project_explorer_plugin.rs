//! Exposes `:explore` (alias `:files`) to open the configured file
//! picker, built-in browser or an external tool, per
//! `config.project.file_picker`.

use crate::core::app::App;
use crate::errors::plugin_error::PluginError;
use crate::plugins::plugin_registry::{Plugin, PluginCommand, PluginMetadata};
use crate::project::open_file_picker;
use ratatui::Frame;

pub struct ProjectExplorerPlugin;

impl ProjectExplorerPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for ProjectExplorerPlugin {
    fn name(&self) -> &str {
        "project_explorer_plugin"
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "project_explorer_plugin".to_string(),
            commands: vec![PluginCommand {
                name: "explore".to_string(),
                description: "Open the file picker (builtin browser or configured external tool)"
                    .to_string(),
                aliases: vec!["files".to_string()],
                handler: handle_explore,
            }],
            keybinds: vec![],
        }
    }

    fn init(&mut self, _app: &mut App) -> Result<(), PluginError> {
        Ok(())
    }

    fn handle_key_event(&mut self, _app: &mut App, _key: crossterm::event::KeyEvent) -> bool {
        false
    }

    fn render(&self, _frame: &mut Frame, _app: &App) -> bool {
        false
    }
}

impl Default for ProjectExplorerPlugin {
    fn default() -> Self {
        Self::new()
    }
}

fn handle_explore(app: &mut App, _args: Vec<String>) -> Result<(), PluginError> {
    open_file_picker(app);
    Ok(())
}
