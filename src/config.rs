

//EDITOR SETTINGS
pub mod editor_settings {
    pub const TAB_WIDTH: u16 = 4;
}

// KEYBINDS
#[allow(dead_code)]  //don't need warnings for unused keybinds
pub mod key_binds {
    use crossterm::event::{KeyCode, KeyModifiers};
    
    pub const KEYBIND_TOGGLE_AREA: (KeyModifiers, KeyCode) = (KeyModifiers::NONE, KeyCode::Esc);
    pub const KEYBIND_BACKSPACE: (KeyModifiers, KeyCode) = (KeyModifiers::NONE, KeyCode::Backspace);
    pub const KEYBIND_TAB: (KeyModifiers, KeyCode) = (KeyModifiers::NONE, KeyCode::Tab);
    pub const KEYBIND_ENTER: (KeyModifiers, KeyCode) = (KeyModifiers::NONE, KeyCode::Enter);
    pub const KEYBIND_DELETE: (KeyModifiers, KeyCode) = (KeyModifiers::NONE, KeyCode::Delete);

    //Cursor Movement
    pub const KEYBIND_UP: (KeyModifiers, KeyCode) = (KeyModifiers::NONE, KeyCode::Up);
    pub const KEYBIND_DOWN: (KeyModifiers, KeyCode) = (KeyModifiers::NONE, KeyCode::Down);
    pub const KEYBIND_LEFT: (KeyModifiers, KeyCode) = (KeyModifiers::NONE, KeyCode::Left);
    pub const KEYBIND_RIGHT: (KeyModifiers, KeyCode) = (KeyModifiers::NONE, KeyCode::Right);

    //Text Selection //move text selection cursor
    pub const KEYBIND_SELECTION_UP: (KeyModifiers, KeyCode) = (KeyModifiers::SHIFT, KeyCode::Up);
    pub const KEYBIND_SELECTION_DOWN: (KeyModifiers, KeyCode) =
        (KeyModifiers::SHIFT, KeyCode::Down);
    pub const KEYBIND_SELECTION_LEFT: (KeyModifiers, KeyCode) =
        (KeyModifiers::SHIFT, KeyCode::Left);
    pub const KEYBIND_SELECTION_RIGHT: (KeyModifiers, KeyCode) =
        (KeyModifiers::SHIFT, KeyCode::Right);

    //WITH MODIFIER AND CHAR  :: ShortCuts
    pub const KEYBIND_SAVE: (KeyModifiers, KeyCode) = (KeyModifiers::CONTROL, KeyCode::Char('s'));
    pub const KEYBIND_COPY: (KeyModifiers, KeyCode) = (KeyModifiers::CONTROL, KeyCode::Char('c'));
    pub const KEYBIND_CUT: (KeyModifiers, KeyCode) = (KeyModifiers::CONTROL, KeyCode::Char('x'));

    pub const KEYBIND_PASTE: (KeyModifiers, KeyCode) = (KeyModifiers::CONTROL, KeyCode::Char('f'));

    pub const KEYBIND_UNDO: (KeyModifiers, KeyCode) = (KeyModifiers::CONTROL, KeyCode::Char('z'));

    pub const KEYBIND_REDO: (KeyModifiers, KeyCode) = (KeyModifiers::CONTROL, KeyCode::Char('y'));
}

//COMMAND BINDS
pub mod command_binds {
    pub const COMMAND_EXIT_DONT_SAVE: &str = "q";
    pub const COMMAND_SAVE_DONT_EXIT: &str = "w";
    pub const COMMAND_SAVE_AND_EXIT: &str = "wq";
    pub const COMMAND_HELP: &str = "h";
}
