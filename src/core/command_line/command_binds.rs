//COMMAND BINDS, TODO for now it is fine but seriously needs betterment, maybe integrate with config system
/// Module for containing command bind consts
pub mod command_binds {
    pub const COMMAND_EXIT_DONT_SAVE: &[&str] = &["q", "quit"];

    pub const COMMAND_EXIT_DONT_SAVE_FORCE: &[&str] = &["q!", "quit!"];
    pub const COMMAND_SAVE_DONT_EXIT: &[&str] = &["w", "write", "save"];

    pub const COMMAND_SAVE_DONT_EXIT_FORCE: &[&str] = &["w!", "write!", "save!"];
    pub const COMMAND_SAVE_AND_EXIT: &[&str] = &["wq", "writequit"];
    pub const COMMAND_HELP: &[&str] = &["h", "help"];

    pub const COMMAND_DEBUG: &[&str] = &["debug", "dbg"];
    pub const COMMAND_CONFIG: &[&str] = &["config", "cfg"];
}
