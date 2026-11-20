use crate::core::app::App;
use crate::errors::plugin_error::PluginError;
use crate::menu::MenuEntry;
use crate::plugins::plugin_registry::{
    KeyContext, Plugin, PluginCommand, PluginKeybinding, PluginMetadata,
};
use crossterm::event::KeyEvent;
use ratatui::Frame;

pub struct ThemePickerPlugin {
    available_themes: Vec<String>, // available themes seen as array of theme names
    current_theme_idx: usize,      // current theme selected, as idx in available themes Vec
}

impl ThemePickerPlugin {
    pub fn new() -> Self {
        ThemePickerPlugin {
            available_themes: vec![],
            current_theme_idx: 0,
        }
    }
}

impl Plugin for ThemePickerPlugin {
    fn name(&self) -> &str {
        "theme_picker_plugin"
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "ThemePickerPlugin".to_string(),
            commands: vec![PluginCommand {
                name: "themepicker".to_string(),
                description: "Browse, preview and apply themes".to_string(),
                aliases: vec!["thp".to_string(), "theme".to_string()],
                handler: |app, _args| {
                    app.open_theme_browser();
                    Ok(())
                },
            }],
            keybinds: vec![PluginKeybinding {
                key: "Ctrl+T".to_string(),
                command: "themepicker".to_string(),
                context: KeyContext::Editor,
            }],
        }
    }

    fn init(&mut self, _app: &mut App) -> Result<(), PluginError> {
        Ok(())
    }

    fn handle_key_event(&mut self, _app: &mut App, _key: KeyEvent) -> bool {
        false
    }

    fn render(&self, _frame: &mut Frame, _app: &App) -> bool {
        false
    }

    fn shutdown(&mut self, app: &mut App) {}

    fn takes_focus(&self) -> bool {
        false
    }

    fn menu_entries(&self) -> Vec<MenuEntry> {
        vec![
            MenuEntry::command(
                "Appearance",
                "Themes",
                "Browse, preview and apply themes",
                "theme",
            )
            .with_key('t'),
            MenuEntry::callback(
                "Appearance",
                "Edit current theme",
                "Change the colors and layout of the active theme",
                |app| app.open_theme_editor_for_active(),
            )
            .with_key('e'),
            MenuEntry::callback(
                "Appearance",
                "New theme",
                "Create a new theme as a copy of the active one",
                |app| app.open_theme_browser_naming(),
            )
            .with_key('n'),
        ]
    }
}
