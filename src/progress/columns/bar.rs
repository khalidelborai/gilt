//! Progress bar column for progress bars.

use std::sync::OnceLock;

use crate::console::{Console, Renderable};
use crate::progress::{ProgressColumn, Task};
use crate::progress_bar::ProgressBar;
use crate::text::Text;

/// A column that renders a progress bar.
///
/// The `Console` used to render the bar segments is constructed once and
/// cached so it is not re-allocated on every refresh tick.
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
    /// Cached `Console` for rendering, initialised lazily on first use.
    console_cache: OnceLock<Console>,
}

impl std::fmt::Debug for BarColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BarColumn")
            .field("bar_width", &self.bar_width)
            .field("style", &self.style)
            .field("complete_style", &self.complete_style)
            .field("finished_style", &self.finished_style)
            .field("pulse_style", &self.pulse_style)
            .finish()
    }
}

impl Clone for BarColumn {
    fn clone(&self) -> Self {
        BarColumn {
            bar_width: self.bar_width,
            style: self.style.clone(),
            complete_style: self.complete_style.clone(),
            finished_style: self.finished_style.clone(),
            pulse_style: self.pulse_style.clone(),
            // Do not clone the cached console; let it be rebuilt lazily.
            console_cache: OnceLock::new(),
        }
    }
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
            console_cache: OnceLock::new(),
        }
    }

    /// Builder: set bar width.
    #[must_use]
    pub fn with_bar_width(mut self, width: Option<usize>) -> Self {
        // Invalidate the cached console since the width changed.
        self.console_cache = OnceLock::new();
        self.bar_width = width;
        self
    }

    /// Get (or lazily initialise) the cached console for rendering.
    fn console(&self) -> &Console {
        self.console_cache.get_or_init(|| {
            Console::builder()
                .width(self.bar_width.unwrap_or(40))
                .color_system("truecolor")
                .build()
        })
    }
}

impl Default for BarColumn {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressColumn for BarColumn {
    fn render(&self, task: &Task) -> Text {
        // An unstarted task should show a pulsing bar (rich parity).
        let pulse = !task.started();
        let bar = ProgressBar::new()
            .with_total(task.total)
            .with_completed(task.completed)
            .with_width(self.bar_width)
            .with_style(&self.style)
            .with_complete_style(&self.complete_style)
            .with_finished_style(&self.finished_style)
            .with_pulse_style(&self.pulse_style)
            .with_pulse(pulse);

        // Use the cached console instead of constructing a new one each tick.
        let console = self.console();
        let opts = console.options();
        let segments = bar.gilt_console(console, &opts);

        let mut text = Text::empty();
        for seg in &segments {
            text.append_str(&seg.text, seg.style().cloned());
        }
        text.end = String::new();
        text
    }
}
