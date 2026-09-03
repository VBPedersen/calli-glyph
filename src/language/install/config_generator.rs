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
            && type_name.chars().all(|c| c.is_ascii_alphabetic() || c == '_');
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