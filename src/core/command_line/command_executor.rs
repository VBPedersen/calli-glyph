use crate::config::command_binds;
use crate::core::app::App;
use crate::core::errors::command_errors::CommandError;

pub fn execute(app: &mut App, command : String, args: Vec<String>) -> Result<(), CommandError> {
    match command.as_ref() {
        command_binds::COMMAND_EXIT_DONT_SAVE => Ok(app.quit()),
        command_binds::COMMAND_SAVE_DONT_EXIT => {
            Ok(app.save(args).expect("save failed"))
        }
        command_binds::COMMAND_SAVE_AND_EXIT => {
            Ok(app.save_and_exit(args)
                .expect("save and exit failed"))
        }
        command_binds::COMMAND_HELP => {Ok(())}
        _ => {Ok(())}
    }
}
