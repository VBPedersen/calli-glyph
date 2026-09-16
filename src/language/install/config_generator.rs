//! Generates a best-effort starter `<lang>.toml` node-kind mapping by
//! introspecting the grammar's own `src/node-types.json` — every official
//! tree-sitter grammar ships this file (it's required for language
//! bindings), so no per-language knowledge needs to be baked into this
//! app to produce *something* usable.
//!
//! This can't replace a hand-tuned config like typescript.toml —
//! `[[parent_rules]]` (function names, class names, etc.) are too
//! language-specific to infer from node-types.json alone — but it means a
//! freshly installed grammar highlights comments, strings, numbers,
//! booleans, keywords, and basic types immediately, instead of nothing.
//! The user can refine it from there exactly like any other config.

use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

/// Reads `<grammar_root>/src/node-types.json` and, if usable, returns the
/// full text of a starter `.toml` config (in the same format `loader.rs`
/// parses). Returns `None` if the file is missing or empty of anything
/// recognizable — better to write nothing than a useless file.
pub fn generate_lang_config(grammar_root: &Path, grammar_name: &str) -> Option<String> {
    let node_types_path = grammar_root.join("src").join("node-types.json");
    let content = std::fs::read_to_string(&node_types_path).ok()?;
    let entries: Vec<Value> = serde_json::from_str(&content).ok()?;

    let mut node_kinds: BTreeMap<String, &'static str> = BTreeMap::new();
    for entry in &entries {
        let Some(type_name) = entry.get("type").and_then(Value::as_str) else {
            continue;
        };
        let named = entry.get("named").and_then(Value::as_bool).unwrap_or(false);
        if let Some(token_type) = classify(type_name, named) {
            node_kinds.insert(type_name.to_string(), token_type);
        }
    }

    if node_kinds.is_empty() {
        return None;
    }

    // Anything mapped to "string" or "comment" is a leaf the walker
    // shouldn't descend into — except types with "template" in the name,
    // which usually contain interpolation (`${...}`) worth still walking.
    let stop_at: Vec<&str> = node_kinds
        .iter()
        .filter(|(name, kind)| matches!(**kind, "string" | "comment") && !name.contains("template"))
        .map(|(name, _)| name.as_str())
        .collect();

    Some(render_toml(grammar_name, &node_kinds, &stop_at))
}

/// Heuristic classification of a single node-types.json entry.
fn classify(type_name: &str, named: bool) -> Option<&'static str> {
    if !named {
        // Anonymous tokens are either punctuation/operators (e.g. "{", "=>")
        // or literal keywords (e.g. "if", "class", "return") — tree-sitter
        // represents keywords as anonymous tokens whose `type` IS the
        // keyword text itself, so a word-like anonymous token is almost
        // always a keyword.
        let is_word = !type_name.is_empty()
            && type_name.len() > 1
            && type_name
                .chars()
                .all(|c| c.is_ascii_alphabetic() || c == '_');
        return if is_word { Some("keyword") } else { None };
    }

    // Hidden/supertype nodes (leading underscore, e.g. "_expression") are
    // never concrete syntax the walker sees directly — skip them.
    if type_name.starts_with('_') {
        return None;
    }

    let t = type_name;
    if t.contains("comment") {
        Some("comment")
    } else if t.contains("string") || t.contains("regex") {
        Some("string")
    } else if t.contains("number") || t.contains("integer") || t.contains("float") {
        Some("number")
    } else if t == "true" || t == "false" || t.contains("boolean") {
        Some("boolean")
    } else if t.contains("type_identifier")
        || t.contains("predefined_type")
        || t == "primitive_type"
    {
        Some("type")
    } else {
        None
    }
}

fn render_toml(
    grammar_name: &str,
    node_kinds: &BTreeMap<String, &'static str>,
    stop_at: &[&str],
) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# Auto-generated starter config for '{}', derived from the grammar's\n\
         # src/node-types.json. This is a best-effort mapping — comments, strings,\n\
         # numbers, booleans, keywords, and basic types — not a hand-tuned config.\n\
         # Feel free to edit: add [[parent_rules]] for context-sensitive highlighting\n\
         # (e.g. function names, class names), or adjust node_kinds/stop_at below.\n\n",
        grammar_name
    ));

    out.push_str("stop_at = [\n");
    for name in stop_at {
        out.push_str(&format!("    \"{}\",\n", name));
    }
    out.push_str("]\n\n");

    out.push_str("[node_kinds]\n");
    for (name, kind) in node_kinds {
        out.push_str(&format!("\"{}\" = \"{}\"\n", name, kind));
    }

    out
}

//████████╗███████╗███████╗████████╗███████╗
//╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//   ██║   █████╗  ███████╗   ██║   ███████╗
//   ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//   ██║   ███████╗███████║   ██║   ███████║
//   ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝

#[cfg(test)]
mod unit_config_generator_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn classifies_common_named_nodes() {
        assert_eq!(classify("comment", true), Some("comment"));
        assert_eq!(classify("line_comment", true), Some("comment"));
        assert_eq!(classify("string", true), Some("string"));
        assert_eq!(classify("template_string", true), Some("string"));
        assert_eq!(classify("regex", true), Some("string"));
        assert_eq!(classify("number", true), Some("number"));
        assert_eq!(classify("integer_literal", true), Some("number"));
        assert_eq!(classify("float", true), Some("number"));
        assert_eq!(classify("true", true), Some("boolean"));
        assert_eq!(classify("false", true), Some("boolean"));
        assert_eq!(classify("type_identifier", true), Some("type"));
        assert_eq!(classify("predefined_type", true), Some("type"));
        assert_eq!(classify("primitive_type", true), Some("type"));
    }

    #[test]
    fn skips_hidden_supertype_nodes() {
        assert_eq!(classify("_expression", true), None);
    }

    #[test]
    fn skips_unrecognized_named_nodes() {
        assert_eq!(classify("identifier", true), None);
        assert_eq!(classify("binary_expression", true), None);
    }

    #[test]
    fn anonymous_word_tokens_become_keywords() {
        assert_eq!(classify("if", false), Some("keyword"));
        assert_eq!(classify("return", false), Some("keyword"));
        assert_eq!(classify("class", false), Some("keyword"));
    }

    #[test]
    fn anonymous_punctuation_is_ignored() {
        assert_eq!(classify("{", false), None);
        assert_eq!(classify("=>", false), None);
        assert_eq!(classify(";", false), None);
        assert_eq!(classify("+=", false), None);
    }

    #[test]
    fn single_char_anonymous_tokens_are_ignored() {
        // Keeps the word-heuristic conservative — avoids misclassifying
        // single-letter punctuation-like anonymous tokens as keywords.
        assert_eq!(classify("_", false), None);
    }

    #[test]
    fn generate_lang_config_reads_node_types_json() {
        let tmp = tempdir().unwrap();
        let src_dir = tmp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        let node_types = r#"[
            {"type": "if", "named": false},
            {"type": "comment", "named": true},
            {"type": "string", "named": true},
            {"type": "identifier", "named": true},
            {"type": "_expression", "named": true}
        ]"#;
        std::fs::write(src_dir.join("node-types.json"), node_types).unwrap();

        let result =
            generate_lang_config(tmp.path(), "testlang").expect("should generate a config");

        assert!(result.contains("\"if\" = \"keyword\""));
        assert!(result.contains("\"comment\" = \"comment\""));
        assert!(result.contains("\"string\" = \"string\""));
        assert!(!result.contains("\"identifier\""));
        assert!(!result.contains("_expression"));
        assert!(result.contains("stop_at"));
    }

    #[test]
    fn generate_lang_config_returns_none_when_file_missing() {
        let tmp = tempdir().unwrap();
        assert!(generate_lang_config(tmp.path(), "nope").is_none());
    }

    #[test]
    fn generate_lang_config_returns_none_when_nothing_classifiable() {
        let tmp = tempdir().unwrap();
        let src_dir = tmp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        // Only punctuation and unrecognized named nodes — nothing to map.
        let node_types = r#"[
            {"type": "{", "named": false},
            {"type": "identifier", "named": true}
        ]"#;
        std::fs::write(src_dir.join("node-types.json"), node_types).unwrap();

        assert!(generate_lang_config(tmp.path(), "nope").is_none());
    }

    #[test]
    fn stop_at_excludes_template_strings_but_includes_comment() {
        let tmp = tempdir().unwrap();
        let src_dir = tmp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        let node_types = r#"[
            {"type": "template_string", "named": true},
            {"type": "comment", "named": true}
        ]"#;
        std::fs::write(src_dir.join("node-types.json"), node_types).unwrap();

        let result = generate_lang_config(tmp.path(), "testlang").unwrap();
        assert!(result.contains("\"template_string\" = \"string\""));

        let stop_at_section = result.split("[node_kinds]").next().unwrap();
        assert!(
            !stop_at_section.contains("template_string"),
            "template strings should be walked, not stopped at, to allow interpolation"
        );
        assert!(stop_at_section.contains("comment"));
    }
}
