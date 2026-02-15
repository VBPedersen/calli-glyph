use crate::core::app::App;
use crate::core::editor::editor::EditAction;
use ratatui::style::Modifier;
use ratatui::text::Span;
use ratatui::widgets::{List, ListItem};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_history(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_undo_stack(frame, app, chunks[0]);
    render_redo_stack(frame, app, chunks[1]);
}

fn render_undo_stack(frame: &mut Frame, app: &App, area: Rect) {
    let undo_stack = &app.editor.undo_redo_manager.undo_stack;

    if undo_stack.is_empty() {
        let text = vec![Line::from(""), Line::from("  No undo history")];

        let block = Block::default()
            .title("Undo Stack - 0 actions")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));

        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = undo_stack
        .iter()
        .enumerate()
        .rev() // Show most recent first
        .map(|(i, action)| map_action_to_item(i, action))
        .collect();

    let block = Block::default()
        .title(format!("Undo Stack - {} actions", undo_stack.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_redo_stack(frame: &mut Frame, app: &App, area: Rect) {
    let redo_stack = &app.editor.undo_redo_manager.redo_stack;

    if redo_stack.is_empty() {
        let text = vec![Line::from(""), Line::from("  No redo history")];

        let block = Block::default()
            .title("Redo Stack - 0 actions")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = redo_stack
        .iter()
        .enumerate()
        .map(|(i, action)| map_action_to_item(i, action))
        .collect();

    let block = Block::default()
        .title(format!("Redo Stack - {} actions", redo_stack.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

/// Function to map editaction to ui element ListItem
fn map_action_to_item(index: usize, action: &EditAction) -> ListItem {
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::ListItem;

    let (action_type, detail, color) = match action {
        EditAction::Insert { pos, c } => (
            "Insert",
            format!("'{}' at {}:{}", c, pos.y, pos.x),
            Color::Green,
        ),
        EditAction::Delete { pos, deleted_char } => (
            "Delete",
            format!("'{}' at {}:{}", deleted_char, pos.y, pos.x),
            Color::Red,
        ),
        EditAction::Replace {
            start, old, new, ..
        } => (
            "Replace",
            format!("'{}' → '{}' at {}:{}", old, new, start.y, start.x),
            Color::Yellow,
        ),
        EditAction::InsertLines { start, lines } => (
            "Insert Lines",
            format!("{} lines at {}:{}", lines.len(), start.y, start.x),
            Color::Green,
        ),
        EditAction::DeleteLines { start, deleted } => (
            "Delete Lines",
            format!("{} lines at {}:{}", deleted.len(), start.y, start.x),
            Color::Red,
        ),
        EditAction::InsertRange { start, lines, .. } => (
            "Insert Range",
            format!("{} lines at {}:{}", lines.len(), start.y, start.x),
            Color::Green,
        ),
        EditAction::DeleteRange { start, deleted, .. } => (
            "Delete Range",
            format!("{} lines at {}:{}", deleted.len(), start.y, start.x),
            Color::Red,
        ),
        EditAction::ReplaceRange {
            start, old, new, ..
        } => (
            "Replace Range",
            format!(
                "{} → {} lines at {}:{}",
                old.len(),
                new.len(),
                start.y,
                start.x
            ),
            Color::Yellow,
        ),
        EditAction::SplitLine { pos, .. } => {
            ("Split Line", format!("at {}:{}", pos.y, pos.x), Color::Cyan)
        }
        EditAction::JoinLine { pos, .. } => {
            ("Join Line", format!("at {}:{}", pos.y, pos.x), Color::Cyan)
        }
        EditAction::Bulk(actions) => (
            "Bulk EditAction",
            format!("containing {} actions", actions.len()),
            Color::Magenta,
        ),
    };

    let content = Line::from(vec![
        Span::styled(
            format!("{:3}: ", index),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            format!("{:13} ", action_type),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::raw(detail),
    ]);

    ListItem::new(content)
}
