use crate::core::help_registry::{HelpPage, HelpRegistry};
use crate::input::actions::{InputAction, PopupAction};
use crate::ui::popups::popup::{Popup, PopupResult, PopupType};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar,
    ScrollbarOrientation, ScrollbarState, Wrap,
};
use ratatui::Frame;
use std::sync::Arc;

enum HelpFocus {
    List,
    Content,
}

pub struct HelpPopup {
    registry: Arc<HelpRegistry>,

    /// What part of popup is focused
    focus: HelpFocus,
    /// indices into registry that match search query
    filtered_pages_indices: Vec<usize>,
    /// Cached rendered lines for the currently selected page.
    rendered_lines: Vec<Line<'static>>,
    /// Currently highlighted index within `filtered_pages_indices`.
    list_cursor: usize,
    /// Vertical scroll offset of the content panel (in rendered lines).
    content_scroll: usize,
    /// Query string content, typed by user
    search_query: String,
    /// Whether search bar is currently active and receiving input
    search_active: bool,
}

impl HelpPopup {
    /// Open the popup in browse mode. all pages listed, nothing pre-selected.
    pub fn browse(registry: Arc<HelpRegistry>) -> Self {
        let filtered: Vec<usize> = (0..registry.len()).collect();
        let mut popup = Self {
            registry,
            list_cursor: 0,
            content_scroll: 0,
            search_query: String::new(),
            search_active: false,
            filtered_pages_indices: filtered,
            rendered_lines: vec![],
            focus: HelpFocus::List,
        };

        popup.refresh_rendered_lines();
        popup
    }

    /// Open the popup in focused mode on a specific topic id (e.g. "debug").
    /// Falls back to browse mode, if the id is not found.
    pub fn focused(registry: Arc<HelpRegistry>, topic: &str) -> Self {
        let mut popup = Self::browse(registry);
        // On focused content should be focused at start
        popup.focus = HelpFocus::Content;
        // On focused also use topic for searching
        popup.search_active = true;
        popup.search_query = topic.to_string();
        popup.refresh_filtered_pages();

        // Find the index of the topic in the full registry and locate it in filtered.
        if let Some(reg_idx) = popup
            .registry
            .get_all()
            .iter()
            .position(|p| p.id == topic.to_lowercase())
        {
            if let Some(filtered_pos) = popup
                .filtered_pages_indices
                .iter()
                .position(|&i| i == reg_idx)
            {
                popup.list_cursor = filtered_pos;
                popup.refresh_rendered_lines();
            }
        }

        popup
    }

    // --------- Helpers -----------

    /// Get selected page by cursor pos in filtered pages indices
    fn selected_page(&self) -> Option<&HelpPage> {
        self.filtered_pages_indices
            .get(self.list_cursor)
            .and_then(|&i| self.registry.get(i))
    }

    /// Re-render the markdown content of the currently selected page into
    /// styled ratatui `Line`s. Call whenever the selection changes.
    fn refresh_rendered_lines(&mut self) {
        self.rendered_lines = match self.selected_page() {
            Some(page) => render_markdown(&page.content),
            None => vec![Line::from("No page selected.")],
        };
        self.content_scroll = 0;
    }

    /// Rebuild filtered_pages_indices based on current query
    fn refresh_filtered_pages(&mut self) {
        self.filtered_pages_indices = self.registry.search(&self.search_query);
        self.list_cursor = 0;
        self.refresh_rendered_lines();
    }

    /// Toggles focus in help popup
    fn toggle_focus(&mut self) {
        match self.focus {
            HelpFocus::List => self.focus = HelpFocus::Content,
            HelpFocus::Content => self.focus = HelpFocus::List,
        }
    }

    /// Activate search functionality
    fn activate_search(&mut self) {
        self.search_active = true;
    }

    /// Deactivate search functionality
    fn deactivate_search(&mut self) {
        self.search_active = false;
    }

    /// Select previous element in list
    fn list_prev(&mut self) {
        if self.list_cursor > 0 {
            self.list_cursor -= 1;
            self.refresh_rendered_lines();
        }
    }

    /// Select next element in list
    fn list_next(&mut self) {
        if self.list_cursor + 1 < self.filtered_pages_indices.len() {
            self.list_cursor += 1;
            self.refresh_rendered_lines();
        }
    }

    /// Scroll up on the help page content in content area
    fn scroll_content_up(&mut self) {
        self.content_scroll = self.content_scroll.saturating_sub(3);
    }

    /// Scroll down on the help page content in content area
    fn scroll_content_down(&mut self) {
        let max = self.rendered_lines.len().saturating_sub(1);
        self.content_scroll = (self.content_scroll + 3).min(max);
    }

    /// Write character to the search bar
    fn push_search_char(&mut self, c: char) {
        self.search_query.push(c);
        self.refresh_filtered_pages();
    }

    /// Remove character from search bar
    fn pop_search_char(&mut self) {
        self.search_query.pop();
        self.refresh_filtered_pages();
    }

    /// Clear content in search bar
    fn clear_search(&mut self) {
        self.search_query.clear();
        self.search_active = false;
        self.refresh_filtered_pages();
    }

    // --------- RENDERING -----------

    /// Renders the list panel of help pages
    fn render_list_panel(&self, area: Rect, frame: &mut Frame) {
        let list_items: Vec<ListItem> = self
            .filtered_pages_indices
            .iter()
            .enumerate()
            .filter_map(|(pos, &reg_idx)| {
                let page = self.registry.get(reg_idx)?;
                let is_selected = pos == self.list_cursor;

                let line = if is_selected {
                    Line::from(vec![
                        Span::styled("░ ", Style::default().fg(Color::Cyan)),
                        Span::styled(
                            page.title.clone(),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled("▎ ", Style::default().fg(Color::DarkGray)),
                        Span::styled(page.title.clone(), Style::default().fg(Color::Gray)),
                    ])
                };
                Some(ListItem::new(line))
            })
            .collect();

        // Show "no results" placeholder when search yields nothing.
        let list_items = if list_items.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                "  no results",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )))]
        } else {
            list_items
        };

        let mut list_state = ListState::default();

        if !self.filtered_pages_indices.is_empty() {
            list_state.select(Some(self.list_cursor));
        }

        let list_block = Block::default()
            .title(Line::from(Span::styled(
                " Topics ",
                Style::default().fg(Color::DarkGray),
            )))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray))
            .style(Style::default().bg(Color::Black));

        let list_widget = List::new(list_items)
            .block(list_block)
            .highlight_style(Style::default()) // We handle highlight styling ourselves above.
            .highlight_symbol("");

        frame.render_stateful_widget(list_widget, area, &mut list_state);
    }

    /// Renders Content panel with help page content parse from markdown and then rendered
    fn render_content_panel(&mut self, area: Rect, frame: &mut Frame) {
        let content_title = self
            .selected_page()
            .map(|p| format!(" {} ", p.title))
            .unwrap_or_else(|| " Help ".to_string());

        let content_inner_area = area.height.saturating_sub(2) as usize;

        let visible_lines: Vec<Line> = self
            .rendered_lines
            .iter()
            .skip(self.content_scroll)
            .take(content_inner_area)
            .cloned()
            .collect();

        let content_block = Block::default()
            .title(Line::from(Span::styled(
                content_title,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray))
            .style(Style::default().bg(Color::Black));

        let content_widget = Paragraph::new(visible_lines)
            .block(content_block)
            .wrap(Wrap { trim: false });

        frame.render_widget(content_widget, area);

        // Scrollbar on content pane when content overflows.
        let total_lines = self.rendered_lines.len();
        if total_lines > content_inner_area {
            let mut scrollbar_state =
                ScrollbarState::new(total_lines).position(self.content_scroll);

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"))
                .track_symbol(Some("│"))
                .thumb_symbol("█")
                .style(Style::default().fg(Color::DarkGray));

            frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
        }
    }

    /// Renders search bar
    fn render_search_bar(&self, area: Rect, frame: &mut Frame) {
        let (search_border_color, search_prefix, cursor) = if self.search_active {
            (Color::Yellow, ">", "_")
        } else {
            (Color::DarkGray, "/", "")
        };

        let search_content = if self.search_query.is_empty() && !self.search_active {
            Line::from(Span::styled(
                " / Search topics...",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ))
        } else {
            Line::from(vec![
                Span::styled(
                    format!(" {} ", search_prefix),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(self.search_query.clone(), Style::default().fg(Color::White)),
                Span::styled(
                    cursor,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::SLOW_BLINK),
                ),
            ])
        };

        let search_block = Block::default()
            .title(Line::from(Span::styled(
                " Search ",
                Style::default().fg(Color::DarkGray),
            )))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(search_border_color));

        let search_widget = Paragraph::new(search_content).block(search_block);
        frame.render_widget(search_widget, area);
    }
}

impl Popup for HelpPopup {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        use ratatui::layout::Direction;

        frame.render_widget(Clear, area);
        let outer_block = Block::default()
            .title(Span::styled(
                " 󰋗 Help ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
            .title_bottom(
                Line::styled(
                    " Tab swap focus ↑↓ navigate   / search   esc close ",
                    Style::default().fg(Color::DarkGray),
                )
                .right_aligned(),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(Color::Blue));

        let inner_block = outer_block.inner(area);
        frame.render_widget(outer_block, area);

        // --------------- VERTICAL LAYOUT ----------------
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(inner_block);

        let body_area = vertical[0];
        let search_area = vertical[1];

        // --------------- HORIZONTAL LAYOUT ----------------
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(body_area);

        let list_area = horizontal[0];
        let content_area = horizontal[1];

        self.render_list_panel(list_area, frame);
        self.render_content_panel(content_area, frame);
        self.render_search_bar(search_area, frame);
    }

    fn get_popup_type(&self) -> PopupType {
        PopupType::Info
    }

    fn handle_input_action(&mut self, action: InputAction) -> PopupResult {
        use crate::input::actions::Direction;

        match action {
            InputAction::Popup(PopupAction::MoveCursor(Direction::Up)) => match self.focus {
                HelpFocus::List => self.list_prev(),
                HelpFocus::Content => self.scroll_content_up(),
            },
            InputAction::Popup(PopupAction::MoveCursor(Direction::Down)) => match self.focus {
                HelpFocus::List => self.list_next(),
                HelpFocus::Content => self.scroll_content_down(),
            },
            InputAction::Popup(PopupAction::ToggleFocus) => self.toggle_focus(),

            // --------- Search ----------
            InputAction::Popup(PopupAction::ToggleSearch) => {
                if self.search_active {
                    self.deactivate_search();
                } else {
                    self.activate_search();
                }
            }

            InputAction::Popup(PopupAction::Close) => {
                if self.search_active || !self.search_query.is_empty() {
                    self.clear_search();
                } else {
                    return PopupResult::Affirmed;
                }
            }
            InputAction::Popup(PopupAction::WriteChar(c)) if self.search_active => {
                self.push_search_char(c);
            }
            InputAction::Popup(PopupAction::Backspace) if self.search_active => {
                self.pop_search_char();
            }
            _ => return PopupResult::None,
        }

        PopupResult::None // so as to not return for each match case
    }

    fn size(&self) -> (u16, u16) {
        (80, 80)
    }
}

// --------- MARKDOWN RENDERING -------------

/// Markdown renderer: converts markdown text into styled ratatui Lines.
/// For now just handles: # h1, ## h2, ### h3, **bold**, `code`, | tables, blank lines, body text.
fn render_markdown(content: &str) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    for raw_line in content.lines() {
        let line = raw_line.trim_end();

        if line.starts_with("### ") {
            // H3 : bold white
            lines.push(Line::from(Span::styled(
                line[4..].to_string(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )));
        } else if line.starts_with("## ") {
            // H2 : bold cyan with leading newline for spacing
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                line[3..].to_string(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )));
        } else if line.starts_with("# ") {
            // H1 : bold yellow, title style
            lines.push(Line::from(Span::styled(
                line[2..].to_string(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )));
            lines.push(Line::from(""));
        } else if line.starts_with("| ") || line.starts_with("|:") || line.starts_with("|--") {
            // Table row : skip separator rows, render data rows dimmed
            if line.contains("---") {
                // Separator row : skip.
            } else {
                let cells: Vec<&str> = line
                    .trim_matches('|')
                    .split('|')
                    .map(|c| c.trim())
                    .collect();

                let spans: Vec<Span> = cells
                    .iter()
                    .enumerate()
                    .flat_map(|(i, cell)| {
                        let style = if i == 0 {
                            Style::default().fg(Color::Green)
                        } else {
                            Style::default().fg(Color::Gray)
                        };
                        vec![
                            Span::styled(format!("  {:<20}", cell), style),
                            Span::raw(" "),
                        ]
                    })
                    .collect();

                lines.push(Line::from(spans));
            }
        } else if line.is_empty() {
            lines.push(Line::from(""));
        } else {
            // Body text : inline code and bold handled inline.
            lines.push(render_inline(line));
        }
    }

    lines
}

/// Render a single line of body text, handling `code` and **bold** inline.
fn render_inline(line: &str) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut remaining = line.to_string();

    while !remaining.is_empty() {
        if let Some(start) = remaining.find('`') {
            if let Some(end) = remaining[start + 1..].find('`') {
                let before = remaining[..start].to_string();
                let code = remaining[start + 1..start + 1 + end].to_string();
                remaining = remaining[start + 1 + end + 1..].to_string();

                if !before.is_empty() {
                    spans.push(Span::styled(before, Style::default().fg(Color::Gray)));
                }
                spans.push(Span::styled(
                    code,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ));
                continue;
            }
        }

        if let Some(start) = remaining.find("**") {
            if let Some(end) = remaining[start + 2..].find("**") {
                let before = remaining[..start].to_string();
                let bold = remaining[start + 2..start + 2 + end].to_string();
                remaining = remaining[start + 2 + end + 2..].to_string();

                if !before.is_empty() {
                    spans.push(Span::styled(before, Style::default().fg(Color::Gray)));
                }
                spans.push(Span::styled(
                    bold,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ));
                continue;
            }
        }

        // No more inline markup — push the rest as plain text.
        spans.push(Span::styled(
            remaining.clone(),
            Style::default().fg(Color::Gray),
        ));
        break;
    }

    Line::from(spans)
}
