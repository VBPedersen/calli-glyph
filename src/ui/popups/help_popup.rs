use crate::core::help_registry::HelpRegistry;
use crate::input::actions::{InputAction, PopupAction};
use crate::ui::popups::popup::{Popup, PopupResult, PopupType};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState};
use ratatui::Frame;
use std::sync::Arc;

pub struct HelpPopup {
    registry: Arc<HelpRegistry>,
    list_cursor: usize,
    search_query: String,
    search_active: bool,
    filtered_pages: Vec<usize>,   // indices into registry
    selected_page: Option<usize>, // optional single indices into registry
}

impl HelpPopup {
    /// Open the popup in browse mode. all pages listed, nothing pre-selected.
    pub fn browse(registry: Arc<HelpRegistry>) -> Self {
        let filtered: Vec<usize> = (0..registry.len()).collect();
        Self {
            registry,
            list_cursor: 0,
            search_query: String::new(),
            search_active: false,
            filtered_pages: filtered,
            selected_page: None,
        }
    }

    /// Open the popup in focused mode on a specific topic id (e.g. "debug").
    /// Falls back to browse mode, if the id is not found.
    pub fn focused(registry: Arc<HelpRegistry>, topic: &str) -> Self {
        let mut popup = Self::browse(registry);

        // Find the index of the topic in the full registry and locate it in filtered.
        if let Some(reg_idx) = popup
            .registry
            .get_all()
            .iter()
            .position(|p| p.id == topic.to_lowercase())
        {}

        popup
    }

    /// Renders the list panel of help pages
    fn render_list_panel(&self, area: Rect, frame: &mut Frame) {
        let list_items: Vec<ListItem> = self
            .filtered_pages
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
            }).collect();

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

        if !self.filtered_pages.is_empty() {
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
}

impl Popup for HelpPopup {
    fn render(&self, frame: &mut Frame, area: Rect) {
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
                    " ↑↓ navigate   / search   esc close ",
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
    }

    fn get_popup_type(&self) -> PopupType {
        PopupType::Info
    }

    fn handle_input_action(&mut self, action: InputAction) -> PopupResult {
        match action {
            InputAction::ToggleActiveArea => PopupResult::Affirmed,
            /*InputAction::ENTER => {},
            InputAction::Popup(PopupAction::MoveCursor(Direction::Up)) => {}*/
            _ => PopupResult::None,
        }
    }

    fn size(&self) -> (u16, u16) {
        (80, 80)
    }
}
