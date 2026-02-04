//! Central registry for plugins to register commands and keybindings

use crate::config::plugins::PluginsConfig;
use crate::core::app::App;
use crate::errors::plugin_error::PluginError;
use crossterm::event::KeyEvent;
use ratatui::Frame;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
//PLUGIN METADATA

/// Metadata of the plugin
pub struct PluginMetadata {
    pub name: String,
    pub commands: Vec<PluginCommand>,
    pub keybinds: Vec<PluginKeybinding>,
}

/// A command provided by plugin
pub struct PluginCommand {
    pub name: String, // e.g. "search"
    pub description: String,
    pub aliases: Vec<String>, // e.g. ["s","find","locate"]
    pub handler: fn(&mut App, Vec<String>) -> Result<(), PluginError>,
}

/// Default keybinding provided by plugin
pub struct PluginKeybinding {
    pub key: String,     // e.g. "Ctrl+F"
    pub command: String, // e.g. "search"
    pub context: KeyContext,
}

/// Where keybinding applies.
/// e.g. whether it is an editor keybindig shortcut example: Ctrl + f,
/// or command line binding, :search
#[derive(Debug, Clone, PartialEq)]
pub enum KeyContext {
    Editor,
    CommandLine,
}

// PLUGIN TRAIT

/// Trait used to define and execute INTERNAL plugin
pub trait Plugin: Send {
    // Plugin name
    fn name(&self) -> &str;

    /// Plugin metadata (commands, keybindings, etc)
    fn metadata(&self) -> PluginMetadata;

    /// Initialize plugin
    fn init(&mut self, app: &mut App) -> Result<(), PluginError>;

    /// Handle input when plugin is active
    /// By standard: should consume input if plugin self is active and input is relavant
    /// Returns true when input consumed, returns false if not
    fn handle_key_event(&mut self, app: &mut App, key: KeyEvent) -> bool;

    /// Render plugin UI
    fn render(&self, frame: &mut Frame, app: &App) -> bool;

    /// Cleanup
    fn shutdown(&mut self, app: &mut App) {}
}

// Command Handler registry

/// Registry for plugin commands
pub struct CommandRegistry {
    // Store command names and their handlers separately
    handlers: HashMap<String, CommandHandler>,
}

// Wrapper type that can be cloned/moved
type CommandHandler = Arc<dyn Fn(&mut App, Vec<String>) -> Result<(), PluginError> + Send + Sync>;

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a command handler
    pub fn register_command<F>(&mut self, name: String, handler: F)
    where
        F: Fn(&mut App, Vec<String>) -> Result<(), PluginError> + Send + Sync + 'static,
    {
        self.handlers.insert(name, Arc::new(handler));
    }

    /// Execute a registered command
    pub fn execute(&self, app: &mut App, name: &str, args: Vec<String>) -> Result<(), PluginError> {
        let handler = self
            .handlers
            .get(name)
            .ok_or_else(|| PluginError::Internal("Command not found".to_string()))?
            .clone(); // Clone the Arc, not the function

        handler(app, args)
    }

    /// Check if command exists
    pub fn has_command(&self, name: &str) -> bool {
        self.handlers.contains_key(name)
    }

    /// Get all registered command names
    pub fn list_commands(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }

    /// Get specific handler by name
    pub fn get_handler(&self, name: &str) -> Option<CommandHandler> {
        self.handlers.get(name).cloned()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Plugin Manager

/// Manages plugins: registered plugins and commands + which plugin is currently active
/// Functionality includes loading, registering,
pub struct PluginManager {
    pub plugins: HashMap<String, Box<dyn Plugin>>,
    command_registry: CommandRegistry,
    active_plugin: Option<String>,
    plugin_config: Option<PluginsConfig>,
    // Runtime keybinding cache
    keybinding_map: HashMap<String, String>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            command_registry: CommandRegistry::new(),
            active_plugin: None,
            plugin_config: None,
            keybinding_map: HashMap::new(),
        }
    }

    /// Build keybinding map from plugins and config
    pub fn rebuild_keybinding_map(&mut self) {
        self.keybinding_map.clear();

        // Collect which plugins have config overrides
        let plugins_with_config: HashSet<String> = if let Some(config) = &self.plugin_config {
            config
                .keybindings
                .keys()
                .filter_map(|k| k.split('.').next())
                .map(|s| s.to_string())
                .collect()
        } else {
            HashSet::new()
        };

        // Add config keybindings
        if let Some(config) = &self.plugin_config {
            for (key, binding) in &config.keybindings {
                if let Some(plugin_name) = key.split('.').next() {
                    self.keybinding_map
                        .insert(binding.clone(), plugin_name.to_string());
                }
            }
        }

        // Add plugin defaults ONLY for plugins without config overrides
        for (name, plugin) in &self.plugins {
            if plugins_with_config.contains(name) {
                continue; // Skip, user configured this plugin
            }

            let metadata = plugin.metadata();
            for keybinding in &metadata.keybinds {
                // Only add if not already in map
                self.keybinding_map
                    .entry(keybinding.key.clone())
                    .or_insert_with(|| name.clone());
            }
        }

        log_info!("Rebuilt keybinding map: {:?}", self.keybinding_map);
    }

    /// Get name of active plugin
    pub fn active_plugin_name(&self) -> Option<String> {
        self.active_plugin.clone()
    }

    /// Activate a plugin (it consumes input)
    pub fn activate_plugin(&mut self, name: &str) {
        self.active_plugin = Some(name.to_string());
    }

    /// Deactivate current plugin
    pub fn deactivate_plugin(&mut self) {
        self.active_plugin = None;
    }

    /// Handle input, send to active plugin first
    pub fn handle_key_event(&mut self, app: &mut App, key: KeyEvent) -> bool {
        if let Some(active) = self.active_plugin.clone() {
            if let Some(plugin) = self.plugins.get_mut(&active) {
                log_info!("Handling key event: {:?}", key);
                return plugin.handle_key_event(app, key);
            }
        }
        false
    }

    /// Render active plugin
    pub fn render(&self, frame: &mut Frame, app: &App) {
        if let Some(active) = &self.active_plugin {
            if let Some(plugin) = self.plugins.get(active) {
                plugin.render(frame, app);
            }
        }
    }

    /// Get command registry
    pub fn command_registry(&self) -> &CommandRegistry {
        &self.command_registry
    }

    /// Get command registry as mutable
    pub fn command_registry_mut(&mut self) -> &mut CommandRegistry {
        &mut self.command_registry
    }

    /// Apply plugin configuration
    /// This modifies keybindings based on config by building keybinding_map
    pub fn apply_config(&mut self, config: &PluginsConfig) {
        self.plugin_config = Some(config.clone());
        self.rebuild_keybinding_map();
        log_info!("Applied plugin config: {:?}", config);
    }

    /// Get configured keybinding for a plugin command
    /// Returns custom keybinding if configured, else uses plugin default
    pub fn get_keybinding_command(&self, plugin_name: &str, command_name: &str) -> Option<String> {
        // Check config first
        if let Some(config) = &self.plugin_config {
            if let Some(custom_binding) = config.get_keybinding(plugin_name, command_name) {
                return Some(custom_binding.clone());
            }
        }

        // Fall back to plugin default
        if let Some(plugin) = self.plugins.get(plugin_name) {
            let metadata = plugin.metadata();
            for keybinding in metadata.keybinds {
                if keybinding.command == command_name {
                    return Some(keybinding.key);
                }
            }
        }

        None
    }

    /// Find plugin by keybinding in keybinding map
    pub fn find_plugin_by_keybinding(&self, key_str: &str) -> Option<String> {
        self.keybinding_map.get(key_str).cloned()
    }

    /// Insert a plugin directly
    pub fn insert_plugin(&mut self, name: String, plugin: Box<dyn Plugin>) {
        self.plugins.insert(name, plugin);
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
