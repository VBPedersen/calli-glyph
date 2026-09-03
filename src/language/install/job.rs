//! Generic background job runner for the language installer.
//!
//! Both grammar builds (git clone + compile) and LSP server installs
//! (spawn npm/pip/cargo/go) are long-running, blocking operations, so they
//! run on a background thread and stream progress back over an mpsc
//! channel — the same pattern `LspClient`'s reader thread already uses.
//! `InstallManager::poll` drains these once per tick from `App::run`,
//! exactly like `LanguageManager::poll_lsp`.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

/// What is being installed. Used as the key in `InstallManager`'s job map.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum JobId {
    Grammar(String),
    LspServer(String),
}

/// A single line of progress, or the final outcome of a job.
pub enum InstallEvent {
    Log(String),
    /// Ok(outcome) on success, Err(message) on failure. Exactly one of
    /// these is sent, always last.
    Done(Result<InstallOutcome, String>),
}

/// Result of a successful install.
pub struct InstallOutcome {
    /// Human-readable summary shown in the UI.
    pub summary: String,
    /// For LSP installs: the resolved absolute path to the installed
    /// binary, when it wasn't already bare-resolvable via PATH. `None` for
    /// grammar installs (which don't need this — the .so's location is
    /// always `grammar_dir`, already known).
    pub resolved_command: Option<String>,
}

/// Current state of a tracked install job.
pub enum JobStatus {
    Running,
    Success(String),
    Failed(String),
}

pub struct InstallJob {
    pub id: JobId,
    pub status: JobStatus,
    pub log: Vec<String>,
    /// Set once a job finishes successfully; carries the resolved binary
    /// path for LSP jobs so `InstallManager` can persist it to config.
    pub outcome: Option<InstallOutcome>,
    receiver: Receiver<InstallEvent>,
}

impl InstallJob {
    /// Spawns `work` on a background thread. `work` receives a `Sender` it
    /// should use to stream `InstallEvent::Log` lines, finishing with
    /// exactly one `InstallEvent::Done`.
    pub fn spawn<F>(id: JobId, work: F) -> Self
    where
        F: FnOnce(Sender<InstallEvent>) + Send + 'static,
    {
        let (tx, rx) = channel();
        thread::spawn(move || work(tx));
        Self {
            id,
            status: JobStatus::Running,
            log: Vec::new(),
            outcome: None,
            receiver: rx,
        }
    }

    /// Drains all pending events for this job. Returns true if the job
    /// finished (successfully or not) during this call, so the caller can
    /// react (e.g. persist config) exactly once.
    pub fn poll(&mut self) -> bool {
        let mut just_finished = false;
        while let Ok(event) = self.receiver.try_recv() {
            match event {
                InstallEvent::Log(line) => {
                    log_info!("[Install] {}", line);
                    self.log.push(line);
                }
                InstallEvent::Done(Ok(outcome)) => {
                    log_info!("[Install] Done: {}", outcome.summary);
                    self.status = JobStatus::Success(outcome.summary.clone());
                    self.outcome = Some(outcome);
                    just_finished = true;
                }
                InstallEvent::Done(Err(err)) => {
                    log_warn!("[Install] Failed: {}", err);
                    self.status = JobStatus::Failed(err);
                    just_finished = true;
                }
            }
        }
        just_finished
    }

    pub fn is_running(&self) -> bool {
        matches!(self.status, JobStatus::Running)
    }
}
