use crate::core::app::App;
use crate::input::actions::{InputAction, ModalAction};
use crate::language::lsp::{ConnectionState, DiagnosticSeverity};
use crate::ui::modal::{Modal, ModalResponse};
use crate::ui::ui::centered_rect;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Tabs};
use ratatui::Frame;
// ----------   TABS   --------------

/// Language Tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LangTab {
    Diagnostics,
    Lsp,
    Syntax,
    Install,
}

impl LangTab {
    fn next(self) -> Self {
        match self {
            Self::Diagnostics => Self::Lsp,
            Self::Lsp => Self::Syntax,
            Self::Syntax => Self::Install,
            Self::Install => Self::Diagnostics,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::Diagnostics => Self::Install,
            Self::Lsp => Self::Diagnostics,
            Self::Syntax => Self::Lsp,
            Self::Install => Self::Syntax,
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Diagnostics => 0,
            Self::Lsp => 1,
            Self::Syntax => 2,
            Self::Install => 3,
        }
    }
}

// ----------   DIAGNOTSTIC FILTER   --------------

/// Diagnostic filter used to filter diagnostics from lsp
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum DiagFilter {
    AllFiles,
    CurrentFile,
    ErrorsOnly,
    WarningsOnly,
}

impl DiagFilter {
    /// Cycles through filters
    fn cycle(self) -> Self {
        match self {
            Self::AllFiles => Self::CurrentFile,
            Self::CurrentFile => Self::ErrorsOnly,
            Self::ErrorsOnly => Self::WarningsOnly,
            Self::WarningsOnly => Self::AllFiles,
        }
    }

    /// Returns label of current DiagFilter
    fn label(self) -> &'static str {
        match self {
            DiagFilter::AllFiles => "All Files",
            DiagFilter::CurrentFile => "Current File",
            DiagFilter::ErrorsOnly => "Errors Only",
            DiagFilter::WarningsOnly => "Warnings Only",
        }
    }
}

// ----------   Install Pnael Focus   --------------

// New enum, near DiagFilter:
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InstallFocus {
    Grammars,
    LspServers,
}

// ----------   MAIN STRUCT   --------------

/// Language panel menu for managing the language related systems via modal
pub struct LangPanel {
    pub tab: LangTab,
    pub selected_diagnostic: usize, // index of diagnostic currently selected
    pub diag_filter: DiagFilter,
    pub diag_scroll: u16,
    /// Flat list of (uri, diag_index, rendered_row) for the currently visible diagnostics,
    /// rebuilt each render so selection always maps to a real diagnostic.
    diag_index_map: Vec<(String, usize, usize)>,
    visible_height: u16, // set each render frame from the actual area height
    install_focus: InstallFocus,
    install_grammar_selected: usize,
    install_lsp_selected: usize,
}

impl LangPanel {
    pub fn new() -> LangPanel {
        Self {
            tab: LangTab::Diagnostics,
            selected_diagnostic: 0,
            diag_filter: DiagFilter::AllFiles,
            diag_scroll: 0,
            diag_index_map: Vec::new(),
            visible_height: 0,
            install_focus: InstallFocus::Grammars,
            install_grammar_selected: 0,
            install_lsp_selected: 0,
        }
    }

    // -------- HELPERS -------------

    /// count of diagnostics in diag to index map
    fn diag_count(self: &Self) -> usize {
        self.diag_index_map.len()
    }

    /// Select next (down) diagnostic
    fn select_down(&mut self) {
        let max = self.diag_count().saturating_sub(1);
        if self.selected_diagnostic < max {
            self.selected_diagnostic += 1;

            // keep selected item inside the visible scroll window
            if let Some((_, _, row)) = self.diag_index_map.get(self.selected_diagnostic) {
                let row = *row as u16;
                let visible_height = self.visible_height;
                if row >= self.diag_scroll + visible_height {
                    self.diag_scroll = row.saturating_sub(visible_height - 1);
                }
            }
        }
    }

    /// Select previous (up) diagnostic
    fn select_up(&mut self) {
        if self.selected_diagnostic > 0 {
            self.selected_diagnostic -= 1;
            if (self.selected_diagnostic as u16) < self.diag_scroll {
                self.diag_scroll = self.selected_diagnostic as u16;
            }
        }
    }

    /// Select previous (up) on install
    fn install_select_up(&mut self) {
        match self.install_focus {
            InstallFocus::Grammars => {
                self.install_grammar_selected = self.install_grammar_selected.saturating_sub(1)
            }
            InstallFocus::LspServers => {
                self.install_lsp_selected = self.install_lsp_selected.saturating_sub(1)
            }
        }
    }

    /// Select next (down) on install
    fn install_select_down(&mut self) {
        match self.install_focus {
            InstallFocus::Grammars => {
                let max = crate::language::install::GRAMMARS.len().saturating_sub(1);
                if self.install_grammar_selected < max {
                    self.install_grammar_selected += 1;
                }
            }
            InstallFocus::LspServers => {
                let max = crate::language::install::LSP_SERVERS
                    .len()
                    .saturating_sub(1);
                if self.install_lsp_selected < max {
                    self.install_lsp_selected += 1;
                }
            }
        }
    }

    /// Confirm action on install screen
    fn install_confirm(&self, app: &mut App) {
        match self.install_focus {
            InstallFocus::Grammars => {
                if let Some(spec) =
                    crate::language::install::GRAMMARS.get(self.install_grammar_selected)
                {
                    let grammar_dir = crate::language::manager::resolve_grammar_dir(
                        &app.config.syntax.grammar_dir,
                    );
                    app.install_manager.start_grammar_install(spec, grammar_dir);
                }
            }
            InstallFocus::LspServers => {
                if let Some(spec) =
                    crate::language::install::LSP_SERVERS.get(self.install_lsp_selected)
                {
                    app.install_manager.start_lsp_install(spec);
                }
            }
        }
    }

    /// Open the grammar config TOML for the current language in the editor.
    ///
    /// Uses `App::open_file` so user needs to confirm save if the current buffer has unsaved changes.
    fn open_grammar_config_in_editor(&self, app: &mut App) {
        let Some(lang_id) = app.language.language_id.clone() else {
            log_warn!("[LangPanel] No language active — cannot open grammar config");
            return;
        };

        let grammar_dir =
            crate::language::manager::resolve_grammar_dir(&app.config.syntax.grammar_dir);
        let config_path = grammar_dir.join(format!("{}.toml", lang_id));

        if !config_path.exists() {
            log_warn!(
                "[LangPanel] Grammar config not found: {}",
                config_path.display()
            );
            return;
        }

        log_info!(
            "[LangPanel] Opening grammar config: {}",
            config_path.display()
        );
        app.open_file(config_path, None);
    }

    /// Jump the editor cursor to the selected diagnostic's line.
    ///
    /// If the diagnostic belongs to a different file than the one currently
    /// open, `App::open_file` is called with the target path so user gets
    /// "save before switching?" confirmation popup if needed.
    fn jump_to_selected(&self, app: &mut App) {
        let Some((uri, diag_idx, _row)) = self.diag_index_map.get(self.selected_diagnostic) else {
            return;
        };

        let lsp = app.language.lsp.as_ref();
        let line = lsp
            .and_then(|l| l.diagnostics.get(uri))
            .and_then(|diags| diags.get(*diag_idx))
            .map(|d| d.line as usize);

        let Some(line) = line else { return };

        let current_uri = app.language.current_uri.as_deref().unwrap_or("");

        if uri.as_str() == current_uri {
            // Same file: just move the cursor
            app.editor.jump_to_line(line);
        } else {
            // Different file: convert LSP URI to filesystem path and open file.
            if let Some(path) = uri_to_path(uri) {
                app.open_file(path, Some(line));
            } else {
                log_warn!("[LangPanel] Cannot resolve URI to path: {}", uri);
            }
        }
    }

    /// Restart the LSP server for the current file.
    fn restart_lsp(&self, app: &mut App) {
        if let Some(path) = app.file_path.clone() {
            let theme = app.language.theme.clone();
            app.language
                .activate_for_file(&path, theme, &app.config.lsp, &app.config.syntax);
            log_info!("[LangPanel] LSP restarted");
        }
    }

    /// Re-run project root detection and log the result.
    fn log_project_root(&self, app: &mut App) {
        let lsp = app.language.lsp.as_ref();
        let root = lsp
            .map(|l| l.workspace_root.display().to_string())
            .unwrap_or_else(|| "—".to_string());
        if root.len() > 0 {
            log_info!("[LangPanel] Project root: {}", root)
        } else {
            log_info!("[LangPanel] No project root is identified");
        }
    }

    /// Reload the grammar config from disk and clear the token cache.
    fn reload_grammar(&self, app: &mut App) {
        if let Some(lang_id) = &app.language.language_id.clone() {
            let grammar_dir =
                crate::language::manager::resolve_grammar_dir(&app.config.syntax.grammar_dir);
            match crate::language::lang_configs::loader::load_lang_config(&grammar_dir, lang_id) {
                Ok(cfg) => {
                    if let Some(syntax) = &mut app.language.syntax {
                        //syntax.reload_config(cfg); TODO implement some reload of only grammar or maybe just make new SyntaxTree and pass to manager
                    }
                    log_info!("[LangPanel] Grammar config reloaded: {}.toml", lang_id);
                }
                Err(e) => log_warn!("[LangPanel] Failed to reload grammar config: {}", e),
            }
        }
    }

    // -------- RENDERING -------------

    /// Renders the diagnostics tab of the modal
    fn render_diagnostics_tab(&mut self, frame: &mut Frame, area: Rect, app: &App) {
        self.visible_height = area.height.saturating_sub(3); // subtract hint bar rows
        let truncation_width = area.width.saturating_sub(24) as usize;
        // Collect diagnostics according to current filter
        let lsp = app.language.lsp.as_ref();
        let current_uri = app.language.current_uri.as_deref().unwrap_or("");

        // Build the flat index map so selection can resolve to real items
        self.diag_index_map.clear();
        let mut rendered_row: usize = 2; // start after filter bar + blank line

        // Sort URIs so current file is always first
        let mut uris: Vec<&String> = lsp
            .map(|l| l.diagnostics.keys().collect())
            .unwrap_or_default();
        uris.sort_by_key(|u| if u.as_str() == current_uri { 0 } else { 1 });

        let mut lines: Vec<Line> = Vec::new();

        // Filter bar
        lines.push(Line::from(vec![
            Span::styled(" Filter: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("[{} ▾]", self.diag_filter.label()),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  f: cycle filter", Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::raw(""));

        // Show all diagnostics resulted from filter

        for uri in &uris {
            let diags = match lsp.and_then(|l| l.diagnostics.get(*uri)) {
                Some(d) => d,
                None => continue,
            };

            // Apply filter
            let filtered: Vec<(usize, &crate::language::lsp::Diagnostic)> = diags
                .iter()
                .enumerate()
                .filter(|(_, d)| match self.diag_filter {
                    DiagFilter::AllFiles => true,
                    DiagFilter::CurrentFile => uri.as_str() == current_uri,
                    DiagFilter::ErrorsOnly => d.severity == DiagnosticSeverity::Error,
                    DiagFilter::WarningsOnly => d.severity == DiagnosticSeverity::Warning,
                })
                .collect();

            if filtered.is_empty() {
                continue;
            }

            // File header
            let filename = uri.rsplit('/').next().unwrap_or(uri.as_str());
            let is_current = uri.as_str() == current_uri;
            let header_style = if is_current {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            lines.push(Line::from(vec![
                Span::styled(if is_current { "▶ " } else { "  " }, header_style),
                Span::styled(filename.to_string(), header_style),
            ]));
            rendered_row += 1;

            for (orig_idx, diag) in &filtered {
                let is_selected = self.diag_index_map.len() == self.selected_diagnostic;
                self.diag_index_map
                    .push((uri.to_string(), *orig_idx, rendered_row));

                let (icon, sev_style) = severity_style(&diag.severity);
                let loc = format!("{}:{}", diag.line + 1, diag.col_start + 1);
                let src = diag.source.as_deref().unwrap_or("");

                let row_style = if is_selected {
                    Style::default().bg(Color::Rgb(80, 80, 100))
                } else {
                    Style::default()
                };

                let mut spans = vec![
                    Span::styled("  ", row_style),
                    Span::styled(icon, sev_style.patch(row_style)),
                    Span::styled(
                        format!("{:<8}", loc),
                        Style::default().fg(Color::DarkGray).patch(row_style),
                    ),
                    Span::styled(
                        truncate(&diag.message, truncation_width),
                        Style::default().fg(Color::White).patch(row_style),
                    ),
                ];
                if !src.is_empty() {
                    spans.push(Span::styled(
                        format!("  [{}]", src),
                        Style::default().fg(Color::DarkGray).patch(row_style),
                    ));
                }
                lines.push(Line::from(spans));
                rendered_row += 1;
            }

            lines.push(Line::raw(""));
            rendered_row += 1;
        }

        if self.diag_index_map.is_empty() {
            lines.push(Line::from(Span::styled(
                "  No diagnostics match the current filter.",
                Style::default().fg(Color::DarkGray),
            )));
        }

        // Hint bar
        lines.push(Line::raw(""));
        lines.push(Line::from(Span::styled(
            " j/k: navigate   Enter: jump to line   f: filter   Esc: close",
            Style::default().fg(Color::DarkGray),
        )));

        let p = Paragraph::new(lines).scroll((self.diag_scroll, 0));
        frame.render_widget(p, area);
    }

    /// Renders the lsp tab of the modal
    fn render_lsp_tab(&mut self, frame: &mut Frame, area: Rect, app: &App) {
        let lsp = app.language.lsp.as_ref();

        let (state_icon, state_label, state_color) = match lsp.map(|l| &l.state) {
            Some(ConnectionState::Ready) => ("●", "Ready", Color::Green),
            Some(ConnectionState::Initializing) => ("◌", "Initializing", Color::Yellow),
            Some(ConnectionState::Disconnected) => ("○", "Disconnected", Color::DarkGray),
            Some(ConnectionState::Failed(e)) => ("✗", e.as_str(), Color::Red),
            None => ("○", "No server", Color::DarkGray),
        };

        let server_name = lsp.map(|l| l.server_name.as_str()).unwrap_or("—");

        let workspace = lsp
            .map(|l| l.workspace_root.display().to_string())
            .unwrap_or_else(|| "—".to_string());

        let uri = app.language.current_uri.as_deref().unwrap_or("—");

        let errors = count_severity(app, DiagnosticSeverity::Error);
        let warnings = count_severity(app, DiagnosticSeverity::Warning);
        let hints = count_severity(app, DiagnosticSeverity::Hint);

        let truncation_width = area.width.saturating_sub(16) as usize;
        let truncated_uri = truncate(uri, truncation_width);
        let lines: Vec<Line> = vec![
            Line::raw(""),
            row_kv("Server", server_name),
            row_kv_styled(
                "State",
                state_label,
                Style::default()
                    .fg(state_color)
                    .add_modifier(Modifier::BOLD),
                state_icon,
            ),
            row_kv("Root", &workspace),
            row_kv("File URI", &truncated_uri),
            Line::raw(""),
            Line::from(vec![
                Span::styled("  Diagnostics   ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("● {} errors  ", errors),
                    Style::default().fg(Color::Red),
                ),
                Span::styled(
                    format!("◆ {} warnings  ", warnings),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    format!("· {} hints", hints),
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
            Line::raw(""),
            section_header("Actions"),
            action_row('r', "Restart LSP server"),
            action_row('l', "Locate project root (.git / .hg / .svn)"),
            action_row('d', "Re-send didOpen for current file"),
            Line::raw(""),
            hint_line("r/l/d: actions   Esc: close"),
        ];

        frame.render_widget(Paragraph::new(lines), area);
    }

    /// Renders the syntax tab of the modal
    fn render_syntax_tab(&mut self, frame: &mut Frame, area: Rect, app: &App) {
        let grammar_id = app.language.language_id.as_deref().unwrap_or("—");
        let syntax_loaded = app.language.syntax.is_some();
        let theme_loaded = app.language.theme.is_some();
        let theme_name = app
            .language
            .theme
            .as_ref()
            .map(|t| t.name.as_str())
            .unwrap_or("—");

        let token_count = app
            .language
            .syntax
            .as_ref()
            .and_then(|s| s.token_cache.as_ref())
            .map(|(_, t)| t.len())
            .unwrap_or(0);

        let grammar_dir =
            crate::language::manager::resolve_grammar_dir(&app.config.syntax.grammar_dir);
        let grammar_dir_str = grammar_dir.display().to_string();

        let (syn_icon, syn_color) = if syntax_loaded {
            ("● Loaded", Color::Green)
        } else {
            ("✗ Not loaded", Color::Red)
        };
        let (thm_icon, thm_color) = if theme_loaded {
            ("● Loaded", Color::Green)
        } else {
            ("✗ Not loaded", Color::Red)
        };

        // Build loaded-grammars section from config
        let mut grammar_rows: Vec<Line> = Vec::new();
        for (name, cfg) in &app.config.syntax.languages {
            let lib_name = format!("tree_sitter_{}", cfg.grammar);
            let filename = crate::language::grammar_loader::platform_lib_name(&lib_name);
            let lib_path = grammar_dir.join(&filename);
            let (icon, color) = if lib_path.exists() {
                ("●", Color::Green)
            } else {
                ("✗", Color::Red)
            };
            grammar_rows.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(icon, Style::default().fg(color)),
                Span::raw("  "),
                Span::styled(format!("{:<14}", name), Style::default().fg(Color::White)),
                Span::styled(
                    format!("{:<18}", cfg.file_extensions.join(", ")),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(filename, Style::default().fg(Color::DarkGray)),
            ]));
        }

        let mut lines: Vec<Line> = vec![
            Line::raw(""),
            row_kv("Grammar dir", &grammar_dir_str),
            Line::raw(""),
            Line::from(vec![
                Span::styled("  Grammar  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    grammar_id,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("   "),
                Span::styled(syn_icon, Style::default().fg(syn_color)),
            ]),
            Line::from(vec![
                Span::styled("  Theme    ", Style::default().fg(Color::DarkGray)),
                Span::styled(theme_name, Style::default().fg(Color::White)),
                Span::raw("   "),
                Span::styled(thm_icon, Style::default().fg(thm_color)),
            ]),
            Line::from(vec![
                Span::styled("  Tokens   ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} cached", token_count),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::raw(""),
            section_header("Loaded grammars"),
        ];

        lines.extend(grammar_rows);

        lines.extend(vec![
            Line::raw(""),
            section_header("Actions"),
            action_row('r', "Reload grammar config from disk"),
            action_row('c', "Clear token cache"),
            action_row('o', "Open grammar config in editor"),
            Line::raw(""),
            hint_line("r/c/o: actions   Esc: close"),
        ]);

        frame.render_widget(Paragraph::new(lines), area);
    }

    /// Renders the install language tab of the modal
    fn render_install_tab(&mut self, frame: &mut Frame, area: Rect, app: &App) {
        use crate::language::install::{JobId, JobStatus, GRAMMARS, LSP_SERVERS};

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(6), Constraint::Length(6)])
            .split(area);

        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[0]);

        // ---- Grammars column ----
        let mut grammar_lines: Vec<Line> = vec![section_header(
            if self.install_focus == InstallFocus::Grammars {
                "▸ Tree-sitter grammars"
            } else {
                "  Tree-sitter grammars"
            },
        )];
        grammar_lines.push(Line::raw(""));
        for (i, spec) in GRAMMARS.iter().enumerate() {
            let (label, color) = grammar_status(app, spec);
            let selected =
                self.install_focus == InstallFocus::Grammars && i == self.install_grammar_selected;
            grammar_lines.push(Line::from(vec![
                Span::raw(if selected { "▶ " } else { "  " }),
                Span::styled(
                    format!("{:<12}", spec.name),
                    Style::default()
                        .fg(if selected { Color::White } else { Color::Gray })
                        .add_modifier(if selected {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                ),
                Span::styled(format!("{:<16}", label), Style::default().fg(color)),
                Span::styled(
                    spec.file_extensions.join(","),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
        frame.render_widget(Paragraph::new(grammar_lines), cols[0]);

        // ---- LSP column ----
        let mut lsp_lines: Vec<Line> = vec![section_header(
            if self.install_focus == InstallFocus::LspServers {
                "▸ LSP servers"
            } else {
                "  LSP servers"
            },
        )];
        lsp_lines.push(Line::raw(""));
        for (i, spec) in LSP_SERVERS.iter().enumerate() {
            let (label, color) = lsp_status(app, spec);
            let selected =
                self.install_focus == InstallFocus::LspServers && i == self.install_lsp_selected;
            lsp_lines.push(Line::from(vec![
                Span::raw(if selected { "▶ " } else { "  " }),
                Span::styled(
                    format!("{:<12}", spec.name),
                    Style::default()
                        .fg(if selected { Color::White } else { Color::Gray })
                        .add_modifier(if selected {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                ),
                Span::styled(format!("{:<16}", label), Style::default().fg(color)),
                Span::styled(spec.command, Style::default().fg(Color::DarkGray)),
            ]));
        }
        frame.render_widget(Paragraph::new(lsp_lines), cols[1]);

        // ---- Log tail for the currently focused/selected job ----
        let selected_id = match self.install_focus {
            InstallFocus::Grammars => GRAMMARS
                .get(self.install_grammar_selected)
                .map(|s| JobId::Grammar(s.name.to_string())),
            InstallFocus::LspServers => LSP_SERVERS
                .get(self.install_lsp_selected)
                .map(|s| JobId::LspServer(s.name.to_string())),
        };

        let mut log_lines: Vec<Line> = vec![section_header("Log")];
        if let Some(id) = &selected_id {
            if let Some(lines) = app.install_manager.job_log(id) {
                for line in lines.iter().rev().take(3).rev() {
                    log_lines.push(Line::styled(
                        format!("  {}", line),
                        Style::default().fg(Color::DarkGray),
                    ));
                }
            }
            if let Some(JobStatus::Failed(err)) = app.install_manager.job_status(id) {
                for line in err.lines().take(3) {
                    log_lines.push(Line::styled(
                        format!("  {}", line),
                        Style::default().fg(Color::Red),
                    ));
                }
            }
        }
        log_lines.push(Line::raw(""));
        log_lines.push(hint_line(
            "h/l: focus list   ↑/↓: select   Enter: install   Esc: close",
        ));
        frame.render_widget(Paragraph::new(log_lines), rows[1]);
    }
}

impl Modal for LangPanel {
    fn handle_input(&mut self, action: InputAction, app: &mut App) -> ModalResponse {
        let InputAction::Modal(modal_action) = action else {
            return ModalResponse::Consumed;
        };

        match modal_action {
            ModalAction::Close => return ModalResponse::Close,
            ModalAction::ScrollUp => match self.tab {
                LangTab::Diagnostics => self.select_up(),
                LangTab::Install => self.install_select_up(),
                _ => self.diag_scroll = self.diag_scroll.saturating_sub(1),
            },
            ModalAction::ScrollDown => match self.tab {
                LangTab::Diagnostics => self.select_down(),
                LangTab::Install => self.install_select_down(),
                _ => self.diag_scroll = self.diag_scroll.saturating_add(1),
            },
            ModalAction::NextTab => self.tab = self.tab.next(),
            ModalAction::PrevTab => self.tab = self.tab.prev(),
            ModalAction::Confirm => match self.tab {
                LangTab::Diagnostics => {
                    self.jump_to_selected(app);
                    return ModalResponse::Close;
                }
                LangTab::Install => self.install_confirm(app),
                _ => {}
            },
            // ----- tab specific char actions -----
            ModalAction::Action(c) => match (self.tab, c) {
                (_, '1') => self.tab = LangTab::Diagnostics,
                (_, '2') => self.tab = LangTab::Lsp,
                (_, '3') => self.tab = LangTab::Syntax,
                (_, '4') => self.tab = LangTab::Install,

                // Diagnostics tab
                (LangTab::Diagnostics, 'f') => {
                    self.diag_filter = self.diag_filter.cycle();
                    self.selected_diagnostic = 0;
                    self.diag_scroll = 0;
                    self.diag_index_map.clear();
                }

                // LSP tab
                (LangTab::Lsp, 'r') => self.restart_lsp(app),
                (LangTab::Lsp, 'l') => self.log_project_root(app),
                (LangTab::Lsp, 'd') => {
                    // Re-send didOpen to trigger a fresh diagnostic pass
                    if let (Some(lsp), Some(uri)) =
                        (&mut app.language.lsp, &app.language.current_uri)
                    {
                        let uri = uri.clone();
                        let lang_id = app.language.language_id.clone().unwrap_or_default();
                        let content = app.editor.editor_content.join("\n");
                        if let Err(e) = lsp.notify_did_open(&uri, &lang_id, &content) {
                            log_warn!("[LangPanel] didOpen failed: {}", e);
                        }
                    }
                }

                // Syntax tab
                (LangTab::Syntax, 'r') => self.reload_grammar(app),
                (LangTab::Syntax, 'c') => {
                    if let Some(syntax) = &mut app.language.syntax {
                        syntax.token_cache = None;
                        log_info!("[LangPanel] Token cache cleared");
                    }
                }
                (LangTab::Syntax, 'o') => self.open_grammar_config_in_editor(app),

                // Install tab
                (LangTab::Install, 'h') => self.install_focus = InstallFocus::Grammars,
                (LangTab::Install, 'l') => self.install_focus = InstallFocus::LspServers,

                _ => {}
            },
        }

        ModalResponse::Consumed
    }

    fn render(&mut self, frame: &mut Frame, app: &App) {
        let area = centered_rect(80, 75, frame.area());

        // render on top of whatever is already drawn
        frame.render_widget(Clear, area);

        let outer = Block::default().borders(Borders::ALL);

        frame.render_widget(outer.clone(), area);
        let inner = outer.inner(area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Tab bar
                Constraint::Min(0),    // Content
            ])
            .split(inner);

        let tab_titles = vec![
            Line::from(" [1] Diagnostics "),
            Line::from(" [2] LSP "),
            Line::from(" [3] Syntax "),
            Line::from(" [4] Install "),
        ];
        let tabs = Tabs::new(tab_titles)
            .select(self.tab.index())
            .style(Style::default().fg(Color::DarkGray))
            .highlight_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(40, 40, 55)),
            )
            .divider("|");
        frame.render_widget(tabs, chunks[0]);

        match self.tab {
            LangTab::Diagnostics => self.render_diagnostics_tab(frame, chunks[1], app),
            LangTab::Lsp => self.render_lsp_tab(frame, chunks[1], app),
            LangTab::Syntax => self.render_syntax_tab(frame, chunks[1], app),
            LangTab::Install => self.render_install_tab(frame, chunks[1], app),
        }
    }
}

// ----------   HELPERS   --------------

/// Returns Line with key and value in specific style
fn row_kv<'a>(key: &'a str, value: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!("  {:<12}", key),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(value.to_string(), Style::default().fg(Color::White)),
    ])
}

/// Returns Line with key and value in passed style
fn row_kv_styled<'a>(key: &'a str, value: &'a str, style: Style, icon: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!("  {:<12}", key),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(format!("{} {}", icon, value), style),
    ])
}

/// Returns section header line from label
fn section_header(label: &str) -> Line<'_> {
    Line::from(Span::styled(
        format!("  {}", label),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    ))
}

/// Returns action row line from key and description
fn action_row(key: char, desc: &str) -> Line<'_> {
    Line::from(vec![
        Span::styled(
            format!("  [{}]  ", key),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(desc.to_string(), Style::default().fg(Color::White)),
    ])
}

/// Returns hint line from text passed
fn hint_line(text: &str) -> Line<'_> {
    Line::from(Span::styled(
        format!(" {}", text),
        Style::default().fg(Color::DarkGray),
    ))
}

/// Style of diagnostic severity
fn severity_style(sev: &DiagnosticSeverity) -> (&'static str, Style) {
    match sev {
        DiagnosticSeverity::Error => ("● ", Style::default().fg(Color::Red)),
        DiagnosticSeverity::Warning => ("◆ ", Style::default().fg(Color::Yellow)),
        DiagnosticSeverity::Information => ("◉ ", Style::default().fg(Color::Cyan)),
        DiagnosticSeverity::Hint => ("· ", Style::default().fg(Color::DarkGray)),
    }
}

/// Counts the number of diagnostics with specific DiagnosticSeverity
fn count_severity(app: &App, sev: DiagnosticSeverity) -> usize {
    app.language
        .lsp
        .as_ref()
        .map(|lsp| {
            lsp.diagnostics
                .values()
                .flat_map(|v| v.iter())
                .filter(|d| d.severity == sev)
                .count()
        })
        .unwrap_or(0)
}

/// Simple truncation
fn truncate(s: &str, max: usize) -> String {
    if max == 0 || s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}

/// Determines the UI display label and color status for a Tree-sitter syntax grammar installation.
///
/// Priority is given to active background installation jobs before checking
/// local disk availability.
///
/// # Returns
/// A tuple containing:
/// - `&'static str`: A formatted display string with status symbols (e.g., "✓ installed").
/// - `Color`: The associated UI theme color for styling the TUI element.
fn grammar_status(
    app: &App,
    spec: &crate::language::install::GrammarSpec,
) -> (&'static str, Color) {
    // Check if an installation job for this grammar is currently active or recently completed
    let id = crate::language::install::JobId::Grammar(spec.name.to_string());
    if let Some(status) = app.install_manager.job_status(&id) {
        return match status {
            crate::language::install::JobStatus::Running => ("… installing", Color::Yellow),
            crate::language::install::JobStatus::Success(_) => ("✓ installed", Color::Green),
            crate::language::install::JobStatus::Failed(_) => ("✗ failed", Color::Red),
        };
    }
    // Fallback: Check if the compiled grammar shared library already exists on disk
    let grammar_dir = crate::language::manager::resolve_grammar_dir(&app.config.syntax.grammar_dir);
    let filename =
        crate::language::grammar_loader::platform_lib_name(&format!("tree_sitter_{}", spec.name));
    if grammar_dir.join(filename).exists() {
        ("✓ installed", Color::Green)
    } else {
        ("· not installed", Color::DarkGray)
    }
}

/// Determines the UI display label and color status for a Language Server (LSP) installation.
///
/// Priority is given to active background installation jobs before checking
/// active server configurations.
///
/// # Returns
/// A tuple containing:
/// - `&'static str`: A formatted display string with status symbols (e.g., "✓ configured").
/// - `Color`: The associated UI theme color for styling the TUI element.
fn lsp_status(app: &App, spec: &crate::language::install::LspSpec) -> (&'static str, Color) {
    // Check if an installation job for this LSP server is currently active or recently completed
    let id = crate::language::install::JobId::LspServer(spec.name.to_string());
    if let Some(status) = app.install_manager.job_status(&id) {
        return match status {
            crate::language::install::JobStatus::Running => ("… installing", Color::Yellow),
            crate::language::install::JobStatus::Success(_) => ("✓ installed", Color::Green),
            crate::language::install::JobStatus::Failed(_) => ("✗ failed", Color::Red),
        };
    }
    // Fallback: Check if the language server is configured in the editor settings
    if app.config.lsp.servers.contains_key(spec.name) {
        ("✓ configured", Color::Green)
    } else {
        ("· not installed", Color::DarkGray)
    }
}

/// Convert an LSP `file://` URI to a [`PathBuf`].
///
/// Returns `None` if the URI does not start with `file://` or if the
/// resulting path is empty.  Percent-decoding is used for the most
/// common case (spaces encoded as `%20`);.
fn uri_to_path(uri: &str) -> Option<std::path::PathBuf> {
    // LSP URIs are always "file://<authority><path>".
    // On Unix:   file:///home/user/foo.rs  -> strip "file://" -> /home/user/foo.rs  ✓
    // On Windows: file:///C:/Users/foo.rs  -> strip "file://" -> /C:/Users/foo.rs
    //   The leading '/' before the drive letter is not valid on Windows, so strip it.
    let path_str = uri.strip_prefix("file://")?;

    // Full percent-decoder — covers the characters LSP servers actually encode.
    let decoded = percent_decode(path_str);

    // On Windows the path looks like "/C:/Users/…" after stripping "file://".
    // Detect this by checking for "/<letter>:/" and drop the leading slash.
    #[cfg(target_os = "windows")]
    let decoded = {
        let bytes = decoded.as_bytes();
        if bytes.len() >= 3
            && bytes[0] == b'/'
            && bytes[1].is_ascii_alphabetic()
            && bytes[2] == b':'
        {
            decoded[1..].to_string()
        } else {
            decoded
        }
    };

    let path = std::path::PathBuf::from(decoded);
    if path.as_os_str().is_empty() {
        None
    } else {
        Some(path)
    }
}

/// Percent-decode a URI path component.
///
/// Handles every `%XX` sequence an LSP server is likely to emit:
/// spaces (`%20`), colons (`%3A`/`%3a`), and anything else in the
/// ASCII range.  Non-UTF-8 sequences are passed through
/// unchanged so the caller still gets a usable (if odd) path.
fn percent_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            // Collect the next two hex digits
            let h1 = chars.next();
            let h2 = chars.next();
            if let (Some(h1), Some(h2)) = (h1, h2) {
                let hex = format!("{}{}", h1, h2);
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    out.push(byte as char);
                    continue;
                }
                // Not valid hex — emit literally
                out.push('%');
                out.push(h1);
                out.push(h2);
            } else {
                out.push('%');
            }
        } else {
            out.push(c);
        }
    }
    out
}
