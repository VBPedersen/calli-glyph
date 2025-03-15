use crate::popup::{Popup, PopupResult, PopupType};
use color_eyre::Report;
use crossterm::event::KeyEvent;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{Color, Line, Span, Style, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use crate::errors::AppError;

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
        .alignment(Alignment::Center);

        // Render the popup in the centered `area`
        frame.render_widget(Clear, area); // Clears the popup area to avoid overlap
        frame.render_widget(popup, area);
    }

    fn handle_key_input(&mut self, key: KeyEvent) -> PopupResult {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Enter => {
                PopupResult::Affirmed // Return Affirmed
            }
            _ => PopupResult::None,
        }
    }

    fn get_popup_type(&self) -> PopupType {
        PopupType::Error
    }
}
