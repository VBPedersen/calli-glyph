mod app_extensions;
mod logger;
mod metrics;
mod state;

use chrono::Local;
pub use logger::{DebugLogger, LogEntry, LogLevel};
pub use metrics::PerformanceMetrics;
use once_cell::sync::Lazy;
pub use state::{
    AppSnapshot, CaptureMode, DebugState, Selection, SnapshotHistory, SnapshotTrigger,
};
use std::sync::Mutex;
use std::time::Instant;

/// Global Debug instance
pub static GLOBAL_LOGGER: Lazy<Mutex<DebugLogger>> =
    Lazy::new(|| Mutex::new(DebugLogger::new(1000)));

//TODO Add logging function for other type of logging, which includes context

/// Global logging function
pub fn debug_log(level: LogLevel, message: impl Into<String>) {
    if let Ok(mut logger) = GLOBAL_LOGGER.lock() {
        logger.push(LogEntry::new(level, message.into(), None));
    }
}

/// Get all logs from startup (thread-safe, read-only)
pub fn get_all_logs() -> Vec<LogEntry> {
    GLOBAL_LOGGER
        .lock()
        .ok()
        .map(|logger| logger.entries().iter().cloned().collect())
        .unwrap_or_default()
}

/// Get logs filtered by level (thread-safe)
pub fn get_logs_by_level(level: LogLevel) -> Vec<LogEntry> {
    GLOBAL_LOGGER
        .lock()
        .ok()
        .map(|logger| {
            logger
                .entries()
                .iter()
                .filter(|e| e.level >= level)
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

/// Get count of logs by level
pub fn get_log_count_by_level(level: LogLevel) -> usize {
    GLOBAL_LOGGER
        .lock()
        .ok()
        .map(|logger| logger.count_by_level(level))
        .unwrap_or(0)
}

/// Get total number of logs
pub fn get_log_count() -> usize {
    GLOBAL_LOGGER
        .lock()
        .ok()
        .map(|logger| logger.len())
        .unwrap_or(0)
}

/// Clear all logs from global logger
pub fn clear_all_logs() {
    if let Ok(mut logger) = GLOBAL_LOGGER.lock() {
        logger.clear();
    }
}
