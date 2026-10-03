use super::popup::{Popup, PopupResult, PopupType};
use crate::core::app::App;
use crate::errors::error::AppError;
use crate::input::actions::InputAction;
use crate::language::lsp::DiagnosticSeverity;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{Line, Span, Style, Text};
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
    fn render(&mut self, frame: &mut Frame, area: Rect, app: &App) {
        let ui = &app.theme_manager.active().ui;

        let button_style = Style::default()
            .bg(ui.severity_color(DiagnosticSeverity::Error))
            .fg(ui.popup_bg());

        let popup_block = Block::default()
            .title("Error?")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ui.severity_color(DiagnosticSeverity::Error)))
            .style(Style::default().fg(ui.popup_fg()).bg(ui.popup_bg()));

        let popup = Paragraph::new(Text::from(vec![
            Line::from(Span::raw(&self.message)),
            Line::from(Span::raw(format!("{}", self.error))), // Empty line
            Line::from(Span::styled(" OK ", button_style)),
        ]))
        .block(popup_block)
        .style(Style::default().fg(ui.popup_fg()).bg(ui.popup_bg()))
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
