use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct PerformanceMetrics {
    pub frame_times: VecDeque<Duration>,
    pub last_frame_time: Instant,
    pub event_count: u64,
    pub render_count: u64,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(120),
            last_frame_time: Instant::now(),
            event_count: 0,
            render_count: 0,
        }
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;

        self.frame_times.push_back(delta);
        if self.frame_times.len() > 120 {
            self.frame_times.pop_front();
        }
        self.render_count += 1;
    }

    pub fn avg_frame_time(&self) -> Duration {
        if self.frame_times.is_empty() {
            return Duration::from_secs(0);
        }
        let sum: Duration = self.frame_times.iter().sum();
        sum / self.frame_times.len() as u32
    }

    /// FPS calculated from avg frame time
    pub fn fps(&self) -> f64 {
        let avg = self.avg_frame_time();
        if avg.as_secs_f64() == 0.0 {
            return 0.0;
        }
        1.0 / avg.as_secs_f64()
    }

    pub fn record_event(&mut self) {
        self.event_count += 1;
    }

    pub fn max_frame_time(&self) -> Duration {
        self.frame_times.iter().max().unwrap().clone()
    }

    pub fn min_frame_time(&self) -> Duration {
        self.frame_times.iter().min().unwrap().clone()
    }

    pub fn reset(&mut self) {
        self.frame_times.clear();
        self.last_frame_time = Instant::now();
        self.event_count = 0;
        self.render_count = 0;
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}
