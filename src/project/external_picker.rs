//! Runs an externally-configured file picker (yazi, ranger, nnn, or
//! anything else registered in `[project.file_picker.external]`).
//!
//! The actual process-spawning and argument-substitution logic
//! (`build_args`, `read_chosen_path`) is kept separate from the
//! terminal-suspend/resume logic (`suspend_terminal`/`resume_terminal`),
//! specifically so the former: the part that varies per picker and is
//! worth getting right. Can be unit tested without actually spawning a
//! process or touching the real terminal.

use crate::config::project::{ExternalPickerSpec, OutputMode};
use std::io::Stdout;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Runs `spec`, starting the picker in `start_dir`. Returns:
/// - `Ok(Some(path))` : the user picked a file
/// - `Ok(None))` :  the picker exited without a selection (cancelled, or a
///   directory-only pick in a tool that allows that)
/// - `Err(message)` : the picker failed to run at all, or its output
///   couldn't be read
pub fn run_external_picker(
    spec: &ExternalPickerSpec,
    start_dir: &Path,
) -> Result<Option<PathBuf>, String> {
    let output_path =
        std::env::temp_dir().join(format!("calliglyph-picker-{}", std::process::id()));
    let _ = std::fs::remove_file(&output_path); // clean slate from any previous run

    let args = build_args(&spec.args, start_dir, &output_path);

    suspend_terminal()?;
    let run_result = match spec.output_mode {
        OutputMode::OutputFile => run_inheriting_stdio(&spec.command, &args, start_dir),
        OutputMode::Stdout => run_capturing_stdout(&spec.command, &args, start_dir),
    };
    resume_terminal()?;

    let outcome = run_result?;

    let result = match spec.output_mode {
        // OutputFile mode doesn't need to check `outcome.success`: a
        // cancelled/failed run naturally leaves the output file
        // missing/empty, which read_chosen_path already treats as "no
        // selection". Checking exit status too would be redundant, and
        // some pickers (notably yazi on Esc) exit nonzero even on a
        // perfectly normal "no selection" cancellation, so treating
        // nonzero as a hard error here would be wrong.
        OutputMode::OutputFile => {
            let path = read_chosen_path(&output_path);
            let _ = std::fs::remove_file(&output_path);
            path
        }
        // Stdout mode has no separate "file exists or not" signal. The
        // only thing distinguishing "cancelled" from "picked, but printed nothing"
        // is the exit status, so this mode
        // needs the check.
        OutputMode::Stdout if outcome.success => outcome
            .stdout
            .filter(|s| !s.trim().is_empty())
            .map(|s| PathBuf::from(s.trim())),
        OutputMode::Stdout => None,
    };

    Ok(result)
}

struct RunOutcome {
    success: bool,
    /// Only populated for `OutputMode::Stdout`.
    stdout: Option<String>,
}

fn run_inheriting_stdio(
    command: &str,
    args: &[String],
    start_dir: &Path,
) -> Result<RunOutcome, String> {
    let status = Command::new(command)
        .args(args)
        .current_dir(start_dir)
        .status()
        .map_err(|e| format!("Failed to run {}: {}", command, e))?;
    Ok(RunOutcome {
        success: status.success(),
        stdout: None,
    })
}

fn run_capturing_stdout(
    command: &str,
    args: &[String],
    start_dir: &Path,
) -> Result<RunOutcome, String> {
    let output = Command::new(command)
        .args(args)
        .current_dir(start_dir)
        .stdin(Stdio::inherit()) // picker may still want /dev/tty for input
        .output()
        .map_err(|e| format!("Failed to run {}: {}", command, e))?;
    Ok(RunOutcome {
        success: output.status.success(),
        stdout: Some(String::from_utf8_lossy(&output.stdout).to_string()),
    })
}

/// Substitutes `{dir}` and `{output}` placeholders in an arg list. Pure
/// and side-effect-free on purpose. It's the part that's picker-specific and most likely
/// to have a typo'd placeholder when someone adds a new picker to config.
fn build_args(template: &[String], start_dir: &Path, output_path: &Path) -> Vec<String> {
    let dir_str = start_dir.to_string_lossy();
    let output_str = output_path.to_string_lossy();
    template
        .iter()
        .map(|arg| {
            arg.replace("{dir}", &dir_str)
                .replace("{output}", &output_str)
        })
        .collect()
}

/// Reads back the path an `OutputMode::OutputFile` picker wrote. Treats a
/// missing or empty file as "no selection" rather than an error. That's
/// the normal outcome when a picker is cancelled (e.g. Esc in yazi without
/// choosing anything), not a failure.
fn read_chosen_path(output_path: &Path) -> Option<PathBuf> {
    let content = std::fs::read_to_string(output_path).ok()?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        None
    } else {
        // Some pickers (nnn in multi-select) can write multiple
        // newline-separated paths; take the first until/unless multi-file
        // open is something the editor supports.
        trimmed.lines().next().map(PathBuf::from)
    }
}

/// Leaves the alternate screen and disables raw mode so the external
/// picker gets a normal, interactive terminal: mirrors exactly what
/// `main.rs` sets up at startup, just run in reverse, temporarily.
fn suspend_terminal() -> Result<(), String> {
    use crossterm::event::DisableMouseCapture;
    use crossterm::execute;
    use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};

    disable_raw_mode().map_err(|e| format!("Failed to disable raw mode: {}", e))?;
    execute!(stdout_handle(), LeaveAlternateScreen, DisableMouseCapture)
        .map_err(|e| format!("Failed to leave alternate screen: {}", e))?;
    Ok(())
}

/// Restores the TUI's terminal state after the external picker exits.
/// Note: this does NOT force ratatui to redraw: the caller is
/// responsible for setting `App.force_full_redraw = true` so the next
/// frame calls `terminal.clear()` before drawing, since ratatui's internal
/// diff buffer has no way to know the physical screen changed underneath
/// it while we were suspended.
fn resume_terminal() -> Result<(), String> {
    use crossterm::event::EnableMouseCapture;
    use crossterm::execute;
    use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen};

    enable_raw_mode().map_err(|e| format!("Failed to enable raw mode: {}", e))?;
    execute!(stdout_handle(), EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| format!("Failed to enter alternate screen: {}", e))?;
    Ok(())
}

fn stdout_handle() -> Stdout {
    std::io::stdout()
}

#[cfg(test)]
mod unit_external_picker_tests {
    use super::*;

    #[test]
    fn build_args_substitutes_output_placeholder() {
        let args = build_args(
            &["--chooser-file".to_string(), "{output}".to_string()],
            Path::new("/home/user/project"),
            Path::new("/tmp/out.txt"),
        );
        assert_eq!(args, vec!["--chooser-file", "/tmp/out.txt"]);
    }

    #[test]
    fn build_args_substitutes_dir_placeholder() {
        let args = build_args(
            &["{dir}".to_string()],
            Path::new("/home/user/project"),
            Path::new("/tmp/out.txt"),
        );
        assert_eq!(args, vec!["/home/user/project"]);
    }

    #[test]
    fn build_args_substitutes_both_placeholders_in_one_arg() {
        // Edge case: a hypothetical picker that wants both in a single
        // combined argument rather than separate ones.
        let args = build_args(
            &["{dir}:{output}".to_string()],
            Path::new("/proj"),
            Path::new("/tmp/out"),
        );
        assert_eq!(args, vec!["/proj:/tmp/out"]);
    }

    #[test]
    fn build_args_leaves_args_without_placeholders_untouched() {
        let args = build_args(
            &["--some-flag".to_string(), "literal-value".to_string()],
            Path::new("/proj"),
            Path::new("/tmp/out"),
        );
        assert_eq!(args, vec!["--some-flag", "literal-value"]);
    }

    #[test]
    fn build_args_handles_empty_template() {
        let args = build_args(&[], Path::new("/proj"), Path::new("/tmp/out"));
        assert!(args.is_empty());
    }

    #[test]
    fn build_args_matches_yazi_default_spec() {
        let yazi_args = vec![
            "{dir}".to_string(),
            "--chooser-file".to_string(),
            "{output}".to_string(),
        ];
        let args = build_args(
            &yazi_args,
            Path::new("/home/me/proj"),
            Path::new("/tmp/calliglyph-picker-123"),
        );
        assert_eq!(
            args,
            vec![
                "/home/me/proj",
                "--chooser-file",
                "/tmp/calliglyph-picker-123"
            ]
        );
    }

    #[test]
    fn read_chosen_path_returns_none_for_missing_file() {
        assert!(read_chosen_path(Path::new("/definitely/does/not/exist")).is_none());
    }

    #[test]
    fn read_chosen_path_returns_none_for_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.txt");
        std::fs::write(&path, "").unwrap();
        assert!(read_chosen_path(&path).is_none());
    }

    #[test]
    fn read_chosen_path_returns_none_for_whitespace_only_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ws.txt");
        std::fs::write(&path, "   \n  \n").unwrap();
        assert!(read_chosen_path(&path).is_none());
    }

    #[test]
    fn read_chosen_path_trims_trailing_newline() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("chosen.txt");
        std::fs::write(&path, "/home/user/project/src/main.rs\n").unwrap();
        assert_eq!(
            read_chosen_path(&path),
            Some(PathBuf::from("/home/user/project/src/main.rs"))
        );
    }

    #[test]
    fn read_chosen_path_takes_first_line_of_multi_line_output() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("multi.txt");
        std::fs::write(&path, "/a/first.rs\n/a/second.rs\n").unwrap();
        assert_eq!(read_chosen_path(&path), Some(PathBuf::from("/a/first.rs")));
    }
}
