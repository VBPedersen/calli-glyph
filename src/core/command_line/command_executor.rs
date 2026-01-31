use crate::core::app::App;
use crate::core::command_line::command::Command;
use crate::core::command_line::commands;
use crate::errors::command_errors::CommandError;

pub fn execute_command(app: &mut App, command: Command) -> Result<(), CommandError> {
    match command {
        Command::Save { args, flags } => commands::file::save_command(app, args, flags),
        Command::SaveAndExit { args, flags } => {
            commands::quit::save_and_exit_command(app, args, flags)
        }
        Command::Quit { args, flags } => commands::quit::exit_command(app, args, flags),
        Command::Help => {
            // TODO: Show help popup or render help screen
            Ok(())
        }
        Command::Debug { args, flags } => {
            if let Err(e) = commands::debug::debug_command(app, args, flags) {
                return Err(e);
            }
            Ok(())
        }
        Command::Config { args, flags } => {
            if let Err(e) = commands::config::config_command(app, args, flags) {
                return Err(e);
            }
            Ok(())
        }
        Command::Plugin { name, args } => app
            .execute_plugin_command(&name, args)
            .map_err(|e| CommandError::ExecutionFailed(e.to_string())),
        Command::Unknown { name, args } => {
            // Try plugin system first, then fail
            log_info!("trying plugin exec: {}", name);
            app.execute_plugin_command(&name, args)
                .map_err(|_| CommandError::ExecutionFailed(name))
        }
    }
}
