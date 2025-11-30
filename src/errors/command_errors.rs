use thiserror::Error;

///general command errors, from more specific ones
#[derive(Error, Debug)]
pub enum CommandError {
    /// The command name is unrecognized, e.g. `:wqz`
    #[error("Unknown command: {0}")]
    UnknownCommand(String),

    /// The command was recognized but its arguments were invalid
    #[error("Invalid arguments for command '{command}': {reason}")]
    InvalidArguments { command: String, reason: String },

    /// A required component (e.g. editor or file) was not available
    #[error("Missing required context: {0}")]
    MissingContext(String),

    /// Something failed internally during execution
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),

    /// Used when a command can't run in the current application state
    #[error("Command cannot be executed in current state: {0}")]
    InvalidState(String),
}
