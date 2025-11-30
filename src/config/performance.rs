use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub tick_rate_ms: u64,
    pub cursor_blink_rate_ms: u64,
    pub undo_history_limit: usize,
    pub clipboard_history_limit: usize,
    pub lazy_redraw: bool, // Only redraw on input
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            tick_rate_ms: 50,
            cursor_blink_rate_ms: 500,
            undo_history_limit: 1000,
            clipboard_history_limit: 100,
            lazy_redraw: false,
        }
    }
}
