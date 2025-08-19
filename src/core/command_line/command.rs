use crate::config::command_binds::*;
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
    QuitForce,
    Help,
    Unknown {
        name: String,
        args: Vec<String>,
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
    let (args, flags) = parse_flags_and_args(raw_args);

    match bind.as_str() {
        COMMAND_SAVE_DONT_EXIT => Command::Save { args, flags },
        COMMAND_SAVE_AND_EXIT => Command::SaveAndExit { args, flags },
        COMMAND_EXIT_DONT_SAVE => Command::QuitForce,
        COMMAND_HELP => Command::Help,
        _ => Command::Unknown { name: bind, args },
    }
}
