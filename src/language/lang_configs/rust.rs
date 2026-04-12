use crate::language::syntax::{LangConfig, ParentRule};
use std::collections::{HashMap, HashSet};

pub fn rust_config() -> LangConfig {
    LangConfig {
        node_kind_map: HashMap::from([
            ("string_literal", "string"),
            ("raw_string_literal", "string"),
            ("char_literal", "string"), // 'a' is its own token in Rust
            ("integer_literal", "number"),
            ("float_literal", "number"),
            ("line_comment", "comment"),
            ("block_comment", "comment"),
            ("boolean_literal", "keyword"), // wraps true/false
            ("type_identifier", "type"),
            ("primitive_type", "type"), // u32, bool, str, etc.
            ("lifetime", "type"),       // e.g. 'a or 'static
            // All anonymous keyword leaves tree-sitter-rust emits
            ("fn", "keyword"),
            ("let", "keyword"),
            ("const", "keyword"),
            ("static", "keyword"),
            ("mut", "keyword"),
            ("pub", "keyword"),
            ("use", "keyword"),
            ("mod", "keyword"),
            ("struct", "keyword"),
            ("enum", "keyword"),
            ("impl", "keyword"),
            ("trait", "keyword"),
            ("type", "keyword"),
            ("where", "keyword"),
            ("return", "keyword"),
            ("if", "keyword"),
            ("else", "keyword"),
            ("match", "keyword"),
            ("for", "keyword"),
            ("while", "keyword"),
            ("loop", "keyword"),
            ("break", "keyword"),
            ("continue", "keyword"),
            ("in", "keyword"),
            ("as", "keyword"),
            ("ref", "keyword"),
            ("move", "keyword"),
            ("self", "keyword"),
            ("super", "keyword"),
            ("crate", "keyword"),
            ("extern", "keyword"),
            ("unsafe", "keyword"),
            ("async", "keyword"),
            ("await", "keyword"),
            ("dyn", "keyword"),
            ("abstract", "keyword"),
            ("become", "keyword"),
            ("box", "keyword"),
            ("do", "keyword"),
            ("final", "keyword"),
            ("macro", "keyword"),
            ("override", "keyword"),
            ("priv", "keyword"),
            ("try", "keyword"),
            ("typeof", "keyword"),
            ("unsized", "keyword"),
            ("virtual", "keyword"),
            ("yield", "keyword"),
        ]),
        stop_at: HashSet::from([
            "string_literal",
            "raw_string_literal",
            "char_literal",
            "line_comment",
            "block_comment",
            "boolean_literal",
        ]),
        parent_rules: vec![
            // fn my_func() , identifier directly inside function_item
            ParentRule {
                node_kind: "identifier",
                parent_kind: "function_item",
                token_type: "function",
            },
            // fn foo() where foo is inside a trait method signature
            ParentRule {
                node_kind: "identifier",
                parent_kind: "function_signature_item",
                token_type: "function",
            },
        ],
    }
}
