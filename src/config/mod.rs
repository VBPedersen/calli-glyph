use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

mod defaults;
mod editor; //Editor settings
mod keymaps; //Keybinding config
mod performance; //Performance settings
mod ui; //UI settings //Default configurations

use crate::config::keymaps::RuntimeKeymaps;
use crate::errors::config_errors::ConfigError;
pub use editor::EditorConfig;
pub use keymaps::KeymapConfig;
pub use performance::PerformanceConfig;
pub use ui::UIConfig;

// Thread local storage for mocking the configuration path during tests.
// This is used by the config_path() function below.
thread_local! {
    static TEST_CONFIG_PATH: std::cell::RefCell<Option<PathBuf>> = std::cell::RefCell::new(None);
}

//REMEMBER TO DELETE config.toml in .config/calliglyph/config.toml
// to test new additions of binds and settings

/// Main struct of config, holds state of all config and keymaps
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)] // if fields are missing, makes them default to avoid errors
pub struct Config {
    pub editor: EditorConfig,
    pub keymaps: KeymapConfig,
    pub ui: UIConfig,
    pub performance: PerformanceConfig,

    // Runtime keymaps (not serialized)
    #[serde(skip)]
    pub runtime_keymaps: Option<RuntimeKeymaps>,
}

/// Result struct for validation state
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl Config {
    pub fn get_config_path() -> Result<PathBuf, ConfigError> {
        // If a test path is set, return it. ONLY USED FOR TESTING
        if let Some(path) = TEST_CONFIG_PATH.with(|cell| cell.borrow().clone()) {
            return Ok(path);
        }

        let config_dir = dirs::config_dir().ok_or(ConfigError::InvalidConfigPath)?;
        Ok(config_dir.join("calliglyph").join("config.toml"))
    }

    ///Tries to load config from path,
    /// func shared by load which falls back, and reload which propagates
    fn try_load() -> Result<Self, ConfigError> {
        let config_path = Self::get_config_path()?;

        let mut config = if config_path.exists() {
            eprintln!("[INFO] [CONFIG] Loading config from: {:?}", config_path);
            let content = fs::read_to_string(&config_path)?;
            toml::from_str(&content)?
        } else {
            eprintln!(
                "[WARN] [CONFIG] Config file not found at {:?}. Using default.",
                config_path
            );
            let config = Config::default();
            config.save()?;
            config
        };

        // Build runtime keymaps
        config.runtime_keymaps = Some(config.keymaps.build_runtime_maps()?);
        Ok(config)
    }

    /// Loads config from path, and falling back to default if errors or unsuccessful
    pub fn load() -> Self {
        Self::try_load().unwrap_or_else(|e| {
            eprintln!("[ERROR] [CONFIG] Error loading configuration: {:?}. Falling back to default configuration.", e);
            Self::default()
        })
    }

    /// Reloads config, and propagates error if unsuccessful
    pub fn reload(&mut self) -> Result<Self, ConfigError> {
        let config = Self::try_load()?;
        *self = config.clone();
        eprintln!("[INFO] [CONFIG] Successfully reloaded configuration.");
        Ok(config)
    }

    pub fn runtime_keymaps(&self) -> &RuntimeKeymaps {
        self.runtime_keymaps
            .as_ref()
            .expect("Runtime keymaps not initialized")
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let config_path = Self::get_config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let toml = toml::to_string_pretty(self)?;
        fs::write(&config_path, toml)?;
        Ok(())
    }

    //TODO complete and show result
    pub fn validate(&self) -> ValidationResult {
        let mut result = ValidationResult {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Check if config file exists
        let config_path = match Self::get_config_path() {
            Ok(path) => path,
            Err(e) => {
                result.valid = false;
                result
                    .errors
                    .push(format!("Cannot determine config path: {}", e));
                return result;
            }
        };

        if !config_path.exists() {
            result.warnings.push(format!(
                "Config file does not exist at {}",
                config_path.display()
            ));
            return result; // don't wanna check more, waste of time if no path
        }

        // Try to read content of file
        let content = match fs::read_to_string(&config_path) {
            Ok(c) => c,
            Err(e) => {
                result.valid = false;
                result.errors.push(format!(
                    "Cannot read config file at {}: {}",
                    config_path.display(),
                    e
                ));
                return result; // don't wanna check more, waste of time if can't read file
            }
        };

        // Try to deserialize the TOML content into the Config struct
        // using a temporary dummy struct for validation to avoid overwriting
        let temp_config: Config = match toml::from_str(&content) {
            Ok(cfg) => cfg,
            Err(e) => {
                result.valid = false;
                result
                    .errors
                    .push(format!("TOML Deserialization error in config file: {}", e));
                result.warnings.push("Check for missing required fields or invalid data types in the TOML structure.".to_string());
                return result; // don't wanna check more, waste of time if can't read deserialize
            }
        };

        // CUSTOM CHECKS on temp config to not make changes

        //TODO implement more validation:
        // Check for all corresponding fields, valid field names and such, both warns and errors

        // Validate editor config
        Self::validate_editor_config(&temp_config.editor, &mut result);

        // Validate UI config
        Self::validate_ui_config(&temp_config.ui, &mut result);

        // Validate performance config
        Self::validate_performance_config(&temp_config.performance, &mut result);

        // Validate keymaps
        Self::validate_keymaps(&temp_config.keymaps, &mut result);

        // Final check, if no errors were encountered, mark as valid.
        if result.errors.is_empty() {
            result.valid = true;
        } else {
            result.valid = false;
        }

        result
    }

    fn validate_editor_config(config: &EditorConfig, result: &mut ValidationResult) {
        // tab width warn, recommended
        if config.tab_width == 0 {
            result.valid = false;
            result
                .errors
                .push("editor.tab_width must be greater than 0".to_string());
        } else if config.tab_width > 8 {
            result.warnings.push(format!(
                "editor.tab_width  is set to {} (recommended range 2-8).",
                config.tab_width
            ));
        }

        // Validate auto_save_delay
        if config.auto_save && config.auto_save_delay_ms < 100 {
            result.warnings.push(
                "editor.auto_save_delay_ms is very low (< 100ms). May cause performance issues."
                    .to_string(),
            );
        }
        // Validate scrolloff
        if config.scrolloff > 15 {
            result.warnings.push(format!(
                "ui.scrolloff is {} (very large). Recommended: 3-10.",
                config.scrolloff
            ));
        }
    }

    fn validate_ui_config(config: &UIConfig, result: &mut ValidationResult) {
        // Theme validation (if theme file should exist)
        if config.theme != "default" {
            result.warnings.push(format!(
                "Custom theme '{}' specified. Ensure theme file exists.",
                config.theme
            ));
        }
    }

    fn validate_performance_config(config: &PerformanceConfig, result: &mut ValidationResult) {
        // Validate tick_rate
        if config.tick_rate_ms == 0 {
            result.valid = false;
            result
                .errors
                .push("performance.tick_rate_ms must be greater than 0".to_string());
        } else if config.tick_rate_ms < 16 {
            result.warnings.push(
                "performance.tick_rate_ms < 16ms (>60 FPS). May use unnecessary CPU.".to_string(),
            );
        } else if config.tick_rate_ms > 200 {
            result
                .warnings
                .push("performance.tick_rate_ms > 200ms. UI may feel sluggish.".to_string());
        }

        // Validate cursor blink rate
        if config.cursor_blink_rate_ms < 100 {
            result.warnings.push(
                "performance.cursor_blink_rate_ms < 100ms. Cursor may blink too fast.".to_string(),
            );
        }

        // Validate history limits
        if config.undo_history_limit == 0 {
            result
                .warnings
                .push("performance.undo_history_limit is 0. Undo will not work.".to_string());
        } else if config.undo_history_limit > 10000 {
            result.warnings.push(format!(
                "performance.undo_history_limit is {} (very large). May use excessive memory.",
                config.undo_history_limit
            ));
        }
    }

    fn validate_keymaps(keymaps: &KeymapConfig, result: &mut ValidationResult) {
        // Try to build runtime keymaps
        match keymaps.build_runtime_maps() {
            Ok(_) => {
                // Keymaps successfully built
            }
            Err(ConfigError::InvalidKeymap(msg)) => {
                result.valid = false;
                result
                    .errors
                    .push(format!("Keymap parsing failed: {}", msg));
            }
            Err(e) => {
                result.valid = false;
                result.errors.push(format!(
                    "Keymap validation failed with unexpected error: {}",
                    e
                ));
            }
        }
        // Check for empty keymaps
        if keymaps.editor.is_empty() {
            result
                .warnings
                .push("keymaps.editor is empty. No editor keybindings defined.".to_string());
        }
        if keymaps.command_line.is_empty() {
            result.warnings.push(
                "keymaps.command_line is empty. No command line keybindings defined.".to_string(),
            );
        }
        if keymaps.debug.is_empty() {
            result
                .warnings
                .push("keymaps.debug is empty. No debug keybindings defined.".to_string());
        }

        // Check for conflicting keybinds within same context
        Self::check_duplicate_keybinds("editor", &keymaps.editor, result);
        Self::check_duplicate_keybinds("command_line", &keymaps.command_line, result);
        Self::check_duplicate_keybinds("debug", &keymaps.debug, result);
    }

    fn check_duplicate_keybinds(
        context: &str,
        keybinds: &std::collections::HashMap<String, String>,
        result: &mut ValidationResult,
    ) {
        let mut key_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for key in keybinds.keys() {
            *key_counts.entry(key.clone()).or_insert(0) += 1;
        }

        for (key, count) in key_counts {
            if count > 1 {
                result.warnings.push(format!(
                    "Duplicate keybinding in {}: '{}' is defined {} times",
                    context, key, count
                ));
            }
        }
    }
}

//ONLY FOR TESTING
impl Config {
    /// Helper function to set the configuration path for testing only.
    /// This block is only compiled when the 'test' feature flag is active.
    pub fn set_test_config_path(path: Option<PathBuf>) {
        TEST_CONFIG_PATH.with(|cell| *cell.borrow_mut() = path);
    }
}

impl Default for Config {
    fn default() -> Self {
        let keymaps = KeymapConfig::default();

        let runtime_keymaps = match keymaps.build_runtime_maps() {
            Ok(maps) => Some(maps),
            Err(e) => {
                panic!("FATAL: Failed to build default runtime keymaps: {:?}. \
                This indicates a bug in the default configuration and the application cannot start.",
                       e);
            }
        };
        Self {
            editor: EditorConfig::default(),
            keymaps,
            ui: UIConfig::default(),
            performance: PerformanceConfig::default(),
            runtime_keymaps,
        }
    }
}

impl Config {
    /// Delete config file
    #[cfg(debug_assertions)] // Only available in debug builds
    pub fn delete_config_file() -> Result<(), ConfigError> {
        let config_path = Self::get_config_path()?;
        if config_path.exists() {
            fs::remove_file(&config_path)?;
            eprintln!("Deleted config file: {:?}", config_path);
        }
        Ok(())
    }
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn summary(&self) -> String {
        if self.valid && !self.has_warnings() {
            "✓ Config is valid with no warnings.".to_string()
        } else if self.valid && self.has_warnings() {
            format!(
                "✓ Config is valid but has {} warning(s).",
                self.warnings.len()
            )
        } else {
            format!(
                "✗ Config is invalid. {} error(s), {} warning(s).",
                self.errors.len(),
                self.warnings.len()
            )
        }
    }

    pub fn detailed_report(&self) -> String {
        let mut report = String::new();

        report.push_str(&self.summary());
        report.push_str("\n\n");

        if !self.errors.is_empty() {
            report.push_str("ERRORS:\n");
            for (i, error) in self.errors.iter().enumerate() {
                report.push_str(&format!("  {}. {}\n", i + 1, error));
            }
            report.push('\n');
        }

        if !self.warnings.is_empty() {
            report.push_str("WARNINGS:\n");
            for (i, warning) in self.warnings.iter().enumerate() {
                report.push_str(&format!("  {}. {}\n", i + 1, warning));
            }
        }

        report
    }
}
