use crate::core::app::App;
use ratatui::style::Modifier;
use ratatui::widgets::List;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, ListItem, Paragraph},
    Frame,
};

pub fn render_clipboard(frame: &mut Frame, app: &App, area: Rect) {
    let clipboard = &app.editor.clipboard;

    if clipboard.copied_text.is_empty() {
        let text = vec![
            Line::from(""),
            Line::from("  Clipboard is empty"),
            Line::from(""),
            Line::from("  Copy some text to see it here."),
        ];

        let block = Block::default()
            .title("Clipboard")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta));

        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    // Create scrollable list of clipboard entries
    let items: Vec<ListItem> = clipboard
        .copied_text
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            // Show preview of entry (first 100 chars)
            let preview = if entry.len() > 100 {
                format!("{}...", &entry[..100])
            } else {
                entry.clone()
            };

            // Replace newlines with visual indicator
            let preview = preview.replace('\n', "↵ ");

            // Count lines in entry
            let line_count = entry.lines().count();
            let char_count = entry.len();

            let content = vec![
                Line::from(vec![
                    Span::styled(
                        format!("{:3}: ", i),
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("[{} lines, {} chars] ", line_count, char_count),
                        Style::default().fg(Color::Blue),
                    ),
                ]),
                Line::from(vec![Span::raw("     "), Span::raw(preview)]),
            ];

            ListItem::new(content)
        })
        .collect();

    let block = Block::default()
        .title(format!(
            "Clipboard ({} entries)",
            clipboard.copied_text.len()
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
