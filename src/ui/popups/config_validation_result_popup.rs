use crate::config::ValidationResult;
use crate::ui::popups::popup::{Popup, PopupResult, PopupType};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::input::input_action::InputAction;

#[derive(Debug)]
pub struct ValidationResultPopup {
    result: ValidationResult,
}

impl ValidationResultPopup {
    pub fn new(result: ValidationResult) -> Self {
        Self { result }
    }
}

impl Popup for ValidationResultPopup {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let border_color = if self.result.errors.is_empty() {
            Color::Yellow
        } else {
            Color::Red
        };

        let main_title = if self.result.errors.is_empty() {
            "Config Validation Complete"
        } else {
            "Config Validation Failed"
        };

        // popup border and title
        let block = Block::default()
            .title(main_title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        frame.render_widget(&block, area); // Render block
        let inner_area = block.inner(area);

        // lines like Header, Errors, Warnings
        let mut lines = Vec::new();

        // Status Header
        let status_style = if self.result.valid && self.result.errors.is_empty() {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        };

        lines.push(Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                if self.result.valid && self.result.errors.is_empty() {
                    "VALID"
                } else {
                    "INVALID"
                },
                status_style,
            ),
        ]));
        lines.push(Line::from("")); // Spacer

        // Errors Section
        if !self.result.errors.is_empty() {
            lines.push(Line::from(Span::styled(
                "ERRORS:",
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )));
            for error in &self.result.errors {
                lines.push(Line::from(Span::styled(
                    format!("  • {}", error),
                    Style::default().fg(Color::Red),
                )));
            }
            lines.push(Line::from("")); // Spacer
        }

        // Warnings Section
        if !self.result.warnings.is_empty() {
            lines.push(Line::from(Span::styled(
                "WARNINGS:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )));
            for warning in &self.result.warnings {
                lines.push(Line::from(Span::styled(
                    format!("  - {}", warning),
                    Style::default().fg(Color::Yellow),
                )));
            }
            lines.push(Line::from("")); // Spacer
        }

        // Footer
        lines.push(Line::from(Span::styled(
            "Press [Enter] to close.",
            Style::default().fg(Color::DarkGray),
        )));

        // Render the Paragraph widget
        let text_content = Text::from(lines);
        let paragraph = Paragraph::new(text_content).style(Style::default().fg(Color::White));

        frame.render_widget(paragraph, inner_area);
    }

    fn get_popup_type(&self) -> PopupType {
        PopupType::Info
    }

    fn handle_input_action(&mut self, action: InputAction) -> PopupResult {
        match action {
            InputAction::ENTER => PopupResult::Affirmed,
            _ => PopupResult::None,
        }
    }
}
