// commands related to closing app, quit, exit, save_and_quit

use crate::core::app::App;
use crate::core::app::PendingState;
use crate::core::command_line::command::CommandFlag;
use crate::core::command_line::commands::file;
use crate::errors::command_errors::CommandError;
use crate::ui::popups::confirmation_popup::ConfirmationPopup;
use crate::ui::popups::popup::PopupResult;
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
                app.pending_states.push_back(PendingState::QuittingAbsolute); // Add Quit to the queue
                return Ok(());
            }
            app.quit();

            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub(crate) fn exit_command(
    app: &mut App,
    args: Vec<String>,
    flags: HashSet<CommandFlag>,
) -> Result<(), CommandError> {
    // If flag force is not inputted
    if !flags.contains(&CommandFlag::Force) {
        if app.content_modified && app.popup_result == PopupResult::None {
            let popup = Box::new(ConfirmationPopup::new(
                "YOU HAVE UNSAVED CHANGES, Confirm quit",
            ));
            app.open_popup(popup);
            app.pending_states.push_back(PendingState::Quitting);
            return Ok(());
        }
    }
    app.quit();

    Ok(())
}
