/// A high-level representation of user intent in the TUI.
/// Context agnostic base actions, extended by contextual actions depending on structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputAction {
    // Universal actions
    TAB,
    ToggleActiveArea,
    ENTER,
    QUIT,
    NoOp,
    // Specific actions
    Editor(EditorAction),
    CommandLine(CommandLineAction),
    Popup(PopupAction),
    Debug(DebugAction),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorAction {
    // Movement
    MoveCursor(Direction),

    // Selection
    MoveSelectionCursor(Direction),

    // Editing
    COPY,
    CUT,
    PASTE,
    UNDO,
    REDO,
    WriteChar(char),
    BACKSPACE,
    DELETE,

    // Misc
    SAVE,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandLineAction {
    WriteChar(char),
    BACKSPACE,
    DELETE,
    MoveCursor(Direction),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopupAction {
    MoveCursor(Direction),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebugAction {
    //Debug related
    DebugNextTab,
    DebugPrevTab,
    DebugScrollUp,
    DebugScrollDown,
    DebugClearLogs,
    DebugClearSnapshots,
    DebugManualSnapshot,
    DebugCycleMode,
    DebugResetMetrics,
    ExitDebug,
    DebugInteract,
}

///direction enum to use in action enum values
#[derive(Clone, Debug, PartialEq, Eq)]
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
