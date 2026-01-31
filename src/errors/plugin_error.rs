use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Plugin Not found: {0}")]
    NotFound(String),

    #[error("Invalid Arguments Detected: {0}")]
    InvalidArgs(String),

    #[error("Internal Plugin Error: {0}")]
    Internal(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
