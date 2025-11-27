use crate::core::app::App;
use crate::core::debug::AppSnapshot;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Screen for viewing specific snapshot
pub fn render_snapshot_viewer(frame: &mut Frame, app: &App, area: Rect) {
    let snapshot_idx = match app.debug_view.selected_snapshot {
        Some(idx) => idx,
        None => {
            let para = Paragraph::new("No snapshot selected").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Snapshot Viewer"),
            );
            frame.render_widget(para, area);
            return;
        }
    };

    let snapshot = match app.debug_state.snapshots.get(snapshot_idx) {
        Some(s) => s,
        None => {
            let para = Paragraph::new("Snapshot not found").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Snapshot Viewer"),
            );
            frame.render_widget(para, area);
            return;
        }
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Header
            Constraint::Percentage(40), // Buffer content
            Constraint::Percentage(20), // Clipboard
            Constraint::Percentage(20), // History
            Constraint::Length(5),      // State info
        ])
        .split(area);

    // Header
    render_snapshot_header(frame, snapshot, snapshot_idx, chunks[0]);

    // Buffer content
    render_snapshot_buffer(frame, snapshot, chunks[1]);

    // Clipboard
    render_snapshot_clipboard(frame, snapshot, chunks[2]);

    // History
    render_snapshot_history(frame, snapshot, chunks[3]);

    // State info
    render_snapshot_state(frame, snapshot, chunks[4]);
}

fn render_snapshot_header(frame: &mut Frame, snapshot: &AppSnapshot, idx: usize, area: Rect) {
    let elapsed = snapshot.timestamp.elapsed();
    let trigger_str = format!("{:?}", snapshot.trigger);

    let text = vec![Line::from(vec![
        Span::styled(
            format!("Snapshot #{} ", idx),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("| Trigger: {} ", trigger_str)),
        Span::raw(format!("| {:.1}s ago", elapsed.as_secs_f64())),
    ])];

    let block = Block::default()
        .title("Snapshot Viewer (Esc to close)")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

fn render_snapshot_buffer(frame: &mut Frame, snapshot: &AppSnapshot, area: Rect) {
    let lines: Vec<Line> = snapshot
        .buffer_content
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let line_num = format!("{:4} ", i + 1);
            Line::from(vec![
                Span::styled(line_num, Style::default().fg(Color::DarkGray)),
                Span::raw(line),
            ])
        })
        .collect();

    let block = Block::default()
        .title(format!(
            "Buffer ({} lines) | Cursor: {:?}",
            snapshot.buffer_lines, snapshot.cursor_pos
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

fn render_snapshot_clipboard(frame: &mut Frame, snapshot: &AppSnapshot, area: Rect) {
    let items: Vec<ListItem> = snapshot
        .clipboard_entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let preview = if entry.len() > 80 {
                format!("{}...", &entry[..80])
            } else {
                entry.clone()
            };
            ListItem::new(Line::from(format!("{}: {}", i, preview)))
        })
        .collect();

    let block = Block::default()
        .title(format!("Clipboard ({} entries)", snapshot.clipboard_size))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_snapshot_history(frame: &mut Frame, snapshot: &AppSnapshot, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Undo stack
    let undo_items: Vec<ListItem> = snapshot
        .undo_stack
        .iter()
        .enumerate()
        .map(|(i, action)| ListItem::new(Line::from(format!("{}: {}", i, action))))
        .collect();

    let undo_block = Block::default()
        .title(format!("Undo ({} actions)", snapshot.undo_depth))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let undo_list = List::new(undo_items).block(undo_block);
    frame.render_widget(undo_list, chunks[0]);

    // Redo stack
    let redo_items: Vec<ListItem> = snapshot
        .redo_stack
        .iter()
        .enumerate()
        .map(|(i, action)| ListItem::new(Line::from(format!("{}: {}", i, action))))
        .collect();

    let redo_block = Block::default()
        .title(format!("Redo ({} actions)", snapshot.redo_depth))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let redo_list = List::new(redo_items).block(redo_block);
    frame.render_widget(redo_list, chunks[1]);
}

fn render_snapshot_state(frame: &mut Frame, snapshot: &AppSnapshot, area: Rect) {
    let text = vec![
        Line::from(format!("Active Area: {}", snapshot.active_area)),
        Line::from(format!(
            "File: {}",
            snapshot.file_path.as_ref().unwrap_or(&"None".to_string())
        )),
        Line::from(format!(
            " Frame Time: {}ms",
            snapshot.frame_time.as_secs_f64() * 1000.0
        )),
    ];

    let block = Block::default()
        .title("State")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}
