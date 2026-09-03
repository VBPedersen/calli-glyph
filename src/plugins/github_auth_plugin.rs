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
//! on the command line while typing/pasting it, and (depending on your
//! `CommandLine` implementation) may end up in in-memory command history.
//! It is never logged. If that's a concern, generate a short-lived,
//! narrowly-scoped token and rotate it.

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
