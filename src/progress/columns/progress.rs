//! Progress count columns for progress bars.

use crate::progress::{ProgressColumn, Task};
use crate::style::Style;
use crate::text::Text;

/// Format a speed value as a compact SI string (e.g. `1.2k`, `3.5M`).
fn format_speed_si(speed: f64) -> String {
    if speed >= 1_000_000.0 {
        format!("{:.1}M", speed / 1_000_000.0)
    } else if speed >= 1_000.0 {
        format!("{:.1}k", speed / 1_000.0)
    } else {
        format!("{:.1}", speed)
    }
}

/// A column that shows percentage complete as `N%`.
///
/// When `total` is `None` (indeterminate task) and `show_speed` is enabled
/// (the default), renders the current speed as `N it/s` — matching rich's
/// behaviour for indeterminate progress.
///
/// This replaces the previous `completed/total` rendering with the
/// `percentage` format that rich uses for `TaskProgressColumn` (P2 fix).
#[derive(Debug, Clone)]
pub struct TaskProgressColumn {
    /// When `true` (default), indeterminate tasks show `N it/s` instead of
    /// nothing.
    pub show_speed: bool,
}

impl TaskProgressColumn {
    /// Create a new TaskProgressColumn.
    pub fn new() -> Self {
        TaskProgressColumn { show_speed: true }
    }

    /// Builder: control whether indeterminate tasks show speed.
    #[must_use]
    pub fn with_show_speed(mut self, show_speed: bool) -> Self {
        self.show_speed = show_speed;
        self
    }
}

impl Default for TaskProgressColumn {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressColumn for TaskProgressColumn {
    fn render(&self, task: &Task) -> Text {
        let style = Style::parse("progress.percentage");
        match task.total {
            Some(_) => {
                // Determinate: show percentage (rich parity).
                let pct = task.percentage();
                Text::new(&format!("{pct:.0}%"), style)
            }
            None => {
                // Indeterminate task: show speed when available and enabled.
                if self.show_speed {
                    if let Some(speed) = task.speed() {
                        let speed_str = format_speed_si(speed);
                        return Text::new(&format!("{} it/s", speed_str), style);
                    }
                }
                Text::new("--%", style)
            }
        }
    }
}

/// A column that shows `M/N` with optional separator customization.
#[derive(Debug, Clone)]
pub struct MofNCompleteColumn {
    /// Separator between M and N.
    pub separator: String,
}

impl MofNCompleteColumn {
    /// Create a new MofNCompleteColumn with the default `/` separator.
    pub fn new() -> Self {
        MofNCompleteColumn {
            separator: "/".to_string(),
        }
    }

    /// Builder: set the separator.
    #[must_use]
    pub fn with_separator(mut self, sep: &str) -> Self {
        self.separator = sep.to_string();
        self
    }
}

impl Default for MofNCompleteColumn {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressColumn for MofNCompleteColumn {
    fn render(&self, task: &Task) -> Text {
        let completed = task.completed as u64;
        let total_str = match task.total {
            Some(t) => format!("{}", t as u64),
            None => "?".to_string(),
        };
        // Pad the completed count to the same width as the total so the
        // separator stays aligned as progress advances (rich parity).
        let completed_str = format!("{:>width$}", completed, width = total_str.len());
        let style = Style::parse("progress.percentage");
        Text::new(
            &format!("{completed_str}{}{total_str}", self.separator),
            style,
        )
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// TaskProgressColumn must render percentage (e.g. "50%") not "50/100".
    #[test]
    fn task_progress_column_renders_percent() {
        let mut task = Task::new(0, "t", Some(100.0));
        task.completed = 50.0;
        let col = TaskProgressColumn::default();
        let rendered = col.render(&task).plain().to_string();
        assert_eq!(rendered, "50%", "should render '50%', got: {rendered}");
    }

    /// At 0% should render "0%".
    #[test]
    fn task_progress_column_renders_zero_percent() {
        let task = Task::new(0, "t", Some(100.0));
        let col = TaskProgressColumn::default();
        let rendered = col.render(&task).plain().to_string();
        assert_eq!(rendered, "0%", "should render '0%', got: {rendered}");
    }

    /// At 100% should render "100%".
    #[test]
    fn task_progress_column_renders_100_percent() {
        let mut task = Task::new(0, "t", Some(100.0));
        task.completed = 100.0;
        let col = TaskProgressColumn::default();
        let rendered = col.render(&task).plain().to_string();
        assert_eq!(rendered, "100%", "should render '100%', got: {rendered}");
    }

    /// Indeterminate task with no speed renders "--%".
    #[test]
    fn task_progress_column_indeterminate_no_speed() {
        let task = Task::new(0, "t", None);
        let col = TaskProgressColumn::default();
        let rendered = col.render(&task).plain().to_string();
        assert_eq!(
            rendered, "--%",
            "indeterminate should render '--%', got: {rendered}"
        );
    }
}
