use crate::core::app::App;
use crate::ui::popups::popup::PopupType;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_state(frame: &mut Frame, app: &App, area: Rect) {
    let ui = &app.theme_manager.active().ui;
    let head = Style::default()
        .fg(ui.accent_blue())
        .add_modifier(Modifier::BOLD);
    let dim = Style::default().fg(ui.hint_text());
    let warn = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);

    let mut lines: Vec<Line> = Vec::new();
    let section = |lines: &mut Vec<Line>, title: &str| {
        if !lines.is_empty() {
            lines.push(Line::raw(""));
        }
        lines.push(Line::from(Span::styled(title.to_string(), head)));
    };
    let kv = |lines: &mut Vec<Line>, k: &str, v: String| {
        lines.push(Line::from(vec![
            Span::styled(format!("  {k:<16}"), dim),
            Span::raw(v),
        ]));
    };

    // Focus
    section(&mut lines, "Focus");
    kv(&mut lines, "active area", format!("{:?}", app.active_area));
    kv(
        &mut lines,
        "welcome",
        if app.welcome.is_some() {
            "shown"
        } else {
            "dismissed"
        }
        .into(),
    );
    let popup = app.popup.as_ref().map(|p| match p.get_popup_type() {
        PopupType::Confirmation => "Confirmation",
        PopupType::Error => "Error",
        PopupType::Info => "Info",
        _ => "Other",
    });
    kv(&mut lines, "popup", popup.unwrap_or("none").to_string());
    let modals: Vec<_> = app.modal_stack.iter().map(|m| m.debug_name()).collect();
    kv(
        &mut lines,
        "modal stack",
        if modals.is_empty() {
            "empty".into()
        } else {
            modals.join(" › ")
        },
    );
    kv(
        &mut lines,
        "active plugin",
        app.plugins
            .active_plugin_name()
            .unwrap_or_else(|| "none".into()),
    );

    // Pending states
    section(
        &mut lines,
        &format!("Pending states ({})", app.pending_states.len()),
    );
    if app.pending_states.is_empty() {
        lines.push(Line::from(Span::styled("  (none)", dim)));
    } else {
        for (i, s) in app.pending_states.iter().enumerate() {
            let marker = if i == 0 { "▶" } else { " " };
            lines.push(Line::from(format!(
                "  {marker} {}. {}",
                i + 1,
                s.describe()
            )));
        }
        if app.popup.is_none() && app.modal_stack.is_empty() {
            lines.push(Line::from(Span::styled(
                "  ⚠ pending states exist but nothing is waiting on the user (stale?)",
                warn,
            )));
        }
    }

    // Buffer
    section(&mut lines, "Buffer");
    kv(
        &mut lines,
        "file",
        app.file_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "[no file]".into()),
    );
    kv(&mut lines, "modified", app.content_modified.to_string());
    kv(
        &mut lines,
        "undo dirty",
        app.editor.undo_redo_manager.is_dirty().to_string(),
    );
    kv(
        &mut lines,
        "lines",
        app.editor.editor_content.len().to_string(),
    );
    kv(
        &mut lines,
        "version",
        app.editor.content_version.to_string(),
    );
    kv(
        &mut lines,
        "cursor",
        format!("{}:{}", app.editor.cursor.y, app.editor.cursor.x),
    );
    kv(
        &mut lines,
        "selection",
        (app.editor.text_selection_start.is_some() && app.editor.text_selection_end.is_some())
            .to_string(),
    );
    kv(
        &mut lines,
        "auto-save",
        app.config.editor.auto_save.to_string(),
    );

    // Project / language
    section(&mut lines, "Project & language");
    kv(
        &mut lines,
        "project root",
        format!("{:?}", app.project_manager.root),
    );
    kv(
        &mut lines,
        "language id",
        format!("{:?}", app.language.language_id),
    );
    kv(
        &mut lines,
        "document uri",
        format!("{:?}", app.language.current_uri),
    );
    kv(
        &mut lines,
        "lsp running",
        app.language.lsp.is_some().to_string(),
    );

    // Plugins
    section(&mut lines, "Plugins");
    let mut names: Vec<_> = app.plugins.plugins.keys().cloned().collect();
    names.sort();
    kv(&mut lines, "loaded", names.join(", "));
    kv(
        &mut lines,
        "commands",
        app.plugins
            .command_registry()
            .list_commands()
            .len()
            .to_string(),
    );
    kv(
        &mut lines,
        "menu entries",
        app.menu.entries().len().to_string(),
    );

    let block = Block::default().borders(Borders::ALL).title(" State ");
    let inner_h = block.inner(area).height as usize;
    let max_scroll = lines.len().saturating_sub(inner_h);
    let offset = app.debug_view.scroll_offset.min(max_scroll) as u16;
    frame.render_widget(Paragraph::new(lines).block(block).scroll((offset, 0)), area);
}
