//! Constrain widget -- limits the width of a renderable to a given number of characters.
//!
//! Port of Python's `rich/constrain.py`.

use std::cmp::min;
use std::fmt;

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::text::Text;

// ---------------------------------------------------------------------------
// Constrain
// ---------------------------------------------------------------------------

/// A widget that constrains the width of its content to a given number of
/// characters.
///
/// When `width` is `Some(w)`, the content is rendered with a maximum width of
/// `min(w, options.max_width)`.  When `width` is `None`, the content passes
/// through unmodified.
#[derive(Debug, Clone)]
pub struct Constrain {
    /// The content to constrain.
    pub renderable: Text,
    /// Maximum width in characters. `None` means no constraint is applied.
    pub width: Option<usize>,
}

impl Constrain {
    /// Create a new `Constrain` widget.
    ///
    /// `width` defaults to `Some(80)` following the Python implementation.
    pub fn new(renderable: Text, width: Option<usize>) -> Self {
        Constrain { renderable, width }
    }

    /// Builder method to set the width.
    #[must_use]
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Measure the minimum and maximum width requirements of the constrained
    /// content.
    ///
    /// If `width` is `Some(w)`, the options are constrained to that width
    /// before measuring.  The resulting measurement is then clamped to the
    /// constrained width.
    pub fn measure(&self, _console: &Console, options: &ConsoleOptions) -> Measurement {
        let measurement = if let Some(w) = self.width {
            let constrained = options.update_width(w);
            self.renderable
                .measure()
                .with_maximum(constrained.max_width)
        } else {
            self.renderable.measure()
        };
        measurement.with_maximum(options.max_width)
    }
}

impl Renderable for Constrain {
    fn rich_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        match self.width {
            None => self.renderable.rich_console(console, options),
            Some(w) => {
                let constrained_width = min(w, options.max_width);
                let child_options = options.update_width(constrained_width);
                self.renderable.rich_console(console, &child_options)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl fmt::Display for Constrain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let w = f.width().unwrap_or(80);
        let mut console = Console::builder()
            .width(w)
            .force_terminal(true)
            .no_color(true)
            .build();
        console.begin_capture();
        console.print(self);
        let output = console.end_capture();
        write!(f, "{}", output.trim_end_matches('\n'))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Style;

    fn make_console(width: usize) -> Console {
        Console::builder()
            .width(width)
            .force_terminal(true)
            .no_color(true)
            .markup(false)
            .build()
    }

    fn segments_to_text(segments: &[Segment]) -> String {
        segments.iter().map(|s| s.text.as_str()).collect()
    }

    // -- Construction -------------------------------------------------------

    #[test]
    fn test_default_construction() {
        let text = Text::new("Hello, world!", Style::null());
        let c = Constrain::new(text.clone(), Some(80));
        assert_eq!(c.width, Some(80));
        assert_eq!(c.renderable.plain(), "Hello, world!");
    }

    #[test]
    fn test_none_width_construction() {
        let text = Text::new("Hello", Style::null());
        let c = Constrain::new(text, None);
        assert_eq!(c.width, None);
    }

    #[test]
    fn test_builder_method() {
        let text = Text::new("Hello", Style::null());
        let c = Constrain::new(text, None).with_width(40);
        assert_eq!(c.width, Some(40));
    }

    // -- Passthrough (width = None) -----------------------------------------

    #[test]
    fn test_none_width_passthrough() {
        let console = make_console(80);
        let opts = console.options();
        let text = Text::new("Hello, world!", Style::null());
        let c = Constrain::new(text.clone(), None);

        let constrained_segments = c.rich_console(&console, &opts);
        let direct_segments = text.rich_console(&console, &opts);

        assert_eq!(
            segments_to_text(&constrained_segments),
            segments_to_text(&direct_segments),
        );
    }

    // -- Width smaller than content -----------------------------------------

    #[test]
    fn test_width_smaller_than_content() {
        let console = make_console(80);
        let opts = console.options();
        // "Hello, world!" is 13 chars; constrain to 5
        let text = Text::new("Hello, world!", Style::null());
        let c = Constrain::new(text, Some(5));

        let segments = c.rich_console(&console, &opts);
        let output = segments_to_text(&segments);

        // Each line should be at most 5 cells wide (the text will wrap)
        for line in output.split('\n') {
            if !line.is_empty() {
                assert!(
                    crate::cells::cell_len(line) <= 5,
                    "Line '{}' exceeds constrained width of 5 (actual: {})",
                    line,
                    crate::cells::cell_len(line),
                );
            }
        }
    }

    // -- Width larger than content ------------------------------------------

    #[test]
    fn test_width_larger_than_content() {
        let console = make_console(80);
        let opts = console.options();
        let text = Text::new("Hi", Style::null());
        let c = Constrain::new(text.clone(), Some(40));

        let constrained_segments = c.rich_console(&console, &opts);
        let direct_segments = text.rich_console(&console, &opts);

        // With width larger than content, the rendering should be the same as
        // rendering into the full console width (since the content fits)
        assert_eq!(
            segments_to_text(&constrained_segments),
            segments_to_text(&direct_segments),
        );
    }

    // -- Width constrains to min(width, max_width) --------------------------

    #[test]
    fn test_width_min_of_width_and_max_width() {
        // Console width is 10, constrain width is 20.
        // Effective constraint should be min(20, 10) = 10.
        let console = make_console(10);
        let opts = console.options();
        let text = Text::new("ABCDEFGHIJKLMNOP", Style::null());

        let c = Constrain::new(text, Some(20));
        let segments = c.rich_console(&console, &opts);
        let output = segments_to_text(&segments);

        for line in output.split('\n') {
            if !line.is_empty() {
                assert!(
                    crate::cells::cell_len(line) <= 10,
                    "Line '{}' exceeds effective width of 10 (actual: {})",
                    line,
                    crate::cells::cell_len(line),
                );
            }
        }
    }

    // -- Measure with width -------------------------------------------------

    #[test]
    fn test_measure_with_width() {
        let console = make_console(80);
        let opts = console.options();
        let text = Text::new("Hello, world!", Style::null());

        // Constrain to 5: measurement maximum should be at most 5
        let c = Constrain::new(text, Some(5));
        let m = c.measure(&console, &opts);
        assert!(m.maximum <= 5, "Expected max <= 5, got {}", m.maximum);
    }

    #[test]
    fn test_measure_without_width() {
        let console = make_console(80);
        let opts = console.options();
        let text = Text::new("Hello", Style::null());

        let c = Constrain::new(text.clone(), None);
        let m = c.measure(&console, &opts);

        // Without width constraint, measure should match text.measure()
        // clamped to options.max_width
        let text_m = text.measure().with_maximum(opts.max_width);
        assert_eq!(m.minimum, text_m.minimum);
        assert_eq!(m.maximum, text_m.maximum);
    }

    #[test]
    fn test_measure_width_larger_than_content() {
        let console = make_console(80);
        let opts = console.options();
        let text = Text::new("Hello", Style::null());

        // Constrain to 40 but text is only 5 wide
        let c = Constrain::new(text, Some(40));
        let m = c.measure(&console, &opts);

        assert_eq!(m.maximum, 5);
    }

    #[test]
    fn test_measure_width_smaller_than_console() {
        let console = make_console(80);
        let opts = console.options();
        let text = Text::new("Hello, world! This is a long sentence.", Style::null());

        let c = Constrain::new(text, Some(10));
        let m = c.measure(&console, &opts);

        assert!(m.maximum <= 10, "Expected max <= 10, got {}", m.maximum,);
    }

    // -- Styled content -----------------------------------------------------

    #[test]
    fn test_styled_content_preserved() {
        let console = make_console(80);
        let opts = console.options();
        let text = Text::styled("Bold text", Style::parse("bold").unwrap());

        let c = Constrain::new(text, Some(40));
        let segments = c.rich_console(&console, &opts);

        // The styled content should still carry its style through
        let has_styled = segments
            .iter()
            .any(|s| s.text.contains("Bold text") && s.style.is_some());
        assert!(has_styled, "Expected styled segment in output");
    }

    // -- Clone and Debug derive checks --------------------------------------

    #[test]
    fn test_clone() {
        let text = Text::new("Hello", Style::null());
        let c = Constrain::new(text, Some(40));
        let cloned = c.clone();
        assert_eq!(cloned.width, c.width);
        assert_eq!(cloned.renderable.plain(), c.renderable.plain());
    }

    #[test]
    fn test_debug() {
        let text = Text::new("Hello", Style::null());
        let c = Constrain::new(text, Some(40));
        let debug = format!("{:?}", c);
        assert!(debug.contains("Constrain"));
        assert!(debug.contains("40"));
    }

    // -- Edge cases ---------------------------------------------------------

    #[test]
    fn test_zero_width() {
        let console = make_console(80);
        let opts = console.options();
        let text = Text::new("Hello", Style::null());
        let c = Constrain::new(text, Some(0));
        let segments = c.rich_console(&console, &opts);
        let output = segments_to_text(&segments);

        // With width 0, all content lines should be empty
        for line in output.split('\n') {
            assert!(
                crate::cells::cell_len(line) == 0,
                "Expected empty line, got '{}'",
                line,
            );
        }
    }

    #[test]
    fn test_empty_text() {
        let console = make_console(80);
        let opts = console.options();
        let text = Text::new("", Style::null());
        let c = Constrain::new(text, Some(40));
        let segments = c.rich_console(&console, &opts);
        let output = segments_to_text(&segments);
        // Empty text should produce only the end segment (newline)
        assert!(output.trim().is_empty());
    }

    #[test]
    fn test_width_equal_to_content() {
        let console = make_console(80);
        let opts = console.options();
        let text = Text::new("Hello", Style::null());
        let c = Constrain::new(text, Some(5));
        let segments = c.rich_console(&console, &opts);
        let output = segments_to_text(&segments);

        // Content exactly fits the constraint -- should not wrap
        let content_lines: Vec<&str> = output.split('\n').filter(|l| !l.is_empty()).collect();
        assert_eq!(content_lines.len(), 1);
        assert_eq!(content_lines[0], "Hello");
    }
}
