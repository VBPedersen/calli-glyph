use super::popup::{Popup, PopupResult, PopupType};
use crate::errors::error::AppError;
use crate::input::input_action::InputAction;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{Color, Line, Span, Style, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

pub struct ErrorPopup {
    pub message: String,
    pub error: AppError,
}

impl ErrorPopup {
    pub fn new(msg: &str, e: AppError) -> Self {
        Self {
            message: msg.to_string(),
            error: e,
        }
    }
}

impl Popup for ErrorPopup {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let button_style = Style::default().bg(Color::White).fg(Color::Black);

        let popup_block = Block::default()
            .title("Error?")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White).bg(Color::Black));

        let popup = Paragraph::new(Text::from(vec![
            Line::from(Span::raw(&self.message)),
            Line::from(Span::raw(format!("{}", self.error))), // Empty line
            Line::from(Span::styled(" OK ", button_style)),
        ]))
        .block(popup_block)
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

        // Render the popup in the centered `area`
        frame.render_widget(Clear, area); // Clears the popup area to avoid overlap
        frame.render_widget(popup, area);
    }

    fn get_popup_type(&self) -> PopupType {
        PopupType::Error
    }

    fn handle_input_action(&mut self, action: InputAction) -> PopupResult {
        match action {
            InputAction::ENTER => PopupResult::Affirmed,
            _ => PopupResult::None,
        }
    }
}
