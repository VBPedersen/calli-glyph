//! the tree-sitter wrapper

use tree_sitter::{Language, Node, Parser, Tree};

pub struct SyntaxTree {
    parser: Parser,
    tree: Option<Tree>,
    /// The last source string fed to the parser.
    /// Stored so highlight_tokens can resolve byte ranges back to text.
    pub source: String,
}

impl SyntaxTree {
    /// Creates a new syntax tree based on selected language
    pub fn new(language: Language) -> SyntaxTree {
        let mut parser = Parser::new();
        parser.set_language(&language).unwrap();
        Self {
            parser,
            tree: None,
            source: String::new(),
        }
    }

    /// Feed (or re-feed) source text to the parser.
    /// Pass `Some(edit)` when precise change information available for
    /// incremental re-parsing (faster). Pass `None` to do a full re-parse
    /// TODO for now fine as simple, optimise later.
    pub fn update(&mut self, new_source: &str, edit: Option<tree_sitter::InputEdit>) {
        if let (Some(old_tree), Some(edit)) = (&mut self.tree, edit) {
            old_tree.edit(&edit);
        }
        self.tree = self.parser.parse(new_source, self.tree.as_ref());
        self.source = new_source.to_string();
    }

    /// Walk the tree and produce (byte_range, token_type) pairs for highlighting.
    /// Should bed called once per render frame, not per line
    pub fn highlight_tokens(&self) -> Vec<(std::ops::Range<usize>, &'static str)> {
        let Some(tree) = &self.tree else {
            return vec![];
        };
        let mut tokens = Vec::new();
        Self::walk_node(tree.root_node(), &self.source, &mut tokens);
        log_trace!("Tokens Highlighted: {:?}", tokens);
        tokens
    }

    /// Recursively walks a node, getting the token type and byte range for each node,
    /// by recursively calling itself with child as new base node
    fn walk_node<'a>(
        node: Node,
        source: &str,
        out: &mut Vec<(std::ops::Range<usize>, &'static str)>,
    ) {
        // Map tree-sitter node kinds to semantic token types
        match node.kind() {
            // Strings
            "string_literal" | "raw_string_literal" => {
                out.push((node.byte_range(), "string"));
                return; // do NOT recurse, avoids double-highlighting the content
            }
            // Numbers
            "integer_literal" | "float_literal" => {
                out.push((node.byte_range(), "number"));
                // These are leaves (child_count == 0), fall through to no recursion
            }
            // Comments
            // line_comment has a "//" child — treat the whole node as one token.
            "line_comment" | "block_comment" => {
                out.push((node.byte_range(), "comment"));
                return;
            }
            // Booleans
            // boolean_literal wraps "true"/"false" child — highlight the whole node
            "boolean_literal" => {
                out.push((node.byte_range(), "keyword"));
                return;
            }
            // Function names
            // The identifier directly inside a function_item is the function name.
            "identifier"
                if node
                    .parent()
                    .map(|p| p.kind() == "function_item")
                    .unwrap_or(false) =>
            {
                out.push((node.byte_range(), "function"));
            }
            // types
            "type_identifier" | "primitive_type" => {
                out.push((node.byte_range(), "type"));
            }
            // keywords, these are anonumours leafs in tree-sitter-rust
            // These are the actual keyword tokens tree-sitter-rust produces.
            // visibility_modifier contains "pub" as a child. matched here
            // when it is reached during recursion.
            "fn" | "let" | "const" | "static" | "mut" | "pub" | "use" | "mod" | "struct"
            | "enum" | "impl" | "trait" | "type" | "where" | "return" | "if" | "else" | "match"
            | "for" | "while" | "loop" | "break" | "continue" | "in" | "as" | "ref" | "move"
            | "self" | "super" | "crate" | "extern" | "unsafe" | "async" | "await" | "dyn" => {
                out.push((node.byte_range(), "keyword"));
                // These are leaves, no children to recurse into
            }
            _ => {}
        };

        // Recurse into children for all other nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_node(child, source, out);
        }
    }

    /// Return tokens that overlap a specific byte range.
    /// Used in tests and optionally by the renderer for per-line filtering.
    pub fn tokens_in_range(
        &self,
        byte_start: usize,
        byte_end: usize,
    ) -> Vec<(std::ops::Range<usize>, &'static str)> {
        if byte_start >= byte_end {
            return vec![]; // zero or negative range, nothing can overlap
        }
        self.highlight_tokens()
            .into_iter()
            .filter(|(r, _)| r.start < byte_end && r.end > byte_start)
            .collect()
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tree(source: &str) -> SyntaxTree {
        let lang: Language = tree_sitter_rust::LANGUAGE.into();
        let mut st = SyntaxTree::new(lang);
        st.update(source, None);
        st
    }

    // ── basic token detection ─────────────────────────────────────────────

    #[test]
    fn print_all_node_kinds() {
        let lang: Language = tree_sitter_rust::LANGUAGE.into();
        let mut parser = Parser::new();
        parser.set_language(&lang).unwrap();

        let source = r#"pub fn main() {
            let x = 42;
            let s = "hello";
            // a comment
            let b = true;
        }"#;

        let tree = parser.parse(source, None).unwrap();

        fn walk(node: Node, source: &str, depth: usize) {
            let indent = "  ".repeat(depth);
            let text = &source[node.byte_range()];
            let preview = if text.len() > 20 { &text[..20] } else { text };
            println!(
                "{}kind={:?} named={} children={} bytes={}..{} text={:?}",
                indent,
                node.kind(),
                node.is_named(),
                node.child_count(),
                node.start_byte(),
                node.end_byte(),
                preview,
            );
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                walk(child, source, depth + 1);
            }
        }

        walk(tree.root_node(), source, 0);
        // This test always "fails" so the output prints — remove when done
        // panic!("scroll up to read the tree");
    }

    #[test]
    fn detects_keyword_fn() {
        let st = make_tree("fn main() {}");
        let tokens = st.highlight_tokens();
        assert!(
            tokens.iter().any(|(_, t)| *t == "keyword"),
            "expected 'keyword' token for 'fn', got: {:?}",
            tokens
        );
    }

    #[test]
    fn detects_string_literal() {
        let st = make_tree(r#"let s = "hello";"#);
        let tokens = st.highlight_tokens();
        assert!(
            tokens.iter().any(|(_, t)| *t == "string"),
            "expected 'string' token, got: {:?}",
            tokens
        );
    }

    #[test]
    fn detects_integer_literal() {
        let st = make_tree("let x = 42;");
        let tokens = st.highlight_tokens();
        assert!(
            tokens.iter().any(|(_, t)| *t == "number"),
            "expected 'number' token, got: {:?}",
            tokens
        );
    }

    #[test]
    fn detects_line_comment() {
        let st = make_tree("// this is a comment\nlet x = 1;");
        let tokens = st.highlight_tokens();
        assert!(
            tokens.iter().any(|(_, t)| *t == "comment"),
            "expected 'comment' token, got: {:?}",
            tokens
        );
    }

    #[test]
    fn detects_block_comment() {
        let st = make_tree("/* block */\nlet x = 1;");
        let tokens = st.highlight_tokens();
        assert!(
            tokens.iter().any(|(_, t)| *t == "comment"),
            "expected 'comment' token for block comment, got: {:?}",
            tokens
        );
    }

    #[test]
    fn detects_type_identifier() {
        let st = make_tree("let x: String = String::new();");
        let tokens = st.highlight_tokens();
        assert!(
            tokens.iter().any(|(_, t)| *t == "type"),
            "expected 'type' token, got: {:?}",
            tokens
        );
    }

    #[test]
    fn detects_function_name() {
        let st = make_tree("fn my_function() {}");
        let tokens = st.highlight_tokens();
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
        let st = make_tree(source);
        let tokens = st.highlight_tokens();
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
        let st = make_tree(source);
        let tokens = st.highlight_tokens();
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
        let st = make_tree("");
        assert!(st.highlight_tokens().is_empty());
    }

    #[test]
    fn no_update_returns_no_tokens() {
        let lang: Language = tree_sitter_rust::LANGUAGE.into();
        let st = SyntaxTree::new(lang); // never call update()
        assert!(
            st.highlight_tokens().is_empty(),
            "tree is None, should return empty"
        );
    }

    #[test]
    fn whitespace_only_returns_no_tokens() {
        let st = make_tree("   \n\n   ");
        assert!(st.highlight_tokens().is_empty());
    }

    #[test]
    fn multiple_keywords_all_detected() {
        let source = "pub fn foo() { let x = 1; return x; }";
        let st = make_tree(source);
        let tokens = st.highlight_tokens();
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
        let st = make_tree(source);
        // Ask only for the first 2 bytes
        let in_range = st.tokens_in_range(0, 2);
        assert!(
            in_range.iter().all(|(r, _)| r.start < 2),
            "tokens_in_range returned tokens outside requested range"
        );
    }

    #[test]
    fn tokens_in_range_empty_range_returns_nothing() {
        let st = make_tree("fn main() {}");
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
        let tokens_before = st.highlight_tokens();
        assert!(tokens_before.iter().any(|(_, t)| *t == "number"));

        // Replace with source that has a string instead
        st.update(r#"let s = "hello";"#, None);
        let tokens_after = st.highlight_tokens();
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
    fn update_stores_new_source() {
        let mut st = make_tree("let x = 1;");
        st.update("fn foo() {}", None);
        assert_eq!(st.source, "fn foo() {}");
    }

    // ── multiline source ─────────────────────────────────────────────────

    #[test]
    fn multiline_tokens_have_correct_byte_offsets() {
        let source = "fn foo() {}\nlet x = 42;";
        let st = make_tree(source);
        let tokens = st.highlight_tokens();

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
        let st = make_tree(source);
        let tokens = st.highlight_tokens();
        let comment = tokens
            .iter()
            .find(|(_, t)| *t == "comment")
            .expect("no comment");
        // Line 2 starts at byte 11
        assert!(comment.0.start >= 11, "comment should start on second line");
        assert!(&source[comment.0.clone()].contains("comment here"));
    }
}
