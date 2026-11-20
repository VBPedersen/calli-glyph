use crate::core::app::App;
use crate::input::actions::{InputAction, ModalAction};
use crate::menu::{MenuAction, MenuEntry, MenuRegistry};
use crate::plugins::plugin_registry::PluginManager;
use crate::ui::modal::{Modal, ModalResponse};
use crate::ui::popups::error_popup::ErrorPopup;
use crate::ui::ui::centered_rect;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
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

    fn current(&self) -> Vec<&MenuEntry> {
        match self.categories.get(self.tab) {
            Some(cat) => self.entries.iter().filter(|e| &e.category == cat).collect(),
            None => vec![],
        }
    }

    fn change_tab(&mut self, delta: isize) {
        let n = self.categories.len() as isize;
        if n == 0 {
            return;
        }
        self.tab = (self.tab as isize + delta).rem_euclid(n) as usize;
        self.cursor = 0;
        self.scroll = 0;
    }

    fn move_cursor(&mut self, delta: isize) {
        let len = self.current().len();
        if len == 0 {
            return;
        }
        self.cursor = (self.cursor as isize + delta).clamp(0, len as isize - 1) as usize;
    }

    fn run(action: MenuAction, app: &mut App) {
        match action {
            MenuAction::Command(name) => {
                if let Err(e) = app.execute_plugin_command(&name, vec![]) {
                    app.open_popup(Box::new(ErrorPopup::new("Menu action failed", e)));
                }
            }
            MenuAction::Callback(f) => f(app),
        }
    }
}

impl Modal for MainMenuModal {
    fn handle_input(&mut self, action: InputAction, app: &mut App) -> ModalResponse {
        let InputAction::Modal(modal_action) = action else {
            return ModalResponse::Consumed;
        };

        match modal_action {
            ModalAction::Close => return ModalResponse::Close,
            ModalAction::ScrollUp => self.move_cursor(-1),
            ModalAction::ScrollDown => self.move_cursor(1),
            ModalAction::NextTab => self.change_tab(1),
            ModalAction::PrevTab => self.change_tab(-1),
            ModalAction::Confirm => {
                if let Some(action) = self.current().get(self.cursor).map(|e| e.action.clone()) {
                    // Run, then close: anything the action pushes (modals,
                    // popups) must end up on top, not under the menu.
                    Self::run(action, app);
                    return ModalResponse::Close;
                }
            }
            ModalAction::Action(c) if c != '\0' => {
                if let Some(action) = self
                    .entries
                    .iter()
                    .find(|e| e.key == Some(c))
                    .map(|e| e.action.clone())
                {
                    Self::run(action, app);
                    return ModalResponse::Close;
                }
            }
            _ => {}
        }
        ModalResponse::Consumed
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

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // tabs
                Constraint::Length(1), // spacer
                Constraint::Min(1),    // entries
                Constraint::Length(1), // description
                Constraint::Length(1), // hint
            ])
            .split(inner);

        // Tabs
        let mut tab_spans = Vec::new();
        for (i, cat) in self.categories.iter().enumerate() {
            let style = if i == self.tab {
                ui.list_highlight_style().add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(ui.hint_text())
            };
            tab_spans.push(Span::styled(format!(" {cat} "), style));
            tab_spans.push(Span::raw(" "));
        }
        frame.render_widget(Paragraph::new(Line::from(tab_spans)), chunks[0]);

        // Keep cursor in view
        let visible = chunks[2].height as usize;
        let len = self.current().len();
        if visible > 0 {
            if self.cursor >= self.scroll + visible {
                self.scroll = self.cursor + 1 - visible;
            }
            if self.cursor < self.scroll {
                self.scroll = self.cursor;
            }
        }

        let width = chunks[2].width as usize;
        let entries = self.current();
        let lines: Vec<Line> = if entries.is_empty() {
            vec![Line::from(Span::styled(
                "  Nothing here yet",
                Style::default().fg(ui.hint_text()),
            ))]
        } else {
            entries
                .iter()
                .enumerate()
                .skip(self.scroll)
                .take(visible)
                .map(|(i, e)| {
                    let hint = e.key.map(|k| format!("[{k}]")).unwrap_or_default();
                    let pad = width
                        .saturating_sub(2 + e.title.chars().count() + hint.chars().count() + 1);
                    let mut line = Line::from(vec![
                        Span::raw(format!("  {}", e.title)),
                        Span::raw(" ".repeat(pad)),
                        Span::styled(hint, Style::default().fg(ui.hint_text())),
                    ]);
                    if i == self.cursor {
                        line = line.style(ui.list_highlight_style().add_modifier(Modifier::BOLD));
                    }
                    line
                })
                .collect()
        };
        frame.render_widget(Paragraph::new(lines), chunks[2]);

        // Description of selection
        if let Some(e) = entries.get(self.cursor) {
            frame.render_widget(
                Paragraph::new(format!("  {}", e.description))
                    .style(Style::default().fg(ui.hint_text())),
                chunks[3],
            );
        }
        let _ = len;

        frame.render_widget(
            Paragraph::new("Tab/S-Tab: category  ↑↓/jk: move  Enter: run  Esc: close")
                .style(Style::default().fg(ui.hint_text())),
            chunks[4],
        );
    }
}

#[cfg(test)]
mod unit_main_menu_tests {
    use super::*;

    fn modal() -> MainMenuModal {
        MainMenuModal::from_entries(vec![
            MenuEntry::command("A", "a1", "", "x"),
            MenuEntry::command("A", "a2", "", "x"),
            MenuEntry::command("B", "b1", "", "x"),
        ])
    }

    #[test]
    fn categories_are_derived_from_entries() {
        assert_eq!(modal().categories, vec!["A", "B"]);
    }

    #[test]
    fn cursor_is_clamped_within_current_tab() {
        let mut m = modal();
        m.move_cursor(5);
        assert_eq!(m.cursor, 1); // tab A has 2 entries
        m.move_cursor(-5);
        assert_eq!(m.cursor, 0);
    }

    #[test]
    fn change_tab_wraps_and_resets_cursor() {
        let mut m = modal();
        m.move_cursor(1);
        m.change_tab(1);
        assert_eq!((m.tab, m.cursor), (1, 0));
        m.change_tab(1);
        assert_eq!(m.tab, 0);
        m.change_tab(-1);
        assert_eq!(m.tab, 1);
    }
}
