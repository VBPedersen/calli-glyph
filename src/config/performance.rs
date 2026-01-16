use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PerformanceConfig {
    pub tick_rate_ms: u64,
    pub cursor_blink_rate_ms: u64,
    pub lazy_redraw: bool, // Only redraw on input
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            tick_rate_ms: 50,
            cursor_blink_rate_ms: 500,
            lazy_redraw: false,
        }
    }
}
