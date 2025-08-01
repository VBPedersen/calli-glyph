/// A high-level representation of user intent in the TUI.
#[derive(Debug, Clone, PartialEq)]
pub enum InputAction {
    MoveCursor(Direction),
    MoveSelectionCursor(Direction),
    TAB,
    ENTER,
    BACKSPACE,
    DELETE,
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
#[derive(Clone, Debug, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

///convert direction to (x,y) vector with i16 values
impl Direction {
    pub fn to_vector(self) -> (i16, i16) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}
