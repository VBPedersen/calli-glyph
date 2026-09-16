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

//████████╗███████╗███████╗████████╗███████╗
//╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//   ██║   █████╗  ███████╗   ██║   ███████╗
//   ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//   ██║   ███████╗███████║   ██║   ███████║
//   ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝

#[cfg(test)]
mod unit_job_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn spawn_starts_in_running_state_with_no_outcome() {
        let job = InstallJob::spawn(JobId::Grammar("test".to_string()), |_tx| {
            // Sender dropped immediately — nothing to send, just confirming
            // the job starts Running before any event arrives.
        });
        assert!(job.is_running());
        assert!(job.outcome.is_none());
        assert!(job.log.is_empty());
    }

    #[test]
    fn poll_collects_logs_and_marks_success() {
        let mut job = InstallJob::spawn(JobId::LspServer("test".to_string()), |tx| {
            let _ = tx.send(InstallEvent::Log("step one".to_string()));
            let _ = tx.send(InstallEvent::Log("step two".to_string()));
            let _ = tx.send(InstallEvent::Done(Ok(InstallOutcome {
                summary: "done".to_string(),
                resolved_command: Some("/usr/bin/test".to_string()),
            })));
        });

        // Give the background thread a moment to push its events through.
        let finished = wait_until_finished(&mut job);

        assert!(finished);
        assert!(!job.is_running());
        assert_eq!(
            job.log,
            vec!["step one".to_string(), "step two".to_string()]
        );
        match &job.status {
            JobStatus::Success(summary) => assert_eq!(summary, "done"),
            _ => panic!("expected Success status"),
        }
        assert_eq!(
            job.outcome.as_ref().unwrap().resolved_command.as_deref(),
            Some("/usr/bin/test")
        );
    }

    #[test]
    fn poll_marks_failure_on_err() {
        let mut job = InstallJob::spawn(JobId::Grammar("bad".to_string()), |tx| {
            let _ = tx.send(InstallEvent::Done(Err("boom".to_string())));
        });

        let finished = wait_until_finished(&mut job);

        assert!(finished);
        assert!(!job.is_running());
        assert!(job.outcome.is_none(), "outcome should stay None on failure");
        match &job.status {
            JobStatus::Failed(msg) => assert_eq!(msg, "boom"),
            _ => panic!("expected Failed status"),
        }
    }

    #[test]
    fn job_id_equality_is_by_kind_and_name() {
        assert_eq!(
            JobId::Grammar("rust".to_string()),
            JobId::Grammar("rust".to_string())
        );
        assert_ne!(
            JobId::Grammar("rust".to_string()),
            JobId::LspServer("rust".to_string())
        );
        assert_ne!(
            JobId::Grammar("rust".to_string()),
            JobId::Grammar("python".to_string())
        );
    }

    /// Polls `job` in a loop for up to a second, since the background
    /// thread's events arrive asynchronously.
    fn wait_until_finished(job: &mut InstallJob) -> bool {
        for _ in 0..100 {
            if job.poll() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        false
    }
}
