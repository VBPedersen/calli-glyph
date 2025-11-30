//file related commands: save, open, write, etc.

use crate::core::app::App;
use crate::errors::command_errors::CommandError;
use std::collections::HashSet;

use crate::core::app::PendingState;
use crate::core::command_line::command::CommandFlag;
use crate::ui::popups::confirmation_popup::ConfirmationPopup;
use crate::ui::popups::popup::PopupResult;
use std::path::Path;

pub fn save_command(
    app: &mut App,
    args: Vec<String>,
    flags: HashSet<CommandFlag>,
) -> Result<(), CommandError> {
    let path = if !args.is_empty() {
        args.first().unwrap().clone()
    } else if let Some(current) = app.file_path.clone() {
        current
    } else {
        "untitled".to_string()
    };

    let new_content = app.editor.editor_content.join("\n");
    let path_ref = Path::new(&path);

    // If flag force is not inputted and file exists and is different, prompt confirmation
    if !flags.contains(&CommandFlag::Force) && path_ref.exists() {
        let has_changes = app.file_has_changes(new_content.clone(), path.clone());

        if has_changes.unwrap() && app.popup_result == PopupResult::None {
            let popup = Box::new(ConfirmationPopup::new("Confirm Overwrite of file"));
            app.open_popup(popup);
            app.pending_states.push(PendingState::Saving(path));
            return Ok(());
        }
    }
    //confirmation wasn't needed, try to save file,
    //if it fails return error else return Ok()
    if let Err(e) = app.save_to_path(path) {
        Err(CommandError::ExecutionFailed(format!(
            "failed to save file: {}",
            e
        )))
    } else {
        Ok(())
    }
}
