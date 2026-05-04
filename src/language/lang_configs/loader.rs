use crate::language::syntax::{LangConfig, ParentRule};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Debug, Deserialize)]
struct LangConfigFile {
    #[serde(default)]
    node_kinds: HashMap<String, String>, // node_kind -> token_type

    #[serde(default)]
    stop_at: Vec<String>,

    #[serde(default)]
    parent_rules: Vec<ParentRuleConfig>,
}

#[derive(Debug, Deserialize)]
struct ParentRuleConfig {
    node_kind: String,
    parent_kind: String,
    token_type: String,
}

pub fn load_lang_config(grammar_dir: &Path, grammar_name: &str) -> Result<LangConfig, String> {
    let config_path = grammar_dir.join(format!("{}.toml", grammar_name));

    if !config_path.exists() {
        return Err(format!(
            "Language config not found: {}",
            config_path.display()
        ));
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read {}: {}", config_path.display(), e))?;

    let file: LangConfigFile = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse {}: {}", config_path.display(), e))?;

    Ok(LangConfig {
        node_kind_map: file.node_kinds,
        stop_at: file.stop_at.into_iter().collect::<HashSet<_>>(),
        parent_rules: file
            .parent_rules
            .into_iter()
            .map(|r| ParentRule {
                node_kind: r.node_kind,
                parent_kind: r.parent_kind,
                token_type: r.token_type,
            })
            .collect(),
    })
}
