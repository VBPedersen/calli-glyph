use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::core::app::App;
use crate::core::debug::LogEntry;
use crate::ui::debug_console::{
    action_history, clipboard_view, log_viewer, logs_list, overview, performance_viewer,
    snapshot_viewer, snapshots_list,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugTab {
    Overview,
    Logs,
    LogViewer,
    Clipboard,
    History,
    Snapshots,
    Performance,
    SnapshotViewer,
}

///Defines view of debug console
#[derive(Debug)]
pub struct DebugView {
    pub active_tab: DebugTab,
    pub scroll_offset: usize,
    pub selected_snapshot: Option<usize>, //snapshot currently selected
    pub viewing_snapshot: bool,           // is viewing snapshot
    pub max_logs: usize,                  // current max amount of logs, should be updated when new
    pub selected_log: Option<usize>,      // log entry currently selected
    pub active_log_entry: Option<LogEntry>, // the log being viewed
    pub viewing_log: bool,                // is viewing log entry
}

impl DebugView {
    pub fn new() -> Self {
        Self {
            active_tab: DebugTab::Overview,
            scroll_offset: 0,
            selected_snapshot: None,
            viewing_snapshot: false,
            max_logs: 0,
            selected_log: None,
            active_log_entry: None,
            viewing_log: false,
        }
    }

    //SNAPSHOT VIEWER

    pub fn select_next_snapshot(&mut self, max: usize) {
        self.selected_snapshot = Some(self.selected_snapshot.map(|i| (i + 1) % max).unwrap_or(0));
    }

    pub fn select_prev_snapshot(&mut self, max: usize) {
        self.selected_snapshot = Some(
            self.selected_snapshot
                .map(|i| (i + max - 1) % max)
                .unwrap_or(max - 1),
        );
    }

    pub fn open_snapshot_viewer(&mut self) {
        if self.selected_snapshot.is_some() {
            self.viewing_snapshot = true;
            self.active_tab = DebugTab::SnapshotViewer;
        }
    }

    pub fn close_snapshot_viewer(&mut self) {
        self.viewing_snapshot = false;
        self.active_tab = DebugTab::Snapshots;
    }

    pub fn open_log_entry(&mut self) {
        if let Some(idx) = self.selected_log {
            let logs = crate::core::debug::get_all_logs();
            // Capture the log once.
            if let Some(entry) = logs.iter().rev().nth(idx) {
                self.active_log_entry = Some(entry.clone());
                self.viewing_log = true;
                self.active_tab = DebugTab::LogViewer;
            }
        }
    }

    pub fn close_log_entry(&mut self) {
        self.viewing_log = false;
        self.active_tab = DebugTab::Logs;
        self.active_log_entry = None;
    }

    //------------------------------------

    pub fn select_next_log(&mut self, max: usize) {
        self.selected_log = Some(self.selected_log.map(|i| (i + 1) % max).unwrap_or(0));
    }

    pub fn select_prev_log(&mut self, max: usize) {
        self.selected_log = Some(
            self.selected_log
                .map(|i| (i + max - 1) % max)
                .unwrap_or(max - 1),
        );
    }

    pub fn next_tab(&mut self) {
        self.active_tab = match self.active_tab {
            DebugTab::Overview => DebugTab::Logs,
            DebugTab::Logs => DebugTab::Clipboard,
            DebugTab::Clipboard => DebugTab::History,
            DebugTab::History => DebugTab::Snapshots,
            DebugTab::Snapshots => DebugTab::Performance,
            DebugTab::Performance => DebugTab::Overview,
            _ => self.active_tab, //exclude any tab not wanted in navigation, and stay
        };
        self.scroll_offset = 0;
    }

    pub fn prev_tab(&mut self) {
        self.active_tab = match self.active_tab {
            DebugTab::Overview => DebugTab::Performance,
            DebugTab::Logs => DebugTab::Overview,
            DebugTab::Clipboard => DebugTab::Logs,
            DebugTab::History => DebugTab::Clipboard,
            DebugTab::Snapshots => DebugTab::History,
            DebugTab::Performance => DebugTab::Snapshots,
            _ => self.active_tab, //exclude any tab not wanted in navigation, and stay
        };
        self.scroll_offset = 0;
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }
}

// ██████╗  █████╗ ███╗   ██╗███████╗██╗         ██████╗ ███████╗███╗   ██╗██████╗ ███████╗██████╗ ██╗███╗   ██╗ ██████╗
// ██╔══██╗██╔══██╗████╗  ██║██╔════╝██║         ██╔══██╗██╔════╝████╗  ██║██╔══██╗██╔════╝██╔══██╗██║████╗  ██║██╔════╝
// ██████╔╝███████║██╔██╗ ██║█████╗  ██║         ██████╔╝█████╗  ██╔██╗ ██║██║  ██║█████╗  ██████╔╝██║██╔██╗ ██║██║  ███╗
// ██╔═══╝ ██╔══██║██║╚██╗██║██╔══╝  ██║         ██╔══██╗██╔══╝  ██║╚██╗██║██║  ██║██╔══╝  ██╔══██╗██║██║╚██╗██║██║   ██║
// ██║     ██║  ██║██║ ╚████║███████╗███████╗    ██║  ██║███████╗██║ ╚████║██████╔╝███████╗██║  ██║██║██║ ╚████║╚██████╔╝
// ╚═╝     ╚═╝  ╚═╝╚═╝  ╚═══╝╚══════╝╚══════╝    ╚═╝  ╚═╝╚══════╝╚═╝  ╚═══╝╚═════╝ ╚══════╝╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝ ╚═════╝

/// Debug panel
pub fn render_debug_panel(frame: &mut Frame, app: &mut App, area: Rect) {
    if !app.debug_state.enabled {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tab bar
            Constraint::Min(0),    // Content
            Constraint::Length(2), // Help bar
        ])
        .split(area);

    render_tabs(frame, chunks[0], &app.debug_view);

    match app.debug_view.active_tab {
        DebugTab::Overview => overview::render_overview(frame, app, chunks[1]),
        DebugTab::Logs => logs_list::render_logs(frame, &mut app.debug_view, chunks[1]),
        DebugTab::LogViewer => log_viewer::render_log_viewer(frame, app, chunks[1]),
        DebugTab::Clipboard => clipboard_view::render_clipboard(frame, app, chunks[1]),
        DebugTab::History => action_history::render_history(frame, app, chunks[1]),
        DebugTab::Snapshots => snapshots_list::render_snapshots_list(frame, app, chunks[1]),
        DebugTab::SnapshotViewer => snapshot_viewer::render_snapshot_viewer(frame, app, chunks[1]),
        DebugTab::Performance => performance_viewer::render_performance(frame, app, chunks[1]),
    }

    help_bar(frame, chunks[2]);
}

fn render_tabs(frame: &mut Frame, area: Rect, view: &DebugView) {
    let titles = vec![
        "Overview",
        "Logs",
        "Clipboard",
        "History",
        "Snapshots",
        "Performance",
    ];
    let selected = match view.active_tab {
        DebugTab::Overview => 0,
        DebugTab::Logs => 1,
        DebugTab::Clipboard => 2,
        DebugTab::History => 3,
        DebugTab::Snapshots => 4,
        DebugTab::Performance => 5,
        _ => {
            return;
        } //ignore other tabs
    };

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Debug Console"),
        )
        .select(selected)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(tabs, area);
}

fn help_bar(frame: &mut Frame, area: Rect) {
    let instructions_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::new().fg(Color::LightYellow));
    let instructions_text = vec![Line::from(
        "ESC/Q: Exit | Tab/Shift+Tab: Switch Tab | s: Snapshot | c: Clear Logs | C: Clear Snapshots",
    )];
    let instructions_paragraph = Paragraph::new(instructions_text).block(instructions_block);
    frame.render_widget(instructions_paragraph, area);
}
