mod app_extensions;
mod logger;
mod metrics;
mod state;

pub use logger::{DebugLogger, LogEntry, LogLevel};
pub use metrics::PerformanceMetrics;
pub use state::{
    AppSnapshot, CaptureMode, DebugState, Selection, SnapshotHistory, SnapshotTrigger,
};
