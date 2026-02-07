use crate::core::app::App;
use crate::core::cursor::CursorPosition;
use crate::errors::plugin_error::PluginError;
use crate::plugins::plugin_registry::{
    KeyContext, Plugin, PluginCommand, PluginKeybinding, PluginMetadata,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;
use std::cmp::{min, PartialEq};
//TODO make found matches highlighted in editor,
// select next and prev scroll to view the actually selected match
// Enter should replace current, and maybe shift enter to replace all

//TODO make possible to toggle to not match case sensitivity and exact words.
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
    current_match_idx: usize,     // current match selected, as idx in matches Vec
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

    /// Find matches in buffer provided, according to search query text
    fn find_matches(&mut self, buffer: &[String]) {
        self.matches.clear();
        if self.search_query.is_empty() {
            return;
        }
        // iterate on query array then matches fo und on
        for (line_idx, line) in buffer.iter().enumerate() {
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
    fn next_match(&mut self, app: &mut App) {
        if !self.matches.is_empty() {
            // increment current match, with modulo for safeguard
            self.current_match_idx = (self.current_match_idx + 1) % self.matches.len();
            self.scroll_to_match(app);
        }
    }

    /// selects previous match possible
    fn prev_match(&mut self, app: &mut App) {
        if !self.matches.is_empty() {
            self.current_match_idx = if self.current_match_idx == 0 {
                // prev from 0 is max
                self.matches.len() - 1
            } else {
                self.current_match_idx - 1
            };
            self.scroll_to_match(app);
        }
    }

    /// Replace current selected match with replace content, and move to next
    fn replace_current_selected(&mut self, app: &mut App) -> Result<(), PluginError> {
        if self.matches.is_empty() || self.current_match_idx > self.matches.len() {
            return Err(PluginError::Internal(
                "Trying to replace when no matches or selected index longer than matches"
                    .to_string(),
            ));
        }

        //current match
        let (line_idx, byte_col) = self.matches[self.current_match_idx];

        let mut buffer = app.editor.editor_content.clone();

        if line_idx > buffer.len() {
            return Err(PluginError::Internal(
                "Line index is larger than buffer length".to_string(),
            ));
        }

        let line = &mut buffer[line_idx];

        // Ensure byte position is at a char boundary
        let actual_byte_col = self.find_char_boundary(line, byte_col);

        // Calculate byte range to replace
        let byte_end = (actual_byte_col + self.search_query.len()).min(line.len());

        // Verify can slice safely
        if !line.is_char_boundary(actual_byte_col) || !line.is_char_boundary(byte_end) {
            return Err(PluginError::Internal(
                "Replace not possible, match is inbetween char boundaries, slice not safe "
                    .to_string(),
            ));
        }

        // Replace
        line.replace_range(actual_byte_col..byte_end, &self.replace_text);

        // Replace editor content with new changes
        app.editor.editor_content = buffer.clone();

        // Find new matches and scroll to
        self.find_matches(&app.editor.editor_content);
        self.scroll_to_match(app);

        Ok(())
    }

    /// Replace all matches with replace content
    fn replace_all(&self, app: &mut App) {
        todo!()
    }

    /// Find the char boundary at or before the given byte position
    fn find_char_boundary(&self, line: &str, byte_pos: usize) -> usize {
        if byte_pos > line.len() {
            return line.len();
        }

        if line.is_char_boundary(byte_pos) {
            return byte_pos;
        }

        // Go backwards to find the char boundary
        let mut pos = byte_pos;
        while pos > 0 && !line.is_char_boundary(pos) {
            pos -= 1;
        }
        pos
    }

    /// Scroll editor to current match
    fn scroll_to_match(&self, app: &mut App) {
        if let Some((line, col)) = self.matches.get(self.current_match_idx) {
            // Move cursor to match position
            app.editor
                .set_cursor_position(&CursorPosition { x: *col, y: *line });

            // Ensure line is visible in viewport
            let viewport_height = app.editor.editor_height as usize;
            let target_scroll = if *line < viewport_height / 2 {
                0
            } else {
                line - viewport_height / 2
            };
            app.editor.set_scroll_offset(target_scroll as i16);
        }
    }

    /// Render dialog for search replace plugin
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

    /// Render the highlight overlay for matches found
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

            // Check if line is before viewport, aka not visible
            if line < scroll_offset {
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
                Style::default().fg(Color::Black).bg(Color::Gray)
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
    fn search_query_visual_width(
        &self,
        line: &str,
        byte_start_idx: usize,
        tab_width: usize,
    ) -> usize {
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
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Esc) => true,
            (_, KeyCode::Char(c)) => {
                match self.focused_field {
                    FocusedField::Search => {
                        self.search_query.push(c);
                    }
                    FocusedField::Replace => {
                        self.replace_text.push(c);
                    }
                }
                self.find_matches(&app.editor.editor_content);
                if !self.matches.is_empty() {
                    self.scroll_to_match(app);
                }
                true
            }
            (KeyModifiers::NONE, KeyCode::Backspace) => {
                match self.focused_field {
                    FocusedField::Search => self.search_query.pop(),
                    FocusedField::Replace => self.replace_text.pop(),
                };
                self.find_matches(&app.editor.editor_content);
                true
            }
            (KeyModifiers::NONE, KeyCode::Enter) => {
                self.replace_current_selected(app);
                true
            }
            (KeyModifiers::SHIFT, KeyCode::Enter) => {
                self.replace_all(app);
                true
            }
            (KeyModifiers::NONE, KeyCode::Tab) => {
                self.focused_field = match self.focused_field {
                    FocusedField::Search => FocusedField::Replace,
                    FocusedField::Replace => FocusedField::Search,
                };
                true
            }
            (KeyModifiers::NONE, KeyCode::Up) => {
                self.prev_match(app);
                true
            }
            (KeyModifiers::NONE, KeyCode::Down) => {
                self.next_match(app);
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
