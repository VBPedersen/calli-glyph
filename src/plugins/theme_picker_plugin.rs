use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState};
use crate::core::app::App;
use crate::errors::plugin_error::PluginError;
use crate::plugins::plugin_registry::{KeyContext, Plugin, PluginCommand, PluginKeybinding, PluginMetadata};

pub struct ThemePickerPlugin {
    available_themes: Vec<String>, // available themes seen as array of theme names
    current_theme_idx: usize,     // current theme selected, as idx in available themes Vec
}

impl ThemePickerPlugin {

    pub fn new() -> Self {
        ThemePickerPlugin {
            available_themes: vec![],
            current_theme_idx: 0,
        }
    }

    /// Populate available themes from ThemeManager and select the currently active one
    fn populate_themes(&mut self, app: &App) {
        // Retrieve loaded theme names from ThemeManager
        self.available_themes = app
            .theme_manager
            .names()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        // Sync index to current active theme
        let active = app.theme_manager.active_name();
        if let Some(idx) = self.available_themes.iter().position(|name| name == active) {
            self.current_theme_idx = idx;
        } else {
            self.current_theme_idx = 0;
        }
    }


    /// selects next theme possible
    fn next_theme(&mut self) {
        if !self.available_themes.is_empty() {
            self.current_theme_idx = (self.current_theme_idx + 1) % self.available_themes.len();
        }
    }

    /// selects previous theme possible
    fn prev_theme(&mut self) {
        if !self.available_themes.is_empty() {
            self.current_theme_idx = if self.current_theme_idx == 0 {
                self.available_themes.len() - 1
            } else {
                self.current_theme_idx - 1
            };
        }
    }

    /// Applies the selected theme and updates active app state/syntax themes
    fn apply_theme(&self, app: &mut App) {
        if let Some(selected_name) = self.available_themes.get(self.current_theme_idx) {
            if app.theme_manager.set_active(selected_name).is_ok() {
                // Update in-memory configuration
                app.config.ui.theme = selected_name.clone();

                // Save config to disk so it persists across restarts
                if let Err(e) = app.config.save() {
                    log_warn!("[ThemePicker] Failed to save config: {}", e);
                } else {
                    log_info!("[ThemePicker] Saved theme '{}' to config", selected_name);
                }

                // Refresh language manager syntax highlights if a file is currently active
                if let Some(ref path) = app.file_path {
                    let syntax_theme = app.theme_manager.active().syntax.clone();
                    app.language.activate_for_file(
                        path,
                        Some(syntax_theme),
                        &app.config.lsp,
                        &app.config.syntax,
                    );
                }
            }
        }
    }

    /// Render the centered modal popup dialog
    fn render_theme_picker(&self, frame: &mut Frame, app: &App) {
        let active_theme = app.theme_manager.active();
        let border_color = active_theme.ui.popup_border();
        let bg_color = active_theme.ui.popup_bg();
        let fg_color = active_theme.ui.popup_fg();

        let border_symbol_set = active_theme.layout.border_style.to_ratatui();

        let block = Block::default()
            .title(" Select Theme (Enter: Apply, Esc: Cancel) ")
            .borders(Borders::ALL)
            .border_set(border_symbol_set)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(bg_color).fg(fg_color));

        let items: Vec<ListItem> = self
            .available_themes
            .iter()
            .enumerate()
            .map(|(idx, name)| {
                let is_active = app.theme_manager.active_name() == name;
                let is_selected = idx == self.current_theme_idx;

                let mut prefix = if is_active { "● " } else { "  " };
                let mut style = Style::default().fg(fg_color);

                if is_selected {
                    style = style
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD);
                    prefix = if is_active { "➤ " } else { "> " };
                }

                let line = Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(name, style),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items).block(block);

        // Center calculation for modal dialog
        let area = frame.area();
        let width = 45.min(area.width);
        let height = (self.available_themes.len() as u16 + 4).clamp(6, 18).min(area.height);

        let popup_area = Rect {
            x: (area.width.saturating_sub(width)) / 2,
            y: (area.height.saturating_sub(height)) / 2,
            width,
            height,
        };

        let mut state = ListState::default();
        state.select(Some(self.current_theme_idx));

        frame.render_widget(Clear, popup_area);
        frame.render_stateful_widget(list, popup_area, &mut state);
    }
}


impl Plugin for ThemePickerPlugin {
    fn name(&self) -> &str {
        "theme_picker_plugin"
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "ThemePickerPlugin".to_string(),
            commands: vec![PluginCommand {
                name: "themepicker".to_string(),
                description: "Open theme picker popup".to_string(),
                aliases: vec!["thp".to_string(), "theme".to_string()],
                handler: |app, _args| {
                    app.plugins.activate_plugin("theme_picker_plugin");
                    Ok(())
                },
            }],
            keybinds: vec![PluginKeybinding {
                key: "Ctrl+T".to_string(),
                command: "themepicker".to_string(),
                context: KeyContext::Editor,
            }],
        }
    }

    fn init(&mut self, app: &mut App) -> Result<(), PluginError> {
        // Sync/scan available themes on init  if empty
        if self.available_themes.is_empty() {
            self.populate_themes(app);
        }
        Ok(())
    }

    fn handle_key_event(&mut self, app: &mut App, key: KeyEvent) -> bool {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Esc) => true,
            (KeyModifiers::NONE, KeyCode::Enter) => {
                self.apply_theme(app);
                true
            }
            (KeyModifiers::NONE, KeyCode::Up) => {
                self.prev_theme();
                true
            }
            (KeyModifiers::NONE, KeyCode::Down) => {
                self.next_theme();
                true
            }
            _ => false,
        }
    }

    fn render(&self, frame: &mut Frame, app: &App) -> bool {
        // render plugin dialog
        self.render_theme_picker(frame, app);
        true
    }

    fn shutdown(&mut self, app: &mut App) {}
}