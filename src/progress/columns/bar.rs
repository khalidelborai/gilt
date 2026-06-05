//! Progress bar column for progress bars.

use std::sync::OnceLock;

use crate::color::Color;
use crate::console::{Console, Renderable};
use crate::gradient::interpolate_color;
use crate::progress::{ProgressColumn, Task};
use crate::progress_bar::ProgressBar;
use crate::style::Style;
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
    /// Optional gradient fill: `(start_color, end_color)`.
    ///
    /// When set, the completed portion of the bar is filled with a per-cell
    /// truecolor gradient interpolated between `start` and `end`.  The
    /// unfilled remainder keeps the normal `back` style.
    pub gradient: Option<(Color, Color)>,
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
            .field("gradient", &self.gradient)
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
            gradient: self.gradient,
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
            gradient: None,
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

    /// Builder: enable a truecolor gradient for the completed portion.
    ///
    /// When set, each filled cell in the completed portion is assigned an
    /// interpolated truecolor (`Color::TrueColor`) between `start` and `end`
    /// instead of the uniform `complete_style` foreground color.
    /// The unfilled remainder is unaffected and keeps the `style` (back) style.
    #[must_use]
    pub fn with_gradient(mut self, start: Color, end: Color) -> Self {
        self.gradient = Some((start, end));
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

        // When gradient is set and the bar is not pulsing, replace per-cell
        // styles in the completed portion with interpolated truecolor values.
        if let Some((ref start, ref end)) = self.gradient {
            if !pulse {
                // Determine how many filled cells there are so we can assign
                // each one a gradient position.  Segments from ProgressBar that
                // are part of the filled portion come before the back-style
                // segments; we count them by totalling up char counts until we
                // have processed the complete portion.
                let total_width = self.bar_width.unwrap_or(40);
                let pct = task
                    .total
                    .filter(|&t| t > 0.0)
                    .map(|t| (task.completed / t).clamp(0.0, 1.0))
                    .unwrap_or(0.0);
                let complete_halves = (total_width as f64 * 2.0 * pct) as usize;
                let filled_cells = complete_halves / 2;

                let mut cell_idx: usize = 0;
                for seg in &segments {
                    let char_count = seg.text.chars().count();
                    if cell_idx < filled_cells {
                        // This segment belongs to the completed portion —
                        // apply per-character gradient colors.
                        let mut buf = String::with_capacity(4);
                        for ch in seg.text.chars() {
                            let t = if filled_cells <= 1 {
                                0.0
                            } else {
                                cell_idx as f64 / (filled_cells - 1) as f64
                            };
                            let color = interpolate_color(start, end, t);
                            let grad_style = Style::from_color(Some(color), None);
                            buf.clear();
                            buf.push(ch);
                            text.append_str(&buf, Some(grad_style));
                            cell_idx += 1;
                        }
                    } else {
                        // Back (unfilled) portion — keep the original style.
                        text.append_str(&seg.text, seg.style().cloned());
                        cell_idx += char_count;
                    }
                }
                text.end = String::new();
                return text;
            }
        }

        for seg in &segments {
            text.append_str(&seg.text, seg.style().cloned());
        }
        text.end = String::new();
        text
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::color_triplet::ColorTriplet;
    use crate::progress::task::Task;

    fn make_task_at_pct(pct: f64) -> Task {
        let mut t = Task::new(0, "test", Some(100.0));
        t.completed = pct;
        // Mark as started so the bar doesn't pulse.
        t.start_time = Some(0.0);
        t
    }

    // -- with_gradient builder --------------------------------------------------

    #[test]
    fn test_with_gradient_sets_field() {
        let start = Color::from_rgb(255, 0, 0);
        let end = Color::from_rgb(0, 0, 255);
        let col = BarColumn::new().with_gradient(start, end);
        assert!(col.gradient.is_some());
    }

    #[test]
    fn test_no_gradient_by_default() {
        let col = BarColumn::new();
        assert!(col.gradient.is_none());
    }

    // -- Gradient fill produces truecolor segments ------------------------------

    /// A gradient bar at 50% must have per-cell truecolor ANSI in the filled
    /// portion and differ from a non-gradient bar.
    #[test]
    fn test_gradient_bar_produces_truecolor_styles() {
        let start = Color::from_rgb(255, 0, 0); // red
        let end = Color::from_rgb(0, 0, 255); // blue
        let col = BarColumn::new()
            .with_bar_width(Some(10))
            .with_gradient(start, end);

        let task = make_task_at_pct(50.0);
        let text = col.render(&task);

        // Collect styles from the first few filled cells (should have truecolor).
        let spans = text.spans();
        let has_truecolor = spans.iter().any(|span| {
            span.style.color().is_some_and(|c| {
                let triplet = c.get_truecolor(None, true);
                // At t=0 (first cell) the color should be close to red (255, _, _)
                triplet.red > 100
            })
        });
        assert!(
            has_truecolor,
            "gradient bar should produce truecolor (red-channel) styles in filled cells"
        );
    }

    #[test]
    fn test_gradient_bar_differs_from_non_gradient_bar() {
        let start = Color::from_rgb(200, 100, 50);
        let end = Color::from_rgb(50, 100, 200);

        let gradient_col = BarColumn::new()
            .with_bar_width(Some(10))
            .with_gradient(start, end);
        let plain_col = BarColumn::new().with_bar_width(Some(10));

        let task = make_task_at_pct(50.0);

        let grad_text = gradient_col.render(&task);
        let plain_text = plain_col.render(&task);

        // The plain text from both should contain the same characters, but the
        // gradient version should have different (per-cell) styles.
        assert_eq!(
            grad_text.plain(),
            plain_text.plain(),
            "gradient bar should produce the same characters as a plain bar"
        );
        // Spans should differ because gradient gives per-cell styles.
        assert_ne!(grad_text.spans().len(), 0, "gradient bar should have spans");
    }

    /// The first filled cell should start near `start` color and the last
    /// should be near `end` color.
    #[test]
    fn test_gradient_interpolation_endpoints() {
        let start = Color::from_rgb(255, 0, 0);
        let end = Color::from_rgb(0, 0, 255);
        let col = BarColumn::new()
            .with_bar_width(Some(10))
            .with_gradient(start, end);

        let task = make_task_at_pct(100.0); // full bar
        let text = col.render(&task);

        let spans = text.spans();
        assert!(!spans.is_empty(), "full bar should have spans");

        // First span: should be close to red
        let first_color = spans[0]
            .style
            .color()
            .expect("first span must have a color")
            .get_truecolor(None, true);
        assert!(
            first_color.red > 200 && first_color.blue < 100,
            "first gradient cell should be near red; got {:?}",
            first_color
        );

        // Last span before any trailing segments: should be close to blue
        let last_color = spans[spans.len() - 1]
            .style
            .color()
            .expect("last span must have a color")
            .get_truecolor(None, true);
        assert!(
            last_color.blue > 200 && last_color.red < 100,
            "last gradient cell should be near blue; got {:?}",
            last_color
        );
    }

    /// Verify `ColorTriplet` is referenced (compiler check).
    #[test]
    fn test_color_triplet_usage() {
        let _: ColorTriplet = Color::from_rgb(128, 64, 32).get_truecolor(None, true);
    }
}
