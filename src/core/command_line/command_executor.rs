use crate::core::app::App;
use crate::core::command_line::command::Command;
use crate::core::command_line::commands;
use crate::core::errors::command_errors::CommandError;

pub fn execute_command(app: &mut App, command: Command) -> Result<(), CommandError> {
    match command {
        Command::Save { args, flags } => commands::file::save_command(app, args, flags),
        Command::SaveAndExit { args, flags } => {
            commands::quit::save_and_exit_command(app, args, flags)
        }
        Command::QuitForce => {
            app.quit();
            Ok(())
        }
        Command::Help => {
            // TODO: Show help popup or render help screen
            Ok(())
        }
        Command::Debug => {
            app.toggle_debug();
            Ok(())
        }
        Command::Unknown { name, .. } => Err(CommandError::UnknownCommand(name)),
    }
}
