//! Bar renderable -- a solid block-character bar.
//!
//! Rust port of Python's `rich/bar.py`.
//!
//! Renders a horizontal bar using Unicode block elements, useful for progress
//! indicators, sparklines, and other proportional visualisations.

use std::fmt;

use crate::color::Color;
use crate::console::{Console, ConsoleOptions, Renderable};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::Style;

// ---------------------------------------------------------------------------
// Block-element constants
// ---------------------------------------------------------------------------

/// Left-aligned block elements used for the *prefix* (the space before the
/// filled region).  Indexed by the number of filled eighths (0..8).
const BEGIN_BLOCK_ELEMENTS: [&str; 8] = [
    "\u{2588}", // 0 - FULL BLOCK  (unused placeholder)
    "\u{2588}", // 1
    "\u{2588}", // 2
    "\u{2590}", // 3 - RIGHT HALF BLOCK
    "\u{2590}", // 4
    "\u{2590}", // 5
    "\u{2595}", // 6 - RIGHT ONE EIGHTH BLOCK
    "\u{2595}", // 7
];

/// Right-aligned block elements used for the *suffix* of the filled region.
/// Indexed by the number of filled eighths (0..8).
const END_BLOCK_ELEMENTS: [&str; 8] = [
    " ",        // 0
    "\u{258F}", // 1 - LEFT ONE EIGHTH BLOCK
    "\u{258E}", // 2 - LEFT ONE QUARTER BLOCK
    "\u{258D}", // 3 - LEFT THREE EIGHTHS BLOCK
    "\u{258C}", // 4 - LEFT HALF BLOCK
    "\u{258B}", // 5 - LEFT FIVE EIGHTHS BLOCK
    "\u{258A}", // 6 - LEFT THREE QUARTERS BLOCK
    "\u{2589}", // 7 - LEFT SEVEN EIGHTHS BLOCK
];

/// Full block character used for the solid filled body of the bar.
const FULL_BLOCK: &str = "\u{2588}";

// ---------------------------------------------------------------------------
// Bar struct
// ---------------------------------------------------------------------------

/// Renders a solid block bar.
///
/// The bar occupies a range `[begin, end)` within a total `size`, rendered
/// using Unicode block elements to achieve sub-character precision (eighths).
#[derive(Debug, Clone)]
pub struct Bar {
    /// The total range represented by the bar.
    pub size: f64,
    /// Start of the filled region (clamped to >= 0).
    pub begin: f64,
    /// End of the filled region (clamped to <= size).
    pub end: f64,
    /// Optional fixed width in cells.  When `None`, the bar uses
    /// `ConsoleOptions::max_width`.
    pub width: Option<usize>,
    /// Visual style applied to the bar (foreground and background colors).
    pub style: Style,
}

impl Bar {
    /// Create a new `Bar` with default style and no fixed width.
    pub fn new(size: f64, begin: f64, end: f64) -> Self {
        Self {
            size,
            begin: begin.max(0.0),
            end: end.min(size),
            width: None,
            style: Style::null(),
        }
    }

    /// Set a fixed width (builder pattern).
    #[must_use]
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the style (builder pattern).
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the foreground color (builder convenience).
    #[must_use]
    pub fn with_color(mut self, color: Color) -> Self {
        self.style = Style::from_color(Some(color), self.style.bgcolor().cloned());
        self
    }

    /// Set the background color (builder convenience).
    #[must_use]
    pub fn with_bgcolor(mut self, bgcolor: Color) -> Self {
        self.style = Style::from_color(self.style.color().cloned(), Some(bgcolor));
        self
    }

    /// Return the measurement for this bar.
    pub fn measure(&self, _console: &Console, options: &ConsoleOptions) -> Measurement {
        match self.width {
            Some(w) => Measurement::new(w, w),
            None => Measurement::new(4, options.max_width),
        }
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl fmt::Display for Bar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bar({}, {}, {})", self.size, self.begin, self.end)
    }
}

// ---------------------------------------------------------------------------
// Renderable
// ---------------------------------------------------------------------------

impl Renderable for Bar {
    fn rich_console(&self, _console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let width = match self.width {
            Some(w) => w.min(options.max_width),
            None => options.max_width,
        };

        // Empty bar when begin >= end
        if self.begin >= self.end {
            return vec![
                Segment::styled(&" ".repeat(width), self.style.clone()),
                Segment::line(),
            ];
        }

        // -- prefix (space before the bar) ----------------------------------

        let prefix_complete_eights =
            (width as f64 * 8.0 * self.begin / self.size) as usize;
        let prefix_bar_count = prefix_complete_eights / 8;
        let prefix_eights_count = prefix_complete_eights % 8;

        let mut prefix = " ".repeat(prefix_bar_count);
        if prefix_eights_count != 0 {
            prefix.push_str(BEGIN_BLOCK_ELEMENTS[prefix_eights_count]);
        }

        // -- body (filled portion) ------------------------------------------

        let body_complete_eights =
            (width as f64 * 8.0 * self.end / self.size) as usize;
        let body_bar_count = body_complete_eights / 8;
        let body_eights_count = body_complete_eights % 8;

        let mut body = FULL_BLOCK.repeat(body_bar_count);
        if body_eights_count != 0 {
            body.push_str(END_BLOCK_ELEMENTS[body_eights_count]);
        }

        // -- suffix (space after the bar) -----------------------------------

        // Use character (cell) count for width arithmetic, not byte count.
        let body_char_len = body.chars().count();
        let suffix = " ".repeat(width.saturating_sub(body_char_len));

        // Combine: skip the prefix portion of body (just like Python's
        // `body[len(prefix):]`).  `prefix` is already the correct number of
        // *characters* wide because it is built from single-cell-wide pieces.
        let prefix_char_len = prefix.chars().count();
        let body_tail: String = body.chars().skip(prefix_char_len).collect();

        let bar_text = format!("{prefix}{body_tail}{suffix}");

        vec![
            Segment::styled(&bar_text, self.style.clone()),
            Segment::line(),
        ]
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::{Console, ConsoleDimensions, ConsoleOptions};


    /// Build a `ConsoleOptions` with a given `max_width`.
    fn make_options(max_width: usize) -> ConsoleOptions {
        ConsoleOptions {
            size: ConsoleDimensions {
                width: max_width,
                height: 25,
            },
            legacy_windows: false,
            min_width: 1,
            max_width,
            is_terminal: false,
            encoding: "utf-8".to_string(),
            max_height: 25,
            justify: None,
            overflow: None,
            no_wrap: false,
            highlight: None,
            markup: None,
            height: None,
        }
    }

    /// Render a `Bar` through its `Renderable` impl and return the
    /// concatenated text (ignoring styles).
    fn render_bar_text(bar: &Bar, max_width: usize) -> String {
        let console = Console::builder().width(max_width).build();
        let opts = make_options(max_width);
        let segments = bar.rich_console(&console, &opts);
        segments.iter().map(|s| s.text.as_str()).collect()
    }

    // -- Construction -------------------------------------------------------

    #[test]
    fn test_new_defaults() {
        let bar = Bar::new(100.0, 0.0, 50.0);
        assert_eq!(bar.size, 100.0);
        assert_eq!(bar.begin, 0.0);
        assert_eq!(bar.end, 50.0);
        assert!(bar.width.is_none());
        assert!(bar.style.is_null());
    }

    #[test]
    fn test_new_clamps_begin() {
        let bar = Bar::new(100.0, -10.0, 50.0);
        assert_eq!(bar.begin, 0.0);
    }

    #[test]
    fn test_new_clamps_end() {
        let bar = Bar::new(100.0, 0.0, 200.0);
        assert_eq!(bar.end, 100.0);
    }

    // -- Builder methods ----------------------------------------------------

    #[test]
    fn test_with_width() {
        let bar = Bar::new(100.0, 0.0, 50.0).with_width(40);
        assert_eq!(bar.width, Some(40));
    }

    #[test]
    fn test_with_style() {
        let style = Style::parse("bold red on blue").unwrap();
        let bar = Bar::new(100.0, 0.0, 50.0).with_style(style.clone());
        assert_eq!(bar.style, style);
    }

    #[test]
    fn test_with_color() {
        let bar = Bar::new(100.0, 0.0, 50.0)
            .with_color(Color::parse("red").unwrap());
        assert_eq!(bar.style.color().unwrap().name, "red");
    }

    #[test]
    fn test_with_bgcolor() {
        let bar = Bar::new(100.0, 0.0, 50.0)
            .with_bgcolor(Color::parse("blue").unwrap());
        assert_eq!(bar.style.bgcolor().unwrap().name, "blue");
    }

    #[test]
    fn test_with_color_and_bgcolor_chained() {
        let bar = Bar::new(100.0, 0.0, 50.0)
            .with_color(Color::parse("red").unwrap())
            .with_bgcolor(Color::parse("blue").unwrap());
        assert_eq!(bar.style.color().unwrap().name, "red");
        assert_eq!(bar.style.bgcolor().unwrap().name, "blue");
    }

    // -- Display trait ------------------------------------------------------

    #[test]
    fn test_display() {
        let bar = Bar::new(100.0, 10.0, 90.0);
        assert_eq!(format!("{bar}"), "Bar(100, 10, 90)");
    }

    // -- Empty bar (begin >= end) -------------------------------------------

    #[test]
    fn test_empty_bar_begin_equals_end() {
        let bar = Bar::new(100.0, 50.0, 50.0).with_width(10);
        let text = render_bar_text(&bar, 10);
        // Should be 10 spaces + newline
        assert_eq!(text, format!("{}\n", " ".repeat(10)));
    }

    #[test]
    fn test_empty_bar_begin_greater_than_end() {
        let bar = Bar::new(100.0, 80.0, 20.0).with_width(10);
        let text = render_bar_text(&bar, 10);
        assert_eq!(text, format!("{}\n", " ".repeat(10)));
    }

    // -- Full bar (begin=0, end=size) ---------------------------------------

    #[test]
    fn test_full_bar() {
        let bar = Bar::new(100.0, 0.0, 100.0).with_width(10);
        let text = render_bar_text(&bar, 10);
        // Full bar: 10 full blocks + newline
        assert_eq!(text, format!("{}\n", FULL_BLOCK.repeat(10)));
    }

    // -- Partial bar with fractional blocks ---------------------------------

    #[test]
    fn test_half_bar() {
        let bar = Bar::new(100.0, 0.0, 50.0).with_width(10);
        let text = render_bar_text(&bar, 10);
        // Should contain some full blocks and then spaces (and a newline)
        assert!(text.contains(FULL_BLOCK));
        assert!(text.ends_with('\n'));
        // The line (excluding newline) should be exactly 10 characters wide
        let line = text.trim_end_matches('\n');
        assert_eq!(line.chars().count(), 10);
    }

    // -- Bar with specific width --------------------------------------------

    #[test]
    fn test_bar_with_specific_width() {
        let bar = Bar::new(100.0, 0.0, 100.0).with_width(20);
        let text = render_bar_text(&bar, 80);
        let line = text.trim_end_matches('\n');
        // Width should be 20 (capped by bar width, not max_width=80)
        assert_eq!(line.chars().count(), 20);
    }

    #[test]
    fn test_bar_width_capped_by_max_width() {
        let bar = Bar::new(100.0, 0.0, 100.0).with_width(100);
        let text = render_bar_text(&bar, 40);
        let line = text.trim_end_matches('\n');
        // bar.width=100 but max_width=40, so capped to 40
        assert_eq!(line.chars().count(), 40);
    }

    // -- Bar with default width (uses max_width) ----------------------------

    #[test]
    fn test_bar_default_width_uses_max_width() {
        let bar = Bar::new(100.0, 0.0, 100.0);
        let text = render_bar_text(&bar, 30);
        let line = text.trim_end_matches('\n');
        assert_eq!(line.chars().count(), 30);
    }

    // -- Measure ------------------------------------------------------------

    #[test]
    fn test_measure_with_fixed_width() {
        let bar = Bar::new(100.0, 0.0, 50.0).with_width(25);
        let console = Console::new();
        let opts = make_options(80);
        let m = bar.measure(&console, &opts);
        assert_eq!(m, Measurement::new(25, 25));
    }

    #[test]
    fn test_measure_without_fixed_width() {
        let bar = Bar::new(100.0, 0.0, 50.0);
        let console = Console::new();
        let opts = make_options(80);
        let m = bar.measure(&console, &opts);
        assert_eq!(m, Measurement::new(4, 80));
    }

    // -- Bar at different positions (begin > 0) -----------------------------

    #[test]
    fn test_bar_offset_position() {
        let bar = Bar::new(100.0, 25.0, 75.0).with_width(20);
        let text = render_bar_text(&bar, 20);
        let line = text.trim_end_matches('\n');
        assert_eq!(line.chars().count(), 20);

        // The first few characters should be spaces (prefix)
        let first_char = line.chars().next().unwrap();
        assert_eq!(first_char, ' ');
    }

    #[test]
    fn test_bar_near_end() {
        let bar = Bar::new(100.0, 80.0, 100.0).with_width(20);
        let text = render_bar_text(&bar, 20);
        let line = text.trim_end_matches('\n');
        assert_eq!(line.chars().count(), 20);

        // First 80% should be spaces
        let chars: Vec<char> = line.chars().collect();
        // 16 spaces (80% of 20)
        for ch in &chars[..16] {
            assert_eq!(*ch, ' ');
        }
    }

    // -- Edge cases ---------------------------------------------------------

    #[test]
    fn test_size_zero() {
        // size=0 makes begin(0) >= end(0), so empty bar
        let bar = Bar::new(0.0, 0.0, 0.0).with_width(10);
        let text = render_bar_text(&bar, 10);
        assert_eq!(text, format!("{}\n", " ".repeat(10)));
    }

    #[test]
    fn test_begin_equals_end_at_zero() {
        let bar = Bar::new(100.0, 0.0, 0.0).with_width(10);
        let text = render_bar_text(&bar, 10);
        assert_eq!(text, format!("{}\n", " ".repeat(10)));
    }

    #[test]
    fn test_very_small_bar() {
        let bar = Bar::new(100.0, 0.0, 1.0).with_width(10);
        let text = render_bar_text(&bar, 10);
        let line = text.trim_end_matches('\n');
        // Width should still be 10
        assert_eq!(line.chars().count(), 10);
    }

    // -- Segment and style verification ------------------------------------

    #[test]
    fn test_segments_have_style() {
        let style = Style::parse("red on blue").unwrap();
        let bar = Bar::new(100.0, 0.0, 50.0)
            .with_width(10)
            .with_style(style.clone());
        let console = Console::builder().width(10).build();
        let opts = make_options(10);
        let segments = bar.rich_console(&console, &opts);

        // First segment is the bar content, second is the newline
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].style.as_ref(), Some(&style));
        assert_eq!(segments[1].text, "\n");
    }

    #[test]
    fn test_segments_end_with_newline() {
        let bar = Bar::new(100.0, 0.0, 50.0).with_width(10);
        let console = Console::builder().width(10).build();
        let opts = make_options(10);
        let segments = bar.rich_console(&console, &opts);

        let last = segments.last().unwrap();
        assert_eq!(last.text, "\n");
    }

    // -- Width consistency across various fill levels ----------------------

    #[test]
    fn test_width_consistency() {
        // Every bar rendered at width=20 should produce exactly 20 chars
        for pct in (0..=100).step_by(5) {
            let bar = Bar::new(100.0, 0.0, pct as f64).with_width(20);
            let text = render_bar_text(&bar, 20);
            let line = text.trim_end_matches('\n');
            assert_eq!(
                line.chars().count(),
                20,
                "width mismatch at {}%",
                pct
            );
        }
    }

    #[test]
    fn test_width_consistency_with_offset() {
        // Bars with a non-zero begin should also maintain exact width
        for begin in (0..=90).step_by(10) {
            let bar = Bar::new(100.0, begin as f64, 100.0).with_width(20);
            let text = render_bar_text(&bar, 20);
            let line = text.trim_end_matches('\n');
            assert_eq!(
                line.chars().count(),
                20,
                "width mismatch with begin={}",
                begin
            );
        }
    }

    // -- Renderable trait dispatch -----------------------------------------

    #[test]
    fn test_renderable_trait() {
        let bar = Bar::new(100.0, 0.0, 50.0).with_width(10);
        let console = Console::builder().width(80).build();
        let opts = make_options(80);
        // Call through the trait object
        let renderable: &dyn Renderable = &bar;
        let segments = renderable.rich_console(&console, &opts);
        assert!(!segments.is_empty());
    }

    // -- Clone and Debug ---------------------------------------------------

    #[test]
    fn test_clone() {
        let bar = Bar::new(100.0, 10.0, 90.0).with_width(40);
        let cloned = bar.clone();
        assert_eq!(cloned.size, bar.size);
        assert_eq!(cloned.begin, bar.begin);
        assert_eq!(cloned.end, bar.end);
        assert_eq!(cloned.width, bar.width);
    }

    #[test]
    fn test_debug() {
        let bar = Bar::new(100.0, 0.0, 50.0);
        let debug = format!("{bar:?}");
        assert!(debug.contains("Bar"));
        assert!(debug.contains("100"));
        assert!(debug.contains("50"));
    }
}
