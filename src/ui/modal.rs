pub mod browser;
pub mod components;
pub mod github_token;
pub mod lang_modal;
pub mod main_menu;
pub mod settings;

use crate::core::app::App;
use crate::input::actions::InputAction;
use ratatui::Frame;

pub trait Modal {
    fn handle_input(&mut self, action: InputAction, app: &mut App) -> ModalResponse;
    fn render(&mut self, frame: &mut Frame, app: &App);
    fn input_mode(&self) -> ModalInputMode {
        ModalInputMode::Normal
    }
    fn debug_name(&self) -> &'static str {
        "modal"
    } // used by the debugger
}

pub enum ModalResponse {
    Consumed,
    Close,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ModalInputMode {
    #[default]
    Normal,
    Text,
    CaptureKey,
}
