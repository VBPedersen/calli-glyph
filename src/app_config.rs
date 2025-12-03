use crate::args::AppLaunchArgs;
use crate::config::Config;
use crate::errors::error::AppError;
use std::fs;
use std::path::PathBuf;
use std::result::Result;

pub struct AppLaunchConfig {
    pub file_path: Option<PathBuf>,
    pub reset_config: bool,
}

impl AppLaunchConfig {
    /// Creates the application configuration from the parsed CLI arguments,
    /// executing arguments in priority order.
    pub fn from_args(args: AppLaunchArgs) -> Result<Self, AppError> {
        let config_path = Config::get_config_path()?;

        // Load the default launch config, and change it based on args after
        let mut temp_app_launch_config = AppLaunchConfig::default();

        if args.reset_config {
            if config_path.exists() {
                //TODO log
                fs::remove_file(&config_path)?;
            } else {
                //TODO log
            }
        }

        if args.file_path.is_some() {
            temp_app_launch_config.file_path = args.file_path;
        }

        //return finalised launch config
        Ok(temp_app_launch_config)
    }
}

impl Default for AppLaunchConfig {
    fn default() -> Self {
        Self {
            file_path: None,
            reset_config: false,
        }
    }
}
