use crate::core::app::ActiveArea;
use crate::core::debug::{LogLevel, Selection};
use crate::input::input_action::InputAction;
use crate::core::app::App;
use crate::core::cursor::CursorPosition;

impl App{

    /// Toggle debug
    pub fn toggle_debug(&mut self) {
        use crate::core::debug::LogLevel;
        self.debug_state.enabled = !self.debug_state.enabled;

        if self.debug_state.enabled {
            self.active_area = ActiveArea::DebugConsole;
            self.debug_state.log(LogLevel::Info, "Debug mode activated");
        } else {
            self.active_area = ActiveArea::Editor;
            self.debug_state
                .log(LogLevel::Info, "Debug mode deactivated");
        }
    }

    pub fn handle_debug_input_action(&mut self, action: InputAction) {
        match action {
            // Debug actions
            InputAction::ExitDebug => {
                if self.active_area == ActiveArea::DebugConsole {
                    self.toggle_debug();
                }
            }

            InputAction::DebugNextTab => {
                self.debug_view.next_tab();
                self.debug_state.log(
                    LogLevel::Debug,
                    format!("Switched to tab: {:?}", self.debug_view.active_tab),
                );
            }

            InputAction::DebugPrevTab => {
                self.debug_view.prev_tab();
                self.debug_state.log(
                    LogLevel::Debug,
                    format!("Switched to tab: {:?}", self.debug_view.active_tab),
                );
            }

            InputAction::DebugScrollUp => {
                self.debug_view.scroll_up();
            }

            InputAction::DebugScrollDown => {
                self.debug_view.scroll_down();
            }

            InputAction::DebugClearLogs => {
                self.debug_state.clear_logs();
                self.debug_state.log(LogLevel::Info, "Logs cleared");
            }

            InputAction::DebugClearSnapshots => {
                self.debug_state.clear_snapshots();
                self.debug_state.log(LogLevel::Info, "Snapshots cleared");
            }
            InputAction::DebugManualSnapshot => {
                self.debug_state.capture_manual_snapshot(
                    self.active_area,
                    self.editor.cursor,
                    Some(Selection {
                        start: self.editor.text_selection_start.unwrap_or(CursorPosition::default()),
                        end: self.editor.text_selection_end.unwrap_or(CursorPosition::default()),
                    }), 
                    self.editor.editor_content.clone(),
                    self.editor.scroll_offset,
                    self.editor.clipboard.copied_text.clone(),
                    self.editor.undo_redo_manager.undo_stack.clone(),
                    self.editor.undo_redo_manager.undo_stack.clone(),
                    self.file_path.clone());
                self.debug_state
                    .log(LogLevel::Info, "Manual snapshot captured");
            }

            InputAction::DebugCycleMode => {
                use crate::core::debug::CaptureMode;
                self.debug_state.capture_mode = match self.debug_state.capture_mode {
                    CaptureMode::None => CaptureMode::OnEvent,
                    CaptureMode::OnEvent => CaptureMode::Manual,
                    CaptureMode::Manual => CaptureMode::EveryFrame,
                    CaptureMode::EveryFrame => CaptureMode::None,
                };
                self.debug_state.log(
                    LogLevel::Info,
                    format!("Capture mode: {:?}", self.debug_state.capture_mode),
                );
            }

            InputAction::DebugResetMetrics => {
                self.debug_state.metrics.reset();
                self.debug_state
                    .log(LogLevel::Info, "Performance metrics reset");
            }
            _ => {}
        }
    }
}