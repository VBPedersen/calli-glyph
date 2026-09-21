use crate::config::ValidationResult;
use crate::core::app::App;
use crate::input::actions::{Direction, InputAction, PopupAction};
use crate::language::lsp::DiagnosticSeverity;
use crate::ui::popups::popup::{Popup, PopupResult, PopupType};
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::{Clear, Wrap};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[derive(Debug)]
pub struct ValidationResultPopup {
    result: ValidationResult,
    scroll_offset: usize,
}

impl ValidationResultPopup {
    pub fn new(result: ValidationResult) -> Self {
        Self {
            result,
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

impl Popup for ValidationResultPopup {
    fn render(&mut self, frame: &mut Frame, area: Rect, app: &App) {
        let ui = &app.theme_manager.active().ui;

        let border_color = if self.result.is_valid() {
            ui.success()
        } else {
            ui.severity_color(DiagnosticSeverity::Error)
        };

        // Render the popup in the centered `area`
        frame.render_widget(Clear, area); // Clears the popup area to avoid overlap

        // popup border and title
        let block = Block::default()
            .title("Config Validation Results")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().fg(ui.popup_fg()).bg(ui.popup_bg()));

        let inner_area = block.inner(area);
        frame.render_widget(block, area); // Render block

        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Summary
                Constraint::Min(1),    // Details
                Constraint::Length(2), // Footer
            ])
            .split(inner_area);

        // Summary
        let summary_style = Style::default()
            .fg(border_color)
            .add_modifier(Modifier::BOLD);

        let summary = Paragraph::new(vec![
            Line::from(Span::styled(self.result.summary(), summary_style)),
            Line::from(""),
        ])
        .style(Style::default().bg(ui.popup_bg()));
        frame.render_widget(summary, chunks[0]);

        // Details
        let mut items = Vec::new();

        if !self.result.errors.is_empty() {
            items.push(Line::from(Span::styled(
                "ERRORS:",
                Style::default()
                    .fg(ui.severity_color(DiagnosticSeverity::Error))
                    .add_modifier(Modifier::BOLD),
            )));
            for (i, error) in self.result.errors.iter().enumerate() {
                items.push(Line::from(vec![
                    Span::styled(
                        format!("  {}. ", i + 1),
                        Style::default().fg(ui.hint_text()),
                    ),
                    Span::styled(
                        error,
                        Style::default().fg(ui.severity_color(DiagnosticSeverity::Error)),
                    ),
                ]));
            }
            items.push(Line::from(""));
        }

        if !self.result.warnings.is_empty() {
            items.push(Line::from(Span::styled(
                "WARNINGS:",
                Style::default()
                    .fg(ui.severity_color(DiagnosticSeverity::Warning))
                    .add_modifier(Modifier::BOLD),
            )));
            for (i, warning) in self.result.warnings.iter().enumerate() {
                items.push(Line::from(vec![
                    Span::styled(
                        format!("  {}. ", i + 1),
                        Style::default().fg(ui.hint_text()),
                    ),
                    Span::styled(
                        warning,
                        Style::default().fg(ui.severity_color(DiagnosticSeverity::Warning)),
                    ),
                ]));
            }
        }

        // Render details with scroll offset and wrap
        let paragraph = Paragraph::new(items)
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default().bg(ui.popup_bg()).fg(ui.popup_fg()))
            .wrap(Wrap { trim: true })
            .scroll((self.scroll_offset as u16, 0));
        frame.render_widget(paragraph, chunks[1]);

        // Footer
        let help = Paragraph::new("↑↓: Scroll | Enter/Esc: Close")
            .style(Style::default().fg(ui.hint_text()));
        frame.render_widget(help, chunks[2]);
    }

    fn get_popup_type(&self) -> PopupType {
        PopupType::Info
    }

    fn handle_input_action(&mut self, action: InputAction) -> PopupResult {
        match action {
            InputAction::ENTER => PopupResult::Affirmed,
            InputAction::ToggleActiveArea => PopupResult::Affirmed,
            InputAction::Popup(PopupAction::MoveCursor(dir)) => match dir {
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
