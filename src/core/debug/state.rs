use super::PerformanceMetrics;
use crate::core::app::ActiveArea;
use crate::core::cursor::{Cursor, CursorPosition};
use crate::core::editor::editor::EditAction;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Main debug state - tracks entire application
#[derive(Debug)]
pub struct DebugState {
    pub enabled: bool,
    //pub logger: DebugLogger,
    pub metrics: PerformanceMetrics,
    pub snapshots: SnapshotHistory,
    pub capture_mode: CaptureMode,
}

#[derive(Debug, Clone)]
pub enum SnapshotTrigger {
    Error(String),
    Command(String),
    KeyPress(String),
    Manual,
    PeriodicSnap,
}

///mode set determines
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureMode {
    None,       // No snapshots
    OnEvent,    // Snapshot on specific events
    EveryFrame, // Snapshot every frame
    Manual,     // Only when explicitly requested
}

///history of snapshots
#[derive(Debug)]
pub struct SnapshotHistory {
    snapshots: VecDeque<AppSnapshot>,
    max_snapshots: usize,
}
/// Snapshot of current application state
#[derive(Debug)]
pub struct AppSnapshot {
    pub timestamp: Instant,
    pub trigger: SnapshotTrigger,

    //editor state
    pub cursor_pos: Cursor,
    pub selection: Option<Selection>,
    pub buffer_content: Vec<String>, // Full buffer content
    pub buffer_lines: usize,
    pub scroll_offset: i16,

    //clipboard state
    pub clipboard_entries: Vec<String>,
    pub clipboard_size: usize,

    //history state
    pub undo_stack: VecDeque<EditAction>,
    pub redo_stack: VecDeque<EditAction>,
    pub undo_depth: usize,
    pub redo_depth: usize,

    // App state
    pub active_area: String,
    pub file_path: Option<PathBuf>,

    //performance at time
    pub frame_time: Duration,
    pub memory_usage: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct Selection {
    pub start: CursorPosition,
    pub end: CursorPosition,
}

impl DebugState {
    pub fn new() -> Self {
        Self {
            enabled: false,
            //logger: DebugLogger::new(1000),
            metrics: PerformanceMetrics::new(),
            snapshots: SnapshotHistory::new(50),
            capture_mode: CaptureMode::OnEvent,
        }
    }

    //if debugging enabled then tick on metrics
    pub fn tick_frame(&mut self) {
        if self.enabled {
            self.metrics.tick()
        }
        return;
    }

    /// Update and potentially capture a snapshot (for background logging)
    pub fn update_and_maybe_snapshot(
        &mut self,
        active_area: ActiveArea,
        trigger: Option<SnapshotTrigger>,
        cursor_pos: Cursor,
        selection: Option<Selection>,
        buffer_content: Vec<String>,
        scroll_offset: i16,
        clipboard_entries: Vec<String>,
        undo_stack: VecDeque<EditAction>,
        redo_stack: VecDeque<EditAction>,
        file_path: Option<PathBuf>,
    ) {
        if !self.enabled {
            return;
        }

        let should_capture = match self.capture_mode {
            CaptureMode::None => false,
            CaptureMode::OnEvent => trigger.is_some(),
            CaptureMode::EveryFrame => true,
            CaptureMode::Manual => matches!(trigger, Some(SnapshotTrigger::Manual)),
        };

        if should_capture {
            self.capture_snapshot_internal(
                active_area,
                trigger.unwrap_or(SnapshotTrigger::PeriodicSnap),
                cursor_pos,
                selection,
                buffer_content,
                scroll_offset,
                clipboard_entries,
                undo_stack,
                redo_stack,
                file_path,
            );
        }
    }

    /// Always captures (for manual snapshots)
    pub fn capture_manual_snapshot(
        &mut self,
        active_area: ActiveArea,
        cursor_pos: Cursor,
        selection: Option<Selection>,
        buffer_content: Vec<String>,
        scroll_offset: i16,
        clipboard_entries: Vec<String>,
        undo_stack: VecDeque<EditAction>,
        redo_stack: VecDeque<EditAction>,
        file_path: Option<PathBuf>,
    ) {
        self.capture_snapshot_internal(
            active_area,
            SnapshotTrigger::Manual,
            cursor_pos,
            selection,
            buffer_content,
            scroll_offset,
            clipboard_entries,
            undo_stack,
            redo_stack,
            file_path,
        );
    }

    /// Internal helper that does the actual capturing
    fn capture_snapshot_internal(
        &mut self,
        active_area: ActiveArea,
        trigger: SnapshotTrigger,
        cursor_pos: Cursor,
        selection: Option<Selection>,
        buffer_content: Vec<String>,
        scroll_offset: i16,
        clipboard_entries: Vec<String>,
        undo_stack: VecDeque<EditAction>,
        redo_stack: VecDeque<EditAction>,
        file_path: Option<PathBuf>,
    ) {
        let snapshot = AppSnapshot {
            timestamp: Instant::now(),
            trigger,
            cursor_pos,
            selection,
            buffer_content: buffer_content.clone(),
            buffer_lines: buffer_content.len(),
            scroll_offset,
            clipboard_entries: clipboard_entries.clone(),
            clipboard_size: clipboard_entries.len(),
            undo_stack: undo_stack.clone(),
            redo_stack: redo_stack.clone(),
            undo_depth: undo_stack.len(),
            redo_depth: redo_stack.len(),
            active_area: format!("{:?}", active_area),
            file_path,
            frame_time: self.metrics.avg_frame_time(),
            memory_usage: Some(self.metrics.memory_usage_mb()),
        };
        self.snapshots.push(snapshot);
    }

    pub fn clear_logs(&mut self) {
        crate::core::debug::clear_all_logs();
    }

    pub fn clear_snapshots(&mut self) {
        self.snapshots.clear();
    }

    pub fn set_capture_mode(&mut self, mode: CaptureMode) {
        self.capture_mode = mode;
        log_info!("Capture mode set to: {:?}", mode);
        //self.log(LogLevel::Info, format!("Capture mode set to: {:?}", mode));
    }
}

impl SnapshotHistory {
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            snapshots: VecDeque::new(),
            max_snapshots,
        }
    }

    pub fn push(&mut self, snapshot: AppSnapshot) {
        if self.snapshots.len() >= self.max_snapshots {
            self.snapshots.pop_front();
        }
        self.snapshots.push_back(snapshot);
    }

    pub fn snapshots(&self) -> &VecDeque<AppSnapshot> {
        &self.snapshots
    }

    pub fn latest(&self) -> Option<&AppSnapshot> {
        self.snapshots.back()
    }

    pub fn get(&self, index: usize) -> Option<&AppSnapshot> {
        self.snapshots.get(index)
    }

    pub fn clear(&mut self) {
        self.snapshots.clear();
    }

    pub fn find_by_trigger(&self, trigger_type: &str) -> Vec<&AppSnapshot> {
        self.snapshots
            .iter()
            .filter(|s| format!("{:?}", s.trigger).contains(trigger_type))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }
}

impl Default for DebugState {
    fn default() -> Self {
        Self::new()
    }
}
