use std::fmt::{Debug, Formatter};
use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
pub trait Popup{
    fn render(&self, frame: &mut Frame, area:Rect);
    fn handle_key_input(&mut self, key:KeyEvent) -> PopupResult;
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
}

pub enum PopupType {
    None,
    Confirmation,
    Warning,
}