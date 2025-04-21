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

    #[error("UNDO failed: {0}")]
    UndoError(#[from] UndoError),

    #[error("REDO failed: {0}")]
    RedoError(#[from] RedoError),

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

#[derive(Debug, Error)]
pub enum UndoError {
    #[error("No action to undo")]
    NoActionToUndo,
    #[error("failed to undo action")]
    FailedToUndo,

}

#[derive(Debug, Error)]
pub enum RedoError {
    #[error("No action to redo")]
    NoActionToRedo,
    #[error("failed to redo action")]
    FailedToRedo,

}



