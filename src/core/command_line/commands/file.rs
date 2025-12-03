//file related commands: save, open, write, etc.

use crate::core::app::App;
use crate::errors::command_errors::CommandError;
use std::collections::HashSet;

use crate::core::app::PendingState;
use crate::core::command_line::command::CommandFlag;
use crate::ui::popups::confirmation_popup::ConfirmationPopup;
use crate::ui::popups::popup::PopupResult;
use std::path::{Path, PathBuf};

pub fn save_command(
    app: &mut App,
    args: Vec<String>,
    flags: HashSet<CommandFlag>,
) -> Result<(), CommandError> {
    let path_buf: PathBuf = if !args.is_empty() {
        PathBuf::from(args.first().unwrap())
    } else if let Some(current) = app.file_path.clone() {
        current.clone()
    } else {
        PathBuf::from("untitled")
    };

    let new_content = app.editor.editor_content.join("\n");
    let path_ref = Path::new(&path_buf);

    // If flag force is not inputted and file exists and is different, prompt confirmation
    if !flags.contains(&CommandFlag::Force) && path_ref.exists() {
        let has_changes = app
            .file_has_changes(new_content.clone(), path_ref)
            .map_err(|e| {
                CommandError::ExecutionFailed(format!(
                    "Failed when checking if file has changes {}",
                    e
                ))
            })?;

        if has_changes && app.popup_result == PopupResult::None {
            let popup = Box::new(ConfirmationPopup::new("Confirm Overwrite of file"));
            app.open_popup(popup);
            app.pending_states.push(PendingState::Saving(path_buf));
            return Ok(());
        }
    }
    //confirmation wasn't needed, try to save file,
    //if it fails return error else return Ok()
    if let Err(e) = app.save_to_path(path_ref) {
        Err(CommandError::ExecutionFailed(format!(
            "failed to save file: {}",
            e
        )))
    } else {
        Ok(())
    }
}
