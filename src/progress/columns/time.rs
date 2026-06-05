//! Time-related columns for progress bars.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::progress::task::TaskId;
use crate::progress::{format_time, ProgressColumn, Task};
use crate::style::Style;
use crate::text::Text;

/// A column that shows elapsed time as `[H:MM:SS]`.
#[derive(Debug, Clone)]
pub struct TimeElapsedColumn;

impl Default for TimeElapsedColumn {
    fn default() -> Self {
        Self
    }
}

impl ProgressColumn for TimeElapsedColumn {
    fn render(&self, task: &Task) -> Text {
        let elapsed = task.elapsed().unwrap_or(0.0);
        let formatted = format_time(elapsed);
        Text::new(&formatted, Style::parse("progress.elapsed"))
    }
}

/// A column that shows estimated remaining time as `[H:MM:SS]` or
/// `-:--:--` when the estimate is unavailable.
///
/// This column rate-limits its ETA calculation to at most every 0.5 s per
/// task (via [`max_refresh`](ProgressColumn::max_refresh)) and caches the
/// last rendered value so the ETA text is not recomputed on every tick.
pub struct TimeRemainingColumn {
    /// Whether to show compact format.
    pub compact: bool,
    /// Whether to show elapsed time when finished.
    pub elapsed_when_finished: bool,
    /// Per-task render cache: `task_id → (last_render_time_secs, cached_text)`.
    cache: Mutex<HashMap<TaskId, (f64, Text)>>,
}

impl std::fmt::Debug for TimeRemainingColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TimeRemainingColumn")
            .field("compact", &self.compact)
            .field("elapsed_when_finished", &self.elapsed_when_finished)
            .finish()
    }
}

impl Clone for TimeRemainingColumn {
    fn clone(&self) -> Self {
        TimeRemainingColumn {
            compact: self.compact,
            elapsed_when_finished: self.elapsed_when_finished,
            // Start with an empty cache in the clone.
            cache: Mutex::new(HashMap::new()),
        }
    }
}

impl TimeRemainingColumn {
    /// Create a new TimeRemainingColumn with default settings.
    pub fn new() -> Self {
        TimeRemainingColumn {
            compact: false,
            elapsed_when_finished: false,
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// Builder: enable compact format.
    #[must_use]
    pub fn with_compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    /// Builder: show elapsed time instead of `0:00` when the task finishes.
    #[must_use]
    pub fn with_elapsed_when_finished(mut self, elapsed_when_finished: bool) -> Self {
        self.elapsed_when_finished = elapsed_when_finished;
        self
    }

    /// Render the ETA text for a task (no caching — always recomputes).
    fn render_fresh(&self, task: &Task) -> Text {
        let style = Style::parse("progress.remaining");

        if task.finished() {
            if self.elapsed_when_finished {
                let elapsed = task.elapsed().unwrap_or(0.0);
                return Text::new(&format_time(elapsed), style);
            }
            return Text::new("0:00", style);
        }

        match task.time_remaining() {
            Some(remaining) if remaining.is_finite() => Text::new(&format_time(remaining), style),
            _ => Text::new("-:--:--", style),
        }
    }
}

impl Default for TimeRemainingColumn {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressColumn for TimeRemainingColumn {
    /// Rate-limit ETA recomputation to twice per second.
    fn max_refresh(&self) -> Option<f64> {
        Some(0.5)
    }

    fn render(&self, task: &Task) -> Text {
        // Use the task's elapsed time (or current wall time) as a clock for
        // cache staleness so that the cache is independent of the system clock.
        let now = task
            .elapsed()
            .unwrap_or_else(crate::progress::task::current_time_secs);

        let mut cache = self.cache.lock().unwrap();
        if let Some(&(last_time, ref cached)) = cache.get(&task.id) {
            if now - last_time < self.max_refresh().unwrap_or(0.5) {
                return cached.clone();
            }
        }

        let fresh = self.render_fresh(task);
        cache.insert(task.id, (now, fresh.clone()));
        fresh
    }
}
