use crate::language::syntax::{LangConfig, ParentRule};
use std::collections::{HashMap, HashSet};

pub fn javascript_config() -> LangConfig {
    LangConfig {
        node_kind_map: HashMap::from([
            // Strings ,JS has template literals too
            ("string", "string"),
            ("template_string", "string"),
            // Numbers
            ("number", "number"),
            // Comments
            ("comment", "comment"),
            // Types (TS-only nodes, ignored in plain JS trees)
            ("type_identifier", "type"),
            ("predefined_type", "type"), // string, number, boolean, void etc.
            // Keywords
            ("var", "keyword"),
            ("let", "keyword"),
            ("const", "keyword"),
            ("function", "keyword"),
            ("return", "keyword"),
            ("if", "keyword"),
            ("else", "keyword"),
            ("for", "keyword"),
            ("while", "keyword"),
            ("do", "keyword"),
            ("break", "keyword"),
            ("continue", "keyword"),
            ("switch", "keyword"),
            ("case", "keyword"),
            ("default", "keyword"),
            ("new", "keyword"),
            ("delete", "keyword"),
            ("typeof", "keyword"),
            ("instanceof", "keyword"),
            ("in", "keyword"),
            ("of", "keyword"),
            ("class", "keyword"),
            ("extends", "keyword"),
            ("import", "keyword"),
            ("export", "keyword"),
            ("from", "keyword"),
            ("async", "keyword"),
            ("await", "keyword"),
            ("try", "keyword"),
            ("catch", "keyword"),
            ("finally", "keyword"),
            ("throw", "keyword"),
            ("this", "keyword"),
            ("super", "keyword"),
            ("static", "keyword"),
            ("get", "keyword"),
            ("set", "keyword"),
            ("yield", "keyword"),
            // TS-only keywords ,safely ignored if not present in the tree
            ("type", "keyword"),
            ("interface", "keyword"),
            ("enum", "keyword"),
            ("as", "keyword"),
            ("implements", "keyword"),
            ("declare", "keyword"),
            ("abstract", "keyword"),
            ("override", "keyword"),
            ("readonly", "keyword"),
            ("namespace", "keyword"),
            ("satisfies", "keyword"),
            // Booleans / null / undefined are identifier-like in JS grammar
            ("true", "keyword"),
            ("false", "keyword"),
            ("null", "keyword"),
            ("undefined", "keyword"),
        ]),
        stop_at: HashSet::from([
            "string",
            "template_string", // don't recurse into template contents
            "comment",
        ]),
        parent_rules: vec![
            // function foo() {}  →  identifier inside function_declaration
            ParentRule {
                node_kind: "identifier",
                parent_kind: "function_declaration",
                token_type: "function",
            },
            // const foo = () => {}  ->  identifier inside variable_declarator,
            // only when the value is an arrow/function expression
            // (best-effort , you'd need to inspect the sibling to be precise,
            //  so this targets the common pattern by checking the declarator)
            ParentRule {
                node_kind: "identifier",
                parent_kind: "function", // anonymous function expression name
                token_type: "function",
            },
            // class Foo  -> type_identifier inside class_declaration
            ParentRule {
                node_kind: "type_identifier",
                parent_kind: "class_declaration",
                token_type: "type",
            },
        ],
    }
}
