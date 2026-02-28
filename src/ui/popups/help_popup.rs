use crate::core::help_registry::HelpRegistry;
use crate::input::actions::{Direction, InputAction, PopupAction};
use crate::ui::popups::popup::{Popup, PopupResult, PopupType};
use ratatui::layout::Rect;
use ratatui::widgets::ListState;
use ratatui::Frame;
use std::sync::Arc;

pub struct HelpPopup {
    registry: Arc<HelpRegistry>,
    scroll_offset: usize,
    search_query: String,
    search_active: bool,
    filtered_pages: Vec<usize>,   // indices into registry
    selected_page: Option<usize>, // optional single indices into registry
}

impl HelpPopup {
    /// Open the popup in browse mode. all pages listed, nothing pre-selected.
    pub fn browse(registry: Arc<HelpRegistry>) -> Self {
        let filtered: Vec<usize> = (0..registry.len()).collect();
        Self {
            registry,
            scroll_offset: 0,
            search_query: String::new(),
            search_active: false,
            filtered_pages: filtered,
            selected_page: None,
        }
    }

    /// Open the popup in focused mode on a specific topic id (e.g. "debug").
    /// Falls back to browse mode, if the id is not found.
    pub fn focused(registry: Arc<HelpRegistry>, topic: &str) -> Self {
        let mut popup = Self::browse(registry);

        // Find the index of the topic in the full registry and locate it in filtered.
        if let Some(reg_idx) = popup
            .registry
            .get_all()
            .iter()
            .position(|p| p.id == topic.to_lowercase())
        {}

        popup
    }
}

//TODO
impl Popup for HelpPopup {
    fn render(&self, frame: &mut Frame, area: Rect) {
        todo!()
    }

    fn get_popup_type(&self) -> PopupType {
        PopupType::Info
    }

    fn handle_input_action(&mut self, action: InputAction) -> PopupResult {
        match action {
            /*InputAction::ENTER => {},
            InputAction::Popup(PopupAction::MoveCursor(Direction::Up)) => {}*/
            _ => PopupResult::None,
        }
    }
}
