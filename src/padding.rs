//! Padding widget -- adds whitespace around renderable content.
//!
//! Port of Python's `rich/padding.py`.

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;

// ---------------------------------------------------------------------------
// PaddingDimensions
// ---------------------------------------------------------------------------

/// CSS-style padding specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaddingDimensions {
    /// Same padding on all four sides.
    Uniform(usize),
    /// (vertical, horizontal) -- top & bottom share first, left & right share second.
    Pair(usize, usize),
    /// (top, right, bottom, left) -- explicit per-side.
    Full(usize, usize, usize, usize),
}

impl PaddingDimensions {
    /// Unpack any variant into `(top, right, bottom, left)`.
    pub fn unpack(&self) -> (usize, usize, usize, usize) {
        match *self {
            PaddingDimensions::Uniform(v) => (v, v, v, v),
            PaddingDimensions::Pair(vert, horiz) => (vert, horiz, vert, horiz),
            PaddingDimensions::Full(t, r, b, l) => (t, r, b, l),
        }
    }
}

// ---------------------------------------------------------------------------
// Padding
// ---------------------------------------------------------------------------

/// A renderable that adds whitespace padding around `Text` content.
#[derive(Debug, Clone)]
pub struct Padding {
    /// The inner content to pad.
    pub content: Text,
    /// Top padding (blank lines above content).
    pub top: usize,
    /// Right padding (spaces after each content line).
    pub right: usize,
    /// Bottom padding (blank lines below content).
    pub bottom: usize,
    /// Left padding (spaces before each content line).
    pub left: usize,
    /// Style applied to the padding whitespace.
    pub style: Style,
    /// If true, expand to fill the available width.
    pub expand: bool,
}

impl Padding {
    /// Create a new `Padding` around the given content.
    pub fn new(content: Text, pad: PaddingDimensions, style: Style, expand: bool) -> Self {
        let (top, right, bottom, left) = pad.unpack();
        Padding {
            content,
            top,
            right,
            bottom,
            left,
            style,
            expand,
        }
    }

    /// Convenience: create padding that acts as a left-indent.
    pub fn indent(content: Text, level: usize) -> Self {
        Padding::new(
            content,
            PaddingDimensions::Full(0, 0, 0, level),
            Style::null(),
            true,
        )
    }

    /// Measure the minimum and maximum width requirements.
    pub fn measure(&self, _console: &Console, options: &ConsoleOptions) -> Measurement {
        let max_width = options.max_width.saturating_sub(self.left + self.right);
        let inner_opts = options.update_width(max_width.max(1));
        // For Text, measure is the cell_len of the content
        let content_width = self.content.cell_len();
        let min_w = content_width + self.left + self.right;
        let max_w = if self.expand {
            options.max_width
        } else {
            min_w.min(options.max_width)
        };
        Measurement::new(min_w.min(inner_opts.max_width + self.left + self.right), max_w)
    }
}

impl Renderable for Padding {
    fn rich_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let mut segments = Vec::new();

        // Compute the total available width
        let width = if self.expand {
            options.max_width
        } else {
            let content_width = self.content.cell_len();
            (content_width + self.left + self.right).min(options.max_width)
        };

        // Compute inner width for the content
        let inner_width = width.saturating_sub(self.left + self.right).max(1);

        // Render the content into lines
        let inner_opts = options.update_width(inner_width);
        let lines = console.render_lines(&self.content, Some(&inner_opts), None, true, false);

        // Left/right padding strings
        let left_pad = " ".repeat(self.left);
        let right_pad_base = self.right;

        // Top blank lines
        let blank_line = " ".repeat(width);
        for _ in 0..self.top {
            segments.push(Segment::styled(&blank_line, self.style.clone()));
            segments.push(Segment::line());
        }

        // Content lines with left/right padding
        for line in &lines {
            // Left padding
            if self.left > 0 {
                segments.push(Segment::styled(&left_pad, self.style.clone()));
            }

            // Content segments
            segments.extend(line.iter().cloned());

            // Right padding -- fill remaining space to reach full width
            let line_len = self.left + Segment::get_line_length(line);
            let remaining = width.saturating_sub(line_len);
            if remaining > 0 {
                segments.push(Segment::styled(
                    &" ".repeat(remaining),
                    self.style.clone(),
                ));
            } else if right_pad_base > 0 && remaining == 0 {
                // Content exactly fills; no extra padding needed
            }

            segments.push(Segment::line());
        }

        // Bottom blank lines
        for _ in 0..self.bottom {
            segments.push(Segment::styled(&blank_line, self.style.clone()));
            segments.push(Segment::line());
        }

        segments
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cells::cell_len;

    // -- PaddingDimensions --------------------------------------------------

    #[test]
    fn test_unpack_uniform() {
        let pd = PaddingDimensions::Uniform(2);
        assert_eq!(pd.unpack(), (2, 2, 2, 2));
    }

    #[test]
    fn test_unpack_pair() {
        let pd = PaddingDimensions::Pair(1, 3);
        assert_eq!(pd.unpack(), (1, 3, 1, 3));
    }

    #[test]
    fn test_unpack_full() {
        let pd = PaddingDimensions::Full(1, 2, 3, 4);
        assert_eq!(pd.unpack(), (1, 2, 3, 4));
    }

    #[test]
    fn test_unpack_uniform_zero() {
        let pd = PaddingDimensions::Uniform(0);
        assert_eq!(pd.unpack(), (0, 0, 0, 0));
    }

    // -- Padding construction -----------------------------------------------

    #[test]
    fn test_padding_new() {
        let text = Text::new("Hello", Style::null());
        let padding = Padding::new(
            text,
            PaddingDimensions::Full(1, 2, 3, 4),
            Style::null(),
            true,
        );
        assert_eq!(padding.top, 1);
        assert_eq!(padding.right, 2);
        assert_eq!(padding.bottom, 3);
        assert_eq!(padding.left, 4);
        assert!(padding.expand);
    }

    #[test]
    fn test_indent() {
        let text = Text::new("Hello", Style::null());
        let padding = Padding::indent(text, 4);
        assert_eq!(padding.top, 0);
        assert_eq!(padding.right, 0);
        assert_eq!(padding.bottom, 0);
        assert_eq!(padding.left, 4);
        assert!(padding.expand);
    }

    // -- Rendering ----------------------------------------------------------

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

    #[test]
    fn test_render_no_padding() {
        let console = make_console(20);
        let text = Text::new("Hello", Style::null());
        let padding = Padding::new(
            text,
            PaddingDimensions::Uniform(0),
            Style::null(),
            false,
        );
        let opts = console.options();
        let segments = padding.rich_console(&console, &opts);
        let output = segments_to_text(&segments);
        assert!(output.contains("Hello"));
    }

    #[test]
    fn test_render_with_left_padding() {
        let console = make_console(20);
        let text = Text::new("Hi", Style::null());
        let padding = Padding::new(
            text,
            PaddingDimensions::Full(0, 0, 0, 4),
            Style::null(),
            true,
        );
        let opts = console.options();
        let segments = padding.rich_console(&console, &opts);
        let output = segments_to_text(&segments);
        // Should have 4 spaces before "Hi"
        assert!(output.contains("    Hi"));
    }

    #[test]
    fn test_render_top_bottom_padding() {
        let console = make_console(20);
        let text = Text::new("X", Style::null());
        let padding = Padding::new(
            text,
            PaddingDimensions::Full(2, 0, 3, 0),
            Style::null(),
            true,
        );
        let opts = console.options();
        let segments = padding.rich_console(&console, &opts);
        let output = segments_to_text(&segments);
        let lines: Vec<&str> = output.split('\n').collect();
        // 2 top blank lines + 1 content line + 3 bottom blank lines = 6 lines
        // (each with a trailing newline, so split gives 7 with last empty)
        let non_empty_lines: Vec<&&str> = lines.iter().filter(|l| !l.is_empty()).collect();
        assert_eq!(non_empty_lines.len(), 6);
    }

    #[test]
    fn test_render_expand_fills_width() {
        let console = make_console(30);
        let text = Text::new("Hi", Style::null());
        let padding = Padding::new(
            text,
            PaddingDimensions::Uniform(1),
            Style::null(),
            true,
        );
        let opts = console.options();
        let segments = padding.rich_console(&console, &opts);
        let output = segments_to_text(&segments);
        let lines: Vec<&str> = output.split('\n').collect();
        // First non-empty line (top padding) should be 30 chars wide
        let top_line = lines[0];
        assert_eq!(cell_len(top_line), 30);
    }

    #[test]
    fn test_render_no_expand_minimal_width() {
        let console = make_console(80);
        let text = Text::new("AB", Style::null());
        let padding = Padding::new(
            text,
            PaddingDimensions::Full(0, 1, 0, 1),
            Style::null(),
            false,
        );
        let opts = console.options();
        let segments = padding.rich_console(&console, &opts);
        let output = segments_to_text(&segments);
        // Width should be content(2) + left(1) + right(1) = 4
        let first_line: &str = output.split('\n').next().unwrap();
        assert_eq!(cell_len(first_line), 4);
    }

    #[test]
    fn test_indent_rendering() {
        let console = make_console(40);
        let text = Text::new("indented", Style::null());
        let padding = Padding::indent(text, 8);
        let opts = console.options();
        let segments = padding.rich_console(&console, &opts);
        let output = segments_to_text(&segments);
        assert!(output.contains("        indented"));
    }

    #[test]
    fn test_measure() {
        let console = make_console(40);
        let text = Text::new("Hello", Style::null());
        let padding = Padding::new(
            text,
            PaddingDimensions::Full(0, 2, 0, 2),
            Style::null(),
            true,
        );
        let opts = console.options();
        let m = padding.measure(&console, &opts);
        // min: 5 + 2 + 2 = 9, max: 40 (expand)
        assert_eq!(m.minimum, 9);
        assert_eq!(m.maximum, 40);
    }

    #[test]
    fn test_measure_no_expand() {
        let console = make_console(40);
        let text = Text::new("Hello", Style::null());
        let padding = Padding::new(
            text,
            PaddingDimensions::Full(0, 2, 0, 2),
            Style::null(),
            false,
        );
        let opts = console.options();
        let m = padding.measure(&console, &opts);
        // min: 9, max: min(9, 40) = 9
        assert_eq!(m.maximum, 9);
    }

    #[test]
    fn test_padding_with_styled_content() {
        let console = make_console(20);
        let text = Text::styled("Bold", Style::parse("bold").unwrap());
        let padding = Padding::new(
            text,
            PaddingDimensions::Uniform(1),
            Style::null(),
            true,
        );
        let opts = console.options();
        let segments = padding.rich_console(&console, &opts);
        let plain: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(plain.contains("Bold"));
    }

    #[test]
    fn test_padding_dimensions_equality() {
        assert_eq!(PaddingDimensions::Uniform(1), PaddingDimensions::Uniform(1));
        assert_ne!(PaddingDimensions::Uniform(1), PaddingDimensions::Uniform(2));
        assert_eq!(PaddingDimensions::Pair(1, 2), PaddingDimensions::Pair(1, 2));
        assert_ne!(PaddingDimensions::Pair(1, 2), PaddingDimensions::Pair(2, 1));
    }
}
