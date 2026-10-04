//! Project view / file browser.
//!
//! `tree` is the pure data model, `browser` is the built-in `Modal` UI
//! built on it, `external_picker` runs a configured external tool
//! instead, and `manager` dispatches between the two based on
//! `config.project.file_picker.mode`.
//!
//! Adding a new external picker is pure configuration, see
//! `config/project.rs`'s `FilePickerConfig::default()` for the shape, and
//! its `user_can_add_a_custom_picker_via_toml_alone` test for proof it
//! needs no code change.

pub mod external_picker;
pub mod manager;
pub mod root_detect;
pub mod tree;

pub use crate::ui::modal::browser::FileBrowserModal;
pub use manager::{open_file_picker, ProjectManager};
pub use root_detect::find_project_root;
pub use tree::{FileNode, VisibleRow};
