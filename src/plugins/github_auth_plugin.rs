//! Exposes a `:githubtoken` command for setting or clearing the GitHub
//! Personal Access Token used to authenticate grammar-repo clones.
//!
//! This is a command rather than a modal text-input field on purpose: the
//! command line is the one input surface in this app already proven to
//! handle arbitrary pasted/typed text (tokens are long, random-looking
//! strings), so reusing it avoids reinventing text entry just for this.
//!
//! Usage:
//!   :githubtoken <token>   set the token (used for all future grammar clones)
//!   :githubtoken clear     remove the stored token, back to anonymous cloning
//!
//! Note on secrecy: like any other command, the token is briefly visible
//! on the command line while typing/pasting it, and
//! TODO might later need to be disregarded for a coming commandline history

use crate::core::app::App;
use crate::errors::plugin_error::PluginError;
use crate::plugins::plugin_registry::{Plugin, PluginCommand, PluginMetadata};
use ratatui::Frame;

pub struct GithubAuthPlugin;

impl GithubAuthPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for GithubAuthPlugin {
    fn name(&self) -> &str {
        "github_auth_plugin"
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "github_auth_plugin".to_string(),
            commands: vec![PluginCommand {
                name: "githubtoken".to_string(),
                description: "Set or clear the GitHub token used for grammar installs \
                               (`:githubtoken <token>` or `:githubtoken clear`)"
                    .to_string(),
                aliases: vec!["ghtoken".to_string()],
                handler: handle_githubtoken,
            }],
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

impl Default for GithubAuthPlugin {
    fn default() -> Self {
        Self::new()
    }
}

fn handle_githubtoken(app: &mut App, args: Vec<String>) -> Result<(), PluginError> {
    let Some(first) = args.first() else {
        return Err(PluginError::Internal(
            "Usage: :githubtoken <token>  or  :githubtoken clear".to_string(),
        ));
    };

    let token = if first.eq_ignore_ascii_case("clear") {
        None
    } else {
        Some(first.clone())
    };

    let cleared = token.is_none();
    app.install_manager
        .set_github_token(token)
        .map_err(PluginError::Internal)?;

    log_info!(
        "[GithubAuthPlugin] GitHub token {}",
        if cleared { "cleared" } else { "set" }
    );

    Ok(())
}

//████████╗███████╗███████╗████████╗███████╗
//╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//   ██║   █████╗  ███████╗   ██║   ███████╗
//   ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//   ██║   ███████╗███████║   ██║   ███████║
//   ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝

// IMPORTANT: `App::default()` constructs an `InstallManager`, whose `new()`
// reads a token file from the real OS config directory (see
// `credentials.rs`). `with_isolated_config_dir` redirects that via
// `XDG_CONFIG_HOME` (Linux only — see the same caveat in manager.rs's tests).
#[cfg(test)]
mod unit_github_auth_plugin_tests {
    use super::*;
    use crate::core::app::App;
    use std::sync::Mutex;
    use tempfile::tempdir;

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
    fn setting_token_enables_has_github_token() {
        with_isolated_config_dir(|| {
            let mut app = App::default();
            // Establish a known starting state explicitly — on Windows/macOS
            // the config-dir redirection above doesn't apply (see module
            // docs), so this app may otherwise legitimately already have a
            // real token configured on this machine.
            app.install_manager.set_github_token(None).ok();
            assert!(!app.install_manager.has_github_token());

            handle_githubtoken(&mut app, vec!["ghp_testtoken123".to_string()]).unwrap();
            assert!(app.install_manager.has_github_token());
        });
    }

    #[test]
    fn clear_removes_token() {
        with_isolated_config_dir(|| {
            let mut app = App::default();
            handle_githubtoken(&mut app, vec!["ghp_testtoken123".to_string()]).unwrap();
            assert!(app.install_manager.has_github_token());

            handle_githubtoken(&mut app, vec!["clear".to_string()]).unwrap();
            assert!(!app.install_manager.has_github_token());
        });
    }

    #[test]
    fn clear_is_case_insensitive() {
        with_isolated_config_dir(|| {
            let mut app = App::default();
            handle_githubtoken(&mut app, vec!["ghp_testtoken123".to_string()]).unwrap();
            handle_githubtoken(&mut app, vec!["CLEAR".to_string()]).unwrap();
            assert!(!app.install_manager.has_github_token());
        });
    }

    #[test]
    fn missing_argument_is_an_error() {
        with_isolated_config_dir(|| {
            let mut app = App::default();
            assert!(handle_githubtoken(&mut app, vec![]).is_err());
        });
    }

    #[test]
    fn plugin_reports_its_command_and_alias() {
        let plugin = GithubAuthPlugin::new();
        let metadata = plugin.metadata();
        assert_eq!(metadata.commands.len(), 1);
        assert_eq!(metadata.commands[0].name, "githubtoken");
        assert!(metadata.commands[0]
            .aliases
            .contains(&"ghtoken".to_string()));
    }

    #[test]
    fn plugin_name_matches_registration_key() {
        let plugin = GithubAuthPlugin::new();
        assert_eq!(plugin.name(), "github_auth_plugin");
        assert_eq!(plugin.metadata().name, "github_auth_plugin");
    }
}
