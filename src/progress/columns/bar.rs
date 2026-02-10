//! Progress bar column for progress bars.

use crate::console::{Console, Renderable};
use crate::progress::{ProgressColumn, Task};
use crate::progress_bar::ProgressBar;
use crate::text::Text;

/// A column that renders a progress bar.
#[derive(Debug, Clone)]
pub struct BarColumn {
    /// Fixed width of the bar, or None for flexible sizing.
    pub bar_width: Option<usize>,
    /// Style for the bar background.
    pub style: String,
    /// Style for the completed portion.
    pub complete_style: String,
    /// Style for a finished bar.
    pub finished_style: String,
    /// Style for pulse animation.
    pub pulse_style: String,
}

impl BarColumn {
    /// Create a new BarColumn with default styles.
    pub fn new() -> Self {
        BarColumn {
            bar_width: Some(40),
            style: "bar.back".to_string(),
            complete_style: "bar.complete".to_string(),
            finished_style: "bar.finished".to_string(),
            pulse_style: "bar.pulse".to_string(),
        }
    }

    /// Builder: set bar width.
    #[must_use]
    pub fn with_bar_width(mut self, width: Option<usize>) -> Self {
        self.bar_width = width;
        self
    }
}

impl Default for BarColumn {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressColumn for BarColumn {
    fn render(&self, task: &Task) -> Text {
        let bar = ProgressBar::new()
            .with_total(task.total)
            .with_completed(task.completed)
            .with_width(self.bar_width)
            .with_style(&self.style)
            .with_complete_style(&self.complete_style)
            .with_finished_style(&self.finished_style)
            .with_pulse_style(&self.pulse_style);

        // Render the bar through the Renderable trait to get segments,
        // then convert to text.
        let console = Console::builder()
            .width(self.bar_width.unwrap_or(40))
            .color_system("truecolor")
            .build();
        let opts = console.options();
        let segments = bar.gilt_console(&console, &opts);

        let mut text = Text::empty();
        for seg in &segments {
            text.append_str(&seg.text, seg.style.clone());
        }
        text.end = String::new();
        text
    }
}
