//! The data model behind the built-in file browser: a lazily-expanded
//! directory tree. Directories don't read their children until first
//! expanded, so opening a large project doesn't walk the whole tree up
//! front, only what the user actually looks at.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FileNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub expanded: bool,
    /// `None` = not yet loaded. Only meaningful when `is_dir` is true.
    pub children: Option<Vec<FileNode>>,
}

impl FileNode {
    /// Builds the root node for `path`, with its immediate children
    /// already loaded (so the browser isn't empty on first render) and
    /// expanded.
    pub fn new_root(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        let children = Some(load_children(&path));
        Self {
            path,
            name,
            is_dir: true,
            expanded: true,
            children,
        }
    }

    /// Toggles a directory open/closed, lazily loading its children the
    /// first time it's expanded. No-op on files.
    pub fn toggle_expand(&mut self) {
        if !self.is_dir {
            return;
        }
        if self.expanded {
            self.expanded = false;
        } else {
            self.expanded = true;
            if self.children.is_none() {
                self.children = Some(load_children(&self.path));
            }
        }
    }

    /// Finds the node at `target_path` anywhere in this subtree (including
    /// `self`), by mutable reference.
    pub fn find_mut(&mut self, target_path: &Path) -> Option<&mut FileNode> {
        if self.path == target_path {
            return Some(self);
        }
        if let Some(children) = &mut self.children {
            for child in children {
                if let Some(found) = child.find_mut(target_path) {
                    return Some(found);
                }
            }
        }
        None
    }

    /// Flattens the currently-visible rows (this node, plus children of
    /// every expanded descendant) into `out`, depth-first, in display
    /// order. Collapsed directories' children are skipped entirely —
    /// that's what makes a node "not visible".
    pub fn flatten_visible(&self, depth: usize, out: &mut Vec<VisibleRow>) {
        out.push(VisibleRow {
            path: self.path.clone(),
            name: self.name.clone(),
            depth,
            is_dir: self.is_dir,
            expanded: self.expanded,
        });
        if self.is_dir && self.expanded {
            if let Some(children) = &self.children {
                for child in children {
                    child.flatten_visible(depth + 1, out);
                }
            }
        }
    }
}

/// One row as the browser displays it, owned/cloned rather than a
/// reference, so the browser's render and input-handling code don't have
/// to fight the borrow checker over holding a flattened view alongside a
/// `&mut` tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleRow {
    pub path: PathBuf,
    pub name: String,
    pub depth: usize,
    pub is_dir: bool,
    pub expanded: bool,
}

/// Reads one directory level: directories first, then files, both
/// alphabetical (case-insensitive) within their group. Hidden entries
/// (dotfiles) are included, TODO maybe filter those out based on toggle, not default behavior.
///
/// Returns an empty `Vec` (rather than erroring) on an unreadable
/// directory, permission errors, races with something deleting the dir,
/// etc. shouldn't crash the browser, just show nothing under that node.
pub fn load_children(dir: &Path) -> Vec<FileNode> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return vec![];
    };

    let mut nodes: Vec<FileNode> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            Some(FileNode {
                path,
                name,
                is_dir,
                expanded: false,
                children: None,
            })
        })
        .collect();

    nodes.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    nodes
}

#[cfg(test)]
mod unit_tree_tests {
    use super::*;
    use tempfile::tempdir;

    fn make_tree(dir: &Path) {
        std::fs::create_dir(dir.join("b_dir")).unwrap();
        std::fs::create_dir(dir.join("a_dir")).unwrap();
        std::fs::write(dir.join("z_file.txt"), "").unwrap();
        std::fs::write(dir.join("a_file.txt"), "").unwrap();
        std::fs::write(dir.join("a_dir").join("nested.txt"), "").unwrap();
    }

    #[test]
    fn load_children_sorts_directories_before_files() {
        let dir = tempdir().unwrap();
        make_tree(dir.path());

        let children = load_children(dir.path());
        let dir_positions: Vec<usize> = children
            .iter()
            .enumerate()
            .filter(|(_, n)| n.is_dir)
            .map(|(i, _)| i)
            .collect();
        let file_positions: Vec<usize> = children
            .iter()
            .enumerate()
            .filter(|(_, n)| !n.is_dir)
            .map(|(i, _)| i)
            .collect();

        assert!(dir_positions.iter().max().unwrap() < file_positions.iter().min().unwrap());
    }

    #[test]
    fn load_children_sorts_alphabetically_within_each_group() {
        let dir = tempdir().unwrap();
        make_tree(dir.path());

        let children = load_children(dir.path());
        let dir_names: Vec<&str> = children
            .iter()
            .filter(|n| n.is_dir)
            .map(|n| n.name.as_str())
            .collect();
        let file_names: Vec<&str> = children
            .iter()
            .filter(|n| !n.is_dir)
            .map(|n| n.name.as_str())
            .collect();

        assert_eq!(dir_names, vec!["a_dir", "b_dir"]);
        assert_eq!(file_names, vec!["a_file.txt", "z_file.txt"]);
    }

    #[test]
    fn load_children_on_unreadable_path_returns_empty_not_panic() {
        let children = load_children(Path::new("/definitely/does/not/exist"));
        assert!(children.is_empty());
    }

    #[test]
    fn new_root_has_children_already_loaded_and_is_expanded() {
        let dir = tempdir().unwrap();
        make_tree(dir.path());

        let root = FileNode::new_root(dir.path().to_path_buf());
        assert!(root.expanded);
        assert!(root.children.is_some());
        assert_eq!(root.children.unwrap().len(), 4);
    }

    #[test]
    fn toggle_expand_on_file_is_a_no_op() {
        let mut file_node = FileNode {
            path: PathBuf::from("/tmp/some_file.txt"),
            name: "some_file.txt".to_string(),
            is_dir: false,
            expanded: false,
            children: None,
        };
        file_node.toggle_expand();
        assert!(!file_node.expanded);
        assert!(file_node.children.is_none());
    }

    #[test]
    fn toggle_expand_lazily_loads_children_only_on_first_expand() {
        let dir = tempdir().unwrap();
        make_tree(dir.path());

        let mut node = FileNode {
            path: dir.path().join("a_dir"),
            name: "a_dir".to_string(),
            is_dir: true,
            expanded: false,
            children: None,
        };
        assert!(node.children.is_none());

        node.toggle_expand();
        assert!(node.expanded);
        assert!(node.children.is_some());
        assert_eq!(node.children.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn toggle_expand_twice_collapses_without_discarding_loaded_children() {
        let dir = tempdir().unwrap();
        make_tree(dir.path());

        let mut node = FileNode {
            path: dir.path().join("a_dir"),
            name: "a_dir".to_string(),
            is_dir: true,
            expanded: false,
            children: None,
        };
        node.toggle_expand(); // expand, loads children
        node.toggle_expand(); // collapse

        assert!(!node.expanded);
        assert!(
            node.children.is_some(),
            "children should stay cached, not be dropped"
        );
    }

    #[test]
    fn find_mut_locates_root() {
        let dir = tempdir().unwrap();
        make_tree(dir.path());
        let mut root = FileNode::new_root(dir.path().to_path_buf());
        let found = root.find_mut(dir.path());
        assert!(found.is_some());
    }

    #[test]
    fn find_mut_locates_nested_child_and_allows_mutation() {
        let dir = tempdir().unwrap();
        make_tree(dir.path());
        let mut root = FileNode::new_root(dir.path().to_path_buf());

        let target = dir.path().join("a_dir");
        let found = root.find_mut(&target).expect("should find a_dir");
        found.toggle_expand();

        // Confirm the mutation actually landed on the real tree, not a copy
        let refetched = root.find_mut(&target).unwrap();
        assert!(refetched.expanded);
    }

    #[test]
    fn find_mut_returns_none_for_unknown_path() {
        let dir = tempdir().unwrap();
        make_tree(dir.path());
        let mut root = FileNode::new_root(dir.path().to_path_buf());
        assert!(root.find_mut(Path::new("/not/in/this/tree")).is_none());
    }

    #[test]
    fn flatten_visible_includes_root_but_not_unexpanded_children() {
        let dir = tempdir().unwrap();
        make_tree(dir.path());
        let mut root = FileNode::new_root(dir.path().to_path_buf());
        root.expanded = false; // collapse root itself

        let mut rows = vec![];
        root.flatten_visible(0, &mut rows);
        assert_eq!(rows.len(), 1); // just the root row
        assert_eq!(rows[0].path, dir.path());
    }

    #[test]
    fn flatten_visible_includes_expanded_but_not_collapsed_subdirs() {
        let dir = tempdir().unwrap();
        make_tree(dir.path());
        let mut root = FileNode::new_root(dir.path().to_path_buf());

        // root expanded (default) but a_dir is not -> a_dir's child should be absent
        let mut rows = vec![];
        root.flatten_visible(0, &mut rows);
        // root + a_dir + b_dir + a_file.txt + z_file.txt = 5, nested.txt NOT included
        assert_eq!(rows.len(), 5);
        assert!(!rows.iter().any(|r| r.name == "nested.txt"));
    }

    #[test]
    fn flatten_visible_includes_grandchildren_once_subdir_expanded() {
        let dir = tempdir().unwrap();
        make_tree(dir.path());
        let mut root = FileNode::new_root(dir.path().to_path_buf());

        let a_dir_path = dir.path().join("a_dir");
        root.find_mut(&a_dir_path).unwrap().toggle_expand();

        let mut rows = vec![];
        root.flatten_visible(0, &mut rows);
        assert!(rows.iter().any(|r| r.name == "nested.txt"));
    }

    #[test]
    fn flatten_visible_depth_increases_with_nesting() {
        let dir = tempdir().unwrap();
        make_tree(dir.path());
        let mut root = FileNode::new_root(dir.path().to_path_buf());
        let a_dir_path = dir.path().join("a_dir");
        root.find_mut(&a_dir_path).unwrap().toggle_expand();

        let mut rows = vec![];
        root.flatten_visible(0, &mut rows);

        let root_row = rows.iter().find(|r| r.path == dir.path()).unwrap();
        let a_dir_row = rows.iter().find(|r| r.name == "a_dir").unwrap();
        let nested_row = rows.iter().find(|r| r.name == "nested.txt").unwrap();

        assert_eq!(root_row.depth, 0);
        assert_eq!(a_dir_row.depth, 1);
        assert_eq!(nested_row.depth, 2);
    }

    #[test]
    fn new_root_on_empty_directory_has_zero_children() {
        let dir = tempdir().unwrap();
        let root = FileNode::new_root(dir.path().to_path_buf());
        assert_eq!(root.children.as_ref().unwrap().len(), 0);

        let mut rows = vec![];
        root.flatten_visible(0, &mut rows);
        assert_eq!(rows.len(), 1); // just the root row, no children
    }
}
