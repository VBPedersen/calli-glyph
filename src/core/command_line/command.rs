use crate::core::command_line::command_binds::command_binds::*;
use std::collections::HashSet;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Save {
        args: Vec<String>,
        flags: HashSet<CommandFlag>,
    },
    SaveAndExit {
        args: Vec<String>,
        flags: HashSet<CommandFlag>,
    },
    Quit {
        args: Vec<String>,
        flags: HashSet<CommandFlag>,
    },
    Help,
    Unknown {
        name: String,
        args: Vec<String>,
    },
    //DEBUG
    Debug {
        args: Vec<String>,
        flags: HashSet<CommandFlag>,
    }, //DEBUG
    Config {
        args: Vec<String>,
        flags: HashSet<CommandFlag>,
    },
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum CommandFlag {
    Force,
    DryRun,
    Backup,
}

///function to parse flags and args to respective data structure
fn parse_flags_and_args(raw_args: Vec<String>) -> (Vec<String>, HashSet<CommandFlag>) {
    let mut args = Vec::new();
    let mut flags = HashSet::new();

    for arg in raw_args {
        match arg.as_str() {
            "--force" => {
                flags.insert(CommandFlag::Force);
            }
            "--dry-run" => {
                flags.insert(CommandFlag::DryRun);
            }
            "--backup" => {
                flags.insert(CommandFlag::Backup);
            }
            _ => args.push(arg),
        }
    }

    (args, flags)
}

///function to parse a command bind string to a Command enum, with possible arguments
pub fn parse_command(bind: String, raw_args: Vec<String>) -> Command {
    let (args, mut flags) = parse_flags_and_args(raw_args);

    match bind.as_str() {
        COMMAND_SAVE_DONT_EXIT => Command::Save { args, flags },
        COMMAND_SAVE_DONT_EXIT_FORCE => {
            // TODO considering if shorthand flags should just be registered like this
            flags.insert(CommandFlag::Force);
            Command::Save { args, flags }
        }
        COMMAND_SAVE_AND_EXIT => Command::SaveAndExit { args, flags },
        COMMAND_EXIT_DONT_SAVE => Command::Quit { args, flags },
        COMMAND_EXIT_DONT_SAVE_FORCE => {
            flags.insert(CommandFlag::Force);
            Command::Quit { args, flags }
        }
        COMMAND_HELP => Command::Help,
        COMMAND_DEBUG => Command::Debug { args, flags },
        COMMAND_CONFIG => Command::Config { args, flags },
        _ => Command::Unknown { name: bind, args },
    }
}
