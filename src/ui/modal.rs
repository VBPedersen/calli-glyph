pub mod lang_modal;

use crate::core::app::App;
use crate::input::actions::InputAction;
use ratatui::Frame;

pub trait Modal {
    fn handle_input(&mut self, action: InputAction, app: &mut App) -> ModalResponse;
    fn render(&self, frame: &mut Frame, app: &App);
}

pub enum ModalResponse {
    Consumed,
    Close,
}
