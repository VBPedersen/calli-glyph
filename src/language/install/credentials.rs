//! Storage for an optional GitHub Personal Access Token, used to
//! authenticate grammar-repo `git clone`s.
//!
//! Deliberately kept in its own file rather than `config.toml`: it's a
//! secret, not a setting, so it shouldn't get swept up in config
//! backups/dotfile-syncing the way the rest of config.toml might be. On
//! Unix it's written with `0600` permissions (owner read/write only).
//!
//! Note: this is plaintext-on-disk storage, same trust model as e.g. a
//! `.netrc` file or `~/.config/gh/hosts.yml`. Recommend a fine-grained PAT
//! scoped to public repo read access, ideally with an expiry — it only
//! exists to raise git's anonymous-clone rate limit, not for anything
//! requiring elevated access.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn token_file_path(config_dir: &Path) -> PathBuf {
    config_dir.join("github_token")
}

/// Returns the stored token, if any. Missing file is not an error — it
/// just means no token has been configured.
pub fn load_github_token(config_dir: &Path) -> Option<String> {
    let path = token_file_path(config_dir);
    let content = fs::read_to_string(path).ok()?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Writes (or clears, if `token` is `None`) the stored token.
pub fn save_github_token(config_dir: &Path, token: Option<&str>) -> io::Result<()> {
    let path = token_file_path(config_dir);
    match token {
        Some(t) if !t.trim().is_empty() => {
            fs::create_dir_all(config_dir)?;
            fs::write(&path, t.trim())?;
            restrict_permissions(&path)?;
        }
        _ => {
            if path.exists() {
                fs::remove_file(&path)?;
            }
        }
    }
    Ok(())
}

#[cfg(unix)]
fn restrict_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(path, perms)
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) -> io::Result<()> {
    // Windows ACLs aren't handled here — the file lives under the user's
    // config directory, which is already scoped to the current user by
    // default on most setups. Contributions welcome.
    Ok(())
}
