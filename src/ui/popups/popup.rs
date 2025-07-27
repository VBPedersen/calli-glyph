use crate::input::input_action::InputAction;
use ratatui::layout::Rect;
use ratatui::Frame;
use std::fmt::{Debug, Formatter};

pub trait Popup {
    fn render(&self, frame: &mut Frame, area: Rect);
    fn get_popup_type(&self) -> PopupType;
    ///function to handle input action on popup,
    /// responsible for dispatching action to correct internal method.
    fn handle_input_action(&mut self, action: InputAction) -> PopupResult;
}

impl Debug for dyn Popup {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Popup trait")
    }
}

#[derive(Debug, PartialEq)]
pub enum PopupResult {
    None,
    Bool(bool),
    String(String),
    Affirmed,
}

pub enum PopupType {
    None,
    Confirmation,
    Warning,
    Error,
}
