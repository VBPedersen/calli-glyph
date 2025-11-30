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

//REMEMBER TO DELETE config.toml in .config/calliglyph/config.toml
// to test new additions of binds and settings

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub editor: EditorConfig,
    pub keymaps: KeymapConfig,
    pub ui: UIConfig,
    pub performance: PerformanceConfig,

    // Runtime keymaps (not serialized)
    #[serde(skip)]
    runtime_keymaps: Option<RuntimeKeymaps>,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = Self::config_path()?;

        let mut config = if config_path.exists() {
            eprintln!("Loading config from: {:?}", config_path); // DEBUG
            let content = fs::read_to_string(&config_path)?;
            toml::from_str(&content)?
        } else {
            let config = Config::default();
            config.save()?;
            config
        };

        // Build runtime keymaps
        config.runtime_keymaps = Some(config.keymaps.build_runtime_maps()?);
        Ok(config)
    }

    pub fn runtime_keymaps(&self) -> &RuntimeKeymaps {
        self.runtime_keymaps
            .as_ref()
            .expect("Runtime keymaps not initialized")
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let toml = toml::to_string_pretty(self)?;
        fs::write(&config_path, toml)?;
        Ok(())
    }

    fn config_path() -> Result<PathBuf, ConfigError> {
        let config_dir = dirs::config_dir().ok_or(ConfigError::InvalidConfigPath)?;
        Ok(config_dir.join("calliglyph").join("config.toml"))
    }

    pub fn reload(&mut self) -> Result<(), ConfigError> {
        *self = Self::load()?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: EditorConfig::default(),
            keymaps: KeymapConfig::default(),
            ui: UIConfig::default(),
            performance: PerformanceConfig::default(),
            runtime_keymaps: None,
        }
    }
}
