//help command

use crate::core::app::App;
use crate::core::command_line::command::CommandFlag;
use crate::core::help_registry::HelpRegistry;
use crate::errors::command_errors::CommandError;
use crate::ui::popups::help_popup::HelpPopup;
use std::collections::HashSet;
use std::sync::Arc;

/// Command handling for help menu browser, opens popup for browsing and viewing help pages
pub fn help_command(
    app: &mut App,
    args: Vec<String>,
    _flags: HashSet<CommandFlag>,
) -> Result<(), CommandError> {
    let registry = Arc::clone(&app.help_registry);
    let popup = match args.first() {
        Some(arg) => HelpPopup::focused(registry, args.first().unwrap()),
        None => HelpPopup::browse(registry),
    };
    app.open_popup(Box::new(popup));
    Ok(())
}
