//! Exposes `:explore` (alias `:files`) to open the configured file
//! picker, built-in browser or an external tool, per
//! `config.project.file_picker`.

use crate::core::app::App;
use crate::errors::plugin_error::PluginError;
use crate::plugins::plugin_registry::{Plugin, PluginCommand, PluginMetadata};
use crate::project::open_file_picker;
use ratatui::Frame;
use std::path::PathBuf;

pub struct ProjectExplorerPlugin;

impl ProjectExplorerPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for ProjectExplorerPlugin {
    fn name(&self) -> &str {
        "project_explorer_plugin"
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "project_explorer_plugin".to_string(),
            commands: vec![
                PluginCommand {
                    name: "explore".to_string(),
                    description: "Open the file picker (builtin browser or configured external tool)"
                        .to_string(),
                    aliases: vec!["files".to_string()],
                    handler: handle_explore,
                },
                PluginCommand {
                    name: "cd".to_string(),
                    description: "Set the project root explicitly (`:cd <path>` or `:cd` for current repo root)"
                        .to_string(),
                    aliases: vec!["project".to_string()],
                    handler: handle_cd,
                }
            ],
            keybinds: vec![],
        }
    }

    fn init(&mut self, _app: &mut App) -> Result<(), PluginError> {
        Ok(())
    }

    fn handle_key_event(&mut self, _app: &mut App, _key: crossterm::event::KeyEvent) -> bool {
        false
    }

    fn render(&self, _frame: &mut Frame, _app: &App) -> bool {
        false
    }
}

impl Default for ProjectExplorerPlugin {
    fn default() -> Self {
        Self::new()
    }
}

fn handle_explore(app: &mut App, _args: Vec<String>) -> Result<(), PluginError> {
    open_file_picker(app);
    Ok(())
}

fn handle_cd(app: &mut App, args: Vec<String>) -> Result<(), PluginError> {
    // No argument: snap back to the auto-detected root (walking up from
    // whatever file is currently open, or cwd). Useful after having
    // explicitly :cd'd somewhere else and wanting the "real" project root
    // back without retyping it.
    if args.is_empty() {
        let start = app
            .file_path
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        app.project_manager.set_root_from_path(&start);
        log_info!(
            "[Project] Root reset to detected project root: {:?}",
            app.project_manager.root
        );
        return Ok(());
    }

    let requested = PathBuf::from(&args[0]);
    let base = app
        .project_manager
        .root
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let resolved = if requested.is_absolute() {
        requested
    } else {
        base.join(requested)
    };
    let resolved = resolved.canonicalize().unwrap_or(resolved);

    if !resolved.is_dir() {
        return Err(PluginError::Internal(format!(
            "'{}' is not a directory",
            resolved.display()
        )));
    }

    app.project_manager.set_root(resolved.clone());
    log_info!("[Project] Root set to {}", resolved.display());
    Ok(())
}

#[cfg(test)]
mod unit_project_explorer_plugin_tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::tempdir;

    // Same disk-isolation caveat as github_auth_plugin.rs's tests:
    // App::default() touches the real OS config dir via ThemeManager /
    // InstallManager unless redirected. See that file for the full note.
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
    fn plugin_reports_both_commands_with_aliases() {
        let plugin = ProjectExplorerPlugin::new();
        let metadata = plugin.metadata();
        assert_eq!(metadata.commands.len(), 2);

        let explore = metadata
            .commands
            .iter()
            .find(|c| c.name == "explore")
            .unwrap();
        assert!(explore.aliases.contains(&"files".to_string()));

        let cd = metadata.commands.iter().find(|c| c.name == "cd").unwrap();
        assert!(cd.aliases.contains(&"project".to_string()));
    }

    #[test]
    fn cd_with_valid_absolute_path_sets_root() {
        with_isolated_config_dir(|| {
            let dir = tempdir().unwrap();
            let mut app = App::default();

            handle_cd(&mut app, vec![dir.path().to_string_lossy().to_string()]).unwrap();

            assert_eq!(
                app.project_manager.root.as_deref(),
                Some(dir.path().canonicalize().unwrap().as_path())
            );
        });
    }

    #[test]
    fn cd_with_nonexistent_path_errors_and_does_not_change_root() {
        with_isolated_config_dir(|| {
            let mut app = App::default();
            let before = app.project_manager.root.clone();

            let result = handle_cd(&mut app, vec!["/definitely/not/a/real/path".to_string()]);

            assert!(result.is_err());
            assert_eq!(app.project_manager.root, before);
        });
    }

    #[test]
    fn cd_with_path_to_a_file_not_a_directory_errors() {
        with_isolated_config_dir(|| {
            let dir = tempdir().unwrap();
            let file = dir.path().join("not_a_dir.txt");
            std::fs::write(&file, "").unwrap();
            let mut app = App::default();

            let result = handle_cd(&mut app, vec![file.to_string_lossy().to_string()]);
            assert!(result.is_err());
        });
    }

    #[test]
    fn cd_with_relative_path_resolves_against_current_root() {
        with_isolated_config_dir(|| {
            let dir = tempdir().unwrap();
            std::fs::create_dir(dir.path().join("subdir")).unwrap();
            let mut app = App::default();
            app.project_manager.set_root(dir.path().to_path_buf());

            handle_cd(&mut app, vec!["subdir".to_string()]).unwrap();

            assert_eq!(
                app.project_manager.root.as_deref(),
                Some(dir.path().join("subdir").canonicalize().unwrap().as_path())
            );
        });
    }

    #[test]
    fn cd_with_no_args_and_no_open_file_falls_back_to_cwd_based_detection() {
        with_isolated_config_dir(|| {
            let mut app = App::default();
            handle_cd(&mut app, vec![]).unwrap();
            // Just confirms it doesn't panic and produces *some* root,
            // since the exact detected directory depends on where tests
            // happen to run from.
            assert!(app.project_manager.root.is_some());
        });
    }

    #[test]
    fn explore_in_builtin_mode_pushes_a_modal() {
        with_isolated_config_dir(|| {
            let mut app = App::default();
            assert_eq!(
                app.config.project.file_picker.mode,
                crate::config::project::PickerMode::Builtin
            );
            let before = app.modal_stack.len();

            handle_explore(&mut app, vec![]).unwrap();

            assert_eq!(app.modal_stack.len(), before + 1);
        });
    }
}
