use std::path::PathBuf;
use clap::Parser;
use ratatui::Frame;
use crate::input::actions::{InputAction, ModalAction};
use crate::project::FileNode;
use crate::theme::UiTheme;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

const MAX_FILES_SHOWN: usize = 12;
const CONTENT_WIDTH: u16 = 56;


const GLYPH_ROWS: usize = 6;
const GLYPH_WIDTH: u16 = 42;
const GLYPH_LETTERS: [[&str; GLYPH_ROWS]; 5] = [
    // G
    [" ██████╗ ", "██╔════╝ ", "██║  ███╗", "██║   ██║", "╚██████╔╝", " ╚═════╝ "],
    // L
    ["██╗     ", "██║     ", "██║     ", "██║     ", "███████╗", "╚══════╝"],
    // Y
    ["██╗   ██╗", "╚██╗ ██╔╝", " ╚████╔╝ ", "  ╚██╔╝  ", "   ██║   ", "   ╚═╝   "],
    // P
    ["██████╗ ", "██╔══██╗", "██████╔╝", "██╔═══╝ ", "██║     ", "╚═╝     "],
    // H
    ["██╗  ██╗", "██║  ██║", "███████║", "██╔══██║", "██║  ██║", "╚═╝  ╚═╝"],
];

#[derive(Clone)]
pub enum WelcomeAction {
    NewFile,
    OpenFilePicker,
    OpenPath(PathBuf),
    EnterDir(PathBuf),
    CommandLine,
    Settings,
    Help,
    Quit,
    Dismiss
}


#[derive(Debug, Clone, Copy, PartialEq)]
enum Section { Actions, Files }

struct Entry {
    section: Section,
    label: String,
    key: Option<char>,
    is_dir: bool,
    action: WelcomeAction,
}

pub struct WelcomeScreen {
    title: String,
    subtitle: String,
    entries: Vec<Entry>,
    hidden_files: usize,
    cursor: usize,
}

impl WelcomeScreen {
    pub fn empty() -> Self {
        Self::build("calli-glyph".into(), "a terminal editor".into(), None)
    }

    pub fn for_folder(root: PathBuf) -> Self {
        let name = root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| root.display().to_string());
        Self::build(name, root.display().to_string(), Some(root))
    }

    fn build(title: String, subtitle: String, folder: Option<PathBuf>) -> Self {
        let act = |key, label: &str, action| Entry {
            section: Section::Actions,
            label: label.into(),
            key: Some(key),
            is_dir: false,
            action,
        };
        let mut entries = vec![
            act('n', "New file", WelcomeAction::NewFile),
            act('o', if folder.is_some() { "Browse project files" } else { "Open file…" },
                WelcomeAction::OpenFilePicker),
            act('c', "Command line", WelcomeAction::CommandLine),
            act('s', "Settings", WelcomeAction::Settings),
            act('h', "Help", WelcomeAction::Help),
            act('q', "Quit", WelcomeAction::Quit),
        ];

        let mut hidden_files = 0;
        if let Some(root) = folder {
            // Reuse the tree so sorting/ignore rules match the browser.
            let mut rows = Vec::new();
            FileNode::new_root(root).flatten_visible(0, &mut rows);
            let children: Vec<_> = rows.into_iter().filter(|r| r.depth == 1).collect();
            hidden_files = children.len().saturating_sub(MAX_FILES_SHOWN);
            for r in children.into_iter().take(MAX_FILES_SHOWN) {
                let action = if r.is_dir {
                    WelcomeAction::EnterDir(r.path.clone())
                } else {
                    WelcomeAction::OpenPath(r.path.clone())
                };
                entries.push(Entry {
                    section: Section::Files,
                    label: r.name,
                    key: None,
                    is_dir: r.is_dir,
                    action,
                });
            }
        }
        Self { title, subtitle, entries, hidden_files, cursor: 0 }
    }

    /// Pure state transition: returns an action for App to dispatch.
    pub fn handle_input(&mut self, action: InputAction) -> Option<WelcomeAction> {
        let InputAction::Modal(m) = action else { return None };
        match m {
            ModalAction::ScrollUp => self.cursor = self.cursor.saturating_sub(1),
            ModalAction::ScrollDown => {
                if self.cursor + 1 < self.entries.len() {
                    self.cursor += 1;
                }
            }
            ModalAction::Confirm => return self.entries.get(self.cursor).map(|e| e.action.clone()),
            ModalAction::Action(c) => {
                return self.entries.iter().find(|e| e.key == Some(c)).map(|e| e.action.clone());
            }
            _ => {}
        }
        None
    }

    pub fn render(&self, frame: &mut Frame, ui: &UiTheme, area: Rect) {
        // Body: title/subtitle + sections
        let mut body: Vec<Line> = vec![
            Line::from(Span::styled(
                &self.title,
                Style::default().fg(ui.foreground()).add_modifier(Modifier::BOLD),
            ))
                .alignment(Alignment::Center),
            Line::from(Span::styled(&self.subtitle, Style::default().fg(ui.hint_text())))
                .alignment(Alignment::Center),
            Line::raw(""),
        ];


        let mut last_section = None;
        for (i, e) in self.entries.iter().enumerate() {
            if last_section != Some(e.section) {
                if last_section.is_some() { body.push(Line::raw("")); }
                let header = match e.section { Section::Actions => "Start", Section::Files => "Files" };
                body.push(Line::from(Span::styled(header, Style::default().fg(ui.hint_text()))));
                last_section = Some(e.section);
            }
            let key = e.key.map(|k| format!("[{k}] ")).unwrap_or_else(|| "    ".into());
            let marker = if e.is_dir { "▸ " } else { "" };
            let name_color = if e.is_dir { ui.accent_blue() } else { ui.foreground() };
            let mut line = Line::from(vec![
                Span::styled(format!("  {key}"), Style::default().fg(ui.hint_text())),
                Span::styled(format!("{marker}{}", e.label), Style::default().fg(name_color)),
            ]);
            if i == self.cursor {
                line = line.style(ui.list_highlight_style().add_modifier(Modifier::BOLD));
            }
            body.push(line);
        }
        if self.hidden_files > 0 {
            body.push(Line::from(Span::styled(
                format!("      … {} more (press o to browse)", self.hidden_files),
                Style::default().fg(ui.hint_text()),
            )));
        }

        // Big logo only if it fits: logo rows + "calli" line + spacer + body
        let big = area.width >= GLYPH_WIDTH + 2
            && area.height as usize >= body.len() + GLYPH_ROWS + 3;

        let mut lines = logo_lines(ui, big);
        lines.push(Line::raw(""));
        lines.extend(body);

        let h = (lines.len() as u16).min(area.height);
        let w = CONTENT_WIDTH.min(area.width);
        let rect = Rect {
            x: area.x + (area.width - w) / 2,
            y: area.y + (area.height - h) / 2,
            width: w,
            height: h,
        };
        frame.render_widget(Paragraph::new(lines), rect);
    }
}

/// One accent per letter. Swap names here if your UiTheme accessors differ.
fn glyph_palette(ui: &UiTheme) -> [Color; 5] {
    [
        ui.accent_cyan(),
        ui.accent_blue(),
        ui.accent_magenta(),
        ui.accent_green(),
        ui.accent_yellow(),
    ]
}

fn logo_lines(ui: &UiTheme, big: bool) -> Vec<Line<'static>> {
    let palette = glyph_palette(ui);

    if !big {
        // Compact fallback for small terminals
        return vec![Line::from(vec![
            Span::styled("calli", Style::default().fg(ui.foreground())),
            Span::styled("GLYPH", Style::default().fg(palette[0]).add_modifier(Modifier::BOLD)),
        ])
            .alignment(Alignment::Center)];
    }

    let shadow = Style::default().fg(ui.hint_text());
    let mut lines = vec![Line::from(Span::styled(
        "c  a  l  l  i",
        Style::default().fg(ui.foreground()).add_modifier(Modifier::BOLD),
    ))
        .alignment(Alignment::Center)];

    for row in 0..GLYPH_ROWS {
        let mut spans = Vec::new();
        for (letter, color) in GLYPH_LETTERS.iter().zip(palette) {
            for ch in letter[row].chars() {
                let style = if ch == '█' { Style::default().fg(color) } else { shadow };
                spans.push(Span::styled(ch.to_string(), style));
            }
        }
        lines.push(Line::from(spans).alignment(Alignment::Center));
    }
    lines
}
