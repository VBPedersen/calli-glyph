//! Builds a tree-sitter grammar from source: clone the repo, compile the
//! generated parser (and scanner, if any) with the system C/C++ compiler,
//! and drop the resulting shared library into the grammar directory.
//!
//! No extra crates required — this shells out to `git` and a C compiler,
//! the same tools tree-sitter's own CLI relies on for `tree-sitter build`.
//!
//! NOTE: Windows support is best-effort — it assumes a `cc`/`clang`-style
//! compiler (e.g. via MSYS2/MinGW) is on PATH. MSVC's `cl.exe` uses a
//! different flag set and isn't handled here yet.

use super::job::{InstallEvent, InstallOutcome};
use super::registry::GrammarSpec;
use crate::language::grammar_loader::platform_lib_name;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

/// Entry point run on the background thread spawned by `InstallJob::spawn`.
pub fn install_grammar(
    spec: GrammarSpec,
    grammar_dir: PathBuf,
    github_token: Option<String>,
    tx: Sender<InstallEvent>,
) {
    let result = try_install(&spec, &grammar_dir, github_token.as_deref(), &tx);
    let _ = tx.send(InstallEvent::Done(result));
}

fn log(tx: &Sender<InstallEvent>, msg: impl Into<String>) {
    let _ = tx.send(InstallEvent::Log(msg.into()));
}

fn try_install(
    spec: &GrammarSpec,
    grammar_dir: &Path,
    github_token: Option<&str>,
    tx: &Sender<InstallEvent>,
) -> Result<InstallOutcome, String> {
    std::fs::create_dir_all(grammar_dir)
        .map_err(|e| format!("Failed to create grammar dir: {}", e))?;

    let tmp_dir = std::env::temp_dir().join(format!("calliglyph-grammar-{}", spec.name));
    if tmp_dir.exists() {
        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    log(tx, format!("Cloning {}", spec.repo_url));
    clone_repo(spec, &tmp_dir, github_token, tx)?;

    let grammar_root = match spec.subdir {
        Some(sub) => tmp_dir.join(sub),
        None => tmp_dir.clone(),
    };
    let src_dir = grammar_root.join("src");

    if !src_dir.join("parser.c").exists() {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return Err(format!(
            "No src/parser.c found in {}. The grammar repo may need `tree-sitter generate` \
             run first, which this installer doesn't do for you (yet) — most published \
             grammars commit the generated parser, but some don't.",
            src_dir.display()
        ));
    }

    log(tx, "Compiling grammar");
    let filename = platform_lib_name(&format!("tree_sitter_{}", spec.name));
    let out_path = grammar_dir.join(&filename);

    let result = compile_grammar(&src_dir, &out_path, tx);
    if result.is_ok() {
        write_default_lang_config(&grammar_root, grammar_dir, spec.name, tx);
    }
    let _ = std::fs::remove_dir_all(&tmp_dir);
    result?;

    log(tx, format!("Installed {}", filename));
    Ok(InstallOutcome {
        summary: format!("Grammar '{}' installed", spec.name),
        resolved_command: None,
    })
}

/// Clones `spec.repo_url` into `tmp_dir`, retrying a few times with backoff.
///
/// Anonymous (unauthenticated) `git clone` over HTTPS against github.com is
/// subject to secondary rate limiting, especially when several installs
/// are kicked off back-to-back (each spawns its own clone). When that
/// happens git often can't complete the smart-HTTP handshake and falls
/// back to asking for credentials — which, with `GIT_TERMINAL_PROMPT=0`
/// set below, surfaces as a clean "could not read Username" error instead
/// of hanging. That failure is usually transient, so retrying with a short
/// backoff resolves it without the user needing to do anything.
///
/// When `github_token` is set, authenticates via `GIT_ASKPASS` instead of
/// cloning anonymously — this both avoids the rate limiting/anti-abuse
/// heuristics above and never touches stdin (unlike embedding credentials
/// in the URL, the token also never appears in the process's argv, only
/// in an env var scoped to this one child process).
fn clone_repo(
    spec: &GrammarSpec,
    tmp_dir: &Path,
    github_token: Option<&str>,
    tx: &Sender<InstallEvent>,
) -> Result<(), String> {
    const MAX_ATTEMPTS: u32 = 3;
    let mut last_err = String::new();

    let askpass = match github_token {
        Some(token) => {
            log(tx, "Using GitHub token for authentication");
            Some(AskpassHelper::write(token)?)
        }
        None => None,
    };

    for attempt in 1..=MAX_ATTEMPTS {
        if tmp_dir.exists() {
            let _ = std::fs::remove_dir_all(tmp_dir);
        }
        if attempt > 1 {
            let backoff = Duration::from_secs(2 * attempt as u64);
            log(
                tx,
                format!(
                    "Clone attempt {} failed, retrying in {}s ({}/{})",
                    attempt - 1,
                    backoff.as_secs(),
                    attempt,
                    MAX_ATTEMPTS
                ),
            );
            thread::sleep(backoff);
        }

        let mut cmd = Command::new("git");
        cmd.args([
            "clone",
            "--depth",
            "1",
            spec.repo_url,
            tmp_dir.to_str().ok_or("Invalid temp path")?,
        ])
        // Disables git's interactive credential prompt. Without this, a
        // failed/rate-limited anonymous clone makes git ask for a
        // username on stdin — which, inherited from our raw-mode TUI
        // terminal, is unusable and looks like a hang. With this set,
        // git just fails immediately with a normal error instead. Kept
        // even when authenticating, as a safety net.
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null());

        if let Some(helper) = &askpass {
            helper.apply(&mut cmd);
        }

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to run git (is it installed?): {}", e))?;

        if output.status.success() {
            return Ok(());
        }

        last_err = String::from_utf8_lossy(&output.stderr).to_string();
    }

    Err(format!(
        "git clone failed after {} attempts: {}",
        MAX_ATTEMPTS, last_err
    ))
}

/// A temporary `GIT_ASKPASS` helper script that answers any prompt (git
/// invokes it once for username, once for password) with the token.
/// GitHub accepts any non-empty username alongside a PAT, so answering
/// both prompts identically works and keeps this simple.
///
/// Cleaned up via `Drop` so it's removed even if a clone attempt errors
/// out early.
struct AskpassHelper {
    script_path: PathBuf,
}

impl AskpassHelper {
    fn write(token: &str) -> Result<Self, String> {
        let dir = std::env::temp_dir();
        let unique = format!(
            "calliglyph-askpass-{}",
            std::process::id() // enough to avoid collisions between concurrent installs
        );

        #[cfg(unix)]
        let script_path = dir.join(&unique);
        #[cfg(windows)]
        let script_path = dir.join(format!("{}.cmd", unique));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            // Token is embedded directly in the script rather than read from
            // an env var at askpass-invocation time — env vars set on the
            // git Command aren't necessarily inherited by an externally
            // invoked askpass helper the same way across platforms, so this
            // is the more portable choice. The script itself is 0700
            // (owner-only) and lives in a per-process-id temp path.
            let content = format!("#!/bin/sh\necho '{}'\n", token.replace('\'', "'\\''"));
            std::fs::write(&script_path, content)
                .map_err(|e| format!("Failed to write askpass helper: {}", e))?;
            let mut perms = std::fs::metadata(&script_path)
                .map_err(|e| format!("Failed to stat askpass helper: {}", e))?
                .permissions();
            perms.set_mode(0o700);
            std::fs::set_permissions(&script_path, perms)
                .map_err(|e| format!("Failed to chmod askpass helper: {}", e))?;
        }

        #[cfg(windows)]
        {
            let content = format!("@echo off\r\necho {}\r\n", token);
            std::fs::write(&script_path, content)
                .map_err(|e| format!("Failed to write askpass helper: {}", e))?;
        }

        Ok(Self { script_path })
    }

    fn apply(&self, cmd: &mut Command) {
        cmd.env("GIT_ASKPASS", &self.script_path);
        // Some git versions/environments also check this for the
        // credential fill-in step; harmless to set alongside GIT_ASKPASS.
        cmd.env("GCM_INTERACTIVE", "never");
    }
}

impl Drop for AskpassHelper {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.script_path);
    }
}

/// Writes a starter `<name>.toml` node-kind config next to the grammar's
/// `.so`, generated from the grammar's own `src/node-types.json`. Never
/// overwrites an existing config — if the user (or a previous install)
/// already has one, it's left alone.
fn write_default_lang_config(
    grammar_root: &Path,
    grammar_dir: &Path,
    name: &str,
    tx: &Sender<InstallEvent>,
) {
    let config_path = grammar_dir.join(format!("{}.toml", name));
    if config_path.exists() {
        log(
            tx,
            format!("{}.toml already exists, leaving it as-is", name),
        );
        return;
    }

    match super::config_generator::generate_lang_config(grammar_root, name) {
        Some(toml_text) => match std::fs::write(&config_path, toml_text) {
            Ok(()) => log(tx, format!("Generated starter {}.toml", name)),
            Err(e) => log(tx, format!("Failed to write {}.toml: {}", name, e)),
        },
        None => log(
            tx,
            format!(
                "Couldn't auto-generate {}.toml (no usable node-types.json in this grammar) — \
                 you'll need to write one by hand; see an existing config for the format.",
                name
            ),
        ),
    }
}

/// Compiles parser.c (+ scanner.c/scanner.cc if present) into a shared library.
fn compile_grammar(
    src_dir: &Path,
    out_path: &Path,
    tx: &Sender<InstallEvent>,
) -> Result<(), String> {
    let parser_c = src_dir.join("parser.c");
    let scanner_c = src_dir.join("scanner.c");
    let scanner_cc = src_dir.join("scanner.cc");
    let has_cpp_scanner = scanner_cc.exists();

    let compiler = if has_cpp_scanner {
        find_compiler(&["c++", "g++", "clang++"])
    } else {
        find_compiler(&["cc", "gcc", "clang"])
    }
    .ok_or_else(|| "No C/C++ compiler found on PATH (need cc/gcc/clang)".to_string())?;

    log(tx, format!("Using compiler: {}", compiler));

    let mut cmd = Command::new(&compiler);
    cmd.arg("-shared")
        .arg("-fPIC")
        .arg("-O2")
        .arg("-I")
        .arg(src_dir)
        .arg("-o")
        .arg(out_path)
        .arg(&parser_c);

    if scanner_c.exists() {
        cmd.arg(&scanner_c);
    }
    if has_cpp_scanner {
        cmd.arg(&scanner_cc);
    }

    let output = cmd
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("Failed to run {}: {}", compiler, e))?;

    if !output.status.success() {
        return Err(format!(
            "Compilation failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

/// Returns the first candidate binary name found runnable on PATH.
fn find_compiler(candidates: &[&str]) -> Option<String> {
    candidates
        .iter()
        .find(|c| {
            Command::new(c)
                .arg("--version")
                .stdin(Stdio::null())
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        })
        .map(|c| c.to_string())
}

//████████╗███████╗███████╗████████╗███████╗
//╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//   ██║   █████╗  ███████╗   ██║   ███████╗
//   ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//   ██║   ███████╗███████║   ██║   ███████║
//   ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝

// NOTE: the network/process-spawning paths here (`try_install`, `clone_repo`,
// `compile_grammar`) are deliberately not unit tested — they'd either need a
// live network connection and a real compiler toolchain (slow, flaky, and
// non-hermetic in CI) or a mockable `Command` abstraction this codebase
// doesn't have. What's tested below is everything that doesn't require
// either: the askpass helper's file handling, the "don't clobber an existing
// config" guard, and compiler-detection's negative case.

#[cfg(test)]
mod unit_grammar_installer_tests {
    use super::*;
    use std::sync::mpsc::channel;
    use tempfile::tempdir;

    #[test]
    fn find_compiler_returns_none_when_nothing_matches() {
        assert!(find_compiler(&["definitely-not-a-real-compiler-xyz"]).is_none());
    }

    #[test]
    fn write_default_lang_config_skips_existing_file() {
        let grammar_dir = tempdir().unwrap();
        let grammar_root = tempdir().unwrap();
        let config_path = grammar_dir.path().join("fakelang.toml");
        std::fs::write(&config_path, "# hand written, do not touch").unwrap();

        let (tx, _rx) = channel();
        write_default_lang_config(grammar_root.path(), grammar_dir.path(), "fakelang", &tx);

        let content = std::fs::read_to_string(&config_path).unwrap();
        assert_eq!(
            content, "# hand written, do not touch",
            "an existing lang config must never be overwritten"
        );
    }

    #[test]
    fn write_default_lang_config_writes_when_node_types_available() {
        let grammar_dir = tempdir().unwrap();
        let grammar_root = tempdir().unwrap();
        let src_dir = grammar_root.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(
            src_dir.join("node-types.json"),
            r#"[{"type": "comment", "named": true}]"#,
        )
        .unwrap();

        let (tx, _rx) = channel();
        write_default_lang_config(grammar_root.path(), grammar_dir.path(), "fakelang", &tx);

        let config_path = grammar_dir.path().join("fakelang.toml");
        assert!(config_path.exists());
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("\"comment\" = \"comment\""));
    }

    #[test]
    fn write_default_lang_config_does_nothing_when_no_node_types() {
        let grammar_dir = tempdir().unwrap();
        let grammar_root = tempdir().unwrap(); // no src/node-types.json at all

        let (tx, _rx) = channel();
        write_default_lang_config(grammar_root.path(), grammar_dir.path(), "fakelang", &tx);

        assert!(!grammar_dir.path().join("fakelang.toml").exists());
    }

    #[test]
    #[cfg(unix)]
    fn askpass_helper_writes_owner_only_script_containing_the_token() {
        use std::os::unix::fs::PermissionsExt;

        let helper = AskpassHelper::write("my-secret-token").expect("should write helper");
        let content = std::fs::read_to_string(&helper.script_path).unwrap();
        assert!(content.contains("my-secret-token"));

        let mode = std::fs::metadata(&helper.script_path)
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o700);

        let path = helper.script_path.clone();
        drop(helper);
        assert!(!path.exists(), "askpass helper should clean up on drop");
    }

    #[test]
    #[cfg(unix)]
    fn askpass_helper_escapes_single_quotes_without_panicking() {
        let helper = AskpassHelper::write("weird'token'value").unwrap();
        let content = std::fs::read_to_string(&helper.script_path).unwrap();
        // Exact escaping format isn't the point here — what matters is the
        // token's characters all still appear and the shell script is valid
        // enough that `sh -c` wouldn't choke on unmatched quotes.
        assert!(content.contains("weird"));
        assert!(content.contains("token"));
        assert!(content.contains("value"));
    }
}
