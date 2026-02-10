//! Task tracking types for progress bars.

use std::collections::VecDeque;
use std::time::SystemTime;

// ---------------------------------------------------------------------------
// TaskId
// ---------------------------------------------------------------------------

/// Unique identifier for a progress task.
pub type TaskId = usize;

// ---------------------------------------------------------------------------
// ProgressSample
// ---------------------------------------------------------------------------

/// A timestamped progress measurement used for speed calculation.
#[derive(Debug, Clone)]
pub struct ProgressSample {
    /// Seconds since the UNIX epoch when this sample was taken.
    pub timestamp: f64,
    /// Cumulative completed count at the time of this sample.
    pub completed: f64,
}

// ---------------------------------------------------------------------------
// Task
// ---------------------------------------------------------------------------

/// A tracked unit of work within a [`Progress`](crate::progress::Progress) display.
///
/// Each task has a description, optional total, and records progress
/// samples for speed estimation.
#[derive(Debug, Clone)]
pub struct Task {
    /// Unique identifier for this task.
    pub id: TaskId,
    /// Human-readable description shown in the progress display.
    pub description: String,
    /// Total number of steps (None = indeterminate).
    pub total: Option<f64>,
    /// Number of steps completed so far.
    pub completed: f64,
    /// Whether this task is visible in the display.
    pub visible: bool,
    /// Arbitrary key-value fields for template substitution.
    pub fields: std::collections::HashMap<String, String>,
    /// Time when this task was started (seconds since epoch).
    pub start_time: Option<f64>,
    /// Time when this task was stopped.
    pub stop_time: Option<f64>,
    /// Time when this task was marked finished.
    pub finished_time: Option<f64>,
    /// Cached speed at finish time.
    pub finished_speed: Option<f64>,
    /// Sliding window of samples for speed calculation.
    pub samples: VecDeque<ProgressSample>,
    /// All recorded progress samples.
    progress: Vec<ProgressSample>,
}

impl Task {
    /// Create a new task with the given id, description, and optional total.
    pub fn new(id: TaskId, description: &str, total: Option<f64>) -> Self {
        Task {
            id,
            description: description.to_string(),
            total,
            completed: 0.0,
            visible: true,
            fields: std::collections::HashMap::new(),
            start_time: None,
            stop_time: None,
            finished_time: None,
            finished_speed: None,
            samples: VecDeque::new(),
            progress: Vec::new(),
        }
    }

    /// Whether this task has been started.
    pub fn started(&self) -> bool {
        self.start_time.is_some()
    }

    /// Whether this task has been finished.
    pub fn finished(&self) -> bool {
        self.finished_time.is_some()
    }

    /// The remaining work (total - completed), if total is known.
    pub fn remaining(&self) -> Option<f64> {
        self.total.map(|t| (t - self.completed).max(0.0))
    }

    /// Elapsed time in seconds since the task was started.
    pub fn elapsed(&self) -> Option<f64> {
        self.start_time.map(|start| {
            let end = self.stop_time.unwrap_or_else(current_time_secs);
            (end - start).max(0.0)
        })
    }

    /// Percentage complete (0..100). Returns 0.0 if total is None or zero.
    pub fn percentage(&self) -> f64 {
        match self.total {
            Some(total) if total > 0.0 => ((self.completed / total) * 100.0).clamp(0.0, 100.0),
            _ => 0.0,
        }
    }

    /// Calculate speed from the sliding window of samples.
    ///
    /// Returns the average rate of change per second computed from the
    /// first and last samples in the window.
    pub fn speed(&self) -> Option<f64> {
        if self.finished() {
            return self.finished_speed;
        }
        if self.samples.len() < 2 {
            return None;
        }
        let first = self.samples.front().expect("samples has >= 2 elements");
        let last = self.samples.back().expect("samples has >= 2 elements");
        let time_delta = last.timestamp - first.timestamp;
        if time_delta <= 0.0 {
            return None;
        }
        let completed_delta = last.completed - first.completed;
        Some(completed_delta / time_delta)
    }

    /// Estimated time remaining in seconds, based on current speed.
    pub fn time_remaining(&self) -> Option<f64> {
        if self.finished() {
            return Some(0.0);
        }
        let remaining = self.remaining()?;
        let speed = self.speed()?;
        if speed <= 0.0 {
            return None;
        }
        Some(remaining / speed)
    }

    /// Record a progress sample for speed estimation.
    ///
    /// Samples older than `speed_estimate_period` seconds are pruned
    /// from the sliding window.
    pub(crate) fn record_sample(&mut self, timestamp: f64, speed_estimate_period: f64) {
        self.samples.push_back(ProgressSample {
            timestamp,
            completed: self.completed,
        });
        self.progress.push(ProgressSample {
            timestamp,
            completed: self.completed,
        });

        // Prune samples outside the estimation window.
        let cutoff = timestamp - speed_estimate_period;
        while let Some(front) = self.samples.front() {
            if front.timestamp < cutoff {
                self.samples.pop_front();
            } else {
                break;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Time helpers
// ---------------------------------------------------------------------------

/// Return the current time as seconds since the UNIX epoch.
pub(crate) fn current_time_secs() -> f64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// Format a duration in seconds as `H:MM:SS`.
pub fn format_time(seconds: f64) -> String {
    let total = seconds.round() as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}
