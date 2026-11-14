use crate::config::theme_edit;
use crate::core::app::App;
use crate::input::actions::{InputAction, ModalAction};
use crate::theme::ThemeManager;
use crate::ui::modal::{Modal, ModalInputMode, ModalResponse};
use crate::ui::ui::centered_rect;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use crate::ui::modal::components::text_field::TextField;

pub struct ThemesModal {
    names: Vec<String>,
    cursor: usize,
    scroll: usize,
    /// Active theme when opened: marked with ● and restored on cancel.
    original: String,
    naming: Option<TextField>,
    message: Option<String>,
}

impl ThemesModal {
    pub fn new(app: &App) -> Self {
        let names: Vec<String> = app.theme_manager.names().into_iter().map(String::from).collect();
        let original = app.theme_manager.active_name().to_string();
        let cursor = names.iter().position(|n| *n == original).unwrap_or(0);
        Self { names, cursor, scroll: 0, original, naming: None, message: None }
    }

    pub fn start_naming(mut self) -> Self {
        self.naming = Some(TextField::new());
        self
    }

    fn selected(&self) -> Option<&String> {
        self.names.get(self.cursor)
    }

    fn move_cursor(&mut self, delta: isize, app: &mut App) {
        if self.names.is_empty() {
            return;
        }
        self.cursor = (self.cursor as isize + delta).clamp(0, self.names.len() as isize - 1) as usize;
        self.message = None;
        if let Some(n) = self.selected() {
            let _ = app.theme_manager.set_active(n); // preview only
        }
    }

    /// Copies the selected theme to a new user theme; returns its name.
    fn create(&self, app: &mut App, name: &str) -> Result<String, String> {
        let src_name = self.selected().cloned().ok_or_else(|| "no theme selected".to_string())?;
        let src = app.theme_manager.ensure_theme_file(&src_name)?;
        let dir = app.theme_manager.themes_dir().to_path_buf();
        let dest = theme_edit::duplicate_theme(&src, &dir, name).map_err(|e| e.to_string())?;
        app.theme_manager.rescan();
        dest.file_stem()
            .and_then(|s| s.to_str())
            .map(String::from)
            .ok_or_else(|| "bad theme file name".to_string())
    }
}

impl Modal for ThemesModal {
    fn handle_input(&mut self, action: InputAction, app: &mut App) -> ModalResponse {
        let InputAction::Modal(m) = action else { return ModalResponse::Consumed };

        if let Some(mut field) = self.naming.take() {
            match m {
                ModalAction::Close => {} // cancel naming, stay in the browser
                ModalAction::Confirm => match self.create(app, field.value()) {
                    Ok(stem) => {
                        app.open_theme_editor(&stem, self.original.clone());
                        return ModalResponse::Close; // editor stays on the stack
                    }
                    Err(e) => {
                        self.message = Some(format!("✗ {e}"));
                        self.naming = Some(field);
                    }
                },
                other => {
                    field.handle(&other);
                    self.naming = Some(field);
                }
            }
            return ModalResponse::Consumed;
        }

        match m {
            ModalAction::Close => {
                let _ = app.theme_manager.set_active(&self.original);
                return ModalResponse::Close;
            }
            ModalAction::ScrollUp => self.move_cursor(-1, app),
            ModalAction::ScrollDown => self.move_cursor(1, app),
            ModalAction::Confirm => {
                if let Some(name) = self.selected().cloned() {
                    match app.apply_theme(&name) {
                        Ok(()) => return ModalResponse::Close,
                        Err(e) => self.message = Some(format!("✗ {e}")),
                    }
                }
            }
            ModalAction::Action('e') => {
                if let Some(name) = self.selected().cloned() {
                    app.open_theme_editor(&name, self.original.clone());
                    return ModalResponse::Close;
                }
            }
            ModalAction::Action('n') => self.naming = Some(TextField::new()),
            _ => {}
        }
        ModalResponse::Consumed
    }
    fn render(&mut self, frame: &mut Frame, app: &App) {
        let ui = &app.theme_manager.active().ui;
        let area = centered_rect(50, 60, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Themes ")
            .borders(Borders::ALL)
            .border_set(app.theme_manager.active().layout.border_style.to_ratatui())
            .border_style(Style::default().fg(ui.popup_border()))
            .style(Style::default().fg(ui.popup_fg()).bg(ui.popup_bg()));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1), Constraint::Length(1)])
            .split(inner);

        let visible = chunks[0].height as usize;
        if visible > 0 {
            if self.cursor >= self.scroll + visible {
                self.scroll = self.cursor + 1 - visible;
            }
            if self.cursor < self.scroll {
                self.scroll = self.cursor;
            }
        }

        let dim = Style::default().fg(ui.hint_text());
        let lines: Vec<Line> = self
            .names
            .iter()
            .enumerate()
            .skip(self.scroll)
            .take(visible)
            .map(|(i, n)| {
                let marker = if *n == self.original { "● " } else { "  " };
                let display = app.theme_manager.get(n).map(|t| t.meta.name.clone()).unwrap_or_default();
                let tag = if ThemeManager::is_builtin(n) { " · built-in" } else { "" };
                let mut line = Line::from(vec![
                    Span::raw(format!("{marker}{n}")),
                    Span::styled(format!("  {display}{tag}"), dim),
                ]);
                if i == self.cursor {
                    line = line.style(ui.list_highlight_style().add_modifier(Modifier::BOLD));
                }
                line
            })
            .collect();
        frame.render_widget(Paragraph::new(lines), chunks[0]);

        let status = match (&self.naming, &self.message) {
            (Some(f), _) => Line::from(vec![
                Span::raw("New theme name: "),
                Span::styled(f.value().to_string(), Style::default().fg(ui.accent_blue())),
                Span::styled("█", Style::default().fg(ui.accent_blue())),
            ]),
            (None, Some(m)) => Line::from(Span::styled(m.clone(), Style::default().fg(ui.accent_blue()))),
            _ => Line::raw(""),
        };
        frame.render_widget(Paragraph::new(status), chunks[1]);

        let hint = if self.naming.is_some() {
            "Enter: create (copy of selection)  Esc: cancel"
        } else {
            "↑↓ preview  Enter: apply  e: edit  n: new  Esc: cancel"
        };
        frame.render_widget(Paragraph::new(hint).style(dim), chunks[2]);
    }

    fn input_mode(&self) -> ModalInputMode {
        if self.naming.is_some() { ModalInputMode::Text } else { ModalInputMode::Normal }
    }

    fn debug_name(&self) -> &'static str { "Themes" }
}