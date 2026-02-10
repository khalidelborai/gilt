//! File size columns for progress bars.

use crate::progress::{ProgressColumn, Task};
use crate::style::Style;
use crate::text::Text;
use crate::utils::filesize;

/// A column that shows the completed amount as a human-readable file size.
#[derive(Debug, Clone)]
pub struct FileSizeColumn;

impl Default for FileSizeColumn {
    fn default() -> Self {
        Self
    }
}

impl ProgressColumn for FileSizeColumn {
    fn render(&self, task: &Task) -> Text {
        let size = task.completed as u64;
        let formatted = filesize::decimal(size, 1, " ");
        Text::new(
            &formatted,
            Style::parse("progress.filesize").unwrap_or_else(|_| Style::null()),
        )
    }
}

/// A column that shows the total as a human-readable file size.
#[derive(Debug, Clone)]
pub struct TotalFileSizeColumn;

impl Default for TotalFileSizeColumn {
    fn default() -> Self {
        Self
    }
}

impl ProgressColumn for TotalFileSizeColumn {
    fn render(&self, task: &Task) -> Text {
        let size = task.total.unwrap_or(0.0) as u64;
        let formatted = filesize::decimal(size, 1, " ");
        Text::new(
            &formatted,
            Style::parse("progress.filesize.total").unwrap_or_else(|_| Style::null()),
        )
    }
}
