//TODO make command for loading, and reloading and related commands for config

use crate::core::app::App;
use crate::core::command_line::command::CommandFlag;
use crate::errors::command_errors::CommandError;
use std::collections::HashSet;

enum ConfigSubcommand {
    Reload,
    Reset,
    Edit,
    Show,
    Set { key: String, value: String },
    InvalidCommandArgument { name: String, args: Vec<String> },
}

///Parses argument strings to sub command enum
fn parse_to_subcommand(args: Vec<String>) -> ConfigSubcommand {
    if args.is_empty() {
        return ConfigSubcommand::Show;
    }

    match args[0].as_str() {
        "reload" => ConfigSubcommand::Reload,
        "reset" => ConfigSubcommand::Reset,
        "edit" => ConfigSubcommand::Edit,
        "show" => ConfigSubcommand::Show,
        "set" => {
            //since set requires key and value (2 args), just check args length,
            // if not long enough = invalid
            if args.len() >= 3 {
                ConfigSubcommand::Set {
                    key: String::from(args[1].clone()),
                    value: String::from(args[2].clone()),
                }
            } else {
                ConfigSubcommand::InvalidCommandArgument {
                    name: "Arguments for Set command invalid".to_string(),
                    args,
                }
            }
        }
        _ => ConfigSubcommand::InvalidCommandArgument {
            name: "Argument for base config command unrecognized".to_string(),
            args,
        },
    }
}

pub fn config_command(
    app: &mut App,
    args: Vec<String>,
    _flags: HashSet<CommandFlag>,
) -> Result<(), CommandError> {
    let sub_command = parse_to_subcommand(args);

    match sub_command {
        ConfigSubcommand::Reload => reload_config_command(app),
        ConfigSubcommand::Reset => reset_config_command(app),
        ConfigSubcommand::Edit => edit_config_command(app),
        ConfigSubcommand::Show => show_config_command(app),
        ConfigSubcommand::Set { key, value } => set_config_command(app, key, value),
        ConfigSubcommand::InvalidCommandArgument { name, args } => {
            Err(CommandError::InvalidArguments {
                command: format!("InvalidArguments: {}", name),
                reason: format!("Args: {:?}", args),
            })
        }
    }
}

///Reloads config, rebuilds keymaps and applies to running app
pub fn reload_config_command(app: &mut App) -> Result<(), CommandError> {
    match app.config.reload() {
        Ok(_) => {
            //TODO
            // Rebuild runtime keymaps
            // Apply config changes to running app
            Ok(())
        }
        Err(e) => Err(CommandError::ExecutionFailed(format!(
            "Failed to reload config: {}",
            e
        ))),
    }
}

pub fn reset_config_command(app: &mut App) -> Result<(), CommandError> {
    match app.config.reload() {
        Ok(_) => {
            //TODO
            //
            Ok(())
        }
        Err(e) => Err(CommandError::ExecutionFailed(format!(
            "Failed to reset config: {}",
            e
        ))),
    }
}

pub fn edit_config_command(app: &mut App) -> Result<(), CommandError> {
    match app.config.reload() {
        Ok(_) => {
            //TODO
            // Open config file inside editor
            Ok(())
        }
        Err(e) => Err(CommandError::ExecutionFailed(format!(
            "Failed to edit config: {}",
            e
        ))),
    }
}

pub fn show_config_command(app: &mut App) -> Result<(), CommandError> {
    match app.config.reload() {
        Ok(_) => {
            //TODO
            // show config possibly in previewer popup or something
            Ok(())
        }
        Err(e) => Err(CommandError::ExecutionFailed(format!(
            "Failed to show config: {}",
            e
        ))),
    }
}

pub fn set_config_command(app: &mut App, key: String, value: String) -> Result<(), CommandError> {
    match app.config.reload() {
        Ok(_) => {
            //TODO
            // set specific key of config temporarily in app memory to value
            Ok(())
        }
        Err(e) => Err(CommandError::ExecutionFailed(format!(
            "Failed to set config key value: {}",
            e
        ))),
    }
}
