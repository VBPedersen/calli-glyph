
use ratatui::{ layout::{Constraint, Direction, Layout}, widgets::{Block, Borders, }, Frame, };
use ratatui::layout::Position;
use ratatui::style::{Color, Style};
use ratatui::widgets::{BorderType, Paragraph};
use crate::app::{ActiveArea, App};


pub fn ui(frame: &mut Frame, app: &App) {
    let editor_content: String = app.editor_content.join("\n");
    let command_input:String = (&app.command_input).to_string();



    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(90),
            Constraint::Percentage(10),
        ])
        .split(frame.area());

    frame.render_widget(
        editor(editor_content,app.scroll_offset),
        layout[0],
    );
    frame.render_widget(
        command_line(command_input),
        layout[1],
    );

    //set cursor with position if it should be visiblie (determined by app logic)
    if app.cursor_visible {
        match app.active_area {
            ActiveArea::Editor => {
                let x = layout[0].x + app.cursor_x as u16 + 1;
                let y = layout[0].y + app.cursor_y as u16 + 1;
                let pos: Position = Position { x, y };
                frame.set_cursor_position(pos);
            },
            ActiveArea::CommandLine => {
                let x = layout[1].x + app.cursor_x as u16 + 1;
                let y = layout[1].y + app.cursor_y as u16 + 1;
                let pos: Position = Position { x, y };
                frame.set_cursor_position(pos);
            },
        }

    }
}

fn editor<'a>(editor_content: String, scroll_offset: u16) -> Paragraph<'a> {
    Paragraph::new(editor_content)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Editor")
                .border_type(BorderType::Rounded)
        ).scroll((scroll_offset, 0))
}

fn command_line<'a>(command_input: String) -> Paragraph<'a> {
    Paragraph::new(command_input)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Command Line")
                .border_type(BorderType::Rounded)
        )
}