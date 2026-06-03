use crate::config::syntax::SyntaxConfig;
use crate::config::{Config, LspConfig};
use crate::language::lsp::{
    CompletionItem, ConnectionState, Diagnostic, HoverResult, LspClient, LspMessage,
};
use crate::language::syntax::SyntaxTree;
use crate::language::theme::Theme;
use crate::language::{grammar_loader, lang_configs, lsp};
use ratatui::style::Style;
use std::fs;
use std::path::{Path, PathBuf};

pub struct LanguageManager {
    pub syntax: Option<SyntaxTree>,
    pub theme: Option<Theme>,
    pub language_id: Option<String>,
    pub lsp: Option<LspClient>,
    pub current_uri: Option<String>,
    pub diagnostics: Vec<Diagnostic>,
    pub completions: Vec<CompletionItem>,
    pub hover: Option<HoverResult>,
}

impl LanguageManager {
    pub fn new() -> Self {
        Self {
            syntax: None,
            theme: None,
            language_id: None,
            lsp: None,
            current_uri: None,
            diagnostics: Vec::new(),
            completions: Vec::new(),
            hover: None,
        }
    }

    /// Actives the tree-sitter according to file type.
    /// Called when a file is opened or language config changes.
    /// TODO use FALLBACK THEME if none provided
    pub fn activate_for_file(
        &mut self,
        path: &Path,
        theme: Option<Theme>,
        lsp_config: &LspConfig,
        syntax_config: &SyntaxConfig,
    ) {
        // Resolve relative paths against CWD before canonicalizing,
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir().unwrap_or_default().join(path)
        };
        let canonical = absolute.canonicalize().unwrap_or(absolute);

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        self.language_id = None;
        self.syntax = None;
        self.theme = theme;

        if syntax_config.enabled {
            if let Some((lang_name, lang_cfg)) = syntax_config.language_for_extension(ext) {
                // Always set language_id even when recognized
                self.language_id = Some(lang_name.to_string());

                let grammar_dir = resolve_grammar_dir(&syntax_config.grammar_dir);
                ensure_default_grammar_configs(&grammar_dir);
                let grammar = grammar_loader::load_grammar(&grammar_dir, &lang_cfg.grammar);
                let lang_config =
                    lang_configs::loader::load_lang_config(&grammar_dir, &lang_cfg.grammar);

                match (grammar, lang_config) {
                    (Ok(lang), Ok(cfg)) => {
                        self.syntax = Some(SyntaxTree::new(lang, Some(cfg)));
                        log_info!(
                            "[Syntax] Loaded grammar '{}' for .{}",
                            lang_cfg.grammar,
                            ext
                        );
                    }
                    (Err(e), _) | (_, Err(e)) => {
                        log_warn!("[Syntax] Failed to load grammar for .{}: {}", ext, e);
                    }
                }
            }
        }

        log_info!(
            "Language manager activated, with language : {:?}",
            self.language_id
        );

        // Reset LSP state
        self.lsp = None;
        self.diagnostics.clear();
        self.completions.clear();
        self.hover = None;

        let uri = lsp::path_to_uri(&canonical);
        self.current_uri = Some(uri.clone());

        // Start LSP server if configured for this extension
        if lsp_config.enabled {
            if let Some((server_name, server_cfg)) = lsp_config.server_for_extension(ext) {
                let markers: Vec<&str> =
                    server_cfg.root_markers.iter().map(|s| s.as_str()).collect();
                let workspace_root = find_project_root(&canonical, &markers)
                    .or_else(|| canonical.parent().map(|p| p.to_path_buf()))
                    .unwrap_or_else(|| std::env::current_dir().unwrap_or(PathBuf::from(".")));
                log_info!(
                    "Attempting to start LSP client with workspace root: {:?}",
                    workspace_root
                );

                match LspClient::start(server_name.to_string(), server_cfg, workspace_root) {
                    Ok(client) => {
                        self.lsp = Some(client);
                        log_info!(
                            "[LSP] Started '{}' for .{}, with command name to run: {}",
                            server_name,
                            ext,
                            server_cfg.command
                        );
                    }
                    Err(e) => {
                        log_warn!("[LSP] Failed to start '{}': {}", server_name, e);
                    }
                }
            }
        }
    }

    /// Clears the language system, including diagnostics, completions and hover results
    pub fn clear(&mut self) {}

    /// Restarts the language system, including lsp, syntax and theming
    pub fn restart(&mut self) {}

    // -------------------
    // Tree-sitter
    // -------------------

    /// Feed the current buffer contents to the parser.
    /// Called whenever the buffer changes (after every edit action).
    pub fn update_source(&mut self, source: &str) {
        if let Some(syntax) = &mut self.syntax {
            // Pass None for edit, full re-parse. Fast enough for standard
            // file sizes, TODO maybe add incremental edits later.
            syntax.update(source, None);
        }
    }

    /// Compute all highlight tokens for the current source in one pass,
    /// then split them into per-line (local-byte-offset, Style) pairs.
    ///
    /// Returns a Vec with one entry per buffer line. Each entry is a Vec of
    /// (local_byte_range, Style) pairs that cover that line.
    ///
    /// Called once per render frame, not per line
    pub fn highlighted_lines(
        &mut self,
        lines: &[String],
        content_version: u64,
    ) -> Vec<Vec<(std::ops::Range<usize>, Style)>> {
        // If no syntax or theme, return empty vecs so callers get no spans
        let Some(syntax) = &mut self.syntax else {
            return vec![vec![]; lines.len()];
        };
        let Some(theme) = &self.theme else {
            return vec![vec![]; lines.len()];
        };

        // Walk the tree ONCE
        let all_tokens = syntax.highlight_tokens(content_version);

        // Build a table mapping line_index to byte_start_of_that_line
        // so as, we can do O(1) lookups when partitioning tokens.
        let mut line_starts: Vec<usize> = Vec::with_capacity(lines.len());
        let mut offset = 0usize;
        for line in lines {
            line_starts.push(offset);
            offset += line.len() + 1; // +1 for the '\n' joining them
        }

        // Allocate one bucket per line
        let mut result: Vec<Vec<(std::ops::Range<usize>, Style)>> = vec![vec![]; lines.len()];

        for (range, token_type) in all_tokens {
            // Find which line this token starts on using binary search
            let line_idx = line_starts
                .partition_point(|&start| start <= range.start)
                .saturating_sub(1);

            if line_idx >= lines.len() {
                continue;
            }

            let line_byte_start = line_starts[line_idx];
            let line_byte_end = line_byte_start + lines[line_idx].len();

            // Clamp the token range to this line's byte bounds
            let local_start = range.start.saturating_sub(line_byte_start);
            let local_end = (range.end - line_byte_start).min(lines[line_idx].len());

            // Skip tokens that are outside this line
            if local_start >= lines[line_idx].len() || local_end == 0 || local_start >= local_end {
                continue;
            }

            result[line_idx].push((local_start..local_end, theme.style_for(token_type)));
        }

        result
    }

    // -------------------
    // LSP
    // -------------------

    /// Notify the server of a buffer change. Call after every edit.
    pub fn notify_change(&mut self, full_text: &str) {
        if let (Some(lsp), Some(uri)) = (&mut self.lsp, &self.current_uri) {
            if lsp.state == ConnectionState::Ready {
                let uri = uri.clone();
                if let Err(e) = lsp.notify_did_change(&uri, full_text) {
                    log_warn!("[LSP] didChange failed: {}", e);
                }
            }
        }
    }

    /// Notify the server the file was saved.
    pub fn notify_save(&mut self) {
        if let (Some(lsp), Some(uri)) = (&mut self.lsp, &self.current_uri) {
            if lsp.state == ConnectionState::Ready {
                let uri = uri.clone();
                if let Err(e) = lsp.notify_did_save(&uri) {
                    log_warn!("[LSP] didSave failed: {}", e);
                }
            }
        }
    }

    /// Request completion items at cursor (0-indexed line/character).
    pub fn request_completion(&mut self, line: u32, character: u32) {
        if let (Some(lsp), Some(uri)) = (&mut self.lsp, &self.current_uri) {
            let uri = uri.clone();
            if let Err(e) = lsp.request_completion(&uri, line, character) {
                log_warn!("[LSP] completion request failed: {}", e);
            }
        }
    }

    /// Request hover docs at cursor position.
    pub fn request_hover(&mut self, line: u32, character: u32) {
        if let (Some(lsp), Some(uri)) = (&mut self.lsp, &self.current_uri) {
            let uri = uri.clone();
            if let Err(e) = lsp.request_hover(&uri, line, character) {
                log_warn!("[LSP] hover request failed: {}", e);
            }
        }
    }

    /// Drain LSP messages. Call once per tick. Returns events for App to act on.
    pub fn poll_lsp(&mut self) -> Vec<LspMessage> {
        let Some(lsp) = &mut self.lsp else {
            return vec![];
        };
        let events = lsp.poll();

        for event in &events {
            match event {
                LspMessage::Diagnostics { uri, diagnostics } => {
                    // Normalise both URIs to lowercase for case-insensitive comparison
                    let normalised_incoming = uri.to_lowercase();
                    let normalised_current =
                        self.current_uri.as_deref().unwrap_or("").to_lowercase();
                    if normalised_incoming == normalised_current {
                        self.diagnostics = diagnostics.clone();
                    }
                }
                LspMessage::CompletionResponse { items, .. } => {
                    self.completions = items.clone();
                }
                LspMessage::HoverResponse { result, .. } => {
                    self.hover = result.clone();
                }
                LspMessage::ServerExited => {
                    log_warn!("[LSP] Server exited unexpectedly");
                    self.lsp = None;
                }
                LspMessage::Error { message, .. } => {
                    log_warn!("[LSP] Server error: {}", message);
                }
                LspMessage::Initialized => {
                    log_info!("[LSP] Server ready");
                }
            }
        }
        // log_trace!("LSP events: {:?}", events);
        events
    }

    /// Diagnostics for a specific buffer line (0-indexed).
    pub fn diagnostics_on_line(&self, line: u32) -> Vec<&Diagnostic> {
        self.diagnostics.iter().filter(|d| d.line == line).collect()
    }

    pub fn lsp_ready(&self) -> bool {
        self.lsp
            .as_ref()
            .map(|l| l.state == ConnectionState::Ready)
            .unwrap_or(false)
    }

    pub fn lsp_status(&self) -> &str {
        match &self.lsp {
            None => "",
            Some(l) => match &l.state {
                ConnectionState::Disconnected => "LSP: off",
                ConnectionState::Initializing => "LSP: starting…",
                ConnectionState::Ready => "LSP: ready",
                ConnectionState::Failed(_) => "LSP: error",
            },
        }
    }
}

// -------------------
// Helpers
// -------------------
pub fn find_project_root(start_path: &Path, root_markers: &[&str]) -> Option<PathBuf> {
    log_info!(
        "[LANGUAGE MANAGER] Finding project root starting at {:?}",
        start_path
    );
    let mut current = start_path.to_path_buf();

    // Iterate through parent directories
    while current.pop() {
        // Check if any of the markers exist in the current directory
        if root_markers
            .iter()
            .any(|marker| current.join(marker).exists())
        {
            log_info!("Found project root: {:?}", current);
            return Some(current);
        }
    }

    log_warn!("[LANGUAGE MANAGER] Could not find project root";"Subsystems might not work as intended, including LSP");
    // No Fallback here just return of NONE
    None
}

/// Select either inputted configured grammar directory or from config, and return selected
pub fn resolve_grammar_dir(configured: &Option<String>) -> PathBuf {
    if let Some(dir) = configured {
        let expanded = dir.replace('~', &dirs::home_dir().unwrap_or_default().to_string_lossy());
        return PathBuf::from(expanded);
    }
    // Default: <config_dir>/calliglyph/grammars/
    Config::get_grammar_dir().unwrap_or_else(|| PathBuf::from("."))
}

/// Ensures that at least defualt grammar configs are present, if not then install from assets
pub fn ensure_default_grammar_configs(grammar_dir: &Path) {
    if let Err(e) = fs::create_dir_all(grammar_dir) {
        log_warn!("[Syntax] Could not create grammar dir: {}", e);
        return;
    }

    let defaults: &[(&str, &str)] = &[
        ("rust.toml", include_str!("../../assets/grammars/rust.toml")),
        (
            "python.toml",
            include_str!("../../assets/grammars/python.toml"),
        ),
        (
            "typescript.toml",
            include_str!("../../assets/grammars/typescript.toml"),
        ),
    ];

    for (filename, content) in defaults {
        let path = grammar_dir.join(filename);
        if !path.exists() {
            // never overwrite user edits
            if let Err(e) = fs::write(&path, content) {
                log_warn!("[Syntax] Could not write default {}: {}", filename, e);
            } else {
                log_info!("[Syntax] Wrote default grammar config: {}", path.display());
            }
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn make_manager(source: &str) -> LanguageManager {
        let mut mgr = LanguageManager::new();
        // Activate for a .rs file with no theme (we test styling separately)
        mgr.activate_for_file(
            Path::new("test.rs"),
            None,
            &LspConfig::default(),
            &SyntaxConfig::default(),
        );
        mgr.update_source(source);
        mgr
    }

    fn make_manager_with_theme(source: &str) -> LanguageManager {
        let theme = Theme {
            name: "test".into(),
            defaults: crate::language::theme::ThemeColor {
                fg: Some("white".into()),
                bold: None,
                italic: None,
            },
            tokens: {
                let mut m = std::collections::HashMap::new();
                m.insert(
                    "keyword".into(),
                    crate::language::theme::ThemeColor {
                        fg: Some("#ff0000".into()),
                        bold: Some(true),
                        italic: None,
                    },
                );
                m.insert(
                    "string".into(),
                    crate::language::theme::ThemeColor {
                        fg: Some("#00ff00".into()),
                        bold: None,
                        italic: None,
                    },
                );
                m.insert(
                    "comment".into(),
                    crate::language::theme::ThemeColor {
                        fg: Some("#888888".into()),
                        bold: None,
                        italic: Some(true),
                    },
                );
                m.insert(
                    "number".into(),
                    crate::language::theme::ThemeColor {
                        fg: Some("#0000ff".into()),
                        bold: None,
                        italic: None,
                    },
                );
                m
            },
        };
        let mut mgr = LanguageManager::new();
        mgr.activate_for_file(
            Path::new("test.rs"),
            Some(theme),
            &LspConfig::default(),
            &SyntaxConfig::default(),
        );
        mgr.update_source(source);
        mgr
    }

    // ── activate_for_file ─────────────────────────────────────────────────

    #[test]
    fn activate_sets_language_id_for_rust() {
        let mut mgr = LanguageManager::new();
        mgr.activate_for_file(
            Path::new("main.rs"),
            None,
            &LspConfig::default(),
            &SyntaxConfig::default(),
        );
        assert_eq!(mgr.language_id.as_deref(), Some("rust"));
    }

    #[test]
    fn activate_sets_language_id_for_python() {
        let mut mgr = LanguageManager::new();
        mgr.activate_for_file(
            Path::new("script.py"),
            None,
            &LspConfig::default(),
            &SyntaxConfig::default(),
        );
        assert_eq!(mgr.language_id.as_deref(), Some("python"));
    }

    #[test]
    fn activate_unknown_extension_clears_syntax() {
        let mut mgr = LanguageManager::new();
        mgr.activate_for_file(
            Path::new("file.xyz"),
            None,
            &LspConfig::default(),
            &SyntaxConfig::default(),
        );
        assert!(mgr.syntax.is_none());
        assert!(mgr.language_id.is_none());
    }

    // ── update_source ─────────────────────────────────────────────────────

    #[test]
    fn update_source_without_activate_does_not_panic() {
        let mut mgr = LanguageManager::new(); // no activate_for_file
        mgr.update_source("fn main() {}"); // syntax is None — should be a no-op
    }

    #[test]
    fn update_source_stores_source_in_syntax() {
        let mut mgr = LanguageManager::new();
        mgr.activate_for_file(
            Path::new("a.rs"),
            None,
            &LspConfig::default(),
            &SyntaxConfig::default(),
        );
        mgr.update_source("let x = 1;");
        assert_eq!(mgr.syntax.as_ref().unwrap().source, "let x = 1;");
    }

    // ── highlighted_lines — no theme ──────────────────────────────────────

    #[test]
    fn no_theme_returns_empty_spans_per_line() {
        let mut mgr = make_manager("fn main() {}");
        let lines = vec!["fn main() {}".to_string()];
        let result = mgr.highlighted_lines(&lines, 0);
        assert_eq!(result.len(), 1);
        assert!(result[0].is_empty(), "no theme means no spans");
    }

    // ── highlighted_lines — with theme ────────────────────────────────────

    #[test]
    fn single_line_keyword_produces_span() {
        let mut mgr = make_manager_with_theme("fn main() {}");
        let lines = vec!["fn main() {}".to_string()];
        let result = mgr.highlighted_lines(&lines, 0);
        assert_eq!(result.len(), 1);
        assert!(
            !result[0].is_empty(),
            "should have at least one span for 'fn'"
        );
        // The 'fn' keyword occupies bytes 0..2 on this line
        let has_fn_span = result[0].iter().any(|(r, _)| r.start == 0 && r.end == 2);
        assert!(
            has_fn_span,
            "expected span covering bytes 0..2 for 'fn', got: {:?}",
            result[0]
        );
    }

    #[test]
    fn string_token_produces_span_on_correct_line() {
        let source = "fn main() {\n    let s = \"hello\";\n}";
        let lines: Vec<String> = source.lines().map(String::from).collect();
        let mut mgr = make_manager_with_theme(source);
        let result = mgr.highlighted_lines(&lines, 0);

        // String is on line index 1
        assert!(
            result[1].iter().any(|(_, _)| true), // at least something on line 1
            "line 1 should have spans"
        );
        // Confirm string span exists on line 1
        let string_line = &result[1];
        // The span should be non-empty
        assert!(
            !string_line.is_empty(),
            "expected highlight spans on line 1"
        );
    }

    #[test]
    fn comment_on_line_produces_span() {
        let source = "// top comment\nlet x = 1;";
        let lines: Vec<String> = source.lines().map(String::from).collect();
        let mut mgr = make_manager_with_theme(source);
        let result = mgr.highlighted_lines(&lines, 0);

        assert_eq!(result.len(), 2);
        assert!(
            !result[0].is_empty(),
            "comment line should have a span, got: {:?}",
            result[0]
        );
    }

    #[test]
    fn spans_do_not_bleed_across_lines() {
        let source = "fn foo() {}\nlet x = 1;";
        let lines: Vec<String> = source.lines().map(String::from).collect();
        let mut mgr = make_manager_with_theme(source);
        let result = mgr.highlighted_lines(&lines, 0);

        // All spans on line 0 must have local byte offsets within line 0's length
        let line0_len = lines[0].len();
        for (range, _) in &result[0] {
            assert!(
                range.end <= line0_len,
                "span {:?} bleeds past end of line 0 (len {})",
                range,
                line0_len
            );
        }
        // All spans on line 1 must have local byte offsets within line 1's length
        let line1_len = lines[1].len();
        for (range, _) in &result[1] {
            assert!(
                range.end <= line1_len,
                "span {:?} bleeds past end of line 1 (len {})",
                range,
                line1_len
            );
        }
    }

    #[test]
    fn empty_source_returns_one_empty_line_bucket() {
        let mut mgr = make_manager_with_theme("");
        let lines = vec!["".to_string()];
        let result = mgr.highlighted_lines(&lines, 0);
        assert_eq!(result.len(), 1);
        assert!(result[0].is_empty());
    }

    #[test]
    fn result_length_matches_line_count() {
        let source = "fn a() {}\nfn b() {}\nfn c() {}";
        let lines: Vec<String> = source.lines().map(String::from).collect();
        let mut mgr = make_manager_with_theme(source);
        let result = mgr.highlighted_lines(&lines, 0);
        assert_eq!(result.len(), lines.len());
    }

    #[test]
    fn local_byte_offsets_are_within_line_bounds() {
        let source = "pub fn main() {\n    let x = 42;\n    // done\n}";
        let lines: Vec<String> = source.lines().map(String::from).collect();
        let mut mgr = make_manager_with_theme(source);
        let result = mgr.highlighted_lines(&lines, 0);

        for (line_idx, spans) in result.iter().enumerate() {
            let line_len = lines[line_idx].len();
            for (range, _) in spans {
                assert!(
                    range.start <= line_len && range.end <= line_len,
                    "line {}: span {:?} out of bounds (line len {})",
                    line_idx,
                    range,
                    line_len
                );
                assert!(
                    range.start < range.end,
                    "line {}: span {:?} is zero or negative width",
                    line_idx,
                    range
                );
            }
        }
    }

    #[test]
    fn number_on_second_line_lands_in_correct_bucket() {
        let source = "let a = 0;\nlet b = 99;";
        let lines: Vec<String> = source.lines().map(String::from).collect();
        let mut mgr = make_manager_with_theme(source);
        let result = mgr.highlighted_lines(&lines, 0);

        // '99' is only on line 1
        let has_number_line1 = result[1].iter().any(|(_, _)| true);
        assert!(has_number_line1, "line 1 should have spans (number 99)");
        // Line 1 content: "let b = 99;"  — '99' is at bytes 8..10
        let number_span = result[1].iter().find(|(r, _)| {
            let text = &lines[1][r.clone()];
            text == "99"
        });
        assert!(
            number_span.is_some(),
            "should find span exactly covering '99' on line 1"
        );
    }

    #[test]
    fn test_find_project_root_with_git() {
        let dir = tempdir().unwrap();
        let project = dir.path().join("my_project");
        let src = project.join("src");
        fs::create_dir_all(&src).unwrap();

        // Create a dummy .git folder
        fs::create_dir(project.join(".git")).unwrap();

        let file_path = src.join("main.rs");
        let root = find_project_root(&file_path, &[".git"]).expect("Should find a root");

        // Canonicalize both to ensure identical formatting (fixes Windows prefix issues)
        let expected = project
            .canonicalize()
            .expect("Failed to canonicalize project path");
        let actual = root
            .canonicalize()
            .expect("Failed to canonicalize found root");

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_find_project_root_returns_none() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("standalone.rs");

        // No .git folder exists
        let root = find_project_root(&file_path, &[".git"]);

        // Should return none
        assert!(root.is_none());
    }

    #[test]
    fn test_find_project_root_logic() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("project");
        let src = root.join("src");
        fs::create_dir_all(&src).unwrap();

        // Create agnostic marker (.git)
        fs::create_dir(root.join(".git")).unwrap();

        let file = src.join("main.rs");
        let discovered = find_project_root(&file, &[".git"]).unwrap();

        // Canonicalize to handle Windows path prefix variations (\\?\ vs C:\)
        assert_eq!(
            discovered.canonicalize().unwrap(),
            root.canonicalize().unwrap()
        );
    }
}
