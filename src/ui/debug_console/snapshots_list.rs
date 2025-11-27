use crate::core::app::App;
use crate::core::debug::SnapshotTrigger;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

/// interactable list of snapshots
pub fn render_snapshots_list(frame: &mut Frame, app: &App, area: Rect) {
    let snapshots = app.debug_state.snapshots.snapshots();

    if snapshots.is_empty() {
        return;
    }

    let items: Vec<ListItem> = snapshots
        .iter()
        .enumerate()
        .rev()
        .map(|(i, snapshot)| {
            let elapsed = snapshot.timestamp.elapsed();
            let (trigger_str, trigger_color) = match &snapshot.trigger {
                SnapshotTrigger::Error(e) => (format!("ERROR: {}", e), Color::Red),
                SnapshotTrigger::Command(c) => (format!("CMD: {}", c), Color::Blue),
                _ => (format!("{:?}", snapshot.trigger), Color::Gray),
            };

            let is_selected = app.debug_view.selected_snapshot == Some(i);
            let style = if is_selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            let content = Line::from(vec![
                Span::styled(format!("#{:03} ", i), Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("[{:.1}s] ", elapsed.as_secs_f64()),
                    Style::default().fg(Color::Gray),
                ),
                Span::styled(trigger_str, Style::default().fg(trigger_color)),
            ]);

            ListItem::new(content).style(style)
        })
        .collect();

    let block = Block::default()
        .title(format!(
            "Snapshots ({}/{}) - Enter to view",
            app.debug_view.selected_snapshot.map(|i| i + 1).unwrap_or(0),
            snapshots.len()
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
