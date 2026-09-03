//! Ties the grammar/LSP installers together with the running app: owns
//! in-flight jobs, polls them each tick, and once a job finishes writes the
//! result into `Config` (and saves it) so it takes effect immediately —
//! no restart needed to start using a freshly installed grammar or server.

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
}

impl InstallManager {
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
        }
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
        let job = InstallJob::spawn(job_id, move |tx| {
            grammar_installer::install_grammar(*spec, grammar_dir, tx);
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
                    file_extensions: spec
                        .file_extensions
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    grammar: spec.name.to_string(),
                },
            );
            if let Err(e) = config.save() {
                log_error!("[Install] Failed to save config after grammar install: {}", e);
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
                    file_extensions: spec
                        .file_extensions
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
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
