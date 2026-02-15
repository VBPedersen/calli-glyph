use crate::core::debug::{LogEntry, LogLevel};
use crate::ui::debug::DebugView;
use ratatui::style::Modifier;
use ratatui::widgets::ListState;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

//TODO scroll when going beyond the logs on the screen
pub fn render_logs(frame: &mut Frame, state: &mut DebugView, area: Rect) {
    let log_entries: Vec<LogEntry> = crate::core::debug::get_all_logs();
    state.max_logs = log_entries.len();

    let mut list_state = ListState::default().with_selected(state.selected_log);
    let entries: Vec<ListItem> = log_entries
        .iter()
        .enumerate()
        .rev()
        .map(|(_i, entry)| {
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
        .title(format!("Event Log ({} entries)", log_entries.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let list = List::new(entries).block(block).highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_stateful_widget(list, area, &mut list_state);
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
