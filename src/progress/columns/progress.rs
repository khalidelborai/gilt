//! Progress count columns for progress bars.

use crate::progress::{ProgressColumn, Task};
use crate::style::Style;
use crate::text::Text;

/// A column that shows `completed/total` counts.
#[derive(Debug, Clone)]
pub struct TaskProgressColumn {
    /// Separator between completed and total.
    pub separator: String,
}

impl TaskProgressColumn {
    /// Create a new TaskProgressColumn with the default separator.
    pub fn new() -> Self {
        TaskProgressColumn {
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

impl Default for TaskProgressColumn {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressColumn for TaskProgressColumn {
    fn render(&self, task: &Task) -> Text {
        let style = Style::parse("progress.percentage").unwrap_or_else(|_| Style::null());
        let completed = task.completed;
        let total_str = match task.total {
            Some(t) => format!("{t}"),
            None => "?".to_string(),
        };
        Text::new(&format!("{completed}{}{total_str}", self.separator), style)
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
        let style = Style::parse("progress.percentage").unwrap_or_else(|_| Style::null());
        Text::new(&format!("{completed}{}{total_str}", self.separator), style)
    }
}
