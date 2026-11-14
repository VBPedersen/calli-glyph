use crate::config::settings;
use crate::config::theme_edit::{self, Row, RowKind, ROWS};
use crate::core::app::App;
use crate::input::actions::{InputAction, ModalAction};
use crate::theme::color::parse_color;
use crate::theme::ThemeManager;
use crate::ui::modal::{Modal, ModalInputMode, ModalResponse};
use crate::ui::ui::centered_rect;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use std::path::PathBuf;
use crate::ui::modal::components::text_field::TextField;

pub struct ThemeEditorModal {
    name: String,
    path: PathBuf,
    previous_active: String,
    keep_active: bool,
    syntax_dirty: bool,
    /// File contents at open, for per-row revert.
    original: toml::Value,
    groups: Vec<&'static str>,
    tab: usize,
    cursor: usize,
    scroll: usize,
    editing: Option<TextField>,
    confirm_reset: bool,
    message: Option<String>,
}

fn to_edit_value(v: &toml::Value) -> Option<toml_edit::Value> {
    match v {
        toml::Value::String(s) => Some(s.as_str().into()),
        toml::Value::Boolean(b) => Some((*b).into()),
        toml::Value::Integer(i) => Some((*i).into()),
        _ => None,
    }
}

impl ThemeEditorModal {
    pub fn new(name: String, path: PathBuf, previous_active: String) -> Self {
        let original = std::fs::read_to_string(&path)
            .ok()
            .and_then(|t| t.parse::<toml::Value>().ok())
            .unwrap_or_else(|| toml::Value::Table(Default::default()));
        Self {
            name,
            path,
            previous_active,
            keep_active: false,
            syntax_dirty: false,
            original,
            groups: theme_edit::groups(),
            tab: 0,
            cursor: 0,
            scroll: 0,
            editing: None,
            confirm_reset: false,
            message: None,
        }
    }

    fn rows(&self) -> Vec<&'static Row> {
        match self.groups.get(self.tab) {
            Some(g) => ROWS.iter().filter(|r| r.group == *g).collect(),
            None => vec![],
        }
    }

    fn current_row(&self) -> Option<&'static Row> {
        self.rows().get(self.cursor).copied()
    }

    fn change_tab(&mut self, delta: isize) {
        let n = self.groups.len() as isize;
        if n == 0 {
            return;
        }
        self.tab = (self.tab as isize + delta).rem_euclid(n) as usize;
        self.cursor = 0;
        self.scroll = 0;
        self.message = None;
    }

    fn move_cursor(&mut self, delta: isize) {
        let len = self.rows().len();
        if len == 0 {
            return;
        }
        self.cursor = (self.cursor as isize + delta).clamp(0, len as isize - 1) as usize;
        self.message = None;
    }

    /// Resolved (defaults included) value of `key` in the edited theme.
    fn resolved(&self, app: &App, key: &str) -> Option<toml::Value> {
        let snap = settings::snapshot(app.theme_manager.get(&self.name)?)?;
        settings::lookup(&snap, key).cloned()
    }

    /// Writes (or removes, with None) a key in the theme file, then reloads it live.
    fn write(&mut self, app: &mut App, key: &str, value: Option<toml_edit::Value>) -> bool {
        match settings::write_setting(&self.path, key, value) {
            Ok(()) => {
                app.theme_manager.rescan();
                if key.starts_with("syntax.") {
                    self.syntax_dirty = true;
                    app.refresh_syntax_theme();
                }
                self.message = Some("✓ Saved".into());
                true
            }
            Err(e) => {
                self.message = Some(format!("✗ {e}"));
                false
            }
        }
    }

    fn adjust(&mut self, app: &mut App, dir: i64) {
        let Some(row) = self.current_row() else { return };
        let RowKind::Setting(kind) = &row.kind else { return };
        let Some(cur) = self.resolved(app, row.key) else { return };
        if let Some(new) = settings::step_value(kind, &cur, dir) {
            self.write(app, row.key, Some(new));
        }
    }

    fn revert(&mut self, app: &mut App) {
        let Some(row) = self.current_row() else { return };
        let value = settings::lookup(&self.original, row.key).and_then(to_edit_value);
        self.write(app, row.key, value);
    }

    fn clear(&mut self, app: &mut App) {
        if let Some(row) = self.current_row() {
            self.write(app, row.key, None);
        }
    }

    fn close(&mut self, app: &mut App) -> ModalResponse {
        if !self.keep_active {
            let _ = app.theme_manager.set_active(&self.previous_active);
            if self.syntax_dirty {
                app.refresh_syntax_theme();
            }
        }
        ModalResponse::Close
    }
}

impl Modal for ThemeEditorModal {
    fn input_mode(&self) -> ModalInputMode {
        if self.editing.is_some() { ModalInputMode::Text } else { ModalInputMode::Normal }
    }
    fn debug_name(&self) -> &'static str { "ThemeEditor" }

    fn handle_input(&mut self, action: InputAction, app: &mut App) -> ModalResponse {
        let InputAction::Modal(m) = action else { return ModalResponse::Consumed };

        // Typing a color value
        if let Some(mut field) = self.editing.take() {
            match m {
                ModalAction::Close => {} // cancel the edit only
                ModalAction::Confirm => {
                    let key = self.current_row().map(|r| r.key);
                    match (key, theme_edit::normalize_color(field.value())) {
                        (Some(key), Some(v)) => {
                            if !self.write(app, key, Some(v.into())) {
                                self.editing = Some(field);
                            }
                        }
                        _ => {
                            self.message =
                                Some("✗ not a color: use #rrggbb, #rgb or a name like 'red'".into());
                            self.editing = Some(field);
                        }
                    }
                }
                other => {
                    field.handle(&other);
                    self.editing = Some(field);
                }
            }
            return ModalResponse::Consumed;
        }

        if !matches!(m, ModalAction::Action('R')) {
            self.confirm_reset = false;
        }

        match m {
            ModalAction::Close => return self.close(app),
            ModalAction::ScrollUp => self.move_cursor(-1),
            ModalAction::ScrollDown => self.move_cursor(1),
            ModalAction::NextTab => self.change_tab(1),
            ModalAction::PrevTab => self.change_tab(-1),
            ModalAction::Confirm => {
                if let Some(row) = self.current_row() {
                    match row.kind {
                        RowKind::Color { .. } => {
                            let cur = self
                                .resolved(app, row.key)
                                .map(|v| settings::display(&v))
                                .unwrap_or_default();
                            self.editing = Some(TextField::with_value(&cur));
                            self.message = None;
                        }
                        RowKind::Setting(_) => self.adjust(app, 1),
                    }
                }
            }
            ModalAction::Action('l') => self.adjust(app, 1),
            ModalAction::Action('h') => self.adjust(app, -1),
            ModalAction::Action('u') => self.revert(app),
            ModalAction::Action('x') => self.clear(app),
            ModalAction::Action('a') => match app.apply_theme(&self.name) {
                Ok(()) => {
                    self.keep_active = true;
                    self.message = Some("✓ Now your active theme".into());
                }
                Err(e) => self.message = Some(format!("✗ {e}")),
            },
            ModalAction::Action('R') => {
                if !ThemeManager::is_builtin(&self.name) {
                    self.message = Some("Only built-in themes can be reset".into());
                } else if !self.confirm_reset {
                    self.confirm_reset = true;
                    self.message =
                        Some(format!("Press R again to restore the shipped '{}' theme", self.name));
                } else {
                    self.confirm_reset = false;
                    match app.theme_manager.reset_builtin(&self.name) {
                        Ok(()) => {
                            self.syntax_dirty = true;
                            app.refresh_syntax_theme();
                            self.message = Some("✓ Restored".into());
                        }
                        Err(e) => self.message = Some(format!("✗ {e}")),
                    }
                }
            }
            _ => {}
        }
        ModalResponse::Consumed
    }

    fn render(&mut self, frame: &mut Frame, app: &App) {
        let ui = &app.theme_manager.active().ui;
        let area = centered_rect(66, 80, frame.area());
        frame.render_widget(Clear, area);

        let tag = if ThemeManager::is_builtin(&self.name) { " (built-in)" } else { "" };
        let block = Block::default()
            .title(format!(" Theme: {}{tag} ", self.name))
            .borders(Borders::ALL)
            .border_set(app.theme_manager.active().layout.border_style.to_ratatui())
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
                Constraint::Length(1), // detail / editor
                Constraint::Length(1), // message
                Constraint::Length(1), // hint
            ])
            .split(inner);
        let dim = Style::default().fg(ui.hint_text());

        let mut tab_spans = Vec::new();
        for (i, g) in self.groups.iter().enumerate() {
            let style = if i == self.tab {
                ui.list_highlight_style().add_modifier(Modifier::BOLD)
            } else {
                dim
            };
            tab_spans.push(Span::styled(format!(" {g} "), style));
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

        let snap = app.theme_manager.get(&self.name).and_then(settings::snapshot);
        let width = chunks[2].width as usize;
        let rows = self.rows();
        let lines: Vec<Line> = rows
            .iter()
            .enumerate()
            .skip(self.scroll)
            .take(visible)
            .map(|(i, row)| {
                let value = snap.as_ref().and_then(|s| settings::lookup(s, row.key)).map(settings::display);
                let (text, swatch) = match (&row.kind, value) {
                    (RowKind::Color { .. }, Some(v)) => {
                        let c = parse_color(&v);
                        (v, Some(c))
                    }
                    (RowKind::Color { .. }, None) => ("(unset)".to_string(), None),
                    (RowKind::Setting(_), Some(v)) => (format!("‹ {v} ›"), None),
                    (RowKind::Setting(_), None) => ("—".to_string(), None),
                };
                let label = format!("  {}", row.label);
                let used = label.chars().count() + text.chars().count() + if swatch.is_some() { 3 } else { 0 };
                let mut spans = vec![
                    Span::raw(label),
                    Span::raw(" ".repeat(width.saturating_sub(used + 1))),
                    Span::raw(text),
                ];
                if let Some(c) = swatch {
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled("██", Style::default().fg(c)));
                }
                let mut line = Line::from(spans);
                if i == self.cursor {
                    line = line.style(ui.list_highlight_style().add_modifier(Modifier::BOLD));
                }
                line
            })
            .collect();
        frame.render_widget(Paragraph::new(lines), chunks[2]);

        let detail = match &self.editing {
            Some(f) => {
                let mut spans = vec![
                    Span::raw("  Edit: "),
                    Span::styled(f.value().to_string(), Style::default().fg(ui.accent_blue())),
                    Span::styled("█ ", Style::default().fg(ui.accent_blue())),
                ];
                match theme_edit::normalize_color(f.value()) {
                    Some(v) => spans.push(Span::styled("██", Style::default().fg(parse_color(&v)))),
                    None => spans.push(Span::styled("✗", Style::default().fg(ui.accent_red()))),
                }
                Line::from(spans)
            }
            None => Line::from(Span::styled(
                format!("  {}", self.current_row().map(|r| r.key).unwrap_or("")),
                dim,
            )),
        };
        frame.render_widget(Paragraph::new(detail), chunks[3]);

        if let Some(msg) = &self.message {
            frame.render_widget(
                Paragraph::new(format!("  {msg}")).style(Style::default().fg(ui.accent_blue())),
                chunks[4],
            );
        }
        let hint = if self.editing.is_some() {
            "Enter: save   Esc: cancel   (#rrggbb, #rgb or a color name)"
        } else {
            "Enter: edit  h/l: change  u: revert  x: clear  a: use theme  R: reset  Esc: close"
        };
        frame.render_widget(Paragraph::new(hint).style(dim), chunks[5]);
    }
}