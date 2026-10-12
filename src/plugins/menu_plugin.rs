use crate::core::app::App;
use crate::errors::plugin_error::PluginError;
use crate::plugins::plugin_registry::{KeyContext, Plugin, PluginCommand, PluginKeybinding, PluginMetadata};
use ratatui::Frame;

pub struct MenuPlugin;

impl MenuPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for MenuPlugin {
    fn name(&self) -> &str {
        "menu_plugin"
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "menu_plugin".to_string(),
            commands: vec![PluginCommand {
                name: "menu".to_string(),
                description: "Open the main menu".to_string(),
                aliases: vec![],
                handler: handle_menu,
            }],
            keybinds: vec![PluginKeybinding{
                key: "Ctrl+M".to_string(),
                command: "menu".to_string(),
                context: KeyContext::Editor,
            }],
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

fn handle_menu(app: &mut App, _args: Vec<String>) -> Result<(), PluginError> {
    app.open_main_menu();
    Ok(())
}
