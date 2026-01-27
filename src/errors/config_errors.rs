use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Failed to serialize config: {0}")]
    SerializeError(#[from] toml::ser::Error),

    #[error("Could not determine config path")]
    InvalidConfigPath,

    #[error("Invalid keymap: {0}")]
    InvalidKeymap(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid value for '{field}': {reason}")]
    InvalidValue { field: String, reason: String },
}
