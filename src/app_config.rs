use crate::args::AppLaunchArgs;
use crate::config::Config;
use crate::core::debug::log;
use crate::errors::error::AppError;
use std::fs;
use std::path::PathBuf;
use std::result::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum LaunchTarget {
    Empty,
    File(PathBuf),
    Dir(PathBuf),
}

impl LaunchTarget {
    pub fn from_path(path: Option<PathBuf>) -> Self {
        match path {
            None => Self::Empty,
            // Existing directories only. A nonexistent path stays a File,
            // so create-on-open behavior can be applied.
            Some(p) if p.is_dir() => {
                log_info!("App launched with dir as argument");
                Self::Dir(p.canonicalize().unwrap_or(p))
            }
            Some(p) => {
                log_info!("App launched with file as argument");
                Self::File(p)
            }
        }
    }
}

pub struct AppLaunchConfig {
    pub target: LaunchTarget,
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
                log_info!("Removing config file");
                fs::remove_file(&config_path)?;
                log_info!("Config file removed");
            } else {
                log_info!("No config file found when trying to reset");
            }
        }

        if args.file_path.is_some() {
            temp_app_launch_config.target = LaunchTarget::from_path(args.file_path);
        }

        //return finalised launch config
        Ok(temp_app_launch_config)
    }
}

impl Default for AppLaunchConfig {
    fn default() -> Self {
        Self {
            target: LaunchTarget::Empty,
            reset_config: false,
        }
    }
}
