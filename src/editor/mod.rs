

//main core editor
pub mod editor;
//undo redo utility
pub mod undo_redo;

// Re-export the Editor struct for simpler imports elsewhere
pub use editor::Editor;