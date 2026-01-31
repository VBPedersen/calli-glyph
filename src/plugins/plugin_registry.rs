//! Central registry for plugins to register commands and keybindings

use crate::core::app::App;
use crate::errors::plugin_error::PluginError;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::Frame;
use std::collections::HashMap;
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
    /// By standard: should consume input if plugin self is active
    fn handle_key_event(&mut self, key: KeyEvent) -> bool;

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
    plugins: HashMap<String, Box<dyn Plugin>>,
    command_registry: CommandRegistry,
    active_plugin: Option<String>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            command_registry: CommandRegistry::new(),
            active_plugin: None,
        }
    }

    /// Load and register a plugin
    pub fn load_plugin(
        &mut self,
        mut plugin: Box<dyn Plugin>,
        app: &mut App,
    ) -> Result<(), PluginError> {
        let name = plugin.name().to_string();
        let metadata = plugin.metadata();

        // Initialize plugin
        plugin.init(app)?;

        // Register all commands the plugin provides
        for cmd in metadata.commands {
            let handler = cmd.handler;
            self.command_registry
                .register_command(cmd.name, move |app, args| handler(app, args));

            // Also register aliases
            for alias in cmd.aliases {
                self.command_registry
                    .register_command(alias, move |app, args| handler(app, args));
            }
        }

        self.plugins.insert(name, plugin);
        Ok(())
    }

    pub fn load_plugin_consuming(
        mut self,
        mut plugin: Box<dyn Plugin>,
        mut app: App,
    ) -> (Self, App, Result<(), PluginError>) {
        let name = plugin.name().to_string();
        let metadata = plugin.metadata();

        // Initialize plugin
        if let Err(e) = plugin.init(&mut app) {
            return (self, app, Err(e));
        }

        // Register all commands the plugin provides
        for cmd in metadata.commands {
            let handler = cmd.handler;
            self.command_registry
                .register_command(cmd.name.clone(), move |app, args| handler(app, args));

            // Also register aliases
            for alias in cmd.aliases {
                self.command_registry
                    .register_command(alias, move |app, args| handler(app, args));
            }
        }

        self.plugins.insert(name, plugin);
        (self, app, Ok(()))
    }

    /// Get name of active plugin
    pub fn active_plugin_name(&self) -> Option<&str> {
        self.active_plugin.as_deref()
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
    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        if let Some(active) = self.active_plugin.clone() {
            if let Some(plugin) = self.plugins.get_mut(&active) {
                log_info!("Handling key event: {:?}", key);
                return plugin.handle_key_event(key);
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

    /// Find which plugin has a keybinding for this key
    pub fn find_plugin_by_keybinding(&self, key_str: &str) -> Option<String> {
        // Iterate through all plugins and check their metadata
        for (name, plugin) in &self.plugins {
            let metadata = plugin.metadata();
            for keybinding in metadata.keybinds {
                if keybinding.key.to_lowercase() == key_str.to_lowercase() {
                    //lowercase in case mismatch e.g. Ctrl+t == Ctrl+T
                    return Some(name.clone());
                }
            }
        }
        None
    }

    /// Get the command name for a plugin's keybinding
    pub fn get_keybinding_command(&self, plugin_name: &str, key_str: &str) -> Option<String> {
        if let Some(plugin) = self.plugins.get(plugin_name) {
            let metadata = plugin.metadata();
            for keybinding in metadata.keybinds {
                if keybinding.key == key_str {
                    return Some(keybinding.command);
                }
            }
        }
        None
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
