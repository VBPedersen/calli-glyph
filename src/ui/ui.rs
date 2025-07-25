use crate::core::app::{ActiveArea, App};
use crate::config::editor_settings;
use crate::core::cursor::CursorPosition;
use ratatui::layout::{Alignment, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::Block,
    Frame,
};
use std::default::Default;
use std::vec;

pub fn ui(frame: &mut Frame, app: &mut App) {
    app.terminal_height = frame.area().height as i16;


    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(1),
            Constraint::Percentage(95),
            Constraint::Length(1),
        ])
        .split(frame.area());
    app.editor.editor_height = layout[1].height;

    let editor_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Length(3), Constraint::Percentage(100)])
        .split(layout[1]);

    app.editor.editor_width = editor_layout[1].width as i16;

    let editor_content: Text = handle_editor_content(
        app.editor.editor_content.clone(),
        app.editor.text_selection_start,
        app.editor.text_selection_end,
        editor_layout[1].width as usize,
        app,
    );

    let command_input: String = app.command_line.input.to_string();
    let file_name_optional: Option<String> = app.file_path.clone();
    let file_to_use = if let Some(file) = file_name_optional {
        file
    } else {
        "untitled".to_string()
    };

    //render widgets : infobar, editor side, editor and command line
    frame.render_widget(
        info_bar(
            file_to_use,
            app.editor.cursor.x,
            app.editor.cursor.y,
            app.editor.visual_cursor_x,
            app.editor.text_selection_start,
            app.editor.text_selection_end,
        ),
        layout[0],
    );
    frame.render_widget(
        editor_side_line(
            editor_content.to_owned(),
            app.editor.scroll_offset as u16,
            editor_layout[1].width as usize,
            app.editor.cursor.y,
        ),
        editor_layout[0],
    );
    frame.render_widget(
        editor(editor_content, app.editor.scroll_offset as u16),
        editor_layout[1],
    );
    frame.render_widget(command_line(command_input), layout[2]);

    //if popup is any, then render it
    if let Some(popup) = &app.popup {
        let popup_area = centered_rect(60, 20, frame.area());
        popup.render(frame, popup_area);
    }

    //set cursor with position if it should be visiblie (determined by app logic)
    if app.cursor_visible {
        match app.active_area {
            ActiveArea::Editor => {
                let x = editor_layout[1].x + app.editor.visual_cursor_x as u16; //using visual x
                let y = editor_layout[1].y
                    + (app.editor.cursor.y - app.editor.scroll_offset).clamp(0, i16::MAX) as u16;
                let pos: Position = Position { x, y };

                frame.set_cursor_position(pos);
            }
            ActiveArea::CommandLine => {
                let x = layout[2].x + app.command_line.cursor.x as u16;
                let y = layout[2].y + app.command_line.cursor.y as u16;
                let pos: Position = Position { x, y };
                frame.set_cursor_position(pos);
            }
            ActiveArea::Popup => {}
        }
    }
}

///returns centered rect based on height,width and current screen Rect to use in layout
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = (area.width - width) / 2;
    let y = (area.height - height) / 2;
    Rect {
        x,
        y,
        width,
        height,
    }
}

//COMPONENTS
fn info_bar<'a>(
    file_name: String,
    cursor_x: i16,
    cursor_y: i16,
    visual_x: i16,
    selection_start: Option<CursorPosition>,
    selection_end: Option<CursorPosition>,
) -> Paragraph<'a> {
    let mut start_x: usize = 0;
    let mut start_y: usize = 0;
    let mut end_x: usize = 0;
    let mut end_y: usize = 0;
    if selection_start.is_some() && selection_end.is_some() {
        start_x = selection_start.unwrap().x;
        start_y = selection_start.unwrap().y;
        end_x = selection_end.unwrap().x;
        end_y = selection_end.unwrap().y;
    }
    let line = Line::from(vec![
        Span::styled(file_name, Style::default().fg(Color::LightCyan)),
        Span::raw(" - "), // Separator
        Span::styled(
            format!(
                "Cursor: ({}, {})   Visual X Cursor ({})  Selection Cursor ({},{}) ({},{})",
                cursor_x, cursor_y, visual_x, start_x, start_y, end_x, end_y
            ),
            Style::default().fg(Color::Magenta),
        ),
    ]);
    Paragraph::new("").block(
        Block::default()
            .title(line)
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(Color::LightCyan).bg(Color::White)),
    )
}

///generates a side bar for line nr display as well as displaying line overflow if existing
fn editor_side_line(
    editor_content: Text,
    scroll_offset: u16,
    editor_width: usize,
    cursor_y: i16,
) -> Paragraph {
    let mut line_nrs: Text = Text::from(vec![]);

    let overflow_marker_style = Style::default().fg(Color::Cyan);
    let current_line_style = Style::default().bg(Color::White).fg(Color::Black);

    for (nr, s) in editor_content.iter().enumerate() {
        let dest_to_cursor_y = cursor_y.abs_diff(nr as i16);

        if s.width() >= editor_width {
            let line = Line::from(vec![
                Span::raw(dest_to_cursor_y.to_string()),
                Span::styled(">", overflow_marker_style),
            ]);
            //if is zero (current line), display actual line nr
            if dest_to_cursor_y == 0 {
                let line = Line::from(vec![
                    Span::styled(dest_to_cursor_y.to_string(), current_line_style),
                    Span::styled(">", overflow_marker_style),
                ]);
                line_nrs.push_line(line);
            } else {
                line_nrs.push_line(line);
            }
        } else if dest_to_cursor_y == 0 {
                let line = Line::from(vec![Span::styled(nr.to_string(), current_line_style)]);
                line_nrs.push_line(line);
            } else {
                line_nrs.push_line(dest_to_cursor_y.to_string());
            }

    }

    Paragraph::new(line_nrs)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .block(
            Block::default(), //.borders(Borders::LEFT | Borders::RIGHT)
                              //.border_type(BorderType::Rounded)
        )
        .scroll((scroll_offset, 0))
}

fn editor(editor_content: Text, scroll_offset: u16) -> Paragraph {
    Paragraph::new(editor_content)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default(), //.borders(Borders::LEFT | Borders::RIGHT)
                              //.border_type(BorderType::Rounded)
        )
        .scroll((scroll_offset, 0))
}

fn command_line<'a>(command_input: String) -> Paragraph<'a> {
    Paragraph::new(command_input)
        .style(Style::default().fg(Color::White).bg(Color::Cyan))
        .block(
            Block::default(), //.borders(Borders::ALL)
                              //.title("")
                              //.border_type(BorderType::Thick)
        )
}

//HELPER FUNCTIONS

///manipulates how the editor content is rendered, specifically how certain characters in the
/// content is interpreted visually
fn handle_editor_content<'a>(
    vec: Vec<String>,
    selection_start: Option<CursorPosition>,
    selection_end: Option<CursorPosition>,
    editor_width: usize,
    app: &mut App,
) -> Text<'a> {
    let editor_vec: Vec<String> = vec.into_iter().map(handle_tab_rendering).collect();

    let mut editor_text: Text = Text::default();

    if selection_start.is_some() {
        editor_text = highlight_text(editor_vec.clone(), selection_start, selection_end);
    } else {
        for (i, s) in editor_vec.into_iter().enumerate() {

            let visual_x = app.editor.visual_cursor_x;

            // Only scroll the line the cursor is on
            let line: Line = if i == app.editor.cursor.y as usize && visual_x > editor_width as i16 {
                let start_idx = (visual_x - editor_width as i16).max(0) as usize;
                Line::from(
                    get_copy_of_editor_content_at_line_between_cursor_editor_width(s, start_idx),
                )
            } else {
                Line::from(s)
            };

            editor_text.push_line(line);
        }
    }

    editor_text
}

///gets a copy of the text content at specific line and range of editor content
pub(crate) fn get_copy_of_editor_content_at_line_between_cursor_editor_width(
    s: String,
    start: usize,
) -> String {
    let mut line_chars_vec: Vec<char> = s.chars().collect();
    line_chars_vec.drain(start..).collect()
}

///manipulates how the editor content \t character is rendered visually
fn handle_tab_rendering(s: String) -> String {
    let mut temp_string: Vec<char> = s.chars().collect();
    let tab_width = editor_settings::TAB_WIDTH;

    let mut i = 0;

    while i < temp_string.len() {
        if temp_string[i] == '\t' {
            let spaces_needed = tab_width as usize - ((i as i16) as usize % tab_width as usize);

            temp_string.remove(i);
            temp_string.splice(i..i, std::iter::repeat(' ').take(spaces_needed));

            i += spaces_needed - 1; // Adjust index for added spaces
        }
        i += 1;
    }

    temp_string.into_iter().collect()
}

//TEXT HIGHLIGTHING

fn highlight_text<'a>(
    text: Vec<String>,
    start: Option<CursorPosition>,
    end: Option<CursorPosition>,
) -> Text<'a> {
    let mut highlighted_lines = Vec::new();

    for (i, line) in text.iter().enumerate() {
        let mut spans = Vec::new();

        if i < start.unwrap().y || i > end.unwrap().y {
            spans.push(Span::raw(line.clone())); // No selection on this line
        } else {
            let start_col = if i == start.unwrap().y {
                start.unwrap().x
            } else {
                0
            };
            let end_col = if i == end.unwrap().y {
                end.unwrap().x
            } else {
                line.len()
            };

            // Ensure selection is within valid bounds
            let start_col = start_col.min(line.len());
            let end_col = end_col.min(line.len());

            spans.push(Span::raw(line[..start_col].to_string())); // Before selection
            spans.push(Span::styled(
                line[start_col..end_col].to_string(), // Highlighted text
                Style::default()
                    .bg(Color::White)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::raw(line[end_col..].to_string())); // After selection
        }

        highlighted_lines.push(Line::from(spans));
    }

    Text::from(highlighted_lines)
}
