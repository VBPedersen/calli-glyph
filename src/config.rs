
//EDITOR SETTINGS
pub mod editor_settings{
    pub(crate) const TAB_WIDTH:u16 = 4;
}



// KEYBINDS

pub mod key_binds{
    use crossterm::event::{KeyCode, KeyModifiers};

    pub(crate) const KEYBIND_TAB:(KeyModifiers, KeyCode) = (KeyModifiers::NONE, KeyCode::Tab);
    pub(crate) const KEYBIND_ENTER:(KeyModifiers, KeyCode) = (KeyModifiers::NONE, KeyCode::Enter);
    pub(crate) const KEYBIND_DELETE:(KeyModifiers, KeyCode) = (KeyModifiers::NONE, KeyCode::Delete);
}




//COMMAND BINDS
pub mod command_binds{
    pub(crate) const COMMAND_EXIT_DONT_SAVE:&str = ":q";
    pub(crate) const COMMAND_SAVE_DONT_EXIT:&str = ":w";
}
