use crate::language::syntax::SyntaxTree;
use crate::language::theme::Theme;
use ratatui::style::Style;
use std::path::Path;

pub struct LanguageManager {
    pub syntax: Option<SyntaxTree>,
    pub theme: Option<Theme>,
    // pub lsp: Option<crate::language::lsp::LspClient>,
    pub language_id: Option<String>,
}

impl LanguageManager {
    pub fn new() -> Self {
        Self {
            syntax: None,
            theme: None,
            /*lsp: None,*/
            language_id: None,
        }
    }

    /// Actives the tree-sitter according to file type.
    /// Called when a file is opened or language config changes.
    /// TODO use FALLBACK THEME if none provided
    pub fn activate_for_file(&mut self, path: &Path, theme: Option<Theme>) {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let (lang, lang_id) = match ext {
            "rs" => (
                Some(tree_sitter::Language::from(tree_sitter_rust::LANGUAGE)),
                Some("rust"),
            ),
            "py" => (
                Some(tree_sitter::Language::from(tree_sitter_python::LANGUAGE)),
                Some("python"),
            ),
            "js" | "ts" => (
                Some(tree_sitter::Language::from(
                    tree_sitter_javascript::LANGUAGE,
                )),
                Some("javascript"),
            ),
            _ => (None, None),
        };

        self.language_id = lang_id.map(String::from);
        self.syntax = lang.map(SyntaxTree::new);
        self.theme = theme;
    }

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
    pub fn highlighted_lines(&self, lines: &[String]) -> Vec<Vec<(std::ops::Range<usize>, Style)>> {
        // If no syntax or theme, return empty vecs so callers get no spans
        let Some(syntax) = &self.syntax else {
            return vec![vec![]; lines.len()];
        };
        let Some(theme) = &self.theme else {
            return vec![vec![]; lines.len()];
        };

        // Walk the tree ONCE
        let all_tokens = syntax.highlight_tokens();

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
}

// ────────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_manager(source: &str) -> LanguageManager {
        let mut mgr = LanguageManager::new();
        // Activate for a .rs file with no theme (we test styling separately)
        mgr.activate_for_file(Path::new("test.rs"), None);
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
        mgr.activate_for_file(Path::new("test.rs"), Some(theme));
        mgr.update_source(source);
        mgr
    }

    // ── activate_for_file ─────────────────────────────────────────────────

    #[test]
    fn activate_sets_language_id_for_rust() {
        let mut mgr = LanguageManager::new();
        mgr.activate_for_file(Path::new("main.rs"), None);
        assert_eq!(mgr.language_id.as_deref(), Some("rust"));
    }

    #[test]
    fn activate_sets_language_id_for_python() {
        let mut mgr = LanguageManager::new();
        mgr.activate_for_file(Path::new("script.py"), None);
        assert_eq!(mgr.language_id.as_deref(), Some("python"));
    }

    #[test]
    fn activate_unknown_extension_clears_syntax() {
        let mut mgr = LanguageManager::new();
        mgr.activate_for_file(Path::new("file.xyz"), None);
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
        mgr.activate_for_file(Path::new("a.rs"), None);
        mgr.update_source("let x = 1;");
        assert_eq!(mgr.syntax.as_ref().unwrap().source, "let x = 1;");
    }

    // ── highlighted_lines — no theme ──────────────────────────────────────

    #[test]
    fn no_theme_returns_empty_spans_per_line() {
        let mgr = make_manager("fn main() {}");
        let lines = vec!["fn main() {}".to_string()];
        let result = mgr.highlighted_lines(&lines);
        assert_eq!(result.len(), 1);
        assert!(result[0].is_empty(), "no theme means no spans");
    }

    // ── highlighted_lines — with theme ────────────────────────────────────

    #[test]
    fn single_line_keyword_produces_span() {
        let mgr = make_manager_with_theme("fn main() {}");
        let lines = vec!["fn main() {}".to_string()];
        let result = mgr.highlighted_lines(&lines);
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
        let mgr = make_manager_with_theme(source);
        let result = mgr.highlighted_lines(&lines);

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
        let mgr = make_manager_with_theme(source);
        let result = mgr.highlighted_lines(&lines);

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
        let mgr = make_manager_with_theme(source);
        let result = mgr.highlighted_lines(&lines);

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
        let mgr = make_manager_with_theme("");
        let lines = vec!["".to_string()];
        let result = mgr.highlighted_lines(&lines);
        assert_eq!(result.len(), 1);
        assert!(result[0].is_empty());
    }

    #[test]
    fn result_length_matches_line_count() {
        let source = "fn a() {}\nfn b() {}\nfn c() {}";
        let lines: Vec<String> = source.lines().map(String::from).collect();
        let mgr = make_manager_with_theme(source);
        let result = mgr.highlighted_lines(&lines);
        assert_eq!(result.len(), lines.len());
    }

    #[test]
    fn local_byte_offsets_are_within_line_bounds() {
        let source = "pub fn main() {\n    let x = 42;\n    // done\n}";
        let lines: Vec<String> = source.lines().map(String::from).collect();
        let mgr = make_manager_with_theme(source);
        let result = mgr.highlighted_lines(&lines);

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
        let mgr = make_manager_with_theme(source);
        let result = mgr.highlighted_lines(&lines);

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
}
