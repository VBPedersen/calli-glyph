use crate::core::app::App;
use crate::core::debug::LogLevel;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Line, Modifier, Span, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

pub fn render_overview(frame: &mut Frame, app: &App, area: Rect) {
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
            Span::raw("Avg Frame Time: "),
            Span::styled(
                format!("{:?}", metrics.avg_frame_time()),
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

fn render_app_state_summary(frame: &mut Frame, app: &App, area: Rect) {
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
    frame.render_widget(paragraph, area);
}

fn render_recent_logs(frame: &mut Frame, app: &App, area: Rect) {
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
    frame.render_widget(list, area);
}

///style of log level to show
fn get_log_level_style(level: LogLevel) -> Style {
    match level {
        LogLevel::Error => Style::default().fg(Color::Red),
        LogLevel::Warn => Style::default().fg(Color::Yellow),
        LogLevel::Info => Style::default().fg(Color::Blue),
        LogLevel::Debug => Style::default().fg(Color::Gray),
        LogLevel::Trace => Style::default().fg(Color::DarkGray),
    }
}
