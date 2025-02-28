use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::Frame;
use std::fmt::{Debug, Formatter};
pub trait Popup {
    fn render(&self, frame: &mut Frame, area: Rect);
    fn handle_key_input(&mut self, key: KeyEvent) -> PopupResult;
    fn get_popup_type(&self) -> PopupType;
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
