use crate::core::app::App;
use crate::input::actions::InputAction;
use crate::menu::{MenuAction, MenuEntry, MenuRegistry};
use crate::plugins::plugin_registry::PluginManager;
use crate::ui::modal::{Modal, ModalResponse};
use crate::ui::ui::centered_rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Clear};
use ratatui::Frame;

pub struct MainMenuModal {
    entries: Vec<MenuEntry>,
    categories: Vec<String>,
    tab: usize,
    cursor: usize,
    scroll: usize,
}

impl MainMenuModal {
    /// Snapshot of the registry, minus commands that aren't currently
    /// registered (e.g. their plugin is disabled in config).
    pub fn new(registry: &MenuRegistry, plugins: &PluginManager) -> Self {
        let entries = registry
            .entries()
            .iter()
            .filter(|e| match &e.action {
                MenuAction::Command(name) => plugins.command_registry().get_handler(name).is_some(),
                MenuAction::Callback(_) => true,
            })
            .cloned()
            .collect();
        Self::from_entries(entries)
    }

    fn from_entries(entries: Vec<MenuEntry>) -> Self {
        let mut categories: Vec<String> = Vec::new();
        for e in &entries {
            if !categories.contains(&e.category) {
                categories.push(e.category.clone());
            }
        }
        Self {
            entries,
            categories,
            tab: 0,
            cursor: 0,
            scroll: 0,
        }
    }
}

impl Modal for MainMenuModal {
    fn handle_input(&mut self, action: InputAction, app: &mut App) -> ModalResponse {
        todo!()
    }

    fn render(&mut self, frame: &mut Frame, app: &App) {
        let ui = &app.theme_manager.active().ui;
        let area = centered_rect(60, 60, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Menu ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ui.popup_border()))
            .style(Style::default().fg(ui.popup_fg()).bg(ui.popup_bg()));
        let inner = block.inner(area);
        frame.render_widget(block, area);
    }
}
