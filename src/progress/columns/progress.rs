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

/// A column that shows `completed/total` counts.
///
/// When `total` is `None` (indeterminate task) and `show_speed` is enabled
/// (the default), renders the current speed as `N it/s` — matching rich's
/// behaviour for indeterminate progress.
#[derive(Debug, Clone)]
pub struct TaskProgressColumn {
    /// Separator between completed and total.
    pub separator: String,
    /// When `true` (default), indeterminate tasks show `N it/s` instead of
    /// `N/?`.
    pub show_speed: bool,
}

impl TaskProgressColumn {
    /// Create a new TaskProgressColumn with the default separator.
    pub fn new() -> Self {
        TaskProgressColumn {
            separator: "/".to_string(),
            show_speed: true,
        }
    }

    /// Builder: set the separator.
    #[must_use]
    pub fn with_separator(mut self, sep: &str) -> Self {
        self.separator = sep.to_string();
        self
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
        let completed = task.completed;
        match task.total {
            Some(t) => {
                let total_str = format!("{t}");
                Text::new(&format!("{completed}{}{total_str}", self.separator), style)
            }
            None => {
                // Indeterminate task: show speed when available and enabled.
                if self.show_speed {
                    if let Some(speed) = task.speed() {
                        let speed_str = format_speed_si(speed);
                        return Text::new(&format!("{completed} {speed_str} it/s"), style);
                    }
                }
                Text::new(&format!("{completed}{sep}?", sep = self.separator), style)
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
