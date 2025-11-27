use crate::core::app::ActiveArea;
use crate::core::app::App;
use crate::core::command_line::command::CommandFlag;
use crate::core::cursor::CursorPosition;
use crate::core::debug::{CaptureMode, LogLevel, Selection};
use crate::core::errors::command_errors::CommandError;
use std::collections::HashSet;

enum DebugSubcommand {
    Enable,
    Disable,
    Console,
    Toggle,
    Clear,
    Snapshot,
    Reset,
    ClearSnapshots,
    ModeNone,
    ModeEvent,
    ModeManual,
    ModeFrame,
}

///Parses argument strings to sub command enum
fn parse_to_subcommand(args: Vec<String>) -> DebugSubcommand {
    if args.is_empty() {
        return DebugSubcommand::Toggle;
    }

    match args[0].as_str() {
        "enable" => DebugSubcommand::Enable,
        "disable" => DebugSubcommand::Disable,
        "console" => DebugSubcommand::Console,
        "toggle" => DebugSubcommand::Toggle,
        "clear" => DebugSubcommand::Clear,
        "snapshot" => DebugSubcommand::Snapshot,
        "reset" => DebugSubcommand::Reset,
        "clear-snapshots" => DebugSubcommand::ClearSnapshots,
        "mode-none" => DebugSubcommand::ModeNone,
        "mode-event" => DebugSubcommand::ModeEvent,
        "mode-manual" => DebugSubcommand::ModeManual,
        "mode-frame" => DebugSubcommand::ModeFrame,
        _ => DebugSubcommand::Toggle,
        //TODO want to make it so default with just :debug is toggle,
        // but if subcommand is not recognized CommandError popup
    }
}

pub fn debug_command(
    app: &mut App,
    args: Vec<String>,
    _flags: HashSet<CommandFlag>,
) -> Result<(), CommandError> {
    let sub_command = parse_to_subcommand(args);

    match sub_command {
        DebugSubcommand::Enable => {
            app.debug_state.enabled = true;
            app.debug_state.log(LogLevel::Info, "Debug enabled");
            Ok(())
        }
        DebugSubcommand::Disable => {
            app.debug_state.enabled = true;
            app.debug_state.log(LogLevel::Info, "Debug enabled");
            Ok(())
        }
        DebugSubcommand::Console => {
            if !app.debug_state.enabled {
                app.debug_state.enabled = true;
                app.debug_state.log(LogLevel::Info, "Debug mode enabled");
            }
            app.active_area = ActiveArea::DebugConsole;
            app.debug_state.log(LogLevel::Info, "Debug console opened");
            Ok(())
        }
        DebugSubcommand::Toggle => {
            if app.active_area == ActiveArea::DebugConsole {
                app.active_area = ActiveArea::Editor;
            } else {
                //makes no sense to navigate to debug console if not enabled
                if !app.debug_state.enabled {
                    app.debug_state.enabled = true;
                }
                app.active_area = ActiveArea::DebugConsole;
            }
            Ok(())
        }
        DebugSubcommand::Clear => {
            app.debug_state.clear_logs();
            app.debug_state.log(LogLevel::Info, "Logs Cleared");
            Ok(())
        }
        DebugSubcommand::Snapshot => {
            app.debug_state.capture_manual_snapshot(
                app.active_area,
                app.editor.cursor,
                Some(Selection {
                    start: app
                        .editor
                        .text_selection_start
                        .unwrap_or(CursorPosition::default()),
                    end: app
                        .editor
                        .text_selection_end
                        .unwrap_or(CursorPosition::default()),
                }),
                app.editor.editor_content.clone(),
                app.editor.scroll_offset,
                app.editor.clipboard.copied_text.clone(),
                app.editor.undo_redo_manager.undo_stack.clone(),
                app.editor.undo_redo_manager.redo_stack.clone(),
                app.file_path.clone(),
            );
            app.debug_state
                .log(LogLevel::Info, "Manual snapshot Captured");
            Ok(())
        }
        DebugSubcommand::Reset => {
            app.debug_state.metrics.reset();
            app.debug_state.log(LogLevel::Info, "Metrics Reset");
            Ok(())
        }
        DebugSubcommand::ClearSnapshots => {
            app.debug_state.clear_snapshots();
            app.debug_state.log(LogLevel::Info, "Snapshots cleared");
            Ok(())
        }
        DebugSubcommand::ModeNone => {
            app.debug_state.set_capture_mode(CaptureMode::None);
            Ok(())
        }
        DebugSubcommand::ModeEvent => {
            app.debug_state.set_capture_mode(CaptureMode::OnEvent);
            Ok(())
        }
        DebugSubcommand::ModeManual => {
            app.debug_state.set_capture_mode(CaptureMode::Manual);
            Ok(())
        }
        DebugSubcommand::ModeFrame => {
            app.debug_state.set_capture_mode(CaptureMode::EveryFrame);
            Ok(())
        }
    }
}
