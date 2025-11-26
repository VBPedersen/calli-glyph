use std::collections::VecDeque;
use std::time::{Duration, Instant};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

#[derive(Debug)]
pub struct PerformanceMetrics {
    pub frame_times: VecDeque<Duration>,
    pub last_frame_time: Instant,
    pub event_count: u64,
    pub render_count: u64,

    // System monitoring
    system: System,
    pid: Pid,
    pub memory_usage_kb: u64,
    pub cpu_usage: f32,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(120),
            last_frame_time: Instant::now(),
            event_count: 0,
            render_count: 0,
            system: System::new_all(),
            pid: sysinfo::get_current_pid().unwrap(),
            memory_usage_kb: 0,
            cpu_usage: 0.0,
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

        // Update system stats every 10 renders
        if self.render_count % 10 == 0 {
            self.update_system_stats();
        }
    }

    /// updates all system related info
    fn update_system_stats(&mut self) {
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[self.pid]),
            true, //if process gone or dead remove from list
            ProcessRefreshKind::new().with_cpu().with_memory(),
        );

        if let Some(process) = self.system.process(self.pid) {
            self.memory_usage_kb = process.memory() / 1024; // Convert to KB
            self.cpu_usage = process.cpu_usage();
        }
    }

    // get mem usage in mb from kb
    pub fn memory_usage_mb(&self) -> f64 {
        self.memory_usage_kb as f64 / 1024.0
    }

    pub fn avg_frame_time(&self) -> Duration {
        if self.frame_times.is_empty() {
            return Duration::from_secs(0);
        }
        let sum: Duration = self.frame_times.iter().sum();
        sum / self.frame_times.len() as u32
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
