//! the tree-sitter wrapper

use std::collections::{HashMap, HashSet};
use std::ops::Range;
use tree_sitter::{Language, Node, Parser, Tree};

pub struct SyntaxTree {
    parser: Parser,
    tree: Option<Tree>,
    /// The last source string fed to the parser.
    /// Stored so highlight_tokens can resolve byte ranges back to text.
    pub source: String,
    config: Option<LangConfig>, // None = no highlighting rules loaded
    // Cache: last walk result + the content_version it was built for
    token_cache: Option<(u64, Vec<(Range<usize>, String)>)>,
}

impl SyntaxTree {
    /// Creates a new syntax tree based on selected language
    pub fn new(language: Language, config: Option<LangConfig>) -> SyntaxTree {
        let mut parser = Parser::new();
        parser.set_language(&language).unwrap();
        Self {
            parser,
            tree: None,
            source: String::new(),
            config,
            token_cache: None,
        }
    }

    /// Feed (or re-feed) source text to the parser.
    /// Pass `Some(edit)` when precise change information available for
    /// incremental re-parsing (faster). Pass `None` to do a full re-parse
    /// TODO for now fine as simple, optimise later.
    pub fn update(&mut self, new_source: &str, edit: Option<tree_sitter::InputEdit>) {
        // take old tree out first
        let mut old_tree = self.tree.take(); // self.tree becomes none here

        if let (Some(tree), Some(edit)) = (&mut old_tree, edit) {
            tree.edit(&edit);
        }
        // Whether to use the old tree if edit was made, or pass None when no edit was made
        // Passing old tree means incremental change, Passing None means full re-parse of file needed
        let tree_to_reuse = if edit.is_some() {
            old_tree.as_ref()
        } else {
            None
        };

        self.tree = self.parser.parse(new_source, tree_to_reuse);
        self.source = new_source.to_string();
    }

    /// Walk the tree and produce (byte_range, token_type) pairs for highlighting.
    /// Should bed called once per render frame, not per line
    pub fn highlight_tokens(&mut self, version: u64) -> &[(Range<usize>, String)] {
        // Return cached result if version hasn't changed
        if let Some((cached_version, _)) = &self.token_cache {
            if *cached_version == version {
                return &self.token_cache.as_ref().unwrap().1;
            }
        }

        let Some(tree) = &self.tree else {
            self.token_cache = Some((version, vec![]));
            return &self.token_cache.as_ref().unwrap().1;
        };
        let Some(config) = &self.config else {
            self.token_cache = Some((version, vec![]));
            return &self.token_cache.as_ref().unwrap().1; // no config = no highlights, same as no theme
        };
        let mut tokens = Vec::new();
        Self::walk_node(tree.root_node(), config, &mut tokens);
        // log_trace!("Tokens Highlighted: {:?}", tokens);

        self.token_cache = Some((version, tokens));
        &self.token_cache.as_ref().unwrap().1
    }

    /// Recursively walks a node, getting the token type and byte range for each node,
    /// by recursively calling itself with child as new base node
    fn walk_node<'a>(node: Node, config: &LangConfig, out: &mut Vec<(Range<usize>, String)>) {
        let kind = node.kind();

        // Check parent rules first (e.g. identifier inside function_item)
        for rule in &config.parent_rules {
            if kind == rule.node_kind {
                if node
                    .parent()
                    .map(|p| p.kind() == rule.parent_kind)
                    .unwrap_or(false)
                {
                    out.push((node.byte_range(), rule.token_type.clone()));
                    // fall through to still recurse unless it's in stop_at
                }
            }
        }

        if let Some(token_type) = config.node_kind_map.get(kind) {
            out.push((node.byte_range(), token_type.clone()));
            if config.stop_at.contains(kind) {
                return; // don't recurse
            }
        }

        // Recurse into children for all other nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_node(child, config, out);
        }
    }

    /// Return tokens that overlap a specific byte range.
    /// Used in tests and optionally by the renderer for per-line filtering.
    pub fn tokens_in_range(
        &mut self,
        byte_start: usize,
        byte_end: usize,
    ) -> Vec<(Range<usize>, String)> {
        if byte_start >= byte_end {
            return vec![]; // zero or negative range, nothing can overlap
        }
        self.highlight_tokens(0)
            .into_iter()
            .filter(|(r, _)| r.start < byte_end && r.end > byte_start)
            .map(|(r, t)| (r.clone(), t.clone()))
            .collect()
    }
}

pub struct LangConfig {
    /// node kinds that map directly to a semantic token type
    /// e.g. ("string_literal", "string"), ("integer_literal", "number")
    pub node_kind_map: HashMap<String, String>,

    /// node kinds that should be highlighted but whose children should NOT be walked
    /// (avoids double-highlighting). Subset of node_kind_map.
    pub stop_at: HashSet<String>,

    /// If the node kind equals this, the parent kind is checked to decide the token type.
    /// e.g. in Rust, an "identifier" whose parent is "function_item" -> "function"
    pub parent_rules: Vec<ParentRule>,
}

pub struct ParentRule {
    pub node_kind: String,
    pub parent_kind: String,
    pub token_type: String,
}

// ────────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_lang_config() -> LangConfig {
        LangConfig {
            node_kind_map: HashMap::from([
                ("fn".to_string(), "keyword".to_string()),
                ("let".to_string(), "keyword".to_string()),
                ("pub".to_string(), "keyword".to_string()),
                ("return".to_string(), "keyword".to_string()),
                ("true".to_string(), "keyword".to_string()),
                ("false".to_string(), "keyword".to_string()),
                ("string_literal".to_string(), "string".to_string()),
                ("char_literal".to_string(), "string".to_string()),
                ("integer_literal".to_string(), "number".to_string()),
                ("float_literal".to_string(), "number".to_string()),
                ("line_comment".to_string(), "comment".to_string()),
                ("block_comment".to_string(), "comment".to_string()),
                ("type_identifier".to_string(), "type".to_string()),
                ("primitive_type".to_string(), "type".to_string()),
            ]),
            stop_at: HashSet::from([
                "string_literal".to_string(),
                "char_literal".to_string(),
                "line_comment".to_string(),
                "block_comment".to_string(),
            ]),
            parent_rules: vec![ParentRule {
                node_kind: "identifier".to_string(),
                parent_kind: "function_item".to_string(),
                token_type: "function".to_string(),
            }],
        }
    }

    fn make_tree(source: &str) -> SyntaxTree {
        let grammar_dir = crate::config::SyntaxConfig::default()
            .grammar_dir
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        let lang = crate::language::grammar_loader::load_grammar(&grammar_dir, "rust")
            .expect("rust grammar .so not found — run tree-sitter build for rust first");

        let mut st = SyntaxTree::new(lang, Some(make_lang_config()));
        st.update(source, None);
        st
    }

    // ── basic token detection ─────────────────────────────────────────────

    #[test]
    fn detects_keyword_fn() {
        let mut st = make_tree("fn main() {}");
        let tokens = st.highlight_tokens(0);
        assert!(
            tokens.iter().any(|(_, t)| *t == "keyword"),
            "expected 'keyword' token for 'fn', got: {:?}",
            tokens
        );
    }

    #[test]
    fn detects_string_literal() {
        let mut st = make_tree(r#"let s = "hello";"#);
        let tokens = st.highlight_tokens(0);
        assert!(
            tokens.iter().any(|(_, t)| *t == "string"),
            "expected 'string' token, got: {:?}",
            tokens
        );
    }

    #[test]
    fn detects_integer_literal() {
        let mut st = make_tree("let x = 42;");
        let tokens = st.highlight_tokens(0);
        assert!(
            tokens.iter().any(|(_, t)| *t == "number"),
            "expected 'number' token, got: {:?}",
            tokens
        );
    }

    #[test]
    fn detects_line_comment() {
        let mut st = make_tree("// this is a comment\nlet x = 1;");
        let tokens = st.highlight_tokens(0);
        assert!(
            tokens.iter().any(|(_, t)| *t == "comment"),
            "expected 'comment' token, got: {:?}",
            tokens
        );
    }

    #[test]
    fn detects_block_comment() {
        let mut st = make_tree("/* block */\nlet x = 1;");
        let tokens = st.highlight_tokens(0);
        assert!(
            tokens.iter().any(|(_, t)| *t == "comment"),
            "expected 'comment' token for block comment, got: {:?}",
            tokens
        );
    }

    #[test]
    fn detects_type_identifier() {
        let mut st = make_tree("let x: String = String::new();");
        let tokens = st.highlight_tokens(0);
        assert!(
            tokens.iter().any(|(_, t)| *t == "type"),
            "expected 'type' token, got: {:?}",
            tokens
        );
    }

    #[test]
    fn detects_function_name() {
        let mut st = make_tree("fn my_function() {}");
        let tokens = st.highlight_tokens(0).to_vec();
        // The identifier 'my_function' inside a function_item should be "function"
        let src = &st.source;
        let fn_token = tokens
            .iter()
            .find(|(r, t)| *t == "function" && &src[r.clone()] == "my_function");
        assert!(
            fn_token.is_some(),
            "expected 'function' token for 'my_function', got: {:?}",
            tokens
        );
    }

    // ── byte ranges are correct ───────────────────────────────────────────

    #[test]
    fn byte_range_for_string_is_correct() {
        let source = r#"let s = "hello";"#;
        let mut st = make_tree(source);
        let tokens = st.highlight_tokens(0);
        let string_tok = tokens
            .iter()
            .find(|(_, t)| *t == "string")
            .expect("no string token");
        let extracted = &source[string_tok.0.clone()];
        // tree-sitter includes the quotes in the string node
        assert!(
            extracted.contains("hello"),
            "byte range should cover the string content, got: {:?}",
            extracted
        );
    }

    #[test]
    fn byte_range_for_comment_covers_full_comment() {
        let source = "// hello world\nlet x = 1;";
        let mut st = make_tree(source);
        let tokens = st.highlight_tokens(0);
        let comment_tok = tokens
            .iter()
            .find(|(_, t)| *t == "comment")
            .expect("no comment token");
        let extracted = &source[comment_tok.0.clone()];
        assert!(
            extracted.contains("hello world"),
            "comment range should cover full text, got: {:?}",
            extracted
        );
    }

    // ── empty and edge cases ─────────────────────────────────────────────

    #[test]
    fn empty_source_returns_no_tokens() {
        let mut st = make_tree("");
        assert!(st.highlight_tokens(0).is_empty());
    }

    #[test]
    fn no_update_returns_no_tokens() {
        let grammar_dir = crate::config::SyntaxConfig::default()
            .grammar_dir
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let lang = crate::language::grammar_loader::load_grammar(&grammar_dir, "rust").unwrap();
        let mut st = SyntaxTree::new(lang, Some(make_lang_config())); // never call update()
        assert!(
            st.highlight_tokens(0).is_empty(),
            "tree is None, should return empty"
        );
    }

    #[test]
    fn whitespace_only_returns_no_tokens() {
        let mut st = make_tree("   \n\n   ");
        assert!(st.highlight_tokens(0).is_empty());
    }

    #[test]
    fn multiple_keywords_all_detected() {
        let source = "pub fn foo() { let x = 1; return x; }";
        let mut st = make_tree(source);
        let tokens = st.highlight_tokens(0);
        let keyword_texts: Vec<&str> = tokens
            .iter()
            .filter(|(_, t)| *t == "keyword")
            .map(|(r, _)| &source[r.clone()])
            .collect();
        assert!(keyword_texts.contains(&"pub"), "missing 'pub'");
        assert!(keyword_texts.contains(&"fn"), "missing 'fn'");
        assert!(keyword_texts.contains(&"let"), "missing 'let'");
        assert!(keyword_texts.contains(&"return"), "missing 'return'");
    }

    // ── tokens_in_range helper ────────────────────────────────────────────

    #[test]
    fn tokens_in_range_filters_correctly() {
        // "fn main() {}" — 'fn' is at bytes 0..2
        let source = "fn main() {}";
        let mut st = make_tree(source);
        // Ask only for the first 2 bytes
        let in_range = st.tokens_in_range(0, 2);
        assert!(
            in_range.iter().all(|(r, _)| r.start < 2),
            "tokens_in_range returned tokens outside requested range"
        );
    }

    #[test]
    fn tokens_in_range_empty_range_returns_nothing() {
        let mut st = make_tree("fn main() {}");
        // A zero-length range should match nothing
        let result = st.tokens_in_range(5, 5);
        assert!(
            result.is_empty(),
            "zero-length range should return no tokens"
        );
    }

    // ── incremental update ────────────────────────────────────────────────

    #[test]
    fn update_replaces_old_tree() {
        let mut st = make_tree("let x = 1;");
        let tokens_before = st.highlight_tokens(0);
        assert!(tokens_before.iter().any(|(_, t)| *t == "number"));

        // Replace with source that has a string instead
        st.update(r#"let s = "hello";"#, None);
        let tokens_after = st.highlight_tokens(1);
        assert!(
            tokens_after.iter().any(|(_, t)| *t == "string"),
            "after update should see string"
        );
        // The old number token should be gone
        assert!(
            !tokens_after.iter().any(|(_, t)| *t == "number"),
            "number token should be gone after update"
        );
    }

    #[test]
    fn update_doesnt_replace_old_tree_but_gets_from_last_cached() {
        let mut st = make_tree("let x = 1;");
        let tokens_before = st.highlight_tokens(0);
        assert!(tokens_before.iter().any(|(_, t)| *t == "number"));

        // Replace with source that has a string instead
        st.update(r#"let s = "hello";"#, None);
        let tokens_after = st.highlight_tokens(0);
        assert!(
            tokens_after.iter().any(|(_, t)| *t == "number"),
            "after update should still see number"
        );
        // The later added string should not be there
        assert!(
            !tokens_after.iter().any(|(_, t)| *t == "string"),
            "string token should not have been added after update"
        );
    }

    #[test]
    fn update_stores_new_source() {
        let mut st = make_tree("let x = 1;");
        st.update("fn foo() {}", None);
        assert_eq!(st.source, "fn foo() {}");
    }

    // ── multiline source ─────────────────────────────────────────────────

    #[test]
    fn multiline_tokens_have_correct_byte_offsets() {
        let source = "fn foo() {}\nlet x = 42;";
        let mut st = make_tree(source);
        let tokens = st.highlight_tokens(0);

        // 'let' starts at byte 12 (after "fn foo() {}\n")
        let let_tok = tokens
            .iter()
            .find(|(r, t)| *t == "keyword" && &source[r.clone()] == "let");
        assert!(
            let_tok.is_some(),
            "expected 'let' keyword in multiline source"
        );
        let (range, _) = let_tok.unwrap();
        assert_eq!(range.start, 12, "'let' should start at byte 12");
    }

    #[test]
    fn comment_on_second_line_has_correct_offset() {
        let source = "let x = 1;\n// comment here";
        let mut st = make_tree(source);
        let tokens = st.highlight_tokens(0);
        let comment = tokens
            .iter()
            .find(|(_, t)| *t == "comment")
            .expect("no comment");
        // Line 2 starts at byte 11
        assert!(comment.0.start >= 11, "comment should start on second line");
        assert!(&source[comment.0.clone()].contains("comment here"));
    }
}
