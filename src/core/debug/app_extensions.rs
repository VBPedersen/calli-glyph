use crate::core::app::ActiveArea;
use crate::core::app::App;
use crate::core::cursor::CursorPosition;
use crate::core::debug::Selection;
use crate::input::input_action::{DebugAction, InputAction};
use crate::ui::debug::DebugTab;

impl App {
    /// Toggle debug
    pub fn toggle_debug(&mut self) {
        self.debug_state.enabled = !self.debug_state.enabled;

        if self.debug_state.enabled {
            self.active_area = ActiveArea::DebugConsole;
            log_info!("Debug mode enabled");
        } else {
            self.active_area = ActiveArea::Editor;
            log_info!("Debug mode disabled");
        }
    }

    ///wrapper for parsing to debug action or reject
    pub fn handle_debug_input_action(&mut self, action: InputAction) {
        match action {
            InputAction::Debug(debug_action) => {
                self.handle_debug_action(debug_action);
            }
            _ => {}
        }
    }

    pub fn handle_debug_action(&mut self, action: DebugAction) {
        match action {
            // Debug actions
            DebugAction::ExitDebug => match self.debug_view.active_tab {
                DebugTab::SnapshotViewer => {
                    self.debug_view.close_snapshot_viewer();
                }
                _ => self.toggle_debug(),
            },

            DebugAction::DebugNextTab => {
                self.debug_view.next_tab();
            }

            DebugAction::DebugPrevTab => {
                self.debug_view.prev_tab();
            }

            DebugAction::DebugScrollUp => match self.debug_view.active_tab {
                DebugTab::Snapshots => {
                    let max = self.debug_state.snapshots.len();
                    self.debug_view.select_next_snapshot(max);
                }
                _ => self.debug_view.scroll_up(),
            },

            DebugAction::DebugScrollDown => match self.debug_view.active_tab {
                DebugTab::Snapshots => {
                    let max = self.debug_state.snapshots.len();
                    self.debug_view.select_prev_snapshot(max);
                }
                _ => self.debug_view.scroll_down(),
            },

            DebugAction::DebugClearLogs => {
                self.debug_state.clear_logs();
                log_info!("Logs cleared");
            }

            DebugAction::DebugClearSnapshots => {
                self.debug_state.clear_snapshots();
                log_info!("Snapshots cleared");
            }
            DebugAction::DebugManualSnapshot => {
                self.debug_state.capture_manual_snapshot(
                    self.active_area,
                    self.editor.cursor,
                    Some(Selection {
                        start: self
                            .editor
                            .text_selection_start
                            .unwrap_or(CursorPosition::default()),
                        end: self
                            .editor
                            .text_selection_end
                            .unwrap_or(CursorPosition::default()),
                    }),
                    self.editor.editor_content.clone(),
                    self.editor.scroll_offset,
                    self.editor.clipboard.copied_text.clone(),
                    self.editor.undo_redo_manager.undo_stack.clone(),
                    self.editor.undo_redo_manager.undo_stack.clone(),
                    self.file_path.clone(),
                );
                log_info!("Manual snapshot captured");
            }

            DebugAction::DebugCycleMode => {
                use crate::core::debug::CaptureMode;
                self.debug_state.capture_mode = match self.debug_state.capture_mode {
                    CaptureMode::None => CaptureMode::OnEvent,
                    CaptureMode::OnEvent => CaptureMode::Manual,
                    CaptureMode::Manual => CaptureMode::EveryFrame,
                    CaptureMode::EveryFrame => CaptureMode::None,
                };
                log_info!("Capture mode: {:?}", self.debug_state.capture_mode);
            }

            DebugAction::DebugResetMetrics => {
                self.debug_state.metrics.reset();
                log_info!("Performance metrics cleared");
            }

            DebugAction::DebugViewSnapshot => {
                self.debug_view.open_snapshot_viewer();
            }

            DebugAction::DebugCloseSnapshotViewer => {
                if self.debug_view.viewing_snapshot {
                    self.debug_view.close_snapshot_viewer();
                } else {
                    self.toggle_debug(); // Exit debug if not viewing snapshot
                }
            }
        }
    }
}
