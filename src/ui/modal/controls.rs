use crate::config::keymap_edit::{self as ke, BindMode, Context};
use crate::config::settings;
use crate::config::KeymapConfig;
use crate::core::app::App;
use crate::input::actions::{InputAction, ModalAction};
use crate::input::input::key_event_to_keybinding_string;
use crate::ui::modal::components::text_field::TextField;
use crate::ui::modal::{Modal, ModalInputMode, ModalResponse};
use crate::ui::ui::centered_rect;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Keymap(Context),
    Plugins,
}

const TABS: [Tab; 4] = [
    Tab::Keymap(Context::Editor),
    Tab::Keymap(Context::CommandLine),
    Tab::Keymap(Context::Debug),
    Tab::Plugins,
];

fn tab_label(t: Tab) -> &'static str {
    match t {
        Tab::Keymap(c) => c.label(),
        Tab::Plugins => "Plugins",
    }
}

struct PluginRow {
    plugin: String,
    command: String,
    description: String,
}

/// A binding waiting for the user to confirm replacing another action's key.
struct Pending {
    key: String,
    add: bool,
    displaced: String,
}

enum Mode {
    Browse,
    Capture { add: bool },
    Typing { add: bool, field: TextField },
}

pub struct ControlsModal {
    tab: usize,
    cursor: usize,
    scroll: usize,
    mode: Mode,
    pending: Option<Pending>,
    plugin_rows: Vec<PluginRow>,
    confirm_reset: bool,
    message: Option<String>,
}

impl ControlsModal {
    pub fn new(app: &App) -> Self {
        let mut names: Vec<&String> = app.plugins.plugins.keys().collect();
        names.sort();
        let mut plugin_rows = Vec::new();
        for name in names {
            for cmd in app.plugins.plugins[name].metadata().commands {
                plugin_rows.push(PluginRow {
                    plugin: name.clone(),
                    command: cmd.name,
                    description: cmd.description,
                });
            }
        }
        Self {
            tab: 0,
            cursor: 0,
            scroll: 0,
            mode: Mode::Browse,
            pending: None,
            plugin_rows,
            confirm_reset: false,
            message: None,
        }
    }

    fn row_count(&self) -> usize {
        match TABS[self.tab] {
            Tab::Keymap(ctx) => ke::actions(ctx).len(),
            Tab::Plugins => self.plugin_rows.len(),
        }
    }

    fn change_tab(&mut self, delta: isize) {
        self.tab = (self.tab as isize + delta).rem_euclid(TABS.len() as isize) as usize;
        self.cursor = 0;
        self.scroll = 0;
        self.message = None;
        self.pending = None;
    }

    fn move_cursor(&mut self, delta: isize) {
        let n = self.row_count();
        if n > 0 {
            self.cursor = (self.cursor as isize + delta).clamp(0, n as isize - 1) as usize;
            self.message = None;
        }
    }

    fn display_rows(&self, app: &App) -> Vec<(String, String)> {
        match TABS[self.tab] {
            Tab::Keymap(ctx) => {
                let map = ke::map_of(&app.config.keymaps, ctx);
                ke::actions(ctx)
                    .iter()
                    .map(|(name, label)| {
                        let keys = ke::keys_for(map, name);
                        (
                            label.to_string(),
                            if keys.is_empty() {
                                "—".into()
                            } else {
                                keys.join(", ")
                            },
                        )
                    })
                    .collect()
            }
            Tab::Plugins => self
                .plugin_rows
                .iter()
                .map(|r| {
                    let key = app
                        .plugins
                        .get_keybinding_command(&r.plugin, &r.command)
                        .unwrap_or_else(|| "—".into());
                    (format!("{} › {}", r.plugin, r.command), key)
                })
                .collect(),
        }
    }

    // Writing

    fn persist_map(&mut self, app: &mut App, ctx: Context, new_map: HashMap<String, String>) {
        // Validate the whole keymap BEFORE touching the file
        let mut km = app.config.keymaps.clone();
        ke::set_map(&mut km, ctx, new_map.clone());
        if let Err(e) = km.build_runtime_maps() {
            self.message = Some(format!("✗ invalid keymap, nothing saved: {e:?}"));
            return;
        }
        let sorted: BTreeMap<String, String> = new_map.into_iter().collect();
        let result = settings::write_config_table(&["keymaps", ctx.section()], &sorted)
            .map_err(|e| e.to_string())
            .and_then(|()| app.reload_config());
        self.message = Some(match result {
            Ok(()) => "✓ Saved".into(),
            Err(e) => format!("✗ {e}"),
        });
    }

    fn current_action(&self, ctx: Context) -> Option<&'static str> {
        ke::actions(ctx).get(self.cursor).map(|(n, _)| *n)
    }

    fn bind_keymap(&mut self, app: &mut App, ctx: Context, action: &str, key: &str, add: bool) {
        let map = ke::map_of(&app.config.keymaps, ctx).clone();
        let mode = if add {
            BindMode::Add
        } else {
            BindMode::Replace
        };
        let new_map = ke::with_binding(&map, action, key, mode);
        self.persist_map(app, ctx, new_map);
    }

    fn try_bind(&mut self, app: &mut App, mods: KeyModifiers, code: KeyCode, add: bool) {
        match TABS[self.tab] {
            Tab::Keymap(ctx) => {
                let Some(action) = self.current_action(ctx) else {
                    return;
                };
                if let Err(e) = ke::validate(ctx, mods, code) {
                    self.message = Some(format!("✗ {e}"));
                    return;
                }
                let Some(key) = ke::format_key(mods, code) else {
                    self.message = Some("✗ that key can't be stored in the config".into());
                    return;
                };
                // Plugin shortcuts are dispatched first and would shadow it
                let plugin_str = key_event_to_keybinding_string(KeyEvent::new(code, mods));
                if let Some(p) = app.plugins.find_plugin_by_keybinding(&plugin_str) {
                    self.message = Some(format!(
                        "✗ {key} is a shortcut of plugin '{p}' and would take priority"
                    ));
                    return;
                }
                let map = ke::map_of(&app.config.keymaps, ctx).clone();
                match ke::find_conflict(&map, mods, code) {
                    Some((_, other)) if other != action => {
                        self.message = Some(format!(
                            "{key} is bound to '{other}'. Enter: replace  Esc: cancel"
                        ));
                        self.pending = Some(Pending {
                            key,
                            add,
                            displaced: other,
                        });
                    }
                    _ => self.bind_keymap(app, ctx, action, &key, add),
                }
            }
            Tab::Plugins => self.bind_plugin(app, mods, code),
        }
    }

    fn bind_plugin(&mut self, app: &mut App, mods: KeyModifiers, code: KeyCode) {
        let Some(row) = self.plugin_rows.get(self.cursor) else {
            return;
        };
        let (plugin, command) = (row.plugin.clone(), row.command.clone());

        if let Err(e) = ke::validate_plugin(mods, code) {
            self.message = Some(format!("✗ {e}"));
            return;
        }
        let binding = key_event_to_keybinding_string(KeyEvent::new(code, mods));
        if binding.is_empty() || binding.ends_with('+') {
            self.message = Some("✗ that key can't be used for a plugin shortcut".into());
            return;
        }
        // Already used by another plugin command?
        if let Some(owner) = app.plugins.find_plugin_by_keybinding(&binding) {
            let same = owner == plugin
                && app
                    .plugins
                    .command_for_keybinding(&plugin, &binding)
                    .as_deref()
                    == Some(command.as_str());
            if !same {
                self.message = Some(format!("✗ {binding} is already used by plugin '{owner}'"));
                return;
            }
        }
        // Plugin shortcuts take priority over every keymap context: don't hide one
        for ctx in [Context::Editor, Context::CommandLine, Context::Debug] {
            if let Some((_, other)) =
                ke::find_conflict(ke::map_of(&app.config.keymaps, ctx), mods, code)
            {
                self.message = Some(format!(
                    "✗ {binding} is used by {}: {other}, and plugin keys would override it",
                    ctx.label()
                ));
                return;
            }
        }
        let name = format!("{plugin}.{command}");
        let result = settings::write_config_path(
            &["plugins", "keybindings", &name],
            Some(binding.clone().into()),
        )
        .map_err(|e| e.to_string())
        .and_then(|()| app.reload_config());
        self.message = Some(match result {
            Ok(()) => format!("✓ {name} → {binding}"),
            Err(e) => format!("✗ {e}"),
        });
    }

    fn apply_pending(&mut self, app: &mut App, p: Pending) {
        if let Tab::Keymap(ctx) = TABS[self.tab] {
            if let Some(action) = self.current_action(ctx) {
                let _ = &p.displaced;
                self.bind_keymap(app, ctx, action, &p.key, p.add);
            }
        }
    }

    fn unbind(&mut self, app: &mut App) {
        match TABS[self.tab] {
            Tab::Keymap(ctx) => {
                let Some(action) = self.current_action(ctx) else {
                    return;
                };
                if ke::is_essential(ctx, action) {
                    self.message = Some("✗ this action is essential: rebind it instead".into());
                    return;
                }
                let map = ke::map_of(&app.config.keymaps, ctx).clone();
                self.persist_map(app, ctx, ke::without_action(&map, action));
            }
            Tab::Plugins => {
                self.message = Some("Plugin shortcuts can be rebound or reset, not removed".into());
            }
        }
    }

    fn reset_row(&mut self, app: &mut App) {
        match TABS[self.tab] {
            Tab::Keymap(ctx) => {
                let Some(action) = self.current_action(ctx) else {
                    return;
                };
                let defaults = KeymapConfig::default();
                let map = ke::map_of(&app.config.keymaps, ctx).clone();
                let new_map = ke::reset_action(&map, ke::map_of(&defaults, ctx), action);
                self.persist_map(app, ctx, new_map);
            }
            Tab::Plugins => {
                let Some(row) = self.plugin_rows.get(self.cursor) else {
                    return;
                };
                let name = format!("{}.{}", row.plugin, row.command);
                let result = settings::write_config_path(&["plugins", "keybindings", &name], None)
                    .map_err(|e| e.to_string())
                    .and_then(|()| app.reload_config());
                self.message = Some(match result {
                    Ok(()) => "✓ Back to the plugin's default shortcut".into(),
                    Err(e) => format!("✗ {e}"),
                });
            }
        }
    }

    fn reset_tab(&mut self, app: &mut App) {
        match TABS[self.tab] {
            Tab::Keymap(ctx) => {
                let defaults = KeymapConfig::default();
                let map = ke::map_of(&defaults, ctx).clone();
                self.persist_map(app, ctx, map);
            }
            Tab::Plugins => {
                self.message = Some("Use u on each plugin row to reset it".into());
            }
        }
    }
}

impl Modal for ControlsModal {
    fn handle_input(&mut self, action: InputAction, app: &mut App) -> ModalResponse {
        let InputAction::Modal(m) = action else {
            return ModalResponse::Consumed;
        };

        match std::mem::replace(&mut self.mode, Mode::Browse) {
            Mode::Capture { add } => {
                // Esc (Close) just cancels; any other key is the new binding
                if let ModalAction::Key(code, mods) = m {
                    self.try_bind(app, mods, code, add);
                }
                return ModalResponse::Consumed;
            }
            Mode::Typing { add, mut field } => {
                match m {
                    ModalAction::Close => {}
                    ModalAction::Confirm => match KeymapConfig::parse_key(field.value().trim()) {
                        Ok((mods, code)) => self.try_bind(app, mods, code, add),
                        Err(_) => {
                            self.message = Some(
                                "✗ not a valid key (e.g. Ctrl+Shift+k, F5, Alt+Left, Esc)".into(),
                            );
                            self.mode = Mode::Typing { add, field };
                        }
                    },
                    other => {
                        field.handle(&other);
                        self.mode = Mode::Typing { add, field };
                    }
                }
                return ModalResponse::Consumed;
            }
            Mode::Browse => {}
        }

        if !matches!(m, ModalAction::Action('R')) {
            self.confirm_reset = false;
        }
        if let Some(p) = self.pending.take() {
            if matches!(m, ModalAction::Confirm) {
                self.apply_pending(app, p);
            } else {
                self.message = Some("Cancelled".into());
            }
            return ModalResponse::Consumed;
        }

        match m {
            ModalAction::Close => return ModalResponse::Close,
            ModalAction::ScrollUp => self.move_cursor(-1),
            ModalAction::ScrollDown => self.move_cursor(1),
            ModalAction::NextTab => self.change_tab(1),
            ModalAction::PrevTab => self.change_tab(-1),
            ModalAction::Confirm => {
                if self.row_count() > 0 {
                    self.mode = Mode::Capture { add: false };
                }
            }
            ModalAction::Action('a') => {
                if matches!(TABS[self.tab], Tab::Keymap(_)) && self.row_count() > 0 {
                    self.mode = Mode::Capture { add: true };
                }
            }
            ModalAction::Action('t') => {
                if self.row_count() > 0 {
                    self.mode = Mode::Typing {
                        add: false,
                        field: TextField::new(),
                    };
                }
            }
            ModalAction::Action('x') => self.unbind(app),
            ModalAction::Action('u') => self.reset_row(app),
            ModalAction::Action('R') => {
                if !self.confirm_reset {
                    self.confirm_reset = true;
                    self.message = Some("Press R again to reset this whole tab to defaults".into());
                } else {
                    self.confirm_reset = false;
                    self.reset_tab(app);
                }
            }
            _ => {}
        }
        ModalResponse::Consumed
    }
    fn render(&mut self, frame: &mut Frame, app: &App) {
        let ui = &app.theme_manager.active().ui;
        let area = centered_rect(70, 80, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Controls ")
            .borders(Borders::ALL)
            .border_set(app.theme_manager.active().layout.border_style.to_ratatui())
            .border_style(Style::default().fg(ui.popup_border()))
            .style(Style::default().fg(ui.popup_fg()).bg(ui.popup_bg()));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1), // detail / capture prompt
                Constraint::Length(1), // message
                Constraint::Length(1), // hint
            ])
            .split(inner);
        let dim = Style::default().fg(ui.hint_text());

        let mut tab_spans = Vec::new();
        for (i, t) in TABS.iter().enumerate() {
            let style = if i == self.tab {
                ui.list_highlight_style().add_modifier(Modifier::BOLD)
            } else {
                dim
            };
            tab_spans.push(Span::styled(format!(" {} ", tab_label(*t)), style));
        }
        frame.render_widget(Paragraph::new(Line::from(tab_spans)), chunks[0]);

        let rows = self.display_rows(app);
        let visible = chunks[2].height as usize;
        if visible > 0 {
            if self.cursor >= self.scroll + visible {
                self.scroll = self.cursor + 1 - visible;
            }
            if self.cursor < self.scroll {
                self.scroll = self.cursor;
            }
        }
        let width = chunks[2].width as usize;
        let lines: Vec<Line> = rows
            .iter()
            .enumerate()
            .skip(self.scroll)
            .take(visible)
            .map(|(i, (label, keys))| {
                let label = format!("  {label}");
                let pad = width.saturating_sub(label.chars().count() + keys.chars().count() + 1);
                let mut line = Line::from(vec![
                    Span::raw(label),
                    Span::raw(" ".repeat(pad)),
                    Span::raw(keys.clone()),
                ]);
                if i == self.cursor {
                    line = line.style(ui.list_highlight_style().add_modifier(Modifier::BOLD));
                }
                line
            })
            .collect();
        frame.render_widget(Paragraph::new(lines), chunks[2]);

        let accent = Style::default().fg(ui.accent_blue());
        let detail = match &self.mode {
            Mode::Capture { add } => Line::from(Span::styled(
                format!(
                    "  Press the key combination to {}…  (Esc cancels)",
                    if *add { "add" } else { "bind" }
                ),
                accent,
            )),
            Mode::Typing { field, .. } => Line::from(vec![
                Span::raw("  Key: "),
                Span::styled(field.value().to_string(), accent),
                Span::styled("█", accent),
            ]),
            Mode::Browse => {
                let text = match TABS[self.tab] {
                    Tab::Keymap(ctx) => self
                        .current_action(ctx)
                        .map(|a| format!("  action: {a}"))
                        .unwrap_or_default(),
                    Tab::Plugins => self
                        .plugin_rows
                        .get(self.cursor)
                        .map(|r| format!("  {}", r.description))
                        .unwrap_or_default(),
                };
                Line::from(Span::styled(text, dim))
            }
        };
        frame.render_widget(Paragraph::new(detail), chunks[3]);

        if let Some(msg) = &self.message {
            frame.render_widget(Paragraph::new(format!("  {msg}")).style(accent), chunks[4]);
        }
        let hint = match self.mode {
            Mode::Browse => {
                "Enter: rebind  a: add  t: type key  x: unbind  u: reset  R: reset tab  Esc: close"
            }
            Mode::Capture { .. } => "Press a key combo   Esc: cancel",
            Mode::Typing { .. } => {
                "Enter: apply   Esc: cancel   (Ctrl+Shift+k, F5, Alt+Left, Esc …)"
            }
        };
        frame.render_widget(Paragraph::new(hint).style(dim), chunks[5]);
    }

    fn input_mode(&self) -> ModalInputMode {
        match self.mode {
            Mode::Browse => ModalInputMode::Normal,
            Mode::Capture { .. } => ModalInputMode::CaptureKey,
            Mode::Typing { .. } => ModalInputMode::Text,
        }
    }

    fn debug_name(&self) -> &'static str {
        "Controls"
    }
}
