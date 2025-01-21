
use ratatui::{ layout::{Constraint, Direction, Layout}, widgets::{Block, Borders, }, Frame, };
use ratatui::style::{Color, Style};
use ratatui::widgets::{BorderType, Paragraph};
use crate::app::App;


pub fn ui(frame: &mut Frame, app: &App) {
    let editor_content:String = (&app.editor_content).to_string();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(90),
            Constraint::Percentage(10),
        ])
        .split(frame.area());

    frame.render_widget(
        read_file_to_editor(editor_content),
        layout[0],
    );
    frame.render_widget(
        Block::new().title("CommandLine").borders(Borders::ALL),
        layout[1],
    );

}

fn read_file_to_editor<'a>(editor_content: String) -> Paragraph<'a> {
    let block = Block::new().title("TextEditor").borders(Borders::ALL);
    Paragraph::new(editor_content)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Title")
                .border_type(BorderType::Rounded)
        )
}
