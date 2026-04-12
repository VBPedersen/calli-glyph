use crate::language::syntax::{LangConfig, ParentRule};
use std::collections::{HashMap, HashSet};

pub fn python_config() -> LangConfig {
    LangConfig {
        node_kind_map: HashMap::from([
            // Strings, tree-sitter-python uses "string" for all flavours
            // (single/double/triple quoted, f-strings are "string" with interpolation children)
            ("string", "string"),
            ("concatenated_string", "string"),
            // Numbers
            ("integer", "number"),
            ("float", "number"),
            ("complex", "number"),
            // Comments, Python only has line comments
            ("comment", "comment"),
            // Types, Python type annotations use these
            ("type", "type"),
            // Keywords, tree-sitter-python emits these as named nodes, not anonymous leaves
            ("def", "keyword"),
            ("class", "keyword"),
            ("return", "keyword"),
            ("yield", "keyword"),
            ("import", "keyword"),
            ("from", "keyword"),
            ("as", "keyword"),
            ("if", "keyword"),
            ("elif", "keyword"),
            ("else", "keyword"),
            ("for", "keyword"),
            ("while", "keyword"),
            ("break", "keyword"),
            ("continue", "keyword"),
            ("pass", "keyword"),
            ("raise", "keyword"),
            ("try", "keyword"),
            ("except", "keyword"),
            ("finally", "keyword"),
            ("with", "keyword"),
            ("and", "keyword"),
            ("or", "keyword"),
            ("not", "keyword"),
            ("in", "keyword"),
            ("is", "keyword"),
            ("lambda", "keyword"),
            ("del", "keyword"),
            ("global", "keyword"),
            ("nonlocal", "keyword"),
            ("assert", "keyword"),
            ("async", "keyword"),
            ("await", "keyword"),
            ("match", "keyword"),
            ("case", "keyword"), // Python 3.10+
            // Boolean/None literals are "true"/"false"/"none" in tree-sitter-python
            ("true", "keyword"),
            ("false", "keyword"),
            ("none", "keyword"),
        ]),
        stop_at: HashSet::from(["string", "concatenated_string", "comment"]),
        parent_rules: vec![
            // def my_func(...) ,identifier directly inside function_definition
            ParentRule {
                node_kind: "identifier",
                parent_kind: "function_definition",
                token_type: "function",
            },
            // async def my_func(...) ,same but wrapped in async_function_definition
            ParentRule {
                node_kind: "identifier",
                parent_kind: "async_function_definition",
                token_type: "function",
            },
            // class MyClass ,identifier directly inside class_definition
            ParentRule {
                node_kind: "identifier",
                parent_kind: "class_definition",
                token_type: "type",
            },
        ],
    }
}
