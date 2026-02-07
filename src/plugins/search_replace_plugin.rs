use crate::core::app::App;
use crate::errors::plugin_error::PluginError;
use crate::plugins::plugin_registry::{
    KeyContext, Plugin, PluginCommand, PluginKeybinding, PluginMetadata,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
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
    current_match_idx: usize,         // current match selected, as idx in matches Vec
}

impl SearchReplacePlugin {
    pub fn new() -> SearchReplacePlugin {
        SearchReplacePlugin {
            search_query: "".to_string(),
            replace_text: "".to_string(),
            focused_field: FocusedField::Search,
            matches: Vec::new(),
            current_match_idx: 0,
        }
    }

    /// find matches in editor by array of strings search
   fn find_matches(&mut self, query: &[String]) {
        self.matches.clear();
        if self.search_query.is_empty() {
            return;
        }
        // iterate on query array then matches fo und on
        for (line_idx, line) in query.iter().enumerate() {
            let mut start_idx = 0;

            while start_idx < line.len() {
                // Ensure at char boundary for current index
                if !line.is_char_boundary(start_idx) {
                    start_idx += 1;
                    continue;
                }

                // Search
                if let Some(match_pos) = line[start_idx..].find(&self.search_query) {
                    let actual_pos = start_idx + match_pos;
                    self.matches.push((line_idx, actual_pos));

                    // Move past this match
                    start_idx = actual_pos + self.search_query.len();
                } else {
                    break;
                }
            }
        }
        self.current_match_idx = 0;
    }

    /// selects next match possible
    fn next_match(&mut self) {
        if !self.matches.is_empty() {
            self.current_match_idx = (self.current_match_idx + 1) % self.matches.len(); // increment current match, with modulo for safeguard
        }
    }

    /// selects previous match possible
    fn prev_match(&mut self) {
        if !self.matches.is_empty() {
            self.current_match_idx = if self.current_match_idx == 0 {
                // prev from 0 is max
                self.matches.len() - 1
            } else {
                self.current_match_idx - 1
            }
        }
    }

    fn render_search_replace_dialog(&self, frame: &mut Frame) -> bool {
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

        let match_info = format!("{}/{}", self.current_match_idx + 1, self.matches.len());

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


    /// Convert byte position to visual column position
    /// Accounts for multibyte characters and tabs
    fn byte_pos_to_visual_col(&self, line: &str, byte_pos: usize, tab_width: usize) -> usize {
        let mut visual_col = 0;
        let mut byte_idx = 0;

        for char in line.chars() {
            if byte_idx >= byte_pos {
                break;
            }

            match char {
                '\t' => {
                    // Tab advances to next tab stop
                    visual_col += tab_width - (visual_col % tab_width);
                }
                _ => {
                    // Normal character (including multibyte like æ ø å)
                    visual_col += 1;
                }
            }

            byte_idx += char.len_utf8();
        }

        visual_col
    }

    /// Get visual width of search query in this line
    /// Also accounts for multibyte and tabs
    fn search_query_visual_width(&self, line: &str, byte_start_idx: usize, tab_width: usize) -> usize {
        // Ensure byte_start_idx is at a char boundary
        if byte_start_idx > line.len() {
            return 0;
        }

        // Find the actual start, ensuring at char boundary
        let mut actual_start = byte_start_idx;
        while actual_start > 0 && !line.is_char_boundary(actual_start) {
            actual_start -= 1;
        }

        let byte_end = (actual_start + self.search_query.len()).min(line.len());

        // Find the actual end, ensuring at char boundary
        let mut actual_end = byte_end;
        while actual_end < line.len() && !line.is_char_boundary(actual_end) {
            actual_end += 1;
        }

        let query_slice = &line[actual_start..actual_end];

        let mut visual_width = 0;
        let mut current_col = self.byte_pos_to_visual_col(line, actual_start, tab_width);

        for ch in query_slice.chars() {
            match ch {
                '\t' => {
                    visual_width += tab_width - (current_col % tab_width);
                    current_col += tab_width - (current_col % tab_width);
                }
                _ => {
                    visual_width += 1;
                    current_col += 1;
                }
            }
        }

        visual_width
    }

    fn render_highlights_overlay(&self, frame: &mut Frame, app: &App, content_area: Rect) {
        use ratatui::style::{Color, Modifier, Style};
        use ratatui::text::{Line, Span};

        let scroll_offset = app.editor.scroll_offset as usize;
        let tab_width = app.config.editor.tab_width as usize; 

        for (idx, &(line, byte_col)) in self.matches.iter().enumerate() {
            // Calculate position relative to visible viewport
            let line_in_viewport = line.saturating_sub(scroll_offset);

            // Only render if visible in current viewport
            if line_in_viewport >= content_area.height as usize {
                continue;
            }

            // Get the actual line content
            if line >= app.editor.editor_content.len() {
                continue;
            }
            let line_content = &app.editor.editor_content[line];

            // Convert byte position to visual column position
            let visual_col = self.byte_pos_to_visual_col(line_content, byte_col, tab_width);
            let visual_width = self.search_query_visual_width(line_content, byte_col, tab_width);

            // Position within content area
            let y = content_area.y + line_in_viewport as u16;
            let x = content_area.x + visual_col as u16;

            // Don't render if outside content area bounds
            if x >= content_area.right() || y >= content_area.bottom() {
                continue;
            }

            // Highlight style
            let style = if idx == self.current_match_idx {
                // Selected match
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                // Non selected match
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Gray)
            };

            // Create highlight area with correct visual width
            let highlight_width = visual_width.min((content_area.right() - x) as usize) as u16;

            if highlight_width == 0 {
                continue; // Skip if zero-width as: non existent
            }

            let highlight_area = Rect {
                x,
                y,
                width: highlight_width,
                height: 1,
            };

            // Get the search text to render
            let byte_end = (byte_col + self.search_query.len()).min(line_content.len());
            let search_slice = &line_content[byte_col..byte_end];
            
            // Render
            let highlight_text = Line::from(Span::styled(search_slice, style));
            frame.render_widget(Paragraph::new(highlight_text), highlight_area);
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

    fn render(&self, frame: &mut Frame, app: &App) -> bool {

        // render plugin dialog
        self.render_search_replace_dialog(frame);

        // render highlight overlay if some
        if let Some(editor_area) = app.layout.get("content") {
            self.render_highlights_overlay(frame, app, editor_area);
        }

        true
    }



    fn shutdown(&mut self, app: &mut App) {}
}
