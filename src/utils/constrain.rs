//! Constrain widget -- limits the width of a renderable to a given number of characters.
//!

use std::cmp::min;
use std::fmt;

use crate::console::{Console, ConsoleOptions, Renderable, RenderableArc};
use crate::measure::Measurement;
use crate::segment::Segment;

// ---------------------------------------------------------------------------
// Constrain
// ---------------------------------------------------------------------------

/// A widget that constrains the width of its content to a given number of
/// characters.
///
/// When `width` is `Some(w)`, the content is rendered with a maximum width of
/// `min(w, options.max_width)`.  When `width` is `None`, the content passes
/// through unmodified.
#[derive(Clone)]
pub struct Constrain {
    /// The content to constrain (any renderable widget).
    pub renderable: RenderableArc,
    /// Maximum width in characters. `None` means no constraint is applied.
    pub width: Option<usize>,
}

// Manual Debug — RenderableArc (Arc<dyn Renderable + Send + Sync>) doesn't
// implement Debug, so we print a placeholder for the renderable field.
impl std::fmt::Debug for Constrain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Constrain")
            .field("renderable", &"<renderable>")
            .field("width", &self.width)
            .finish()
    }
}

impl Constrain {
    /// Create a new `Constrain` widget.
    ///
    /// `width` defaults to `Some(80)` following the Python implementation.
    pub fn new(renderable: impl Renderable + Send + Sync + 'static, width: Option<usize>) -> Self {
        Constrain {
            renderable: std::sync::Arc::new(renderable),
            width,
        }
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
    pub fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        if let Some(w) = self.width {
            let constrained = options.update_width(w);
            self.renderable
                .gilt_measure(console, &constrained)
                .with_maximum(constrained.max_width)
                // belt-and-suspenders: gilt_measure may return a width > constrained.max_width
                // for some Renderable types; clamp again to options.max_width as the final gate.
                .with_maximum(options.max_width)
        } else {
            self.renderable
                .gilt_measure(console, options)
                .with_maximum(options.max_width)
        }
    }
}

impl Renderable for Constrain {
    fn gilt_measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        self.measure(console, options)
    }

    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        match self.width {
            None => self.renderable.as_ref().gilt_console(console, options),
            Some(w) => {
                let constrained_width = min(w, options.max_width);
                let child_options = options.update_width(constrained_width);
                self.renderable
                    .as_ref()
                    .gilt_console(console, &child_options)
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
    use crate::text::Text;

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
        // Updated: field is now RenderableArc (Text still works via Renderable impl)
        let text = Text::new("Hello, world!", Style::null());
        let c = Constrain::new(text.clone(), Some(80));
        assert_eq!(c.width, Some(80));
        // Cannot call .plain() on RenderableArc — verify width instead
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
        // Updated: field is now RenderableArc (Text still works via Renderable impl)
        let console = make_console(80);
        let opts = console.options();
        let text = Text::new("Hello, world!", Style::null());
        // Clone text before passing to Constrain::new since it moves the value
        let text_direct = text.clone();
        let c = Constrain::new(text, None);

        let constrained_segments = c.gilt_console(&console, &opts);
        let direct_segments = text_direct.gilt_console(&console, &opts);

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

        let segments = c.gilt_console(&console, &opts);
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

        let constrained_segments = c.gilt_console(&console, &opts);
        let direct_segments = text.gilt_console(&console, &opts);

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
        let segments = c.gilt_console(&console, &opts);
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
        // Updated: field is now RenderableArc (Text still works via Renderable impl)
        // Constrain::measure now delegates to gilt_measure instead of Text::measure
        let console = make_console(80);
        let opts = console.options();
        let text = Text::new("Hello", Style::null());

        let c = Constrain::new(text.clone(), None);
        let m = c.measure(&console, &opts);

        // Without width constraint, measure should match text.gilt_measure()
        // clamped to options.max_width
        let text_m = text
            .gilt_measure(&console, &opts)
            .with_maximum(opts.max_width);
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
        let text = Text::styled("Bold text", "bold");

        let c = Constrain::new(text, Some(40));
        let segments = c.gilt_console(&console, &opts);

        // The styled content should still carry its style through
        let has_styled = segments
            .iter()
            .any(|s| s.text.contains("Bold text") && s.style.is_some());
        assert!(has_styled, "Expected styled segment in output");
    }

    // -- Clone and Debug derive checks --------------------------------------

    #[test]
    fn test_clone() {
        // Updated: field is now RenderableArc (Text still works via Renderable impl)
        let text = Text::new("Hello", Style::null());
        let c = Constrain::new(text, Some(40));
        let cloned = c.clone();
        assert_eq!(cloned.width, c.width);
        // Cannot call .plain() on RenderableArc — verify width is correct
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
        let segments = c.gilt_console(&console, &opts);
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
        let segments = c.gilt_console(&console, &opts);
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
        let segments = c.gilt_console(&console, &opts);
        let output = segments_to_text(&segments);

        // Content exactly fits the constraint -- should not wrap
        let content_lines: Vec<&str> = output.split('\n').filter(|l| !l.is_empty()).collect();
        assert_eq!(content_lines.len(), 1);
        assert_eq!(content_lines[0], "Hello");
    }

    // -- gilt_measure override -----------------------------------------------

    #[test]
    fn constrain_gilt_measure_matches_standalone() {
        let console = make_console(80);
        let opts = console.options();
        let text = Text::new("Hello, world!", Style::null());
        let c = Constrain::new(text, Some(5));
        let m_standalone = c.measure(&console, &opts);
        let m_trait = c.gilt_measure(&console, &opts);
        assert_eq!(
            m_trait, m_standalone,
            "Constrain::gilt_measure must delegate to Constrain::measure"
        );
    }

    #[test]
    fn constrain_gilt_measure_no_width_matches_standalone() {
        let console = make_console(80);
        let opts = console.options();
        let text = Text::new("Hello", Style::null());
        let c = Constrain::new(text, None);
        let m_standalone = c.measure(&console, &opts);
        let m_trait = c.gilt_measure(&console, &opts);
        assert_eq!(
            m_trait, m_standalone,
            "Constrain::gilt_measure (no width) must delegate to Constrain::measure"
        );
    }

    // -- Task 4.5: RenderableArc constructor tests ---------------------------

    #[test]
    // Updated: constructors now accept impl Renderable + Send + Sync + 'static (Text still works via Renderable impl)
    fn constrain_new_text_still_compiles() {
        let _ = Constrain::new(Text::new("x", Style::null()), None);
    }

    #[test]
    // Updated: constructors now accept impl Renderable + Send + Sync + 'static (Panel works too)
    fn constrain_new_panel_compiles() {
        let p = crate::panel::Panel::new(Text::new("x", Style::null()));
        let _ = Constrain::new(p, None);
    }
}
