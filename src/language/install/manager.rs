//! Ties the grammar/LSP installers together with the running app: owns
//! in-flight jobs, polls them each tick, and once a job finishes writes the
//! result into `Config` (and saves it) so it takes effect immediately —
//! no restart needed to start using a freshly installed grammar or server.

use super::credentials;
use super::grammar_installer;
use super::job::{InstallJob, JobId, JobStatus};
use super::lsp_installer;
use super::registry::{GrammarSpec, LspSpec, GRAMMARS, LSP_SERVERS};
use crate::config::lsp::LspServerConfig;
use crate::config::syntax::SyntaxLanguageConfig;
use crate::config::Config;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct InstallManager {
    jobs: HashMap<JobId, InstallJob>,
    github_token: Option<String>,
}

impl InstallManager {
    pub fn new() -> Self {
        // Best-effort load — a missing/unreadable token file just means
        // grammar clones proceed anonymously, same as before this existed.
        let github_token = Config::get_config_base_dir()
            .ok()
            .and_then(|dir| credentials::load_github_token(&dir));

        Self {
            jobs: HashMap::new(),
            github_token,
        }
    }

    pub fn has_github_token(&self) -> bool {
        self.github_token.is_some()
    }

    /// Sets (or clears, with `None`) the GitHub token used to authenticate
    /// grammar clones, persisting it to disk immediately.
    pub fn set_github_token(&mut self, token: Option<String>) -> Result<(), String> {
        let dir = Config::get_config_base_dir().map_err(|e| e.to_string())?;
        credentials::save_github_token(&dir, token.as_deref())
            .map_err(|e| format!("Failed to save GitHub token: {}", e))?;
        self.github_token = token;
        Ok(())
    }

    pub fn grammar_catalog(&self) -> &'static [GrammarSpec] {
        GRAMMARS
    }

    pub fn lsp_catalog(&self) -> &'static [LspSpec] {
        LSP_SERVERS
    }

    pub fn job_status(&self, id: &JobId) -> Option<&JobStatus> {
        self.jobs.get(id).map(|j| &j.status)
    }

    pub fn job_log(&self, id: &JobId) -> Option<&[String]> {
        self.jobs.get(id).map(|j| j.log.as_slice())
    }

    pub fn is_installing(&self, id: &JobId) -> bool {
        self.jobs.get(id).map(|j| j.is_running()).unwrap_or(false)
    }

    /// Kicks off a grammar build in the background. No-op if already running.
    pub fn start_grammar_install(&mut self, spec: &'static GrammarSpec, grammar_dir: PathBuf) {
        let id = JobId::Grammar(spec.name.to_string());
        if self.is_installing(&id) {
            return;
        }
        let job_id = id.clone();
        let github_token = self.github_token.clone();
        let job = InstallJob::spawn(job_id, move |tx| {
            grammar_installer::install_grammar(*spec, grammar_dir, github_token, tx);
        });
        self.jobs.insert(id, job);
    }

    /// Kicks off an LSP server install in the background. No-op if already running.
    pub fn start_lsp_install(&mut self, spec: &'static LspSpec) {
        let id = JobId::LspServer(spec.name.to_string());
        if self.is_installing(&id) {
            return;
        }
        let job_id = id.clone();
        let job = InstallJob::spawn(job_id, move |tx| {
            lsp_installer::install_lsp(*spec, tx);
        });
        self.jobs.insert(id, job);
    }

    /// Drains all job channels. Call once per app tick (see `App::run`).
    /// On success, updates `config` and saves it.
    pub fn poll(&mut self, config: &mut Config) {
        let mut finished_ids: Vec<JobId> = Vec::new();
        for job in self.jobs.values_mut() {
            if job.poll() {
                finished_ids.push(job.id.clone());
            }
        }
        for id in finished_ids {
            if let Some(job) = self.jobs.get(&id) {
                if matches!(job.status, JobStatus::Success(_)) {
                    let resolved_command = job
                        .outcome
                        .as_ref()
                        .and_then(|o| o.resolved_command.clone());
                    apply_success(&id, config, resolved_command);
                }
            }
        }
    }
}

impl Default for InstallManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Registers a successfully installed grammar/server into `Config` and saves it.
/// `resolved_command`, when present, is the absolute path to the installed
/// LSP binary — used instead of the bare command name so the server still
/// launches correctly regardless of this process's (possibly stale) PATH.
fn apply_success(id: &JobId, config: &mut Config, resolved_command: Option<String>) {
    match id {
        JobId::Grammar(name) => {
            let Some(spec) = GRAMMARS.iter().find(|g| g.name == name) else {
                return;
            };
            config.syntax.languages.insert(
                spec.name.to_string(),
                SyntaxLanguageConfig {
                    file_extensions: spec.file_extensions.iter().map(|s| s.to_string()).collect(),
                    grammar: spec.name.to_string(),
                },
            );
            if let Err(e) = config.save() {
                log_error!(
                    "[Install] Failed to save config after grammar install: {}",
                    e
                );
            }
        }
        JobId::LspServer(name) => {
            let Some(spec) = LSP_SERVERS.iter().find(|s| s.name == name) else {
                return;
            };
            config.lsp.servers.insert(
                spec.name.to_string(),
                LspServerConfig {
                    command: resolved_command.unwrap_or_else(|| spec.command.to_string()),
                    args: spec.args.iter().map(|s| s.to_string()).collect(),
                    enabled: true,
                    file_extensions: spec.file_extensions.iter().map(|s| s.to_string()).collect(),
                    root_markers: spec.root_markers.iter().map(|s| s.to_string()).collect(),
                    initialization_options: Default::default(),
                },
            );
            if let Err(e) = config.save() {
                log_error!("[Install] Failed to save config after LSP install: {}", e);
            }
        }
    }
}

//████████╗███████╗███████╗████████╗███████╗
//╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//   ██║   █████╗  ███████╗   ██║   ███████╗
//   ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//   ██║   ███████╗███████║   ██║   ███████║
//   ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝

// IMPORTANT: `apply_success` calls `config.save()`, which writes to the
// real OS config directory (via `Config::get_config_base_dir()`) — not a
// path these tests control. `with_isolated_config_dir` redirects that via
// `XDG_CONFIG_HOME`, which the `dirs` crate honors on Linux; on macOS and
// Windows `dirs::config_dir()` does NOT consult that variable, so these
// tests will write to your real per-user config location on those
// platforms. If that's a concern, run these specifically on Linux/CI, or
// treat a save failure/side effect here as a known limitation to fix by
// making `Config::save()`'s target directory injectable.
#[cfg(test)]
mod unit_manager_tests {
    use super::*;
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
    fn apply_success_registers_grammar_in_config() {
        with_isolated_config_dir(|| {
            let mut config = Config::default();
            apply_success(&JobId::Grammar("rust".to_string()), &mut config, None);

            let entry = config
                .syntax
                .languages
                .get("rust")
                .expect("rust should be registered");
            assert_eq!(entry.grammar, "rust");
            assert!(entry.file_extensions.contains(&"rs".to_string()));
        });
    }

    #[test]
    fn apply_success_registers_lsp_with_resolved_command() {
        with_isolated_config_dir(|| {
            let mut config = Config::default();
            apply_success(
                &JobId::LspServer("python".to_string()),
                &mut config,
                Some("/home/user/.local/bin/pyright-langserver".to_string()),
            );

            let entry = config
                .lsp
                .servers
                .get("python")
                .expect("python lsp should be registered");
            assert_eq!(entry.command, "/home/user/.local/bin/pyright-langserver");
            assert!(entry.enabled);
        });
    }

    #[test]
    fn apply_success_falls_back_to_bare_command_when_unresolved() {
        with_isolated_config_dir(|| {
            let mut config = Config::default();
            apply_success(&JobId::LspServer("go".to_string()), &mut config, None);

            let entry = config.lsp.servers.get("go").unwrap();
            assert_eq!(entry.command, "gopls");
        });
    }

    #[test]
    fn apply_success_ignores_unknown_ids() {
        with_isolated_config_dir(|| {
            let mut config = Config::default();
            let before_syntax = config.syntax.languages.len();
            let before_lsp = config.lsp.servers.len();

            apply_success(
                &JobId::Grammar("not-a-real-grammar".to_string()),
                &mut config,
                None,
            );
            apply_success(
                &JobId::LspServer("not-a-real-server".to_string()),
                &mut config,
                None,
            );

            assert_eq!(config.syntax.languages.len(), before_syntax);
            assert_eq!(config.lsp.servers.len(), before_lsp);
        });
    }

    #[test]
    fn new_install_manager_has_no_running_jobs() {
        with_isolated_config_dir(|| {
            let manager = InstallManager::new();
            let id = JobId::Grammar("rust".to_string());
            assert!(!manager.is_installing(&id));
            assert!(manager.job_status(&id).is_none());
            assert!(manager.job_log(&id).is_none());
        });
    }

    #[test]
    fn catalogs_are_exposed_and_non_empty() {
        with_isolated_config_dir(|| {
            let manager = InstallManager::new();
            assert!(!manager.grammar_catalog().is_empty());
            assert!(!manager.lsp_catalog().is_empty());
        });
    }
}
