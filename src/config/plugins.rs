use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// TODO add custom config fields that plugins can utilize internally, specifically for configuring each plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PluginsConfig {
    /// Which plugins to load (default: all built-in plugins)
    pub enabled: Vec<String>,
    /// Override keybindings per plugin
    pub keybindings: HashMap<String, String>,
}

impl Default for PluginsConfig {
    fn default() -> Self {
        Self {
            enabled: vec![
                "test_plugin".to_string(),           //testplugin
                "search_replace_plugin".to_string(), // search and replace plugin
            ],
            keybindings: HashMap::new(),
        }
    }
}

impl PluginsConfig {
    /// Check if plugin by name is active
    pub fn is_enabled(&self, str: &str) -> bool {
        self.enabled.iter().any(|p| p == str)
    }

    /// Get custom keybinding for a plugin command
    /// Format: get_keybinding("test_plugin", "test") -> Some("Ctrl+T")
    pub fn get_keybinding(&self, plugin: &str, command: &str) -> Option<&String> {
        let key = format!("{}.{}", plugin, command);
        self.keybindings.get(&key)
    }
}
