//! Integration tests for the language installer (`src/language/install/`)
//! and its `:githubtoken` command, exercised the same way `app_tests.rs`
//! exercises `:w` — through the real command line → command executor →
//! plugin dispatch pipeline, not by calling internal functions directly.
//!
//! These assume `GithubAuthPlugin` has been registered in
//! `App::load_plugins_from_config` and enabled in `PluginsConfig`'s
//! defaults (see the integration notes given alongside
//! `github_auth_plugin.rs`). If that wiring isn't in place, the
//! `:githubtoken`/`:ghtoken` tests below will fail at the "command not
//! found" stage rather than actually setting a token — which is exactly
//! what should happen, and is a decent smoke test for the wiring itself.
//!
//! IMPORTANT — disk isolation: `InstallManager` persists the GitHub token
//! (and `apply_success`, exercised indirectly via `InstallManager`,
//! persists config changes) to the OS-standard config directory, not a
//! path these tests control directly. `with_isolated_config_dir` redirects
//! that via `XDG_CONFIG_HOME`, which the `dirs` crate honors on Linux; on
//! macOS/Windows `dirs::config_dir()` does not consult that variable, so
//! these tests will touch the real per-user config/token files on those
//! platforms. Run on Linux/CI if that's a concern.

use calliglyph::app_config::AppLaunchConfig;
use calliglyph::config::Config;
use calliglyph::core::app::*;
use calliglyph::input::actions::InputAction;
use std::sync::Mutex;
use tempfile::tempdir;

// Serializes tests that mutate the process-wide XDG_CONFIG_HOME env var.
static CONFIG_DIR_TEST_LOCK: Mutex<()> = Mutex::new(());

fn with_isolated_config_dir<F: FnOnce()>(f: F) {
    // Recovers from a poisoned lock rather than propagating PoisonError —
    // the shared state here is just an env var, so a prior test panicking
    // doesn't leave anything actually corrupted to worry about.
    let _guard = CONFIG_DIR_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let dir = tempdir().unwrap();
    let prev = std::env::var("XDG_CONFIG_HOME").ok();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());

    // catch_unwind, not a bare call: if `f` panics (e.g. an assertion
    // failure), the env var restore below must still run, or every test
    // after this one silently inherits a stale/deleted XDG_CONFIG_HOME.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));

    match prev {
        Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
        None => std::env::remove_var("XDG_CONFIG_HOME"),
    }

    if let Err(payload) = result {
        std::panic::resume_unwind(payload);
    }
}

fn create_app() -> App {
    App::new(Config::default(), AppLaunchConfig::default())
}

#[test]
fn githubtoken_command_sets_token_end_to_end() {
    with_isolated_config_dir(|| {
        let mut app = create_app();
        app.active_area = ActiveArea::CommandLine;
        app.command_line.input = ":githubtoken ghp_integrationtest".to_string();
        app.process_input_action(InputAction::ENTER);

        assert!(
            app.install_manager.has_github_token(),
            "expected :githubtoken to store a token — is GithubAuthPlugin registered \
             in App::load_plugins_from_config and enabled in PluginsConfig defaults?"
        );
    });
}

#[test]
fn githubtoken_clear_removes_token_end_to_end() {
    with_isolated_config_dir(|| {
        let mut app = create_app();
        app.active_area = ActiveArea::CommandLine;

        app.command_line.input = ":githubtoken ghp_integrationtest".to_string();
        app.process_input_action(InputAction::ENTER);
        assert!(app.install_manager.has_github_token());

        app.command_line.input = ":githubtoken clear".to_string();
        app.process_input_action(InputAction::ENTER);
        assert!(!app.install_manager.has_github_token());
    });
}

#[test]
fn ghtoken_alias_also_works() {
    with_isolated_config_dir(|| {
        let mut app = create_app();
        app.active_area = ActiveArea::CommandLine;
        app.command_line.input = ":ghtoken ghp_aliastest".to_string();
        app.process_input_action(InputAction::ENTER);

        assert!(app.install_manager.has_github_token());
    });
}

#[test]
fn githubtoken_with_no_argument_does_not_crash_the_app() {
    with_isolated_config_dir(|| {
        let mut app = create_app();
        // Establish a known starting state explicitly — XDG_CONFIG_HOME
        // isolation only works on Linux (see module docs above), so on
        // other platforms this app may otherwise pick up a real token
        // already configured on the machine.
        app.install_manager.set_github_token(None).ok();

        app.active_area = ActiveArea::CommandLine;
        app.command_line.input = ":githubtoken".to_string();
        app.process_input_action(InputAction::ENTER);

        // The command's error should surface as an error popup (same as any
        // other failed command — see App::on_command_enter), not a panic,
        // and should certainly not have set a token.
        assert!(!app.install_manager.has_github_token());
    });
}

#[test]
fn install_manager_catalogs_are_reachable_from_a_fresh_app() {
    // Doesn't touch disk/network — just confirms InstallManager and the
    // static registry are correctly constructed as part of App::new/Default.
    let app = create_app();
    assert!(!app.install_manager.grammar_catalog().is_empty());
    assert!(!app.install_manager.lsp_catalog().is_empty());
}

#[test]
fn default_app_also_has_a_working_install_manager() {
    with_isolated_config_dir(|| {
        let mut app = App::default();
        assert!(!app.install_manager.grammar_catalog().is_empty());

        // Establish a known state explicitly rather than assuming no token
        // is configured — XDG_CONFIG_HOME isolation only works on Linux
        // (see module docs above), so this app may otherwise legitimately
        // pick up a real token already configured on this machine.
        app.install_manager.set_github_token(None).ok();
        assert!(!app.install_manager.has_github_token());
    });
}
