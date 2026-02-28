use crate::errors::error::AppError;
use crate::ui::popups::help_popup::HelpPopup;
use std::fs;
use std::path::{Path, PathBuf};

/// Struct for holding data on single help page
pub struct HelpPage {
    pub id: String,        // string id of page like "editor" or "debugger"
    pub title: String,     // title of page
    pub content: String,   // raw markdown content
    pub tags: Vec<String>, // for searching
    pub summary: String,   // one liner for listing
}

impl HelpPage {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        content: impl Into<String>,
        tags: Vec<&str>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            tags: tags.into_iter().map(|s| s.to_lowercase()).collect(),
            summary: summary.into(),
        }
    }

    /// Parse a HelpPage from raw markdown file content.
    /// Expects a front matter (context) block at the top:
    ///
    /// ```markdown
    /// ---
    /// id: debug
    /// title: Developer Console
    /// summary: Real-time internal state inspector
    /// tags: debugger, console, logs, :debug
    /// ---
    ///
    /// # Developer Console
    /// ...rest of the doc...
    /// ```
    ///
    /// Returns None if the file has no valid front matter or is missing the required `id` field.
    fn from_str(raw: &str) -> Option<Self> {
        let raw = raw.trim_start();

        // must start with "---" to identify context block
        if !raw.starts_with("---") {
            return None;
        }

        // Find context block by start and end of context block
        let after_opening = raw.get(3..)?.trim_start_matches("\n");
        let closing_pos = after_opening.find("\n---")?;
        let front_matter = &after_opening[..closing_pos];

        // Content starts after the closing --- and optional newline
        let content = after_opening
            .get(closing_pos + 4..) // skip past \n---
            .unwrap_or("")
            .trim_start_matches('\n')
            .to_string();

        // Parse front matter key: value lines
        let mut id = None;
        let mut title = None;
        let mut summary = None;
        let mut tags: Vec<String> = Vec::new();

        for line in front_matter.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();
                match key {
                    "id" => id = Some(value.to_string()),
                    "title" => title = Some(value.to_string()),
                    "summary" => summary = Some(value.to_string()),
                    "tags" => {
                        tags = value
                            .split(',')
                            .map(|t| t.trim().to_lowercase())
                            .filter(|t| !t.is_empty())
                            .collect();
                    }
                    _ => {} // Unknown keys ignored
                }
            }
        }

        Some(Self {
            id: id?, // id is required. if none, page is skipped
            title: title.unwrap_or_else(|| "Untitled".to_string()),
            summary: summary.unwrap_or_default(),
            tags,
            content,
        })
    }

    /// Returns true if query matches page
    /// matches against every property of page except content (CaSe-iNsenSItiVE)
    pub fn matches(&self, query: &str) -> bool {
        if query.is_empty() {
            return true; // true since empty is always match, as function is used to filter
        }

        let q = query.to_lowercase();
        self.id.to_lowercase().contains(&q)
            || self.title.to_lowercase().contains(&q)
            || self.tags.iter().any(|s| s.contains(&q))
            || self.summary.to_lowercase().contains(&q)
    }
}

/// Registry for holding all help pages
pub struct HelpRegistry {
    pages: Vec<HelpPage>,
}

impl HelpRegistry {
    /// Function to build registry by loading help docs
    /// Automatically discover and load all `.md` markdown files from `docs_dir`.
    /// Each file must have a valid front matter (context) block — files without one are
    /// skipped with a warning
    ///
    /// Usage:
    /// ```rust
    /// use std::sync::Arc;
    /// use calliglyph::core::help_registry::HelpRegistry;
    /// let registry = HelpRegistry::load_from(HelpRegistry::default_docs_path())
    ///     .unwrap_or_else(|e| {
    ///         eprintln!("Warning: {e}");
    ///         HelpRegistry::empty()
    ///     });
    /// let registry = Arc::new(registry);
    /// ```
    pub fn load_from(docs_dir: impl AsRef<Path>) -> Result<Self, AppError> {
        let dir = docs_dir.as_ref();

        if !dir.exists() || !dir.is_dir() {
            return Err(AppError::InternalError(format!(
                "{} {:?}",
                "Path not found for help pages : {}".to_string(),
                dir.to_path_buf()
            )));
        }

        let mut pages: Vec<HelpPage> = fs::read_dir(dir)
            .map_err(|e| {
                AppError::InternalError(format!("{} {:?}", "directory not found :".to_string(), e))
            })?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .path() // filter for markdown files only
                    .extension()
                    .map(|extension| extension == "md")
                    .unwrap_or(false)
            })
            .filter_map(|entry| {
                // try to read and parse to help page
                let path = entry.path();
                let raw = fs::read_to_string(&path)
                    .map_err(|e| log_warn!("[Help] could not read file {}: {}", path.display(), e))
                    .ok()?;

                HelpPage::from_str(&raw).or_else(|| {
                    log_warn!("[Help] could not parse {}", path.display());
                    None
                })
            })
            .collect();

        // Sort alphabetically on title
        pages.sort_by(|a, b| a.title.cmp(&b.title));
        Ok(Self { pages })
    }

    /// Function to get default path of docs.
    /// Resolves the "docs/help" path for both dev (cargo run) and installed builds.
    pub fn default_docs_path() -> PathBuf {
        // Try relative to the executable (works for installed/release builds).
        if let Ok(executable) = std::env::current_exe() {
            let temp = executable
                .parent() // target/release
                .and_then(|p| p.parent()) // target
                .and_then(|p| p.parent()) // root
                .map(|root| root.join("docs/help")); // docs help folder

            if let Some(path) = temp {
                if path.exists() {
                    return path;
                }
            }
        }

        // Fallback (for cargo run not installed builds)
        PathBuf::from("docs/help")
    }

    /// Empty the registry
    pub fn empty() -> Self {
        Self { pages: Vec::new() }
    }

    /// Function to find single help page by id
    pub fn find_by_id(&self, query: &str) -> Option<&HelpPage> {
        todo!()
    }

    /// Function to find all matching help pages
    pub fn search(&self, query: &str) -> Vec<&HelpPage> {
        todo!()
    }

    /// Function to get all help pages in registry as array
    pub fn get_all(&self) -> &[HelpPage] {
        &self.pages
    }

    /// Get page at index
    pub fn get(&self, index: usize) -> Option<&HelpPage> {
        self.pages.get(index)
    }

    /// Get length of pages Vec = amount of help pages
    pub fn len(&self) -> usize {
        self.pages.len()
    }

    /// Returns true if no help pages, False if any
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"---
id: debug
title: Developer Console
summary: Real-time internal state inspector
tags: debugger, console, :debug
---

# Developer Console

Some content here.
"#;

    const NO_FRONT_MATTER: &str = "# Just a markdown file with no front matter";

    const MISSING_ID: &str = "---
title: Oops
summary: No id field
---
content
";

    #[test]
    fn parses_valid_page() {
        let page = HelpPage::from_str(VALID).unwrap();
        assert_eq!(page.id, "debug");
        assert_eq!(page.title, "Developer Console");
        assert_eq!(page.summary, "Real-time internal state inspector");
        assert_eq!(page.tags, vec!["debugger", "console", ":debug"]);
        assert!(page.content.contains("# Developer Console"));
    }

    #[test]
    fn rejects_no_front_matter() {
        assert!(HelpPage::from_str(NO_FRONT_MATTER).is_none());
    }

    #[test]
    fn rejects_missing_id() {
        assert!(HelpPage::from_str(MISSING_ID).is_none());
    }

    #[test]
    fn search_matches_tag() {
        let page = HelpPage::from_str(VALID).unwrap();
        assert!(page.matches(":debug"));
        assert!(page.matches("console"));
        assert!(!page.matches("banana"));
    }

    #[test]
    fn empty_query_matches_all() {
        let page = HelpPage::from_str(VALID).unwrap();
        assert!(page.matches(""));
    }
}
