use crate::core::app::App;
use crate::errors::plugin_error::PluginError;
use crate::plugins::plugin_registry::{
    KeyContext, Plugin, PluginCommand, PluginKeybinding, PluginMetadata,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::Frame;

pub struct TestPlugin {
    message: String,
}

impl TestPlugin {
    pub fn new() -> TestPlugin {
        TestPlugin {
            message: String::new(),
        }
    }
}

impl Plugin for TestPlugin {
    fn name(&self) -> &str {
        "test_plugin"
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "TestPlugin".to_string(),
            commands: vec![PluginCommand {
                name: "test".to_string(),
                description: "test plugin command".to_string(),
                aliases: vec!["testtest".to_string()],
                handler: |app, _args| {
                    app.plugins.activate_plugin("test_plugin");
                    Ok(())
                },
            }],
            keybinds: vec![PluginKeybinding {
                key: "Ctrl+T".to_string(),
                command: "test".to_string(),
                context: KeyContext::Editor,
            }],
        }
    }

    fn init(&mut self, _app: &mut App) -> Result<(), PluginError> {
        log_info!("testplugin init");
        Ok(())
    }

    fn handle_key_event(&mut self, _app: &mut App, key: KeyEvent) -> bool {
        log_info!("testplugin handle_key_event");
        match key.code {
            KeyCode::Esc => true,
            KeyCode::Char(c) => {
                self.message.push(c);
                log_info!("Test plugin received: {}", c);
                true
            }
            KeyCode::Backspace => {
                self.message.pop();
                true
            }
            _ => false,
        }
    }

    fn render(&self, frame: &mut Frame, _app: &App) -> bool {
        use ratatui::style::{Color, Style};
        use ratatui::widgets::{Block, Borders, Paragraph};
        log_info!("testplugin render");
        let block = Block::default()
            .title("Test Plugin")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));

        let content = vec![
            Line::from(format!("Message: {}", self.message)),
            Line::from(""),
            Line::from("Type to write | ESC to close"),
        ];

        let paragraph = Paragraph::new(content).block(block);
        // Render plugin as overlay/popup
        let plugin_area = Rect {
            x: frame.area().width / 4,
            y: frame.area().height / 4,
            width: frame.area().width / 2,
            height: frame.area().height / 2,
        };
        frame.render_widget(paragraph, plugin_area);
        true
    }

    fn shutdown(&mut self, _app: &mut App) {}
}
