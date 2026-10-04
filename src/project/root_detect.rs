//! Project root auto-detection: walks up from a starting file or
//! directory looking for a `.git`, `.hg`, or `.svn` marker: the exact
//! same algorithm (and marker list) documented for LSP workspace-root
//! detection, so "which folder is my project root" means the same thing
//! everywhere in the editor, not one thing for diagnostics and another
//! for the file browser.
//!
//! Deliberately a pure function with no `App`/`Config` dependency: given
//! a path, it either finds a marker or falls back to the starting
//! directory. Nothing here needs to know about the rest of the editor,
//! which is what makes it cheap to unit test exhaustively.

use std::path::{Path, PathBuf};

const MARKERS: &[&str] = &[".git", ".hg", ".svn"];

/// Finds the project root for `start`, which may be a file or a
/// directory (or not exist at all: e.g. a buffer that's never been
/// saved). Walks up from `start`'s directory looking for a marker.
/// Returns the first ancestor that has one. If none is found, returns
/// `start`'s own directory. If `start` has no usable directory at all
/// (e.g. a bare filename with no parent, or doesn't exist and isn't
/// absolute), falls back to the current working directory.
pub fn find_project_root(start: &Path) -> PathBuf {
    let begin_dir = starting_directory(start);

    let mut current = Some(begin_dir.clone());
    while let Some(dir) = current {
        if MARKERS.iter().any(|marker| dir.join(marker).exists()) {
            return dir;
        }
        current = dir.parent().map(Path::to_path_buf);
    }

    begin_dir
}

/// Resolves the directory to start walking from: `start` itself if it's
/// already a directory, otherwise its parent, otherwise cwd.
fn starting_directory(start: &Path) -> PathBuf {
    if start.is_dir() {
        return start.to_path_buf();
    }
    if let Some(parent) = start.parent() {
        if !parent.as_os_str().is_empty() {
            return parent.to_path_buf();
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

#[cfg(test)]
mod unit_root_detect_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn finds_git_marker_in_starting_directory_itself() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();

        assert_eq!(find_project_root(dir.path()), dir.path());
    }

    #[test]
    fn finds_git_marker_several_levels_up() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        let nested = dir.path().join("src").join("core").join("deep");
        std::fs::create_dir_all(&nested).unwrap();

        assert_eq!(find_project_root(&nested), dir.path());
    }

    #[test]
    fn finds_hg_marker() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".hg")).unwrap();
        assert_eq!(find_project_root(dir.path()), dir.path());
    }

    #[test]
    fn finds_svn_marker() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".svn")).unwrap();
        assert_eq!(find_project_root(dir.path()), dir.path());
    }

    #[test]
    fn starting_from_a_file_walks_up_from_its_directory_not_the_file_itself() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        let file = dir.path().join("main.rs");
        std::fs::write(&file, "").unwrap();

        assert_eq!(find_project_root(&file), dir.path());
    }

    #[test]
    fn no_marker_anywhere_falls_back_to_starting_directory() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("a").join("b");
        std::fs::create_dir_all(&nested).unwrap();

        // No .git/.hg/.svn anywhere up this tempdir's chain (assuming the
        // OS temp root itself isn't inside a repo, which it never is in
        // practice) — should fall back to the starting dir unchanged.
        assert_eq!(find_project_root(&nested), nested);
    }

    #[test]
    fn nearest_marker_wins_over_a_more_distant_one() {
        // A repo nested inside another repo (e.g. a git submodule checked
        // out without proper submodule config, or just two unrelated
        // repos one inside the other) — the closer one should win.
        let outer = tempdir().unwrap();
        std::fs::create_dir(outer.path().join(".git")).unwrap();
        let inner_repo = outer.path().join("vendored-project");
        std::fs::create_dir_all(&inner_repo).unwrap();
        std::fs::create_dir(inner_repo.join(".git")).unwrap();
        let deep = inner_repo.join("src");
        std::fs::create_dir(&deep).unwrap();

        assert_eq!(find_project_root(&deep), inner_repo);
    }

    #[test]
    fn nonexistent_file_path_falls_back_to_its_would_be_parent() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        // A path that doesn't exist on disk yet (e.g. "save as" into a new
        // file within a real repo) should still resolve via its parent.
        let not_yet_created = dir.path().join("new_file.rs");

        assert_eq!(find_project_root(&not_yet_created), dir.path());
    }

    #[test]
    fn starting_directory_for_a_plain_directory_is_itself() {
        let dir = tempdir().unwrap();
        assert_eq!(starting_directory(dir.path()), dir.path());
    }

    #[test]
    fn starting_directory_for_a_file_is_its_parent() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("foo.txt");
        std::fs::write(&file, "").unwrap();
        assert_eq!(starting_directory(&file), dir.path());
    }

    #[test]
    fn starting_directory_for_bare_relative_filename_falls_back_to_cwd() {
        // "untitled" (no directory component at all) — Path::parent()
        // returns Some("") in this case, which starting_directory should
        // treat as "no usable parent" and fall back to cwd, not silently
        // resolve to an empty path.
        let result = starting_directory(Path::new("untitled"));
        assert!(!result.as_os_str().is_empty());
    }

    #[test]
    fn repeated_calls_are_idempotent() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        let first = find_project_root(dir.path());
        let second = find_project_root(dir.path());
        assert_eq!(first, second);
    }
}
