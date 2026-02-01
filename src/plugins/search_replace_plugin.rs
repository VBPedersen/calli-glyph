use crate::core::app::App;
use crate::errors::plugin_error::PluginError;
use crate::plugins::plugin_registry::{
    KeyContext, Plugin, PluginCommand, PluginKeybinding, PluginMetadata,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::Frame;
use std::cmp::{max, min};

pub struct SearchReplacePlugin {}

impl SearchReplacePlugin {
    pub fn new() -> SearchReplacePlugin {
        SearchReplacePlugin {}
    }
}

impl Plugin for SearchReplacePlugin {
    fn name(&self) -> &str {
        "search_replace_plugin"
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "Search&ReplacePlugin".to_string(),
            commands: vec![PluginCommand {
                name: "search".to_string(),
                description: "Open search and replace dialog".to_string(),
                aliases: vec!["s".to_string(), "find".to_string()],
                handler: |app, _args| {
                    app.plugins.activate_plugin("search_replace_plugin");
                    Ok(())
                },
            }],
            keybinds: vec![PluginKeybinding {
                key: "Ctrl+F".to_string(),
                command: "search".to_string(),
                context: KeyContext::Editor,
            }],
        }
    }

    fn init(&mut self, app: &mut App) -> Result<(), PluginError> {
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => true,
            _ => false,
        }
    }

    fn render(&self, frame: &mut Frame, app: &App) -> bool {
        use ratatui::style::{Color, Style};
        use ratatui::widgets::{Block, Borders, Paragraph};
        let block = Block::default()
            .title("Search&Replace")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White));

        let content = vec![Line::from("Search:"), Line::from("Replace:")];

        let paragraph = Paragraph::new(content).block(block);

        let width = (frame.area().width / 3).clamp(30, frame.area().width);
        let height = 5.min(frame.area().height);
        // Render plugin as overlay/popup
        let plugin_area = Rect {
            x: frame.area().width.saturating_sub(width),
            y: min(frame.area().height, 1),
            width,
            height,
        };
        frame.render_widget(paragraph, plugin_area);
        true
    }

    fn shutdown(&mut self, app: &mut App) {}
}
