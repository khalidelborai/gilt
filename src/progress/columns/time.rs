//! Time-related columns for progress bars.

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
        Text::new(
            &formatted,
            Style::parse("progress.elapsed").unwrap_or_else(|_| Style::null()),
        )
    }
}

/// A column that shows estimated remaining time as `[H:MM:SS]` or
/// `-:--:--` when the estimate is unavailable.
#[derive(Debug, Clone)]
pub struct TimeRemainingColumn {
    /// Whether to show compact format.
    pub compact: bool,
    /// Whether to show elapsed time when finished.
    pub elapsed_when_finished: bool,
}

impl TimeRemainingColumn {
    /// Create a new TimeRemainingColumn with default settings.
    pub fn new() -> Self {
        TimeRemainingColumn {
            compact: false,
            elapsed_when_finished: false,
        }
    }
}

impl Default for TimeRemainingColumn {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressColumn for TimeRemainingColumn {
    fn render(&self, task: &Task) -> Text {
        let style = Style::parse("progress.remaining").unwrap_or_else(|_| Style::null());

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
