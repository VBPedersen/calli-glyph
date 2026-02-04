use crate::core::app::App;
use crate::errors::plugin_error::PluginError;
use crate::plugins::plugin_registry::{
    KeyContext, Plugin, PluginCommand, PluginKeybinding, PluginMetadata,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::Wrap;
use ratatui::Frame;
use std::cmp::{min, PartialEq};
//TODO make found matches highlighted in editor,
// select next and prev scroll to view the actually selected match
// Enter should replace current, and maybe shift enter to replace all

#[derive(PartialEq)]
enum FocusedField {
    Search,
    Replace,
}

pub struct SearchReplacePlugin {
    search_query: String,
    replace_text: String,
    focused_field: FocusedField,
    matches: Vec<(usize, usize)>, // matches seen as Vec of x,y position of matches
    current_match: usize,         // current match selected, as idx in matches Vec
}

impl SearchReplacePlugin {
    pub fn new() -> SearchReplacePlugin {
        SearchReplacePlugin {
            search_query: "".to_string(),
            replace_text: "".to_string(),
            focused_field: FocusedField::Search,
            matches: Vec::new(),
            current_match: 0,
        }
    }

    /// find matches in editor by array of strings search
    fn find_matches(&mut self, query: &[String]) {
        self.matches.clear(); // new search overrides old
        if self.search_query.is_empty() {
            return;
        }

        // iterate on query array then matches fo und on
        for (line_idx, line) in query.iter().enumerate() {
            let mut start_idx = 0;
            while let Some(match_pos) = line[start_idx..].find(&self.search_query) {
                let actual_pos = start_idx + match_pos;
                self.matches.push((line_idx, actual_pos));
                start_idx = actual_pos + 1;
            }
        }
        self.current_match = 0;
    }

    /// selects next match possible
    fn next_match(&mut self) {
        if !self.matches.is_empty() {
            self.current_match = (self.current_match + 1) % self.matches.len(); // increment current match, with modulo for safeguard
        }
    }

    /// selects previous match possible
    fn prev_match(&mut self) {
        if !self.matches.is_empty() {
            self.current_match = if self.current_match == 0 {
                // prev from 0 is max
                self.matches.len() - 1
            } else {
                self.current_match - 1
            }
        }
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

    fn init(&mut self, _app: &mut App) -> Result<(), PluginError> {
        Ok(())
    }

    fn handle_key_event(&mut self, app: &mut App, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => true,
            KeyCode::Char(c) => {
                match self.focused_field {
                    FocusedField::Search => {
                        self.search_query.push(c);
                    }
                    FocusedField::Replace => {
                        self.replace_text.push(c);
                    }
                }
                log_info!("finding matches {}", self.search_query);
                self.find_matches(&app.editor.editor_content);
                log_info!("found : {}", self.matches.len());
                true
            }
            KeyCode::Backspace => {
                match self.focused_field {
                    FocusedField::Search => self.search_query.pop(),
                    FocusedField::Replace => self.replace_text.pop(),
                };
                self.find_matches(&app.editor.editor_content);
                true
            }
            KeyCode::Enter => {
                // TODO replace
                true
            }
            KeyCode::Tab => {
                self.focused_field = match self.focused_field {
                    FocusedField::Search => FocusedField::Replace,
                    FocusedField::Replace => FocusedField::Search,
                };
                true
            }
            KeyCode::Up => {
                self.prev_match();
                true
            }
            KeyCode::Down => {
                self.next_match();
                true
            }
            _ => false,
        }
    }

    fn render(&self, frame: &mut Frame, _app: &App) -> bool {
        use ratatui::style::{Color, Style};
        use ratatui::widgets::{Block, Borders, Paragraph};
        let block = Block::default()
            .title("Search&Replace")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White));

        let search_focused = self.focused_field == FocusedField::Search;
        let search_style = if search_focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let replace_style = if !search_focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let match_info = format!("{}/{}", self.current_match + 1, self.matches.len());

        let content = vec![
            Line::from(vec![
                Span::styled("Search: ", search_style),
                Span::raw(&self.search_query),
                Span::raw(" "),
            ]),
            Line::from(vec![
                Span::styled("Replace: ", replace_style),
                Span::raw(&self.replace_text),
            ]),
            Line::from(
                Span::styled(format!("match: {}",&match_info), Style::default().fg(Color::Gray))
            ),
            Line::from(
                Span::styled("↑↓: Navigate | Tab: Switch | Enter: Replace | Esc: Close | Shift + Enter : replace all", Style::default().fg(Color::DarkGray)),)
        ];

        let paragraph = Paragraph::new(content)
            .block(block)
            .wrap(Wrap { trim: true });

        let width = (frame.area().width / 3).clamp(30, frame.area().width);
        let height = 8.min(frame.area().height);
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
