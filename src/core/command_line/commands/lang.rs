use crate::core::app::App;
use crate::core::command_line::command::CommandFlag;
use crate::errors::command_errors::CommandError;
use crate::ui::modal::lang_modal::lang_panel::LangPanel;
use std::collections::HashSet;

enum LangSubcommand {
    Open,
    Restart,
    Clear,
}

///Parses argument strings to sub command enum
fn parse_to_subcommand(args: Vec<String>) -> LangSubcommand {
    if args.is_empty() {
        return LangSubcommand::Open;
    }

    match args[0].as_str() {
        "open" => LangSubcommand::Open,
        "restart" | "reset" => LangSubcommand::Restart,
        "clear" => LangSubcommand::Clear,
        _ => LangSubcommand::Open,
    }
}

/// Executes DebugSubcommands of the debug main command
pub fn lang_command(
    app: &mut App,
    args: Vec<String>,
    _flags: HashSet<CommandFlag>,
) -> Result<(), CommandError> {
    let sub_command = parse_to_subcommand(args);

    match sub_command {
        LangSubcommand::Open => {
            app.modal_stack
                .push(Box::new(LangPanel::new()));
            Ok(())
        }
        LangSubcommand::Restart => {
            app.language.restart();
            log_info!("Language System Restarted");
            Ok(())
        }
        LangSubcommand::Clear => {
            app.language.clear();
            log_info!("Language System Cleared");
            Ok(())
        }
    }
}
