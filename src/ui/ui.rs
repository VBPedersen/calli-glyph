use crate::config::EditorConfig;
use crate::core::app::{ActiveArea, App};
use crate::core::cursor::CursorPosition;
use crate::ui::debug;
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
use std::path::PathBuf;
use std::vec;

pub fn ui(frame: &mut Frame, app: &mut App) {
    // Tick frame timer for debug metrics
    app.debug_state.tick_frame();

    // Route to appropriate UI based on active area
    match app.active_area {
        ActiveArea::DebugConsole => {
            debug::render_debug_panel(frame, app, frame.area());
        }
        _ => {
            render_editor_ui(frame, app);
        }
    }
}

fn render_editor_ui(frame: &mut Frame, app: &mut App) {
    app.terminal_height = frame.area().height as i16;

    // just array of constraints to use in layout,
    // mutable, so can step by step add to constraints
    let mut constraints = vec![];

    // check for status bar enabled
    if app.config.ui.show_status_bar {
        constraints.push(Constraint::Length(1));
    }

    // Editor area
    constraints.push(Constraint::Min(1));

    // Command Line
    constraints.push(Constraint::Length(1));

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(frame.area());

    // --------------- Calculate areas ------------------
    // Find status bar area if enabled
    let mut layout_idx = 0;
    let status_bar_area = if app.config.ui.show_status_bar {
        let area = layout[layout_idx];
        layout_idx += 1;
        Some(area)
    } else {
        None
    };

    let editor_area = layout[layout_idx];
    layout_idx += 1;
    let command_area = layout[layout_idx];

    app.editor.editor_height = editor_area.height;

    // Editor layout with optional line numbers
    let (line_number_area, content_area) = if app.config.editor.line_numbers {
        let line_count = app.editor.editor_content.len();
        let line_num_width = (line_count.to_string().len() as u16).max(2) + 1;

        let editor_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Length(line_num_width), Constraint::Min(1)])
            .split(editor_area);

        (Some(editor_layout[0]), editor_layout[1])
    } else {
        (None, editor_area)
    };
    //----------------------------------------------------------

    // Update app layout areas with new found areas
    app.update_layout(
        status_bar_area,
        editor_area,
        line_number_area,
        content_area,
        command_area,
    );

    app.editor.editor_width = content_area.width as i16;

    let editor_content: Text = handle_editor_content(
        app.editor.editor_content.clone(),
        app.editor.text_selection_start,
        app.editor.text_selection_end,
        content_area.width as usize,
        app,
    );

    let command_input: String = app.command_line.input.to_string();
    let file_name_optional: Option<PathBuf> = app.file_path.clone();
    let file_to_use = if let Some(file) = file_name_optional {
        file.to_str().unwrap().to_string()
    } else {
        "untitled".to_string()
    };

    //render widgets : infobar, editor side, editor and command line

    // Render status/info bar if enabled
    if let Some(status_area) = status_bar_area {
        frame.render_widget(
            info_bar(
                file_to_use,
                app.editor.cursor.x,
                app.editor.cursor.y,
                app.editor.visual_cursor_x,
                app.editor.scroll_offset,
                app.editor.text_selection_start,
                app.editor.text_selection_end,
                app.content_modified,
            ),
            status_area,
        );
    }

    // Render line number side line if enabled
    if let Some(ln_area) = line_number_area {
        frame.render_widget(
            editor_side_line(
                editor_content.clone(),
                app.editor.scroll_offset as u16,
                content_area.width as usize,
                app.editor.cursor.y,
                &app.config.editor,
            ),
            ln_area,
        );
    }

    // Render editor content
    frame.render_widget(
        editor(
            editor_content,
            app.editor.scroll_offset as u16,
            app.editor.cursor.y,
            &app.config.editor,
            content_area.height,
            app.editor.editor_content.len(),
        ),
        content_area,
    );
    // Render command line
    frame.render_widget(command_line(command_input), command_area);

    // Render popup if active
    if let Some(popup) = &app.popup {
        let popup_area = centered_rect(60, 20, frame.area());
        popup.render(frame, popup_area);
    }

    //set cursor with position if it should be visible (determined by app logic)
    let should_show_cursor = if app.config.ui.cursor_blink {
        app.cursor_visible
    } else {
        true // Always visible if blink disabled
    };

    // Show cursor when it should, get position of current active area either editor or commandline
    //TODO implment custom rendering to make styles possible, underline, block and line
    if should_show_cursor {
        match app.active_area {
            ActiveArea::Editor => {
                let x = content_area.x + app.editor.visual_cursor_x as u16; //using visual x
                let y = content_area.y
                    + (app.editor.cursor.y - app.editor.scroll_offset).clamp(0, i16::MAX) as u16;
                let pos: Position = Position { x, y };

                frame.set_cursor_position(pos);
            }
            ActiveArea::CommandLine => {
                let x = command_area.x + app.command_line.cursor.x as u16;
                let y = command_area.y + app.command_line.cursor.y as u16;
                let pos: Position = Position { x, y };
                frame.set_cursor_position(pos);
            }
            ActiveArea::Popup => {}
            _ => {}
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
    scroll_offset: i16,
    selection_start: Option<CursorPosition>,
    selection_end: Option<CursorPosition>,
    is_content_modified: bool,
) -> Paragraph<'a> {
    let modified_indicator = if is_content_modified { "[+]" } else { "" };

    let selection_cursor_info = if selection_start.is_some() && selection_end.is_some() {
        let start = selection_start.unwrap();
        let end = selection_end.unwrap();
        format!(" | Sel: ({},{}) → ({},{})", start.x, start.y, end.x, end.y)
    } else {
        String::new()
    };

    let line = Line::from(vec![
        Span::styled(modified_indicator, Style::default().fg(Color::White)),
        Span::styled(file_name, Style::default().fg(Color::LightCyan)),
        Span::raw(" - "), // Separator
        Span::styled(
            format!(
                "Cursor (X{}:Y{}) (VisX: {}), ScrollOff({})",
                cursor_x, cursor_y, visual_x, scroll_offset
            ),
            Style::default().fg(Color::Magenta),
        ),
        Span::styled(selection_cursor_info, Style::default().fg(Color::Yellow)),
    ]);

    Paragraph::new("").block(
        Block::default()
            .title(line)
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(Color::LightCyan).bg(Color::DarkGray)),
    )
}

///generates a side bar for line nr display as well as displaying line overflow if existing
fn editor_side_line<'a>(
    editor_content: Text,
    scroll_offset: u16,
    editor_width: usize,
    cursor_y: i16,
    config: &EditorConfig,
) -> Paragraph<'a> {
    let mut line_nrs: Text = Text::from(vec![]);

    let overflow_marker_style = Style::default().fg(Color::Cyan);
    let current_line_style = Style::default()
        .bg(Color::Yellow)
        .fg(Color::Black)
        .add_modifier(Modifier::BOLD);
    let normal_line_style = Style::default().fg(Color::Gray);

    for (nr, s) in editor_content.iter().enumerate() {
        let line_index = nr;
        let is_current_line = cursor_y as usize == line_index;

        // Calculate line number to display
        let line_num_display = if config.relative_line_numbers && !is_current_line {
            cursor_y.abs_diff(nr as i16).to_string()
        } else {
            (line_index + 1).to_string()
        };

        // If content of line is longer than editor
        let has_overflow = s.width() >= editor_width;

        let line = if has_overflow {
            Line::from(vec![
                Span::styled(
                    line_num_display,
                    if is_current_line {
                        current_line_style
                    } else {
                        normal_line_style
                    },
                ),
                Span::styled(">", overflow_marker_style),
            ])
        } else {
            Line::from(vec![Span::styled(
                line_num_display,
                if is_current_line {
                    current_line_style
                } else {
                    normal_line_style
                },
            )])
        };

        line_nrs.push_line(line);
    }

    Paragraph::new(line_nrs)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .block(Block::default())
        .scroll((scroll_offset, 0))
}

fn editor<'a>(
    editor_content: Text<'a>,
    scroll_offset: u16,
    cursor_y: i16,
    config: &EditorConfig,
    viewport_height: u16,
    content_length: usize,
) -> Paragraph<'a> {
    // Apply current line highlighting if enabled
    let mut lines_vec = if config.highlight_current_line {
        let mut lines = Vec::new();
        for (i, line) in editor_content.lines.iter().enumerate() {
            if i == cursor_y as usize {
                // Highlight current line
                let highlighted_spans: Vec<Span> = line
                    .spans
                    .iter()
                    .map(|span| {
                        Span::styled(span.content.clone(), span.style.bg(Color::Rgb(77, 77, 77)))
                    })
                    .collect();
                lines.push(Line::from(highlighted_spans));
            } else {
                lines.push(line.clone());
            }
        }
        lines
    } else {
        editor_content.lines
    };

    // Add empty lines at the end for bottom margin effect
    let visible_lines_start = scroll_offset as usize;
    let visible_lines_end = (scroll_offset + viewport_height) as usize;

    // If we're scrolled past the actual content, add empty placeholder lines
    if visible_lines_start < content_length && visible_lines_end > content_length {
        let empty_lines_needed = visible_lines_end - content_length;
        for _ in 0..empty_lines_needed {
            lines_vec.push(Line::from(Span::styled(
                "~",
                Style::default().fg(Color::Blue),
            )));
        }
    } else if visible_lines_start >= content_length {
        // Entirely in the margin area
        for _ in 0..viewport_height {
            lines_vec.push(Line::from(Span::styled(
                "~",
                Style::default().fg(Color::Blue),
            )));
        }
    }

    let styled_content = Text::from(lines_vec);

    Paragraph::new(styled_content)
        .style(Style::default().fg(Color::White))
        .block(Block::default())
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
    let editor_vec: Vec<String> = vec
        .into_iter()
        .map(|s| {
            // If show whitespaces render white space " " as "·"
            let with_tabs = handle_tab_rendering(s, app.config.editor.tab_width);
            if app.config.editor.show_whitespace {
                with_tabs.replace(" ", "·")
            } else {
                with_tabs
            }
        })
        .collect();

    let mut editor_text: Text = Text::default();

    if selection_start.is_some() {
        editor_text = highlight_text(editor_vec.clone(), selection_start, selection_end);
    } else {
        for (i, s) in editor_vec.into_iter().enumerate() {
            let visual_x = app.editor.visual_cursor_x;

            // Line wrapping and horizontal scroll
            let line: Line = if app.config.editor.wrap_lines {
                // Simple wrap TODO make actual wrapping solution that is intelligent
                Line::from(s)
            } else if i == app.editor.cursor.y as usize && visual_x > editor_width as i16 {
                // Horizontal scroll for current line
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
fn get_copy_of_editor_content_at_line_between_cursor_editor_width(
    s: String,
    start: usize,
) -> String {
    let mut line_chars_vec: Vec<char> = s.chars().collect();
    line_chars_vec.drain(start..).collect()
}

///manipulates how the editor content \t character is rendered visually
fn handle_tab_rendering(s: String, tab_width: u16) -> String {
    let mut temp_string: Vec<char> = s.chars().collect();

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

//TODO error when shift selecting up into å æ ø multi byte chars
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
