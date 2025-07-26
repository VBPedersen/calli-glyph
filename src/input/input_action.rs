use crossterm::event::KeyCode;

/// A high-level representation of user intent in the TUI.
#[derive(Debug, Clone)]
pub enum InputAction {
    MoveCursor(Direction),
    MoveSelectionCursor(Direction),
    TAB,
    ENTER,
    BACKSPACE,
    DELETE,
    SAVE,
    COPY,
    CUT,
    PASTE,
    UNDO,
    REDO,
    ToggleActiveArea,
    WriteChar(char),
    QUIT,
    NoOp,
}




///direction enum to use in action enum values
#[derive(Clone, Debug,PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}