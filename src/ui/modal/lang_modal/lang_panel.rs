use crate::core::app::App;
use crate::input::actions::InputAction;
use crate::language::manager::LanguageManager;
use crate::ui::modal::{Modal, ModalResponse};
use crate::ui::ui::centered_rect;
use ratatui::widgets::Clear;
use ratatui::Frame;

/// Language panel menu for managing the language related systems via modal
pub struct LangPanel {
    pub selected_diagnostic: Option<usize>, // diagnostic currently selected
}

impl LangPanel {
    pub fn new(lang: &LanguageManager) -> LangPanel {
        Self {
            selected_diagnostic: None,
        }
    }
}

impl Modal for LangPanel {
    fn handle_input(&mut self, action: InputAction, app: &mut App) -> ModalResponse {
        match action {
            InputAction::ENTER => {
                /* if let Some(diag) = self.selected_diagnostic() {
                    app.editor.jump_to_line(diag.line as usize);
                }*/
                ModalResponse::Close
            }
            InputAction::Modal(_) => ModalResponse::Consumed,
            _ => ModalResponse::Consumed,
        }
    }

    fn render(&self, frame: &mut Frame, app: &App) {
        let diags = &app.language.diagnostics;
        // render on top of whatever is already drawn
        let area = centered_rect(80, 60, frame.area());
        frame.render_widget(Clear, area);
    }
}
