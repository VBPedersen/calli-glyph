use thiserror::Error;
use super::command_errors::CommandError;
use super::editor_errors::EditorError;


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
}


