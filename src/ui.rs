use std::default::Default;
use std::vec;
use ratatui::{layout::{Constraint, Direction, Layout}, widgets::{Block}, Frame, };
use ratatui::layout::{Alignment, Position};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Paragraph};
use crate::app::{ActiveArea, App};

use crate::config::editor_settings;
use crate::cursor::CursorPosition;

pub fn ui(frame: &mut Frame, app: &mut App) {
    app.terminal_height = frame.area().height as i16;

    let editor_content: Text = handle_editor_content(app.editor.editor_content.clone(), app.editor.text_selection_start, app.editor.text_selection_end);


    let command_input:String = app.command_line.input.to_string();
    let file_name_optional:Option<String> = app.file_path.clone();
    let file_to_use: String;
    if file_name_optional.is_some() {
        file_to_use = file_name_optional.unwrap();
    } else {
        file_to_use = "untitled".to_string();
    }

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(1),
            Constraint::Percentage(95),
            Constraint::Length(1),
        ])
        .split(frame.area());


    frame.render_widget(
        info_bar(file_to_use, app.editor.cursor.x, app.editor.cursor.y, app.editor.visual_cursor_x,app.editor.text_selection_start,app.editor.text_selection_end),
        layout[0],
    );
    frame.render_widget(
        editor(editor_content, app.scroll_offset as u16),
        layout[1],
    );
    frame.render_widget(
        command_line(command_input),
        layout[2],
    );

    //set cursor with position if it should be visiblie (determined by app logic)
    if app.cursor_visible {
        match app.active_area {
            ActiveArea::Editor => {
                let x = layout[1].x + app.editor.visual_cursor_x as u16; //using visual x
                let y = layout[1].y + (app.editor.cursor.y - app.scroll_offset).clamp(0,i16::MAX) as u16;
                let pos: Position = Position { x, y };
                frame.set_cursor_position(pos);
            },
            ActiveArea::CommandLine => {
                let x = layout[2].x + app.command_line.cursor.x as u16;
                let y = layout[2].y + app.command_line.cursor.y as u16;
                let pos: Position = Position { x, y };
                frame.set_cursor_position(pos);
            },
        }

    }
}


//COMPONENTS
fn info_bar<'a>(file_name:String, cursor_x: i16, cursor_y:i16,visual_x:i16,selection_start:Option<CursorPosition>,selection_end:Option<CursorPosition>) -> Paragraph<'a> {
    let mut start_x:usize = 0;
    let mut start_y:usize = 0;
    let mut end_x:usize = 0;
    let mut end_y:usize = 0;
    if selection_start.is_some() && selection_end.is_some() {
        start_x = selection_start.unwrap().x;
        start_y = selection_start.unwrap().y;
        end_x = selection_end.unwrap().x;
        end_y = selection_end.unwrap().y;
    }
    let line = Line::from(vec![
        Span::styled(file_name, Style::default().fg(Color::LightCyan)),
        Span::raw(" - "), // Separator
        Span::styled(format!("Cursor: ({}, {})   Visual X Cursor ({})  Selection Cursor ({},{}) ({},{})",
                             cursor_x, cursor_y, visual_x,start_x,start_y,end_x,end_y), Style::default().fg(Color::Magenta)),
    ]);
    Paragraph::new("")
        .block(
            Block::default()
                .title(line)
                .title_alignment(Alignment::Center)
                .style(Style::default().fg(Color::LightCyan).bg(Color::White))
        )
}

fn editor(editor_content: Text, scroll_offset: u16) -> Paragraph {
    Paragraph::new(editor_content)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                //.borders(Borders::LEFT | Borders::RIGHT)
                //.border_type(BorderType::Rounded)
        ).scroll((scroll_offset, 0))
}

fn command_line<'a>(command_input: String) -> Paragraph<'a> {
    Paragraph::new(command_input)
        .style(Style::default().fg(Color::White).bg(Color::Cyan))
        .block(
            Block::default()
                //.borders(Borders::ALL)
                //.title("")
                //.border_type(BorderType::Thick)
        )
}

//HELPER FUNCTIONS



///manipulates how the editor content is rendered, specifically how certain characters in the
/// content is interpreted visually
fn handle_editor_content<'a>(vec: Vec<String>, selection_start:Option<CursorPosition>, selection_end:Option<CursorPosition>) -> Text<'a> {
    let mut editor_vec: Vec<String> = Vec::new();
    for s in vec.into_iter() {
        let processed_string = handle_tab_rendering(s);
        editor_vec.push(processed_string);
    }


    let mut editor_text:Text= Text::default();

    //if some text is selected, calculate highlight
    if selection_start.is_some() {
        editor_text = highlight_text(editor_vec, selection_start, selection_end);
    } else {
        for s in editor_vec.iter() {

            let line:Line = Line::from(s.to_string());
            editor_text.push_line(line);
        }
    }

    editor_text
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

fn highlight_text<'a>(text: Vec<String>, start: Option<CursorPosition>, end: Option<CursorPosition>) -> Text<'a> {
    let mut highlighted_lines = Vec::new();

    for (i, line) in text.iter().enumerate() {
        let mut spans = Vec::new();

        if i < start.unwrap().y || i > end.unwrap().y {
            spans.push(Span::raw(line.clone())); // No selection on this line
        } else {
            let start_col = if i == start.unwrap().y { start.unwrap().x } else { 0 };
            let end_col = if i == end.unwrap().y { end.unwrap().x } else { line.len() };

            // Ensure selection is within valid bounds
            let start_col = start_col.min(line.len());
            let end_col = end_col.min(line.len());

            spans.push(Span::raw(line[..start_col].to_string())); // Before selection
            spans.push(Span::styled(
                line[start_col..end_col].to_string(), // Highlighted text
                Style::default().bg(Color::White).fg(Color::Black).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::raw(line[end_col..].to_string())); // After selection
        }

        highlighted_lines.push(Line::from(spans));
    }

    Text::from(highlighted_lines)
}
