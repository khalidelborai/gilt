//! Spinner column for progress bars.

use crate::progress::{ProgressColumn, Task};
use crate::status::spinner::Spinner;
use crate::style::Style;
use crate::text::Text;

/// A column that renders a spinner animation.
#[derive(Debug, Clone)]
pub struct SpinnerColumn {
    /// Name of the spinner (from the SPINNERS registry).
    pub spinner_name: String,
    /// Style for the spinner frame.
    pub style: Option<Style>,
    /// Text shown when the task is finished.
    pub finished_text: Text,
}

impl SpinnerColumn {
    /// Create a new SpinnerColumn with the given spinner name.
    pub fn new(name: &str) -> Self {
        SpinnerColumn {
            spinner_name: name.to_string(),
            style: None,
            finished_text: Text::styled(
                "\u{2714}",
                Style::parse("green").unwrap_or_else(|_| Style::null()),
            ),
        }
    }

    /// Builder: set the style.
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Builder: set the finished text.
    #[must_use]
    pub fn with_finished_text(mut self, text: Text) -> Self {
        self.finished_text = text;
        self
    }
}

impl Default for SpinnerColumn {
    fn default() -> Self {
        Self::new("dots")
    }
}

impl ProgressColumn for SpinnerColumn {
    fn render(&self, task: &Task) -> Text {
        if task.finished() {
            return self.finished_text.clone();
        }

        let mut spinner = match Spinner::new(&self.spinner_name) {
            Ok(s) => s,
            Err(_) => return Text::new("?", Style::null()),
        };
        if let Some(ref style) = self.style {
            spinner = spinner.with_style(style.clone());
        }

        let elapsed = task.elapsed().unwrap_or(0.0);
        spinner.render(elapsed)
    }

    fn max_refresh(&self) -> Option<f64> {
        // Spinners typically update at ~12.5 FPS
        Some(0.08)
    }
}
