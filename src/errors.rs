use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("EditorError: {0}")]
    EditorError(#[from] EditorError),

    #[error("Internal error: {0}")]
    InternalError(String),

}


///general editor errors, from more specific ones
#[derive(Error, Debug)]
pub enum EditorError {
    #[error("Failed to copy text: {0}")]
    TextSelectionError(#[from] TextSelectionError),

    #[error("Clipboard failed: {0}")]
    ClipboardError(#[from] ClipboardError),

    #[error("File: {0} not found")]
    FileNotFound(String),

}

#[derive(Debug, Error)]
pub enum TextSelectionError {
    #[error("No text currently selected in editor")]
    NoTextSelected,

    #[error("Invalid selection range.")]
    InvalidRange,
}

#[derive(Debug, Error)]
pub enum ClipboardError {
    #[error("No text currently copied")]
    NoCopiedText,

}

