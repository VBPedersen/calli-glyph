// commands related to closing app, quit, exit, save_and_quit

use crate::core::app::App;
use crate::core::app::PendingState;
use crate::core::command_line::command::CommandFlag;
use crate::core::command_line::commands::file;
use crate::errors::command_errors::CommandError;
use std::collections::HashSet;

pub(crate) fn save_and_exit_command(
    app: &mut App,
    args: Vec<String>,
    flags: HashSet<CommandFlag>,
) -> Result<(), CommandError> {
    match file::save_command(app, args, flags) {
        Ok(_) => {
            // If a save confirmation is needed, push Quit AFTER Saving
            if app
                .pending_states
                .iter()
                .any(|s| matches!(s, PendingState::Saving(_)))
            {
                app.pending_states.push_back(PendingState::Quitting); // Add Quit to the queue
                return Ok(());
            }
            app.quit();

            Ok(())
        }
        Err(e) => Err(e),
    }
}
