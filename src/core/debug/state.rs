use super::{DebugLogger, PerformanceMetrics};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Main debug state - tracks entire application
#[derive(Debug)]
pub struct DebugState {
    pub enabled: bool,
    pub logger: DebugLogger,
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

#[derive(Clone, Debug)]
pub struct ClipboardSnapshot {
    pub content_preview: String,
    pub full_length: usize,
    pub line_count: usize,
}

#[derive(Clone, Debug)]
pub struct ActionSnapshot {
    pub action_type: String,
    pub description: String,
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
    pub cursor_pos: (usize, usize),
    pub buffer_lines: usize,
    pub buffer_content_preview: String,
    pub mode: String,
    pub active_area: String,

    //clipboard state
    pub clipboard_entries: Vec<ClipboardSnapshot>,
    pub clipboard_size: usize,

    //history state
    pub undo_stack: Vec<ActionSnapshot>,
    pub redo_stack: Vec<ActionSnapshot>,
    pub undo_depth: usize,
    pub redo_depth: usize,

    //performance at time
    pub frame_time: Duration,
    pub fps: f64,
    pub memory_usage: Option<u64>,
}

impl DebugState {
    pub fn new() -> Self {
        Self {
            enabled: false,
            logger: DebugLogger::new(1000),
            metrics: PerformanceMetrics::new(),
            snapshots: SnapshotHistory::new(50),
            capture_mode: CaptureMode::OnEvent,
        }
    }

    //TODO
    pub fn log(&mut self, level: super::LogLevel, message: impl Into<String>) {}

    //if debugging enabled then tick on metrics
    pub fn tick_frame(&mut self) {
        if self.enabled {
            self.metrics.tick()
        }
        return;
    }

    //TODO
    /// Update and potentially capture a snapshot
    pub fn update_and_maybe_snapshot(
        &mut self,
        app: &crate::core::app::App,
        trigger: Option<SnapshotTrigger>,
    ) {
    }

    //TODO
    ///Manually capture snapshot
    pub fn capture_snapshot(
        &mut self,
        app: &crate::core::app::App,
        trigger: Option<SnapshotTrigger>,
    ) {
    }

    pub fn clear_logs(&mut self) {
        self.logger.clear();
    }

    pub fn clear_snapshots(&mut self) {
        self.snapshots.clear();
    }

    pub fn set_capture_mode(&mut self, mode: CaptureMode) {
        self.capture_mode = mode;
        self.log(
            super::LogLevel::Info,
            format!("Capture mode set to: {:?}", mode),
        );
    }
}

//TODO need functionality
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
