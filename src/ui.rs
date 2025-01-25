use std::fmt::Debug;
use color_eyre::owo_colors::OwoColorize;
use ratatui::{layout::{Constraint, Direction, Layout}, widgets::{Block, Borders, }, Frame, };
use ratatui::layout::{Alignment, Position};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{BorderType, Paragraph};
use crate::app::{ActiveArea, App};


pub fn ui(frame: &mut Frame, app: &mut App) {
    app.terminal_height = frame.area().height as i16;
    let editor_content: String = app.editor_content.join("\n");
    let command_input:String = app.command_input.to_string();
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
        info_bar(file_to_use, app.cursor_x, app.cursor_y),
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
                let x = layout[1].x + app.cursor_x as u16;
                let y = layout[1].y + (app.cursor_y - app.scroll_offset).clamp(0,i16::MAX) as u16;
                let pos: Position = Position { x, y };
                frame.set_cursor_position(pos);
            },
            ActiveArea::CommandLine => {
                let x = layout[2].x + app.cursor_x as u16;
                let y = layout[2].y + app.cursor_y as u16;
                let pos: Position = Position { x, y };
                frame.set_cursor_position(pos);
            },
        }

    }
}

fn info_bar<'a>(file_name:String, cursor_x: i16, cursor_y:i16) -> Paragraph<'a> {
    let line = Line::from(vec![
        Span::styled(file_name, Style::default().fg(Color::LightCyan)),
        Span::raw(" - "), // Separator
        Span::styled(format!("Cursor: ({}, {})", cursor_x, cursor_y), Style::default().fg(Color::Magenta)),
    ]);
    Paragraph::new("")
        .block(
            Block::default()
                .title(line)
                .title_alignment(Alignment::Center)
                .style(Style::default().fg(Color::LightCyan).bg(Color::White))
        )
}

fn editor<'a>(editor_content: String, scroll_offset: u16) -> Paragraph<'a> {
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
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                //.borders(Borders::ALL)
                //.title("")
                //.border_type(BorderType::Thick)
        )
}