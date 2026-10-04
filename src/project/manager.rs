//! Ties the file picker config to the running app: knows the current
//! project root, and dispatches "open the picker" to either the built-in
//! `FileBrowserModal` or a configured external command.

use super::browser::FileBrowserModal;
use super::external_picker;
use crate::config::project::PickerMode;
use crate::core::app::App;
use std::path::PathBuf;

/// Holds project-level state.
/// TODO for now minimal: just the root, but home for later project view functionality
/// e.g. (open-file lists, a cached tree, workspace markers) without that state
/// needing to live loose on `App`.
pub struct ProjectManager {
    pub root: Option<PathBuf>,
}

impl ProjectManager {
    pub fn new() -> Self {
        ProjectManager { root: None }
    }

    pub fn set_root(&mut self, root: PathBuf) {
        self.root = Some(root);
    }
}

impl Default for ProjectManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Opens the configured file picker. A free function rather than a method
/// on `ProjectManager`. It needs `&mut App` for both reading config and
/// (for the built-in picker) pushing a modal / (for external pickers)
/// calling `app.open_file`, and `&mut self` on the manager plus `&mut App`
/// at the same time is exactly the double-borrow trap `app.plugins`
/// already works around.
///
/// For an external picker, this call blocks (synchronously) until the
/// picker exits. Acceptable since suspending the TUI to hand the
/// terminal to the external program is the whole point. There's nothing
/// useful for the editor to do concurrently while the user is in external picker (like yazi).
pub fn open_file_picker(app: &mut App) {
    let picker_config = app.config.project.file_picker.clone();
    let start_dir = app
        .project_manager
        .root
        .clone()
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));

    match picker_config.mode {
        PickerMode::Builtin => {
            app.modal_stack
                .push(Box::new(FileBrowserModal::new(start_dir)));
        }
        PickerMode::External => {
            let Some(name) = &picker_config.active_external else {
                log_warn!("[Project] mode = \"external\" but no active_external is configured");
                return;
            };
            let Some(spec) = picker_config.external.get(name) else {
                log_warn!(
                    "[Project] active_external = '{}' but no matching [project.file_picker.external.{}] entry exists",
                    name, name
                );
                return;
            };

            match external_picker::run_external_picker(spec, &start_dir) {
                Ok(Some(path)) => app.open_file(path, None),
                Ok(None) => log_info!("[Project] Picker closed without a selection"),
                Err(e) => log_warn!("[Project] External picker '{}' failed: {}", name, e),
            }

            // Terminal was suspended/resumed around the external process.
            // ratatui's diff buffer has no way to know the physical screen
            // changed underneath it, so force a full redraw next frame.
            app.force_full_redraw = true;
        }
    }
}

#[cfg(test)]
mod unit_manager_tests {
    use super::*;

    #[test]
    fn new_manager_has_no_root() {
        let manager = ProjectManager::new();
        assert!(manager.root.is_none());
    }

    #[test]
    fn set_root_updates_root() {
        let mut manager = ProjectManager::new();
        manager.set_root(PathBuf::from("/home/user/project"));
        assert_eq!(manager.root, Some(PathBuf::from("/home/user/project")));
    }

    #[test]
    fn default_matches_new() {
        let manager = ProjectManager::default();
        assert!(manager.root.is_none());
    }
}
