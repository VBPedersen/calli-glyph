use super::command_errors::CommandError;
use super::config_errors::ConfigError;
use super::editor_errors::EditorError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("EditorError: {0}")]
    EditorFailure(#[from] EditorError),

    #[error("Command execution failed: {0}")]
    CommandFailure(#[from] CommandError),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Config related failure: {0}")]
    ConfigFailure(#[from] ConfigError),
}
