//EDITOR SETTINGS
pub mod editor_settings {
    pub(crate) const TAB_WIDTH: u16 = 4;
}

// KEYBINDS

pub mod key_binds {
    use crossterm::event::{KeyCode, KeyModifiers};

    pub(crate) const KEYBIND_TOGGLE_AREA: (KeyModifiers, KeyCode) =
        (KeyModifiers::NONE, KeyCode::Esc);
    pub(crate) const KEYBIND_BACKSPACE: (KeyModifiers, KeyCode) =
        (KeyModifiers::NONE, KeyCode::Backspace);
    pub(crate) const KEYBIND_TAB: (KeyModifiers, KeyCode) = (KeyModifiers::NONE, KeyCode::Tab);
    pub(crate) const KEYBIND_ENTER: (KeyModifiers, KeyCode) = (KeyModifiers::NONE, KeyCode::Enter);
    pub(crate) const KEYBIND_DELETE: (KeyModifiers, KeyCode) =
        (KeyModifiers::NONE, KeyCode::Delete);

    //Cursor Movement
    pub(crate) const KEYBIND_UP: (KeyModifiers, KeyCode) = (KeyModifiers::NONE, KeyCode::Up);
    pub(crate) const KEYBIND_DOWN: (KeyModifiers, KeyCode) = (KeyModifiers::NONE, KeyCode::Down);
    pub(crate) const KEYBIND_LEFT: (KeyModifiers, KeyCode) = (KeyModifiers::NONE, KeyCode::Left);
    pub(crate) const KEYBIND_RIGHT: (KeyModifiers, KeyCode) = (KeyModifiers::NONE, KeyCode::Right);

    //Text Selection //move text selection cursor
    pub(crate) const KEYBIND_SELECTION_UP: (KeyModifiers, KeyCode) =
        (KeyModifiers::SHIFT, KeyCode::Up);
    pub(crate) const KEYBIND_SELECTION_DOWN: (KeyModifiers, KeyCode) =
        (KeyModifiers::SHIFT, KeyCode::Down);
    pub(crate) const KEYBIND_SELECTION_LEFT: (KeyModifiers, KeyCode) =
        (KeyModifiers::SHIFT, KeyCode::Left);
    pub(crate) const KEYBIND_SELECTION_RIGHT: (KeyModifiers, KeyCode) =
        (KeyModifiers::SHIFT, KeyCode::Right);

    //WITH MODIFIER AND CHAR  :: ShortCuts
    pub(crate) const KEYBIND_SAVE: (KeyModifiers, KeyCode) =
        (KeyModifiers::CONTROL, KeyCode::Char('s'));
    pub(crate) const KEYBIND_COPY: (KeyModifiers, KeyCode) =
        (KeyModifiers::CONTROL, KeyCode::Char('c'));
    pub(crate) const KEYBIND_CUT: (KeyModifiers, KeyCode) =
        (KeyModifiers::CONTROL, KeyCode::Char('x'));

    pub(crate) const KEYBIND_PASTE: (KeyModifiers, KeyCode) =
        (KeyModifiers::CONTROL, KeyCode::Char('f'));

    pub(crate) const KEYBIND_UNDO: (KeyModifiers, KeyCode) =
        (KeyModifiers::CONTROL, KeyCode::Char('z'));

    pub(crate) const KEYBIND_REDO: (KeyModifiers, KeyCode) =
        (KeyModifiers::CONTROL, KeyCode::Char('y'));
}

//COMMAND BINDS
pub mod command_binds {
    pub(crate) const COMMAND_EXIT_DONT_SAVE: &str = "q";
    pub(crate) const COMMAND_SAVE_DONT_EXIT: &str = "w";
    pub(crate) const COMMAND_SAVE_AND_EXIT: &str = "wq";
    pub(crate) const COMMAND_HELP: &str = "h";
}
