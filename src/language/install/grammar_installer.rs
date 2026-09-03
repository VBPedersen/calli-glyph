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
pub fn install_grammar(spec: GrammarSpec, grammar_dir: PathBuf, tx: Sender<InstallEvent>) {
    let result = try_install(&spec, &grammar_dir, &tx);
    let _ = tx.send(InstallEvent::Done(result));
}

fn log(tx: &Sender<InstallEvent>, msg: impl Into<String>) {
    let _ = tx.send(InstallEvent::Log(msg.into()));
}

fn try_install(
    spec: &GrammarSpec,
    grammar_dir: &Path,
    tx: &Sender<InstallEvent>,
) -> Result<InstallOutcome, String> {
    std::fs::create_dir_all(grammar_dir)
        .map_err(|e| format!("Failed to create grammar dir: {}", e))?;

    let tmp_dir = std::env::temp_dir().join(format!("calliglyph-grammar-{}", spec.name));
    if tmp_dir.exists() {
        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    log(tx, format!("Cloning {}", spec.repo_url));
    clone_repo(spec, &tmp_dir, tx)?;

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
fn clone_repo(spec: &GrammarSpec, tmp_dir: &Path, tx: &Sender<InstallEvent>) -> Result<(), String> {
    const MAX_ATTEMPTS: u32 = 3;
    let mut last_err = String::new();

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

        let output = Command::new("git")
            .args([
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
            // git just fails immediately with a normal error instead.
            .env("GIT_TERMINAL_PROMPT", "0")
            .stdin(Stdio::null())
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
        log(tx, format!("{}.toml already exists, leaving it as-is", name));
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
