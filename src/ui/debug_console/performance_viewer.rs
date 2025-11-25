use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Sparkline},
    Frame,
};

use crate::core::app::App;
use crate::core::debug::PerformanceMetrics;

pub fn render_performance(frame: &mut Frame, app: &App, area: Rect) {
    let metrics = &app.debug_state.metrics;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // Stats
            Constraint::Min(8),     // Frame time graph
        ])
        .split(area);

    // Performance statistics
    render_performance_stats(frame, metrics, chunks[0]);

    // Frame time visualization
    render_frame_time_graph(frame, metrics, chunks[1]);
}

fn render_performance_stats(f: &mut Frame, metrics: &PerformanceMetrics, area: Rect) {
    let avg_ms = metrics.avg_frame_time().as_secs_f64() * 1000.0;
    let min_ms = metrics.min_frame_time().as_secs_f64() * 1000.0;
    let max_ms = metrics.max_frame_time().as_secs_f64() * 1000.0;
    let fps = metrics.fps();

    // Color code FPS
    let fps_color = if fps >= 55.0 {
        Color::Green
    } else if fps >= 30.0 {
        Color::Yellow
    } else {
        Color::Red
    };

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  Current FPS: "),
            Span::styled(
                format!("{:.1}", fps),
                Style::default().fg(fps_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(format!("  Average Frame Time: {:.2}ms", avg_ms)),
        Line::from(format!("  Min Frame Time:     {:.2}ms", min_ms)),
        Line::from(format!("  Max Frame Time:     {:.2}ms", max_ms)),
        Line::from(""),
        Line::from(format!("  Total Renders: {}", metrics.render_count)),
        Line::from(format!("  Total Events:  {}", metrics.event_count)),
    ];

    let block = Block::default()
        .title("Performance Metrics")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

fn render_frame_time_graph(frame: &mut Frame, metrics: &PerformanceMetrics, area: Rect) {
    // Convert frame times to u64 for sparkline (in microseconds)
    let data: Vec<u64> = metrics
        .frame_times
        .iter()
        .map(|d| d.as_micros() as u64)
        .collect();

    if data.is_empty() {
        let text = Paragraph::new("No frame data yet...").block(
            Block::default()
                .title("Frame Time History")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(text, area);
        return;
    }

    let max_value = *data.iter().max().unwrap_or(&1);

    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .title(format!("Frame Time History (last {} frames)", data.len()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .data(&data)
        .max(max_value)
        .style(Style::default().fg(Color::Green));

    frame.render_widget(sparkline, area);
}
