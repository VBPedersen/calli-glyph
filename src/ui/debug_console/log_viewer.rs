use crate::core::app::App;
use crate::core::debug::{LogEntry, LogLevel};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

pub fn render_log_viewer(frame: &mut Frame, app: &App, area: Rect) {
    let log_idx = match app.debug_view.selected_log {
        Some(idx) => idx,
        None => {
            render_error_placeholder(frame, "No log entry selected", area);
            return;
        }
    };

    let log = match app.debug_view.active_log_entry.clone() {
        Some(entry) => entry,
        None => {
            render_error_placeholder(frame, "No log entry found in active var", area);
            return;
        }
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header/Metadata
            Constraint::Min(1),    // Full Message
            Constraint::Length(3), // Footer (Shortcut hints)
        ])
        .split(area);

    render_log_header(frame, &log, log_idx, chunks[0]);
    render_log_content(frame, &log, chunks[1]);
    render_log_footer(frame, chunks[2]);
}

fn render_error_placeholder(frame: &mut Frame, msg: &str, area: Rect) {
    let para = Paragraph::new(msg)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Log Viewer"));
    frame.render_widget(para, area);
}

fn render_log_header(frame: &mut Frame, log: &LogEntry, idx: usize, area: Rect) {
    let style = get_log_level_style(log.level);
    let elapsed = log.timestamp.elapsed();

    let text = vec![Line::from(vec![
        Span::styled(
            format!(" LOG #{} ", idx),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | Level: "),
        Span::styled(
            format!("{:?} ", log.level),
            style.add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(" | {:.1}s ago ", elapsed.as_secs_f64())),
        Span::styled(
            format!(" | {}", log.time_at.to_rfc3339()),
            Style::default().fg(Color::DarkGray),
        ),
    ])];

    let block = Block::default()
        .title(" Log Information ")
        .borders(Borders::ALL)
        .border_style(style);

    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn render_log_content(frame: &mut Frame, log: &LogEntry, area: Rect) {
    let block = Block::default()
        .title(" Message Content ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let paragraph = Paragraph::new(log.message.as_str())
        .block(block)
        .wrap(Wrap { trim: false }) // Preserve formatting/indentation
        .style(Style::default().fg(Color::White));

    frame.render_widget(paragraph, area);
}

fn render_log_footer(frame: &mut Frame, area: Rect) {
    let text = Line::from(vec![
        Span::styled(
            " Esc ",
            Style::default().bg(Color::DarkGray).fg(Color::White),
        ),
        Span::raw(" Back to List  "),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    frame.render_widget(
        Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center),
        area,
    );
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
