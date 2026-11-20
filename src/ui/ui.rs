use crate::config::EditorConfig;
use crate::core::app::{ActiveArea, App};
use crate::core::cursor::CursorPosition;
use crate::language::lsp::{Diagnostic, DiagnosticSeverity};
use crate::theme::UiTheme;
use crate::ui::debug;
use ratatui::layout::{Alignment, Position, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::Block,
    Frame,
};
use std::default::Default;
use std::ops::Range;
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

    // render any modals on top, back to front
    // Can't call modal.render() with &mut app borrow active, so extract first
    let modal_count = app.modal_stack.len();
    for i in 0..modal_count {
        // temporarily take the modal out to avoid borrow conflict
        let mut modal = app.modal_stack.remove(i);
        modal.render(frame, app);
        app.modal_stack.insert(i, modal);
    }
}

fn render_editor_ui(frame: &mut Frame, app: &mut App) {
    app.terminal_height = frame.area().height as i16;

    // Theme is cloned once up front
    let theme = app.theme_manager.active().ui.clone();

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

    // Feed the current buffer to the language manager so the syntax tree is
    // up-to-date, then compute all highlight tokens in a single tree walk.
    if app.content_modified
        || app
            .language
            .syntax
            .as_ref()
            .map(|s| s.source.is_empty())
            .unwrap_or(false)
    {
        let full_source = app.editor.editor_content.join("\n");
        app.language.update_source(&full_source);
    }

    let syntax_highlights: Vec<Vec<(std::ops::Range<usize>, Style)>> = app
        .language
        .highlighted_lines(&app.editor.editor_content, app.editor.content_version);

    let editor_content: Text = handle_editor_content(
        app.editor.editor_content.clone(),
        app.editor.text_selection_start,
        app.editor.text_selection_end,
        content_area.width as usize,
        app,
        &syntax_highlights,
        &theme,
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
                app.language.diagnostics.clone(),
                &theme,
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
                &app.language,
                &theme,
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
            &theme,
        ),
        content_area,
    );

    // Welcome view replaces the editor area (line numbers included, since
    // editor_area covers both). Drawn after the editor so it covers it.
    if let Some(welcome) = &app.welcome {
        frame.render_widget(ratatui::widgets::Clear, editor_area);
        welcome.render(frame, &theme, editor_area);
    }

    // Render command line
    frame.render_widget(command_line(command_input, &theme), command_area);

    // Render popup if active. `take()` to use "extract, render,
    // put back" pattern.
    if let Some(mut popup) = app.popup.take() {
        let (w, h) = popup.size();
        let popup_area = centered_rect(w, h, frame.area());
        popup.render(frame, popup_area, app);
        app.popup = Some(popup);
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
pub fn centered_rect(percent_width: u16, percent_height: u16, area: Rect) -> Rect {
    let width = area.width * percent_width / 100;
    let height = area.height * percent_height / 100;
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
#[allow(clippy::too_many_arguments)]
fn info_bar<'a>(
    file_name: String,
    cursor_x: i16,
    cursor_y: i16,
    visual_x: i16,
    scroll_offset: i16,
    selection_start: Option<CursorPosition>,
    selection_end: Option<CursorPosition>,
    is_content_modified: bool,
    diagnostics: Vec<Diagnostic>,
    theme: &UiTheme,
) -> Paragraph<'a> {
    let modified_indicator = if is_content_modified { "[+]" } else { "" };

    let selection_cursor_info = if selection_start.is_some() && selection_end.is_some() {
        let start = selection_start.unwrap();
        let end = selection_end.unwrap();
        format!(" | Sel: ({},{}) → ({},{})", start.x, start.y, end.x, end.y)
    } else {
        String::new()
    };

    let (errors, warnings) = (
        diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .count(),
        diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Warning)
            .count(),
    );

    let line = Line::from(vec![
        Span::styled(
            format!(" ●{} ◆{}", errors, warnings),
            if errors > 0 {
                Style::default().fg(theme.severity_color(DiagnosticSeverity::Error))
            } else if warnings > 0 {
                Style::default().fg(theme.severity_color(DiagnosticSeverity::Warning))
            } else {
                Style::default().fg(theme.hint_text())
            },
        ),
        Span::styled(modified_indicator, Style::default().fg(theme.foreground())),
        Span::styled(file_name, Style::default().fg(theme.accent_cyan())),
        Span::raw(" - "), // Separator
        Span::styled(
            format!(
                "Cursor (X{}:Y{}) (VisX: {}), ScrollOff({})",
                cursor_x, cursor_y, visual_x, scroll_offset
            ),
            Style::default().fg(theme.accent_magenta()),
        ),
        Span::styled(
            selection_cursor_info,
            Style::default().fg(theme.severity_color(DiagnosticSeverity::Warning)),
        ),
    ]);

    Paragraph::new("").block(
        Block::default()
            .title(line)
            .title_alignment(Alignment::Center)
            .style(theme.status_bar_style()),
    )
}

///generates a side bar for line nr display as well as displaying line overflow if existing
#[allow(clippy::too_many_arguments)]
fn editor_side_line<'a>(
    editor_content: Text,
    scroll_offset: u16,
    editor_width: usize,
    cursor_y: i16,
    config: &EditorConfig,
    language: &crate::language::manager::LanguageManager,
    theme: &UiTheme,
) -> Paragraph<'a> {
    let mut line_nrs: Text = Text::from(vec![]);

    let overflow_marker_style = Style::default().fg(theme.accent_cyan());
    let current_line_style = theme.list_highlight_style();
    let normal_line_style = Style::default().fg(theme.line_number());

    for (nr, s) in editor_content.iter().enumerate() {
        let line_index = nr;
        let is_current_line = cursor_y as usize == nr;

        // Calculate line number to display
        let line_num_display = if config.relative_line_numbers && !is_current_line {
            cursor_y.abs_diff(nr as i16).to_string()
        } else {
            (line_index + 1).to_string()
        };

        // Diagnostic marker for this line
        let diags = language.diagnostics_on_line(nr as u32);

        let (marker_char, marker_style) = if diags
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Error)
        {
            (
                "● ",
                Style::default()
                    .fg(theme.severity_color(DiagnosticSeverity::Error))
                    .add_modifier(Modifier::BOLD),
            )
        } else if diags
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Warning)
        {
            (
                "◆ ",
                Style::default().fg(theme.severity_color(DiagnosticSeverity::Warning)),
            )
        } else if diags
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Information)
        {
            (
                "◉ ",
                Style::default().fg(theme.severity_color(DiagnosticSeverity::Information)),
            )
        } else if diags.iter().any(|d| d.severity == DiagnosticSeverity::Hint) {
            (
                "· ",
                Style::default().fg(theme.severity_color(DiagnosticSeverity::Hint)),
            )
        } else {
            ("", Style::default())
        };

        let marker = Span::styled(marker_char, marker_style);

        // If content of line is longer than editor
        let has_overflow = s.width() >= editor_width;

        let line = if has_overflow {
            Line::from(vec![
                marker,
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
            Line::from(vec![
                marker,
                Span::styled(
                    line_num_display,
                    if is_current_line {
                        current_line_style
                    } else {
                        normal_line_style
                    },
                ),
            ])
        };

        line_nrs.push_line(line);
    }

    Paragraph::new(line_nrs)
        .style(
            Style::default()
                .bg(theme.background())
                .fg(theme.foreground()),
        )
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
    theme: &UiTheme,
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
                        Span::styled(span.content.clone(), span.style.bg(theme.current_line_bg()))
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
                Style::default().fg(theme.accent_blue()),
            )));
        }
    } else if visible_lines_start >= content_length {
        // Entirely in the margin area
        for _ in 0..viewport_height {
            lines_vec.push(Line::from(Span::styled(
                "~",
                Style::default().fg(theme.accent_blue()),
            )));
        }
    }

    let styled_content = Text::from(lines_vec);

    Paragraph::new(styled_content)
        .style(Style::default().fg(theme.foreground()))
        .block(Block::default())
        .scroll((scroll_offset, 0))
}

fn command_line<'a>(command_input: String, theme: &UiTheme) -> Paragraph<'a> {
    Paragraph::new(command_input)
        .style(theme.command_line_style())
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
    syntax_highlights: &[Vec<(Range<usize>, Style)>],
    theme: &UiTheme,
) -> Text<'a> {
    let tab_width = app.config.editor.tab_width as usize;
    let show_ws = app.config.editor.show_whitespace;
    let wrap = app.config.editor.wrap_lines;
    let visual_x = app.editor.visual_cursor_x;
    let cursor_y = app.editor.cursor.y.max(0) as usize;
    let selection = match (selection_start, selection_end) {
        (Some(s), Some(e)) => Some((s, e)),
        _ => None,
    };
    let selection_style = theme.selection_style().add_modifier(Modifier::BOLD);

    let mut editor_text: Text = Text::default();
    for (i, src) in vec.iter().enumerate() {
        let tokens = syntax_highlights
            .get(i)
            .map(|t| t.as_slice())
            .unwrap_or(&[]);
        let line_chars = src.chars().count();
        let sel = selection.and_then(|(s, e)| selection_on_line(i, line_chars, s, e));

        let mut chars = render_line(src, tokens, sel, tab_width, show_ws, selection_style);

        // Selected empty lines get a visible placeholder cell
        if src.is_empty() {
            if let Some((s, e)) = selection {
                if i > s.y && i <= e.y {
                    chars.push((' ', selection_style));
                }
            }
        }

        // Horizontal scroll of the cursor line (colors and selection are kept)
        if !wrap && i == cursor_y && visual_x > editor_width as i16 {
            let skip = (visual_x - editor_width as i16).max(0) as usize;
            chars.drain(..skip.min(chars.len()));
        }

        editor_text.push_line(Line::from(to_spans(&chars)));
    }
    editor_text
}

/// Selected char range [start, end) on line `i`, or None.
fn selection_on_line(
    i: usize,
    line_chars: usize,
    start: CursorPosition,
    end: CursorPosition,
) -> Option<(usize, usize)> {
    if i < start.y || i > end.y {
        return None;
    }
    let s = if i == start.y { start.x } else { 0 };
    let e = if i == end.y { end.x } else { line_chars };
    let (s, e) = (s.min(line_chars), e.min(line_chars));
    (s < e).then_some((s, e))
}

/// Maps syntax byte ranges onto per-char styles of the source line.
fn char_styles(src: &str, tokens: &[(Range<usize>, Style)]) -> Vec<Style> {
    let offsets: Vec<usize> = src.char_indices().map(|(b, _)| b).collect();
    let mut styles = vec![Style::default(); offsets.len()];
    for (range, style) in tokens {
        let s = offsets.partition_point(|&o| o < range.start);
        let e = offsets.partition_point(|&o| o < range.end);
        for st in styles.iter_mut().take(e).skip(s) {
            *st = *style; // later (inner) tokens win
        }
    }
    styles
}

/// One display line as (char, style): syntax + selection overlay, then tab
/// expansion and whitespace dots applied per char so styles stay attached.
fn render_line(
    src: &str,
    tokens: &[(Range<usize>, Style)],
    selected: Option<(usize, usize)>,
    tab_width: usize,
    show_whitespace: bool,
    selection_style: Style,
) -> Vec<(char, Style)> {
    let tab_width = tab_width.max(1);
    let styles = char_styles(src, tokens);
    let blank = if show_whitespace { '·' } else { ' ' };
    let mut out = Vec::with_capacity(src.len());
    let mut col = 0usize;

    for (i, ch) in src.chars().enumerate() {
        let mut style = styles[i];
        if let Some((s, e)) = selected {
            if i >= s && i < e {
                style = style.patch(selection_style);
            }
        }
        match ch {
            '\t' => {
                let n = tab_width - col % tab_width;
                out.extend(std::iter::repeat((blank, style)).take(n));
                col += n;
            }
            ' ' => {
                out.push((blank, style));
                col += 1;
            }
            c => {
                out.push((c, style));
                col += 1;
            }
        }
    }
    out
}

/// Groups consecutive chars with the same style into spans.
fn to_spans<'a>(chars: &[(char, Style)]) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    let mut buf = String::new();
    let mut cur: Option<Style> = None;
    for &(c, st) in chars {
        if cur != Some(st) {
            if let Some(s) = cur {
                spans.push(Span::styled(std::mem::take(&mut buf), s));
            }
            cur = Some(st);
        }
        buf.push(c);
    }
    if let Some(s) = cur {
        spans.push(Span::styled(buf, s));
    }
    spans
}

#[cfg(test)]
mod unit_render_line_tests {
    use super::*;
    use ratatui::style::Color;

    fn red() -> Style {
        Style::default().fg(Color::Red)
    }
    fn sel() -> Style {
        Style::default().bg(Color::Blue)
    }

    #[test]
    fn syntax_color_survives_under_selection() {
        // "let x": `let` (bytes 0..3) is red; select chars 1..5
        let out = render_line("let x", &[(0..3, red())], Some((1, 5)), 4, false, sel());
        assert_eq!(out[0].1, red()); // unselected keyword keeps color
        assert_eq!(out[1].1, red().patch(sel())); // selected keyword: color + bg
        assert_eq!(out[3].1, sel()); // selected plain text: bg only
    }

    #[test]
    fn multibyte_chars_use_char_indices_for_selection_and_bytes_for_tokens() {
        // "åb=1": 'å' is 2 bytes. Token on `b` (bytes 2..3), select chars 1..2 (just `b`)
        let out = render_line("åb=1", &[(2..3, red())], Some((1, 2)), 4, false, sel());
        assert_eq!(out[0], ('å', Style::default()));
        assert_eq!(out[1], ('b', red().patch(sel())));
        assert_eq!(out[2].1, Style::default());
    }

    #[test]
    fn tabs_expand_to_the_next_stop_and_inherit_selection() {
        let out = render_line("a\tb", &[], Some((1, 2)), 4, false, sel());
        assert_eq!(out.len(), 5); // a + 3 spaces + b
        assert!(out[1..4].iter().all(|(c, s)| *c == ' ' && *s == sel()));
        assert_eq!(out[4].0, 'b');
    }

    #[test]
    fn whitespace_dots_do_not_shift_token_alignment() {
        // token on `x` (bytes 2..3) in "a x"; dots must not misalign it
        let out = render_line("a x", &[(2..3, red())], None, 4, true, sel());
        assert_eq!(out[1].0, '·');
        assert_eq!(out[2], ('x', red()));
    }

    #[test]
    fn selection_on_line_cases() {
        let (s, e) = (CursorPosition { x: 2, y: 1 }, CursorPosition { x: 3, y: 3 });
        assert_eq!(selection_on_line(0, 10, s, e), None);
        assert_eq!(selection_on_line(1, 10, s, e), Some((2, 10)));
        assert_eq!(selection_on_line(2, 10, s, e), Some((0, 10)));
        assert_eq!(selection_on_line(3, 10, s, e), Some((0, 3)));
        assert_eq!(selection_on_line(3, 2, s, e), Some((0, 2))); // clamped
    }

    #[test]
    fn to_spans_groups_equal_styles() {
        let spans = to_spans(&[('a', red()), ('b', red()), ('c', Style::default())]);
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].content, "ab");
    }
}
