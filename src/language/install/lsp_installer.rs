//! Installs LSP servers via whichever package manager is available on the
//! system (npm/pip/cargo/go), falling back to manual instructions when
//! there's no reliable automatable path (e.g. rust-analyzer).
//!
//! Important: after a package manager install, the binary often lands
//! somewhere that isn't on *our* process's `PATH` (it was captured when the
//! editor launched) even though it'll be on PATH in a fresh shell — npm's
//! global prefix, `~/.local/bin` for pip --user, `$GOPATH/bin` for go
//! install, etc. So instead of trusting bare-name PATH lookups after
//! install, we actively resolve the absolute path via the package
//! manager's own introspection and hand that back so it can be written
//! into config — independent of whatever PATH this process happens to have.
//!
//! Windows note: `npm` (and anything installed via npm/nvm) is a `.cmd`
//! shim, not a `.exe`. `Command::new("npm")` calls `CreateProcess`
//! directly, which — unlike PowerShell or `cmd.exe` — does not consult
//! `PATHEXT` or know how to run `.cmd`/`.bat` files, so it fails with
//! "program not found" even when `npm` works fine when you type it
//! yourself. `run_command` below routes through `cmd /C` on Windows so
//! PATHEXT resolution happens the same way it does in the shell; `which`
//! is likewise PATHEXT-aware when searching directories directly.

use super::job::{InstallEvent, InstallOutcome};
use super::registry::{LspSpec, PackageManager};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;

/// Entry point run on the background thread spawned by `InstallJob::spawn`.
pub fn install_lsp(spec: LspSpec, tx: Sender<InstallEvent>) {
    let result = try_install(&spec, &tx);
    let _ = tx.send(InstallEvent::Done(result));
}

fn log(tx: &Sender<InstallEvent>, msg: impl Into<String>) {
    let _ = tx.send(InstallEvent::Log(msg.into()));
}

fn try_install(spec: &LspSpec, tx: &Sender<InstallEvent>) -> Result<InstallOutcome, String> {
    // Already resolvable? Nothing to install.
    if let Some(path) = which(spec.command) {
        log(tx, format!("'{}' already on PATH", spec.command));
        return Ok(InstallOutcome {
            summary: format!("'{}' found on PATH", spec.command),
            resolved_command: Some(path.to_string_lossy().to_string()),
        });
    }

    let Some(pm) = spec.package_manager else {
        return Err(format!(
            "No automatic installer available for '{}'.\n{}",
            spec.name, spec.manual_instructions
        ));
    };

    if which(pm.binary()).is_none() {
        return Err(format!(
            "'{}' not found on PATH, needed to install '{}'.\n{}",
            pm.binary(),
            spec.name,
            spec.manual_instructions
        ));
    }

    let packages: Vec<&str> = spec.package.split_whitespace().collect();
    log(
        tx,
        format!(
            "Installing {} via {} ({})",
            spec.name,
            pm.binary(),
            spec.package
        ),
    );

    let args = pm.install_args(&packages);
    let output = run_command(pm.binary())
        .args(&args)
        .stdin(Stdio::null()) // never let an installer's prompt hijack the raw-mode terminal
        .output()
        .map_err(|e| format!("Failed to run {}: {}", pm.binary(), e))?;

    if !output.status.success() {
        return Err(format!(
            "{} install failed:\n{}\n\nManual instructions: {}",
            pm.binary(),
            String::from_utf8_lossy(&output.stderr),
            spec.manual_instructions
        ));
    }

    log(tx, "Install command succeeded, locating binary");

    // Bare PATH lookup first (covers cases where PATH already includes it)
    if let Some(path) = which(spec.command) {
        return Ok(InstallOutcome {
            summary: format!("'{}' installed successfully", spec.name),
            resolved_command: Some(path.to_string_lossy().to_string()),
        });
    }

    // Otherwise ask the package manager where it puts things.
    for dir in extra_bin_dirs(pm) {
        if let Some(found) = find_in_dir(&dir, spec.command) {
            log(tx, format!("Found binary at {}", found.display()));
            return Ok(InstallOutcome {
                summary: format!("'{}' installed successfully", spec.name),
                resolved_command: Some(found.to_string_lossy().to_string()),
            });
        }
    }

    Err(format!(
        "Install command succeeded but couldn't locate '{}' afterwards. \
         It may be installed somewhere non-standard.\n\nManual instructions: {}",
        spec.command, spec.manual_instructions
    ))
}

/// Builds a `Command` for running an external program in a way that
/// resolves the same as it would in the user's own shell.
///
/// On Windows this routes through `cmd /C`, because `Command::new` alone
/// calls `CreateProcess` directly and — unlike `cmd.exe`/PowerShell —
/// doesn't search `PATHEXT` or know how to execute `.cmd`/`.bat` shims
/// (which is exactly what `npm`, and anything it installs, usually is).
/// On other platforms this is just `Command::new(program)`.
fn run_command(program: &str) -> Command {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("cmd");
        cmd.arg("/C").arg(program);
        cmd
    }
    #[cfg(not(windows))]
    {
        Command::new(program)
    }
}

/// Manual PATH search — deliberately not relying on `Command::new(...).status()`
/// success/failure, since that only tells us whether *this process's* PATH
/// resolved it, not the absolute location we need to persist to config.
fn which(binary: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        if let Some(found) = find_in_dir(&dir, binary) {
            return Some(found);
        }
    }
    None
}

/// Looks for `binary` directly inside `dir`, trying every extension
/// Windows would consider executable (respecting `PATHEXT` if set) as well
/// as the bare name (covers Unix-style shim scripts with no extension,
/// which some Windows installers — e.g. nvm-windows — also drop alongside
/// the `.cmd`).
fn find_in_dir(dir: &Path, binary: &str) -> Option<PathBuf> {
    for filename in candidate_filenames(binary) {
        let candidate = dir.join(filename);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn candidate_filenames(name: &str) -> Vec<String> {
    #[cfg(windows)]
    {
        let pathext =
            std::env::var("PATHEXT").unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD;.PS1".to_string());
        let mut names: Vec<String> = pathext
            .split(';')
            .filter(|e| !e.is_empty())
            .map(|ext| format!("{}{}", name, ext.to_lowercase()))
            .collect();
        names.push(name.to_string()); // bare name, for Unix-style shims
        names
    }
    #[cfg(not(windows))]
    {
        vec![name.to_string()]
    }
}

/// Where each package manager tends to put global/user install targets,
/// beyond whatever this process's PATH already covers.
fn extra_bin_dirs(pm: PackageManager) -> Vec<PathBuf> {
    match pm {
        PackageManager::Npm => {
            let Ok(output) = run_command("npm")
                .args(["config", "get", "prefix"])
                .stdin(Stdio::null())
                .output()
            else {
                return vec![];
            };
            if !output.status.success() {
                return vec![];
            }
            let prefix = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if prefix.is_empty() {
                return vec![];
            }
            let prefix = PathBuf::from(prefix);
            #[cfg(windows)]
            {
                // On Windows npm drops shims directly in the prefix root,
                // not a "bin" subdirectory.
                vec![prefix]
            }
            #[cfg(not(windows))]
            {
                vec![prefix.join("bin")]
            }
        }
        PackageManager::Pip => {
            for py in ["python3", "python"] {
                let Ok(output) = run_command(py)
                    .args(["-c", "import site; print(site.USER_BASE)"])
                    .stdin(Stdio::null())
                    .output()
                else {
                    continue;
                };
                if !output.status.success() {
                    continue;
                }
                let base = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if base.is_empty() {
                    continue;
                }
                let base = PathBuf::from(base);
                #[cfg(windows)]
                {
                    return vec![base.join("Scripts")];
                }
                #[cfg(not(windows))]
                {
                    return vec![base.join("bin")];
                }
            }
            vec![]
        }
        PackageManager::Cargo => {
            let cargo_home = std::env::var_os("CARGO_HOME")
                .map(PathBuf::from)
                .or_else(|| dirs::home_dir().map(|h| h.join(".cargo")));
            cargo_home.map(|c| vec![c.join("bin")]).unwrap_or_default()
        }
        PackageManager::Go => {
            if let Ok(output) = run_command("go")
                .args(["env", "GOBIN"])
                .stdin(Stdio::null())
                .output()
            {
                if output.status.success() {
                    let gobin = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !gobin.is_empty() {
                        return vec![PathBuf::from(gobin)];
                    }
                }
            }
            if let Ok(output) = run_command("go")
                .args(["env", "GOPATH"])
                .stdin(Stdio::null())
                .output()
            {
                if output.status.success() {
                    let gopath = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !gopath.is_empty() {
                        return vec![PathBuf::from(gopath).join("bin")];
                    }
                }
            }
            vec![]
        }
    }
}

//████████╗███████╗███████╗████████╗███████╗
//╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//   ██║   █████╗  ███████╗   ██║   ███████╗
//   ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//   ██║   ███████╗███████║   ██║   ███████║
//   ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝

// NOTE: `try_install` itself (which actually spawns package managers) isn't
// unit tested here for the same reason as grammar_installer's network path —
// it needs real npm/pip/cargo/go installs to exercise meaningfully, which
// belongs in a manual/CI-with-network-and-toolchains test, not a fast unit
// test. What's covered below is the PATH-resolution logic, which is the part
// that was actually broken (twice) before this file existed in its current
// form.

#[cfg(test)]
mod unit_lsp_installer_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    #[cfg(not(windows))]
    fn candidate_filenames_is_just_the_bare_name_on_unix() {
        assert_eq!(candidate_filenames("npm"), vec!["npm".to_string()]);
    }

    #[test]
    #[cfg(windows)]
    fn candidate_filenames_includes_common_windows_extensions() {
        let names = candidate_filenames("npm");
        assert!(names.iter().any(|n| n.eq_ignore_ascii_case("npm.cmd")));
        assert!(names.iter().any(|n| n.eq_ignore_ascii_case("npm.exe")));
        assert!(names.contains(&"npm".to_string()));
    }

    #[test]
    fn find_in_dir_locates_a_matching_file() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join(test_bin_filename("mytool")), "").unwrap();
        assert!(find_in_dir(dir.path(), "mytool").is_some());
    }

    #[test]
    fn find_in_dir_returns_none_when_absent() {
        let dir = tempdir().unwrap();
        assert!(find_in_dir(dir.path(), "nonexistent-tool-xyz").is_none());
    }

    #[test]
    #[cfg(windows)]
    fn find_in_dir_locates_cmd_shims_not_just_exe() {
        // This is the exact bug that motivated this file: npm ships as a
        // .cmd shim on Windows, and a naive "<name>.exe" check never finds it.
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("npm.cmd"), "").unwrap();
        assert!(find_in_dir(dir.path(), "npm").is_some());
    }

    // `which()` reads the process-wide PATH env var, so mutating it in a
    // test is inherently a little dangerous under cargo's default parallel
    // test execution — this lock only protects against other tests in this
    // same file doing the same thing, not unrelated tests elsewhere in the
    // crate that happen to spawn a process concurrently. Kept deliberately
    // narrow in scope (set → check → restore) to minimize the window.
    static PATH_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn which_finds_a_binary_placed_on_a_custom_path() {
        let _guard = PATH_TEST_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join(test_bin_filename("uniquetool")), "").unwrap();

        let prev = std::env::var_os("PATH");
        std::env::set_var("PATH", dir.path());
        let found = which("uniquetool");
        match prev {
            Some(p) => std::env::set_var("PATH", p),
            None => std::env::remove_var("PATH"),
        }

        assert!(found.is_some());
    }

    #[cfg(not(windows))]
    fn test_bin_filename(name: &str) -> String {
        name.to_string()
    }
    #[cfg(windows)]
    fn test_bin_filename(name: &str) -> String {
        format!("{}.exe", name)
    }
}
