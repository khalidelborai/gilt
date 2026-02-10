//! Alignment widget -- positions renderable content within available space.
//!
//! Port of Python's `rich/align.py`. Named `align_widget` to avoid conflict
//! with the `align` keyword.

use std::fmt;

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;

// ---------------------------------------------------------------------------
// Alignment enums
// ---------------------------------------------------------------------------

/// Horizontal alignment method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalAlign {
    /// Align content to the left edge.
    Left,
    /// Center content horizontally.
    Center,
    /// Align content to the right edge.
    Right,
}

/// Vertical alignment method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAlign {
    /// Align content to the top.
    Top,
    /// Center content vertically.
    Middle,
    /// Align content to the bottom.
    Bottom,
}

// ---------------------------------------------------------------------------
// Align
// ---------------------------------------------------------------------------

/// A widget that aligns its content horizontally (and optionally vertically)
/// within the available console space.
#[derive(Debug, Clone)]
pub struct Align {
    /// The content to align.
    pub content: Text,
    /// Horizontal alignment.
    pub align: HorizontalAlign,
    /// Optional style for the padding whitespace.
    pub style: Option<Style>,
    /// Optional vertical alignment (requires `height`).
    pub vertical: Option<VerticalAlign>,
    /// Whether to pad lines on the right to fill available width.
    pub pad: bool,
    /// Override width (if `None`, uses `options.max_width`).
    pub width: Option<usize>,
    /// Override height for vertical alignment.
    pub height: Option<usize>,
}

impl Align {
    /// Create a new `Align` widget.
    pub fn new(
        content: Text,
        align: HorizontalAlign,
        style: Option<Style>,
        vertical: Option<VerticalAlign>,
        pad: bool,
        width: Option<usize>,
        height: Option<usize>,
    ) -> Self {
        Align {
            content,
            align,
            style,
            vertical,
            pad,
            width,
            height,
        }
    }

    /// Left-align content.
    pub fn left(content: Text) -> Self {
        Align::new(content, HorizontalAlign::Left, None, None, true, None, None)
    }

    /// Center content.
    pub fn center(content: Text) -> Self {
        Align::new(
            content,
            HorizontalAlign::Center,
            None,
            None,
            true,
            None,
            None,
        )
    }

    /// Right-align content.
    pub fn right(content: Text) -> Self {
        Align::new(
            content,
            HorizontalAlign::Right,
            None,
            None,
            true,
            None,
            None,
        )
    }

    /// Measure the minimum and maximum width requirements.
    pub fn measure(&self, _console: &Console, options: &ConsoleOptions) -> Measurement {
        let content_width = self.content.cell_len();
        Measurement::new(content_width, options.max_width)
    }

    /// Generate vertically-padded blank lines above or below content.
    fn vertical_pad_lines(
        &self,
        lines: Vec<Vec<Segment>>,
        width: usize,
        height: usize,
    ) -> Vec<Vec<Segment>> {
        let content_height = lines.len();
        if content_height >= height {
            return lines;
        }

        let pad_style = self.style.clone().unwrap_or_else(Style::null);
        let blank_segment = Segment::styled(&" ".repeat(width), pad_style);
        let blank_line = vec![blank_segment];

        let excess = height - content_height;
        match self.vertical.unwrap_or(VerticalAlign::Top) {
            VerticalAlign::Top => {
                let mut result = lines;
                for _ in 0..excess {
                    result.push(blank_line.clone());
                }
                result
            }
            VerticalAlign::Middle => {
                let top = excess / 2;
                let bottom = excess - top;
                let mut result = Vec::with_capacity(height);
                for _ in 0..top {
                    result.push(blank_line.clone());
                }
                result.extend(lines);
                for _ in 0..bottom {
                    result.push(blank_line.clone());
                }
                result
            }
            VerticalAlign::Bottom => {
                let mut result = Vec::with_capacity(height);
                for _ in 0..excess {
                    result.push(blank_line.clone());
                }
                result.extend(lines);
                result
            }
        }
    }
}

impl Renderable for Align {
    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let width = self.width.unwrap_or(options.max_width);
        let pad_style = self.style.clone().unwrap_or_else(Style::null);

        // Render content into lines
        let render_opts = options.update_width(width);
        let mut rendered_lines =
            console.render_lines(&self.content, Some(&render_opts), None, false, false);

        // Apply horizontal alignment to each line
        for line in rendered_lines.iter_mut() {
            let line_width = Segment::get_line_length(line);
            if line_width >= width {
                continue;
            }
            let excess = width - line_width;

            match self.align {
                HorizontalAlign::Left => {
                    if self.pad {
                        line.push(Segment::styled(&" ".repeat(excess), pad_style.clone()));
                    }
                }
                HorizontalAlign::Center => {
                    let left = excess / 2;
                    let right = excess - left;
                    if left > 0 {
                        line.insert(0, Segment::styled(&" ".repeat(left), pad_style.clone()));
                    }
                    if self.pad && right > 0 {
                        line.push(Segment::styled(&" ".repeat(right), pad_style.clone()));
                    }
                }
                HorizontalAlign::Right => {
                    line.insert(0, Segment::styled(&" ".repeat(excess), pad_style.clone()));
                }
            }
        }

        // Apply vertical alignment if height is specified
        if let Some(height) = self.height {
            rendered_lines = self.vertical_pad_lines(rendered_lines, width, height);
        }

        // Flatten lines into segments with newlines between them
        let mut segments = Vec::new();
        let line_count = rendered_lines.len();
        for (i, line) in rendered_lines.into_iter().enumerate() {
            segments.extend(line);
            if i + 1 < line_count || self.height.is_some() {
                segments.push(Segment::line());
            }
        }
        // Always end with a newline
        if let Some(last) = segments.last() {
            if last.text != "\n" {
                segments.push(Segment::line());
            }
        }

        segments
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl fmt::Display for Align {
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
    use crate::utils::cells::cell_len;

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

    fn get_content_lines(output: &str) -> Vec<&str> {
        output.split('\n').filter(|l| !l.is_empty()).collect()
    }

    // -- Constructor helpers ------------------------------------------------

    #[test]
    fn test_left_constructor() {
        let align = Align::left(Text::new("X", Style::null()));
        assert_eq!(align.align, HorizontalAlign::Left);
        assert!(align.pad);
    }

    #[test]
    fn test_center_constructor() {
        let align = Align::center(Text::new("X", Style::null()));
        assert_eq!(align.align, HorizontalAlign::Center);
    }

    #[test]
    fn test_right_constructor() {
        let align = Align::right(Text::new("X", Style::null()));
        assert_eq!(align.align, HorizontalAlign::Right);
    }

    // -- Horizontal alignment -----------------------------------------------

    #[test]
    fn test_left_align() {
        let console = make_console(10);
        let align = Align::left(Text::new("Hi", Style::null()));
        let opts = console.options();
        let segments = align.gilt_console(&console, &opts);
        let output = segments_to_text(&segments);
        let lines = get_content_lines(&output);
        assert!(!lines.is_empty());
        let line = lines[0];
        assert!(line.starts_with("Hi"));
        // With pad=true, should fill to width
        assert_eq!(cell_len(line), 10);
    }

    #[test]
    fn test_center_align() {
        let console = make_console(10);
        let align = Align::center(Text::new("AB", Style::null()));
        let opts = console.options();
        let segments = align.gilt_console(&console, &opts);
        let output = segments_to_text(&segments);
        let lines = get_content_lines(&output);
        let line = lines[0];
        // "AB" is 2 chars, excess=8, left=4, right=4
        assert!(line.starts_with("    AB"));
        assert_eq!(cell_len(line), 10);
    }

    #[test]
    fn test_right_align() {
        let console = make_console(10);
        let align = Align::right(Text::new("AB", Style::null()));
        let opts = console.options();
        let segments = align.gilt_console(&console, &opts);
        let output = segments_to_text(&segments);
        let lines = get_content_lines(&output);
        let line = lines[0];
        // "AB" is 2 chars, 8 spaces on the left
        assert!(line.starts_with("        AB"));
    }

    #[test]
    fn test_center_odd_excess() {
        let console = make_console(11);
        let align = Align::center(Text::new("AB", Style::null()));
        let opts = console.options();
        let segments = align.gilt_console(&console, &opts);
        let output = segments_to_text(&segments);
        let lines = get_content_lines(&output);
        let line = lines[0];
        // "AB" is 2, excess=9, left=4, right=5
        assert_eq!(cell_len(line), 11);
        // First 4 chars should be spaces
        assert!(line.starts_with("    AB"));
    }

    // -- No padding ---------------------------------------------------------

    #[test]
    fn test_no_pad_left() {
        let console = make_console(10);
        let align = Align::new(
            Text::new("Hi", Style::null()),
            HorizontalAlign::Left,
            None,
            None,
            false,
            None,
            None,
        );
        let opts = console.options();
        let segments = align.gilt_console(&console, &opts);
        let output = segments_to_text(&segments);
        let lines = get_content_lines(&output);
        let line = lines[0];
        // No right padding, so just "Hi"
        assert_eq!(line, "Hi");
    }

    // -- Vertical alignment -------------------------------------------------

    #[test]
    fn test_vertical_top() {
        let console = make_console(10);
        let align = Align::new(
            Text::new("X", Style::null()),
            HorizontalAlign::Left,
            None,
            Some(VerticalAlign::Top),
            true,
            None,
            Some(5),
        );
        let opts = console.options();
        let segments = align.gilt_console(&console, &opts);
        let output = segments_to_text(&segments);
        let lines = get_content_lines(&output);
        // 1 content line + 4 blank = 5 total
        assert_eq!(lines.len(), 5);
        assert!(lines[0].contains('X'));
    }

    #[test]
    fn test_vertical_middle() {
        let console = make_console(10);
        let align = Align::new(
            Text::new("X", Style::null()),
            HorizontalAlign::Left,
            None,
            Some(VerticalAlign::Middle),
            true,
            None,
            Some(5),
        );
        let opts = console.options();
        let segments = align.gilt_console(&console, &opts);
        let output = segments_to_text(&segments);
        let lines = get_content_lines(&output);
        assert_eq!(lines.len(), 5);
        // Content should be at index 2 (middle)
        assert!(lines[2].contains('X'));
    }

    #[test]
    fn test_vertical_bottom() {
        let console = make_console(10);
        let align = Align::new(
            Text::new("X", Style::null()),
            HorizontalAlign::Left,
            None,
            Some(VerticalAlign::Bottom),
            true,
            None,
            Some(5),
        );
        let opts = console.options();
        let segments = align.gilt_console(&console, &opts);
        let output = segments_to_text(&segments);
        let lines = get_content_lines(&output);
        assert_eq!(lines.len(), 5);
        // Content should be at last position
        assert!(lines[4].contains('X'));
    }

    // -- Custom width -------------------------------------------------------

    #[test]
    fn test_custom_width() {
        let console = make_console(80);
        let align = Align::new(
            Text::new("AB", Style::null()),
            HorizontalAlign::Center,
            None,
            None,
            true,
            Some(20),
            None,
        );
        let opts = console.options();
        let segments = align.gilt_console(&console, &opts);
        let output = segments_to_text(&segments);
        let lines = get_content_lines(&output);
        let line = lines[0];
        assert_eq!(cell_len(line), 20);
    }

    // -- Measure ------------------------------------------------------------

    #[test]
    fn test_measure() {
        let console = make_console(40);
        let align = Align::center(Text::new("Hello", Style::null()));
        let opts = console.options();
        let m = align.measure(&console, &opts);
        assert_eq!(m.minimum, 5);
        assert_eq!(m.maximum, 40);
    }

    // -- Enum equality ------------------------------------------------------

    #[test]
    fn test_horizontal_align_equality() {
        assert_eq!(HorizontalAlign::Left, HorizontalAlign::Left);
        assert_ne!(HorizontalAlign::Left, HorizontalAlign::Right);
    }

    #[test]
    fn test_vertical_align_equality() {
        assert_eq!(VerticalAlign::Top, VerticalAlign::Top);
        assert_ne!(VerticalAlign::Top, VerticalAlign::Bottom);
    }

    // -- Content fills width ------------------------------------------------

    #[test]
    fn test_content_fills_width_no_alignment_needed() {
        let console = make_console(5);
        let align = Align::center(Text::new("ABCDE", Style::null()));
        let opts = console.options();
        let segments = align.gilt_console(&console, &opts);
        let output = segments_to_text(&segments);
        let lines = get_content_lines(&output);
        assert!(lines[0].contains("ABCDE"));
    }

    // -- With style ---------------------------------------------------------

    #[test]
    fn test_with_style() {
        let console = make_console(10);
        let style = Style::parse("bold").unwrap();
        let align = Align::new(
            Text::new("X", Style::null()),
            HorizontalAlign::Center,
            Some(style),
            None,
            true,
            None,
            None,
        );
        let opts = console.options();
        let segments = align.gilt_console(&console, &opts);
        // Padding segments should have the bold style
        let padding_segments: Vec<&Segment> = segments
            .iter()
            .filter(|s| s.text.trim().is_empty() && !s.text.contains('\n') && !s.text.is_empty())
            .collect();
        assert!(!padding_segments.is_empty());
        for seg in padding_segments {
            assert!(seg.style.is_some());
        }
    }
}
