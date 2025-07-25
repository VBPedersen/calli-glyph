use super::popup::{Popup, PopupResult, PopupType};
use crossterm::event::KeyEvent;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

#[derive(Debug)]
pub struct ConfirmationPopup {
    pub message: String,
    pub selected_option: bool, // true = Yes, false = No
}

impl ConfirmationPopup {
    pub fn new(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
            selected_option: true,
        }
    }
}
impl Popup for ConfirmationPopup {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let selected_style = Style::default().bg(Color::White).fg(Color::Black);
        let non_selected_style = Style::default().bg(Color::Black).fg(Color::White);

        // Highlight correct option
        let yes_style = if self.selected_option {
            selected_style
        } else {
            non_selected_style
        };
        let no_style = if !self.selected_option {
            selected_style
        } else {
            non_selected_style
        };

        let popup_block = Block::default()
            .title("Confirm?")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White).bg(Color::Black));

        let popup = Paragraph::new(Text::from(vec![
            Line::from(Span::raw(&self.message)),
            Line::from(Span::raw("")), // Empty line
            Line::from(vec![
                Span::styled(" Yes ", yes_style),
                Span::raw("  "), // Space between "Yes" and "No"
                Span::styled(" No ", no_style),
            ]),
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
            KeyCode::Left | KeyCode::Right => {
                self.selected_option = !self.selected_option; // Toggle between Yes/No
                PopupResult::None
            }
            KeyCode::Enter => {
                PopupResult::Bool(self.selected_option) // Return true if "Yes" was selected
            }
            _ => PopupResult::None,
        }
    }

    fn get_popup_type(&self) -> PopupType {
        PopupType::Confirmation
    }
}
