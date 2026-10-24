use crate::config::settings::{Apply, SettingDef, SETTINGS};
use crate::config::{settings, Config};
use crate::core::app::App;
use crate::errors::error::AppError;
use crate::input::actions::{InputAction, ModalAction};
use crate::ui::modal::{Modal, ModalResponse};
use crate::ui::ui::centered_rect;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

pub struct SettingsModal {
    categories: Vec<&'static str>,
    tab: usize,
    cursor: usize,
    scroll: usize,
    message: Option<String>,
}

impl SettingsModal {
    pub fn new() -> SettingsModal {
        Self {
            categories: settings::categories(),
            tab: 0,
            cursor: 0,
            scroll: 0,
            message: None,
        }
    }

    /// Generates rows of settings by categories from config/settings
    fn rows(&self) -> Vec<&'static SettingDef> {
        match self.categories.get(self.tab) {
            Some(cat) => SETTINGS.iter().filter(|d| d.category == *cat).collect(),
            None => vec![],
        }
    }

    /// Changes the tab by delta amount
    fn change_tab(&mut self, delta: isize) {
        let n = self.categories.len() as isize;
        if n == 0 {
            return;
        }
        self.tab = (self.tab as isize + delta).rem_euclid(n) as usize;
        self.cursor = 0;
        self.scroll = 0;
        self.message = None;
    }

    /// moves the cursor on setting rows
    fn move_cursor(&mut self, delta: isize) {
        let len = self.rows().len();
        if len == 0 {
            return;
        }
        self.cursor = (self.cursor as isize + delta).clamp(0, len as isize - 1) as usize;
        self.message = None;
    }

    /// Adjusts the value of the row item selected in direction dir
    fn adjust(&mut self, app: &mut App, dir: i64) {
        let Some(def) = self.rows().get(self.cursor).copied() else {
            return;
        };
        let Some(root) = settings::snapshot(&app.config) else {
            return;
        };
        let Some(cur) = settings::lookup(&root, def.key) else {
            self.message = Some(format!("✗ '{}' not found in config", def.key));
            return;
        };
        if let Some(new) = settings::step_value(&def.kind, cur, dir) {
            self.commit(app, def, Some(new));
        }
    }

    /// Resets setting value to default
    fn reset(&mut self, app: &mut App) {
        if let Some(def) = self.rows().get(self.cursor).copied() {
            self.commit(app, def, None);
        }
    }

    /// commits a change to write via settings::write_setting and conclude on result whether succeeded and whether a restart is needed
    fn commit(&mut self, app: &mut App, def: &'static SettingDef, value: Option<toml_edit::Value>) {
        let result = Config::get_config_path()
            .map_err(|e| AppError::from(e))
            .and_then(|p| settings::write_setting(&p, def.key, value));
        match result {
            Ok(()) => {
                app.reload_config();
                self.message = Some(match def.apply {
                    Apply::Live => "✓ Saved".to_string(),
                    Apply::Restart => "✓ Saved, restart to apply".to_string(),
                });
            }
            Err(e) => self.message = Some(format!("✗ {e}")),
        }
    }
}

impl Modal for SettingsModal {
    fn handle_input(&mut self, action: InputAction, app: &mut App) -> ModalResponse {
        let InputAction::Modal(m) = action else {
            return ModalResponse::Consumed;
        };
        match m {
            ModalAction::Close => return ModalResponse::Close,
            ModalAction::ScrollUp => self.move_cursor(-1),
            ModalAction::ScrollDown => self.move_cursor(1),
            ModalAction::NextTab => self.change_tab(1),
            ModalAction::PrevTab => self.change_tab(-1),
            ModalAction::Confirm | ModalAction::Action('l') => self.adjust(app, 1),
            ModalAction::Action('h') => self.adjust(app, -1),
            ModalAction::Action('r') => self.reset(app),
            ModalAction::Action('o') => {
                // escape hatch: edit the raw file
                if let Ok(path) = Config::get_config_path() {
                    app.open_file(path, None);
                    return ModalResponse::Close;
                }
            }
            _ => {}
        }
        ModalResponse::Consumed
    }

    fn render(&mut self, frame: &mut Frame, app: &App) {
        let ui = &app.theme_manager.active().ui;
        let area = centered_rect(60, 70, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Settings ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ui.popup_border()))
            .style(Style::default().fg(ui.popup_fg()).bg(ui.popup_bg()));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // tabs
                Constraint::Length(1),
                Constraint::Min(1),    // rows
                Constraint::Length(1), // description
                Constraint::Length(1), // message
                Constraint::Length(1), // hint
            ])
            .split(inner);

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

        let visible = chunks[2].height as usize;
        if visible > 0 {
            if self.cursor >= self.scroll + visible {
                self.scroll = self.cursor + 1 - visible;
            }
            if self.cursor < self.scroll {
                self.scroll = self.cursor;
            }
        }

        let snap = settings::snapshot(&app.config);
        let width = chunks[2].width as usize;
        let rows = self.rows();
        let lines: Vec<Line> = rows
            .iter()
            .enumerate()
            .skip(self.scroll)
            .take(visible)
            .map(|(i, def)| {
                let value = snap
                    .as_ref()
                    .and_then(|r| settings::lookup(r, def.key))
                    .map(|v| format!("‹ {} ›", settings::display(v)))
                    .unwrap_or_else(|| "—".to_string());
                let label = if def.apply == Apply::Restart {
                    format!("  {} ↻", def.label)
                } else {
                    format!("  {}", def.label)
                };
                let pad = width.saturating_sub(label.chars().count() + value.chars().count() + 1);
                let mut line = Line::from(vec![
                    Span::raw(label),
                    Span::raw(" ".repeat(pad)),
                    Span::raw(value),
                ]);
                if i == self.cursor {
                    line = line.style(ui.list_highlight_style().add_modifier(Modifier::BOLD));
                }
                line
            })
            .collect();
        frame.render_widget(Paragraph::new(lines), chunks[2]);

        if let Some(def) = rows.get(self.cursor) {
            frame.render_widget(
                Paragraph::new(format!("  {}", def.description))
                    .style(Style::default().fg(ui.hint_text())),
                chunks[3],
            );
        }
        if let Some(msg) = &self.message {
            frame.render_widget(
                Paragraph::new(format!("  {msg}")).style(Style::default().fg(ui.accent_blue())),
                chunks[4],
            );
        }
        frame.render_widget(
            Paragraph::new("h/l ←/→: change  Enter: next  r: reset  o: open file  Tab: category  Esc: close  (↻ restart)")
                .style(Style::default().fg(ui.hint_text())),
            chunks[5],
        );
    }
}
