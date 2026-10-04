//! The default, built-in file browser: a popup/overlay using the `Modal`
//! trait, with no external dependencies.
//! This is what runs when `[project.file_picker] mode =
//! "builtin"` (the default), and is always available as a fallback even
//! when an external picker is configured, in case it isn't installed.

use crate::project::tree::{FileNode, VisibleRow};
use crate::core::app::App;
use crate::input::actions::{InputAction, ModalAction};
use crate::ui::modal::{Modal, ModalResponse};
use crate::ui::ui::centered_rect;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use std::path::PathBuf;

pub struct FileBrowserModal {
    root: FileNode,
    rows: Vec<VisibleRow>,
    cursor: usize,
    scroll: usize,
}

// IMPL functionality and controls
impl FileBrowserModal {
    pub fn new(start_dir: PathBuf) -> Self {
        let root = FileNode::new_root(start_dir);
        let mut modal = Self {
            root,
            rows: Vec::new(),
            cursor: 0,
            scroll: 0,
        };
        modal.rebuild_rows();
        modal
    }

    fn rebuild_rows(&mut self) {
        self.rows.clear();
        self.root.flatten_visible(0, &mut self.rows);
        if self.cursor >= self.rows.len() {
            self.cursor = self.rows.len().saturating_sub(1);
        }
    }

    fn selected(&self) -> Option<&VisibleRow> {
        self.rows.get(self.cursor)
    }

    fn move_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
        self.keep_cursor_in_view();
    }

    fn move_down(&mut self) {
        if self.cursor + 1 < self.rows.len() {
            self.cursor += 1;
        }
        self.keep_cursor_in_view();
    }

    fn jump_to_top(&mut self) {
        self.cursor = 0;
        self.scroll = 0;
    }

    fn jump_to_bottom(&mut self) {
        self.cursor = self.rows.len().saturating_sub(1);
        self.keep_cursor_in_view();
    }

    fn keep_cursor_in_view(&mut self) {
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
        }
    }

    /// Enter/Right on a directory expands it; on a file, nothing (use
    /// `confirm` to open).
    fn expand_selected(&mut self) {
        if let Some(row) = self.selected().cloned() {
            if row.is_dir && !row.expanded {
                if let Some(node) = self.root.find_mut(&row.path) {
                    node.toggle_expand();
                }
                self.rebuild_rows();
            }
        }
    }

    /// Left on an expanded directory collapses it; on a collapsed
    /// directory or a file, moves the cursor to its parent row instead
    /// (standard file-browser "go up a level" behavior).
    fn collapse_selected(&mut self) {
        let Some(row) = self.selected().cloned() else {
            return;
        };

        if row.is_dir && row.expanded {
            if let Some(node) = self.root.find_mut(&row.path) {
                node.toggle_expand();
            }
            self.rebuild_rows();
            return;
        }

        // Jump to parent: nearest row above with a smaller depth.
        if let Some(parent_idx) = self.rows[..self.cursor]
            .iter()
            .enumerate()
            .rev()
            .find(|(_, r)| r.depth < row.depth)
            .map(|(i, _)| i)
        {
            self.cursor = parent_idx;
            self.keep_cursor_in_view();
        }
    }

    /// "Enter" the selected directory: re-roots the *browser's own view*
    /// at that directory (so you're now browsing from inside it, same as
    /// closing and reopening with that path), and also updates
    /// `app.project_manager.root` to match. So the new root sticks for
    /// `:explore` next time, not just for this one browser session.
    /// No-op on a file.
    fn enter_as_root(&mut self, app: &mut App) {
        let Some(row) = self.selected().cloned() else {
            return;
        };
        if !row.is_dir {
            return;
        }

        self.root = FileNode::new_root(row.path.clone());
        self.cursor = 0;
        self.scroll = 0;
        self.rebuild_rows();
        app.project_manager.set_root(row.path);
    }

    /// Enter on a directory toggles expand; on a file, opens it and
    /// closes the browser.
    fn confirm_selected(&mut self, app: &mut App) -> ModalResponse {
        let Some(row) = self.selected().cloned() else {
            return ModalResponse::Consumed;
        };

        if row.is_dir {
            if let Some(node) = self.root.find_mut(&row.path) {
                node.toggle_expand();
            }
            self.rebuild_rows();
            ModalResponse::Consumed
        } else {
            app.open_file(row.path, None);
            ModalResponse::Close
        }
    }
}

impl Modal for FileBrowserModal {
    fn handle_input(&mut self, action: InputAction, app: &mut App) -> ModalResponse {
        let InputAction::Modal(modal_action) = action else {
            return ModalResponse::Consumed;
        };

        match modal_action {
            ModalAction::Close => return ModalResponse::Close,
            ModalAction::ScrollUp => self.move_up(),
            ModalAction::ScrollDown => self.move_down(),
            ModalAction::Confirm => return self.confirm_selected(app),
            ModalAction::Action('l') => self.expand_selected(),
            ModalAction::Action('h') => self.collapse_selected(),
            ModalAction::Action('r') => self.enter_as_root(app),
            ModalAction::Action('g') => self.jump_to_top(),
            ModalAction::Action('b') => self.jump_to_bottom(),
            _ => {}
        }
        ModalResponse::Consumed
    }

    fn render(&mut self, frame: &mut Frame, app: &App) {
        let ui = &app.theme_manager.active().ui;
        let area = centered_rect(70, 80, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Files ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ui.popup_border()))
            .style(Style::default().fg(ui.popup_fg()).bg(ui.popup_bg()));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(inner);

        self.render_rows(frame, ui, chunks[0]);

        let hint = Paragraph::new(
            "↑↓/jk: move  l/→: expand  h/←: collapse/up  r: enter as root  Enter: open  g/b: top/bottom  Esc: close",
        )
        .style(Style::default().fg(ui.hint_text()));
        frame.render_widget(hint, chunks[1]);
    }
}

// IMPL Render
impl FileBrowserModal {
    fn render_rows(&mut self, frame: &mut Frame, ui: &crate::theme::UiTheme, area: Rect) {
        let visible_height = area.height as usize;

        // Keep the cursor inside the visible window — recompute scroll
        // here rather than only on move_up/move_down, since area height
        // (and thus what fits) isn't known until render time.
        if self.cursor >= self.scroll + visible_height {
            self.scroll = self.cursor + 1 - visible_height;
        }
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
        }

        let lines: Vec<Line> = self
            .rows
            .iter()
            .enumerate()
            .skip(self.scroll)
            .take(visible_height)
            .map(|(i, row)| {
                let indent = "  ".repeat(row.depth);
                let marker = if row.is_dir {
                    if row.expanded {
                        "▾ "
                    } else {
                        "▸ "
                    }
                } else {
                    "  "
                };
                let name_style = if row.is_dir {
                    Style::default().fg(ui.accent_blue())
                } else {
                    Style::default().fg(ui.foreground())
                };

                let line = Line::from(vec![
                    Span::raw(indent),
                    Span::styled(marker, Style::default().fg(ui.hint_text())),
                    Span::styled(row.name.clone(), name_style),
                ]);

                if i == self.cursor {
                    line.style(ui.list_highlight_style().add_modifier(Modifier::BOLD))
                } else {
                    line
                }
            })
            .collect();

        frame.render_widget(Paragraph::new(lines), area);
    }
}



#[cfg(test)]
mod unit_browser_tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::tempdir;

    fn make_tree(dir: &std::path::Path) {
        std::fs::create_dir(dir.join("src")).unwrap();
        std::fs::write(dir.join("src").join("main.rs"), "").unwrap();
        std::fs::write(dir.join("Cargo.toml"), "").unwrap();
    }

    // Same disk-isolation pattern as the plugin tests elsewhere in this
    // codebase — enter_as_root touches app.project_manager via a real App.
    static CONFIG_DIR_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn with_isolated_config_dir<F: FnOnce()>(f: F) {
        let _guard = CONFIG_DIR_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let dir = tempdir().unwrap();
        let prev = std::env::var("XDG_CONFIG_HOME").ok();
        std::env::set_var("XDG_CONFIG_HOME", dir.path());
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        match prev {
            Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
            None => std::env::remove_var("XDG_CONFIG_HOME"),
        }
        if let Err(payload) = result {
            std::panic::resume_unwind(payload);
        }
    }

    #[test]
    fn new_starts_with_cursor_at_root() {
        let dir = tempdir().unwrap();
        make_tree(dir.path());
        let modal = FileBrowserModal::new(dir.path().to_path_buf());
        assert_eq!(modal.cursor, 0);
        assert_eq!(modal.rows[0].path, dir.path());
    }

    #[test]
    fn move_down_advances_cursor() {
        let dir = tempdir().unwrap();
        make_tree(dir.path());
        let mut modal = FileBrowserModal::new(dir.path().to_path_buf());
        modal.move_down();
        assert_eq!(modal.cursor, 1);
    }

    #[test]
    fn move_down_at_last_row_does_not_overflow() {
        let dir = tempdir().unwrap();
        make_tree(dir.path());
        let mut modal = FileBrowserModal::new(dir.path().to_path_buf());
        let row_count = modal.rows.len();
        for _ in 0..(row_count + 5) {
            modal.move_down();
        }
        assert_eq!(modal.cursor, row_count - 1);
    }

    #[test]
    fn move_up_at_first_row_does_not_underflow() {
        let dir = tempdir().unwrap();
        make_tree(dir.path());
        let mut modal = FileBrowserModal::new(dir.path().to_path_buf());
        modal.move_up();
        assert_eq!(modal.cursor, 0);
    }

    #[test]
    fn jump_to_top_resets_cursor_and_scroll() {
        let dir = tempdir().unwrap();
        make_tree(dir.path());
        let mut modal = FileBrowserModal::new(dir.path().to_path_buf());
        modal.move_down();
        modal.move_down();
        modal.jump_to_top();
        assert_eq!(modal.cursor, 0);
        assert_eq!(modal.scroll, 0);
    }

    #[test]
    fn jump_to_bottom_moves_cursor_to_last_row() {
        let dir = tempdir().unwrap();
        make_tree(dir.path());
        let mut modal = FileBrowserModal::new(dir.path().to_path_buf());
        let row_count = modal.rows.len();
        modal.jump_to_bottom();
        assert_eq!(modal.cursor, row_count - 1);
    }

    #[test]
    fn jump_to_bottom_from_already_at_bottom_is_a_no_op() {
        let dir = tempdir().unwrap();
        make_tree(dir.path());
        let mut modal = FileBrowserModal::new(dir.path().to_path_buf());
        modal.jump_to_bottom();
        let after_first = modal.cursor;
        modal.jump_to_bottom();
        assert_eq!(modal.cursor, after_first);
    }

    #[test]
    fn jump_to_bottom_reflects_newly_expanded_rows() {
        let dir = tempdir().unwrap();
        make_tree(dir.path());
        let mut modal = FileBrowserModal::new(dir.path().to_path_buf());
        let before_expand_bottom = {
            modal.jump_to_bottom();
            modal.cursor
        };

        modal.cursor = 1; // "src"
        modal.expand_selected();
        modal.jump_to_bottom();

        // Depth-first flatten order after expanding "src" is:
        // root, src, main.rs (src's child), Cargo.toml (root's other
        // child, a sibling of src, so it still sorts after src's whole
        // subtree) — so the bottom row is Cargo.toml, not main.rs.
        assert!(modal.cursor > before_expand_bottom, "expanding should reveal more rows below");
        assert_eq!(modal.rows[modal.cursor].name, "Cargo.toml");
        assert!(modal.rows.iter().any(|r| r.name == "main.rs"), "main.rs should be visible somewhere");
    }

    #[test]
    fn expand_selected_on_collapsed_dir_reveals_its_children() {
        let dir = tempdir().unwrap();
        make_tree(dir.path());
        let mut modal = FileBrowserModal::new(dir.path().to_path_buf());

        modal.cursor = 1;
        assert_eq!(modal.rows[1].name, "src");
        assert!(!modal.rows[1].expanded);

        modal.expand_selected();
        assert!(modal.rows.iter().any(|r| r.name == "main.rs"));
    }

    #[test]
    fn expand_selected_on_file_does_nothing() {
        let dir = tempdir().unwrap();
        make_tree(dir.path());
        let mut modal = FileBrowserModal::new(dir.path().to_path_buf());

        let file_row_idx = modal.rows.iter().position(|r| r.name == "Cargo.toml").unwrap();
        modal.cursor = file_row_idx;
        let row_count_before = modal.rows.len();
        modal.expand_selected();
        assert_eq!(modal.rows.len(), row_count_before);
    }

    #[test]
    fn collapse_selected_on_expanded_dir_hides_its_children() {
        let dir = tempdir().unwrap();
        make_tree(dir.path());
        let mut modal = FileBrowserModal::new(dir.path().to_path_buf());
        modal.cursor = 1; // "src"
        modal.expand_selected();
        assert!(modal.rows.iter().any(|r| r.name == "main.rs"));

        modal.collapse_selected();
        assert!(!modal.rows.iter().any(|r| r.name == "main.rs"));
    }

    #[test]
    fn collapse_selected_on_file_jumps_cursor_to_parent() {
        let dir = tempdir().unwrap();
        make_tree(dir.path());
        let mut modal = FileBrowserModal::new(dir.path().to_path_buf());
        modal.cursor = 1; // "src"
        modal.expand_selected();
        let main_rs_idx = modal.rows.iter().position(|r| r.name == "main.rs").unwrap();
        modal.cursor = main_rs_idx;

        modal.collapse_selected();
        assert_eq!(modal.rows[modal.cursor].name, "src");
    }

    #[test]
    fn rebuild_rows_clamps_cursor_when_rows_shrink() {
        let dir = tempdir().unwrap();
        make_tree(dir.path());
        let mut modal = FileBrowserModal::new(dir.path().to_path_buf());
        modal.cursor = 1;
        modal.expand_selected();
        modal.cursor = modal.rows.len() - 1;

        modal.collapse_selected();
        assert!(modal.cursor < modal.rows.len());
    }

    #[test]
    fn enter_as_root_on_file_does_nothing() {
        with_isolated_config_dir(|| {
            let dir = tempdir().unwrap();
            make_tree(dir.path());
            let mut modal = FileBrowserModal::new(dir.path().to_path_buf());
            let mut app = App::default();

            let file_idx = modal.rows.iter().position(|r| r.name == "Cargo.toml").unwrap();
            modal.cursor = file_idx;
            let root_path_before = modal.root.path.clone();

            modal.enter_as_root(&mut app);

            assert_eq!(modal.root.path, root_path_before);
            assert!(app.project_manager.root.is_none());
        });
    }

    #[test]
    fn enter_as_root_on_directory_reroots_the_browser_view() {
        with_isolated_config_dir(|| {
            let dir = tempdir().unwrap();
            make_tree(dir.path());
            let mut modal = FileBrowserModal::new(dir.path().to_path_buf());
            let mut app = App::default();

            modal.cursor = 1; // "src"
            let src_path = dir.path().join("src");

            modal.enter_as_root(&mut app);

            assert_eq!(modal.root.path, src_path);
            assert_eq!(modal.cursor, 0);
            // The browser's new root should show main.rs directly (it's
            // now inside src/, not nested under a "src" row anymore).
            assert!(modal.rows.iter().any(|r| r.name == "main.rs"));
        });
    }

    #[test]
    fn enter_as_root_on_directory_updates_project_manager_root() {
        with_isolated_config_dir(|| {
            let dir = tempdir().unwrap();
            make_tree(dir.path());
            let mut modal = FileBrowserModal::new(dir.path().to_path_buf());
            let mut app = App::default();

            modal.cursor = 1; // "src"
            let src_path = dir.path().join("src");

            modal.enter_as_root(&mut app);

            assert_eq!(app.project_manager.root, Some(src_path));
        });
    }
}
