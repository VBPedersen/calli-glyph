use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame,
};

use crate::core::app::App;
use crate::core::debug::{LogLevel, SnapshotTrigger};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugTab {
    Overview = 0,
    Logs = 1,
    Clipboard = 2,
    History = 3,
    Snapshots = 4,
    Performance = 5,
}

///Defines view of debug console
#[derive(Debug)]
pub struct DebugView {
    pub active_tab: DebugTab,
    pub scroll_offset: usize,
}

impl DebugView {
    pub fn new() -> Self {
        Self {
            active_tab: DebugTab::Overview,
            scroll_offset: 0,
        }
    }

    pub fn next_tab(&mut self) {
        self.active_tab = match self.active_tab {
            DebugTab::Overview => DebugTab::Logs,
            DebugTab::Logs => DebugTab::Clipboard,
            DebugTab::Clipboard => DebugTab::History,
            DebugTab::History => DebugTab::Snapshots,
            DebugTab::Snapshots => DebugTab::Performance,
            DebugTab::Performance => DebugTab::Overview,
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
pub fn render_debug_panel(frame: &mut Frame, app: &App, area: Rect) {
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
        DebugTab::Overview => render_overview(frame, app, chunks[1]),
        //DebugTab::Logs => render_logs(f, app, chunks[1]),
        //DebugTab::Clipboard => render_clipboard(f, app, chunks[1]),
        //DebugTab::History => render_history(f, app, chunks[1]),
        //DebugTab::Snapshots => render_snapshots(f, app, chunks[1]),
        //DebugTab::Performance => render_performance(f, app, chunks[1]),
        _ => {}
    }

    help_bar(frame,chunks[2]);
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

fn get_log_level_style(level: LogLevel) -> Style {
    match level {
        LogLevel::Error => Style::default().fg(Color::Red),
        LogLevel::Warn => Style::default().fg(Color::Yellow),
        LogLevel::Info => Style::default().fg(Color::Blue),
        LogLevel::Debug => Style::default().fg(Color::Gray),
        LogLevel::Trace => Style::default().fg(Color::DarkGray),
    }
}

//  ██████╗ ██╗   ██╗███████╗██████╗ ██╗   ██╗██╗███████╗██╗    ██╗
// ██╔═══██╗██║   ██║██╔════╝██╔══██╗██║   ██║██║██╔════╝██║    ██║
// ██║   ██║██║   ██║█████╗  ██████╔╝██║   ██║██║█████╗  ██║ █╗ ██║
// ██║   ██║╚██╗ ██╔╝██╔══╝  ██╔══██╗╚██╗ ██╔╝██║██╔══╝  ██║███╗██║
// ╚██████╔╝ ╚████╔╝ ███████╗██║  ██║ ╚████╔╝ ██║███████╗╚███╔███╔╝
//  ╚═════╝   ╚═══╝  ╚══════╝╚═╝  ╚═╝  ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝
fn render_overview(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Performance summary
            Constraint::Length(9), // App state
            Constraint::Min(0),    // Recent logs
        ])
        .split(area);

    render_performance_summary(frame, app, chunks[0]);
    render_app_state_summary(frame, app, chunks[1]);
    render_recent_logs(frame, app, chunks[2]);
}

fn render_performance_summary(frame: &mut Frame, app: &App, area: Rect) {
    let metrics = &app.debug_state.metrics;

    let text = vec![
        Line::from(vec![
            Span::raw("FPS: "),
            Span::styled(
                format!("{:.1}", metrics.fps()),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(format!(
            "Avg Frame: {:.2}ms",
            metrics.avg_frame_time().as_secs_f64() * 1000.0
        )),
        Line::from(format!(
            "Min/Max: {:.2}ms / {:.2}ms",
            metrics.min_frame_time().as_secs_f64() * 1000.0,
            metrics.max_frame_time().as_secs_f64() * 1000.0
        )),
        Line::from(format!(
            "Renders: {} | Events: {}",
            metrics.render_count, metrics.event_count
        )),
    ];

    let block = Block::default()
        .title("Performance")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

fn render_app_state_summary(f: &mut Frame, app: &App, area: Rect) {
    let logger = &app.debug_state.logger;

    let text = vec![
        Line::from(format!("Active Area: {:?}", app.active_area)),
        Line::from(format!("Capture Mode: {:?}", app.debug_state.capture_mode)),
        Line::from(""),
        Line::from(format!("Total Logs: {}", logger.entries().len())),
        Line::from(format!(
            "  Errors: {} | Warnings: {}",
            logger.count_by_level(LogLevel::Error),
            logger.count_by_level(LogLevel::Warn)
        )),
        Line::from(""),
        Line::from(format!("Snapshots: {}", app.debug_state.snapshots.len())),
    ];

    let block = Block::default()
        .title("Application State")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

fn render_recent_logs(f: &mut Frame, app: &App, area: Rect) {
    let entries: Vec<ListItem> = app
        .debug_state
        .logger
        .entries()
        .iter()
        .rev()
        .take(area.height.saturating_sub(2) as usize)
        .map(|entry| {
            let level_style = get_log_level_style(entry.level);
            let elapsed = entry.timestamp.elapsed();

            let content = Line::from(vec![
                Span::styled(
                    format!("{:5}", entry.level),
                    level_style.add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("[{:.1}s] ", elapsed.as_secs_f64()),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(&entry.message),
            ]);

            ListItem::new(content)
        })
        .collect();

    let block = Block::default()
        .title("Recent Logs")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let list = List::new(entries).block(block);
    f.render_widget(list, area);
}
