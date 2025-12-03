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
    pub fn reload(&mut self) -> Result<(), ConfigError> {
        *self = Self::try_load()?;
        eprintln!("[INFO] [CONFIG] Successfully reloaded configuration.");
        Ok(())
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

        //TODO implement custom checks for specific fields in config
        // Check for all corresponding fields, valid field names and such, both warns and errors

        // tab width warn, recommended
        if temp_config.editor.tab_width == 0 || temp_config.editor.tab_width > 8 {
            result.warnings.push(format!(
                "Editor 'tab_width' is set to {} (recommended range 2-8).",
                temp_config.editor.tab_width
            ));
        }

        // check if can build runtime keymaps
        match temp_config.keymaps.build_runtime_maps() {
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

        // Final check, if no errors were encountered, mark as valid.
        if result.errors.is_empty() {
            result.valid = true;
        } else {
            result.valid = false;
        }

        result
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
            std::fs::remove_file(&config_path)?;
            eprintln!("Deleted config file: {:?}", config_path);
        }
        Ok(())
    }
}
