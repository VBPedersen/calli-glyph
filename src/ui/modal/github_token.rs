use crate::core::app::App;
use crate::input::actions::{InputAction, ModalAction};
use crate::ui::modal::components::text_field::TextField;
use crate::ui::modal::{Modal, ModalInputMode, ModalResponse};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

pub struct GithubTokenModal {
    field: TextField,
    message: Option<String>,
}

impl GithubTokenModal {
    pub fn new() -> Self {
        Self {
            field: TextField::new(),
            message: None,
        }
    }
}

fn fixed_centered(w: u16, h: u16, area: Rect) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    Rect {
        x: area.x + (area.width - w) / 2,
        y: area.y + (area.height - h) / 2,
        width: w,
        height: h,
    }
}

impl Modal for GithubTokenModal {
    fn handle_input(&mut self, action: InputAction, app: &mut App) -> ModalResponse {
        let InputAction::Modal(m) = action else {
            return ModalResponse::Consumed;
        };
        match m {
            ModalAction::Close => return ModalResponse::Close,
            ModalAction::Confirm => {
                let token = self.field.value().trim().to_string();
                if token.is_empty() {
                    self.message = Some("Paste a token first".into());
                } else {
                    match app.install_manager.set_github_token(Some(token)) {
                        Ok(()) => {
                            log_info!("[GithubToken] token set via dialog");
                            return ModalResponse::Close;
                        }
                        Err(e) => self.message = Some(format!("✗ {e}")),
                    }
                }
            }
            // Tab: remove the stored token
            ModalAction::NextTab => match app.install_manager.set_github_token(None) {
                Ok(()) => self.message = Some("✓ Stored token removed".into()),
                Err(e) => self.message = Some(format!("✗ {e}")),
            },
            other => {
                if self.field.handle(&other) {
                    self.message = None;
                }
            }
        }
        ModalResponse::Consumed
    }
    fn render(&mut self, frame: &mut Frame, app: &App) {
        let ui = &app.theme_manager.active().ui;
        let area = fixed_centered(64, 11, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" GitHub token ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ui.popup_border()))
            .style(Style::default().fg(ui.popup_fg()).bg(ui.popup_bg()));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let dim = Style::default().fg(ui.hint_text());
        let stored = if app.install_manager.has_github_token() {
            "set ✓"
        } else {
            "none"
        };

        // Show only the tail if the token is wider than the dialog
        let avail = inner.width.saturating_sub(12) as usize;
        let masked = self.field.masked(1);
        let skip = masked.chars().count().saturating_sub(avail);
        let shown: String = masked.chars().skip(skip).collect();

        let mut lines = vec![
            Line::from(Span::styled("Authenticates grammar-repo clones", dim)),
            Line::from(Span::styled("(avoids anonymous GitHub rate limits).", dim)),
            Line::raw(""),
            Line::from(vec![Span::styled("Stored token: ", dim), Span::raw(stored)]),
            Line::raw(""),
            Line::from(vec![
                Span::raw("Token: "),
                Span::styled(shown, Style::default().fg(ui.accent_blue())),
                Span::styled(
                    "█",
                    Style::default()
                        .fg(ui.accent_blue())
                        .add_modifier(Modifier::SLOW_BLINK),
                ),
            ]),
            Line::from(Span::styled(
                format!("       ({} chars)", self.field.len()),
                dim,
            )),
        ];
        lines.push(match &self.message {
            Some(m) => Line::from(Span::styled(
                m.clone(),
                Style::default().fg(ui.accent_blue()),
            )),
            None => Line::raw(""),
        });
        lines.push(Line::from(Span::styled(
            "Enter: save   Tab: remove stored   Esc: cancel",
            dim,
        )));
        frame.render_widget(Paragraph::new(lines), inner);
    }

    fn input_mode(&self) -> ModalInputMode {
        ModalInputMode::Text
    }

    fn debug_name(&self) -> &'static str {
        "GithubToken"
    }
}
