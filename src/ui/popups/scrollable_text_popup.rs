use crate::input::input_action::{Direction, InputAction};
use crate::ui::popups::popup::{Popup, PopupResult, PopupType};
use ratatui::layout::Direction::Vertical;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

pub struct ScrollableTextPopup<'a> {
    title: String,
    text: Paragraph<'a>,
    scroll_offset: usize,
}

impl<'a> ScrollableTextPopup<'a> {
    pub fn new(title: String, lines: Vec<Line<'a>>) -> Self {
        Self {
            title,
            text: Paragraph::new(lines),
            scroll_offset: 0,
        }
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }
}

impl Popup for ScrollableTextPopup<'_> {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let border_color = Color::White;

        // popup border and title
        let block = Block::default()
            .title(self.title.clone())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let inner_area = block.inner(area);
        frame.render_widget(block, area); // Render block

        let chunks = Layout::default()
            .direction(Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(inner_area);

        // Render text with scroll offset and wrap
        let paragraph = self
            .text
            .clone()
            .block(Block::default().borders(Borders::NONE))
            .wrap(Wrap { trim: true })
            .scroll((self.scroll_offset as u16, 0));
        frame.render_widget(paragraph, chunks[0]);

        // Footer
        let help = Paragraph::new("↑↓: Scroll | Enter/Esc: Close")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help, chunks[1]);
    }

    fn get_popup_type(&self) -> PopupType {
        PopupType::Info
    }

    fn handle_input_action(&mut self, action: InputAction) -> PopupResult {
        match action {
            InputAction::ENTER => PopupResult::Affirmed,
            InputAction::ToggleActiveArea => PopupResult::Affirmed,
            InputAction::MoveCursor(dir) => match dir {
                Direction::Up => {
                    self.scroll_up();
                    PopupResult::None
                }
                Direction::Down => {
                    self.scroll_down();
                    PopupResult::None
                }
                _ => PopupResult::None,
            },
            _ => PopupResult::None,
        }
    }
}
