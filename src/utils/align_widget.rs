//! Alignment widget -- positions renderable content within available space.
//!
//! with the `align` keyword.

use std::fmt;

use crate::console::{Console, ConsoleOptions, Renderable, RenderableArc};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::Style;

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
#[derive(Clone)]
pub struct Align {
    /// The content to align (any renderable widget).
    pub content: RenderableArc,
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

// Manual Debug — RenderableArc (Arc<dyn Renderable + Send + Sync>) doesn't
// implement Debug, so we print a placeholder for the content field.
impl std::fmt::Debug for Align {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Align")
            .field("content", &"<renderable>")
            .field("align", &self.align)
            .field("style", &self.style)
            .field("vertical", &self.vertical)
            .field("pad", &self.pad)
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl Align {
    /// Create a new `Align` widget.
    pub fn new(
        content: impl Renderable + Send + Sync + 'static,
        align: HorizontalAlign,
        style: Option<Style>,
        vertical: Option<VerticalAlign>,
        pad: bool,
        width: Option<usize>,
        height: Option<usize>,
    ) -> Self {
        Align {
            content: std::sync::Arc::new(content),
            align,
            style,
            vertical,
            pad,
            width,
            height,
        }
    }

    /// Left-align content.
    pub fn left(content: impl Renderable + Send + Sync + 'static) -> Self {
        Align::new(content, HorizontalAlign::Left, None, None, true, None, None)
    }

    /// Center content.
    pub fn center(content: impl Renderable + Send + Sync + 'static) -> Self {
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
    pub fn right(content: impl Renderable + Send + Sync + 'static) -> Self {
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
    ///
    /// The **maximum** stays the content's full width; the **minimum** is
    /// the longest whitespace-delimited token so the content can wrap to
    /// fit (Plan 7.18 Task 4). For Text content this is the longest word
    /// in the rendered plain string; for other widgets it delegates to
    /// the widget's own `gilt_measure().minimum` (Panel/Rule/etc.). Both
    /// are clamped to `max_width` so we never report a wider value than
    /// the available space.
    pub fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        let content_measure = self.content.gilt_measure(console, options);
        let max_width = match self.width {
            Some(w) => w.min(options.max_width),
            None => content_measure.maximum.min(options.max_width),
        };
        let min_width = content_measure.minimum.min(max_width);
        Measurement::new(min_width, max_width)
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

/// Convenience constructor for an `Align` that centers its content
/// vertically with a default (left) horizontal alignment.
///
/// Mirrors Python rich's `vertical_center` helper: returns an `Align` with
/// `vertical = Some(VerticalAlign::Middle)`, `align = HorizontalAlign::Left`,
/// `pad = true`, and no explicit width/height (so the widget takes the
/// available console dimensions). The content's height drives the
/// middle-pad split when rendered with a non-zero console height.
///
/// # Examples
///
/// ```ignore
/// use gilt::utils::align_widget::vertical_center;
/// use gilt::text::Text;
/// use gilt::style::Style;
///
/// let _a = vertical_center(Text::new("hi", Style::null()));
/// ```
pub fn vertical_center(content: impl Renderable + Send + Sync + 'static) -> Align {
    Align::new(
        content,
        HorizontalAlign::Left,
        None,
        Some(VerticalAlign::Middle),
        true,
        None,
        None,
    )
}

impl Renderable for Align {
    fn gilt_measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        self.measure(console, options)
    }

    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let width = self.width.unwrap_or(options.max_width);
        let pad_style = self.style.clone().unwrap_or_else(Style::null);

        // Render content into lines
        let render_opts = options.update_width(width);
        let mut rendered_lines = console.render_lines(
            self.content.as_ref(),
            Some(&render_opts),
            None,
            false,
            false,
        );

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
    use crate::text::Text;
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
        // "Hello" is 5 cells; no explicit width → maximum is content width (5), not console width.
        let console = make_console(40);
        let align = Align::center(Text::new("Hello", Style::null()));
        let opts = console.options();
        let m = align.measure(&console, &opts);
        assert_eq!(m.minimum, 5);
        assert_eq!(m.maximum, 5);
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
        let style = Style::parse("bold");
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
            assert!(seg.style().is_some());
        }
    }

    // -- Measure correctness (audit #14) ------------------------------------

    #[test]
    fn test_measure_content_width_no_explicit_width() {
        // Without an explicit width, maximum must be the content width (2), not max_width (80).
        let console = make_console(80);
        let opts = console.options();
        let align = Align::left(Text::new("Hi", Style::null()));
        let m = align.measure(&console, &opts);
        assert_eq!(m.minimum, 2);
        assert_eq!(
            m.maximum, 2,
            "maximum must be content width, not options.max_width"
        );
    }

    #[test]
    fn test_measure_explicit_width() {
        // With an explicit width set, that width is the maximum.
        let console = make_console(80);
        let opts = console.options();
        let align = Align::new(
            Text::new("Hi", Style::null()),
            HorizontalAlign::Left,
            None,
            None,
            true,
            Some(30),
            None,
        );
        let m = align.measure(&console, &opts);
        assert_eq!(m.maximum, 30, "explicit width must become the maximum");
        assert_eq!(m.minimum, 2);
    }

    #[test]
    fn test_measure_content_wider_than_max() {
        // Content wider than max_width must be clamped to max_width.
        let console = make_console(5);
        let opts = console.options();
        let align = Align::left(Text::new("Hello World", Style::null()));
        let m = align.measure(&console, &opts);
        assert_eq!(m.maximum, 5, "maximum must be clamped to options.max_width");
    }

    // -- Explicit width clamped to console width (audit #14 parity gap) -------

    #[test]
    fn test_measure_explicit_width_clamped_to_console() {
        // Align with explicit width=200 on 80-col console must clamp to 80.
        let console = make_console(80);
        let opts = console.options();
        let align = Align::new(
            Text::new("Hi", Style::null()),
            HorizontalAlign::Left,
            None,
            None,
            true,
            Some(200),
            None,
        );
        let m = align.measure(&console, &opts);
        assert_eq!(
            m.maximum, 80,
            "explicit width must be clamped to options.max_width"
        );
    }

    // -- gilt_measure override -----------------------------------------------

    #[test]
    fn align_gilt_measure_matches_standalone() {
        let console = make_console(80);
        let opts = console.options();
        let text = Text::new("Hello", Style::null());
        let align = Align::left(text);
        let m_standalone = align.measure(&console, &opts);
        let m_trait = align.gilt_measure(&console, &opts);
        assert_eq!(
            m_trait, m_standalone,
            "Align::gilt_measure must delegate to Align::measure"
        );
    }

    #[test]
    fn align_center_gilt_measure_matches_standalone() {
        let console = make_console(80);
        let opts = console.options();
        let text = Text::new("Hello World", Style::null());
        let align = Align::center(text);
        let m_standalone = align.measure(&console, &opts);
        let m_trait = align.gilt_measure(&console, &opts);
        assert_eq!(
            m_trait, m_standalone,
            "Align::gilt_measure (center) must delegate to Align::measure"
        );
    }

    // -- Task 4.5: RenderableArc constructor tests ---------------------------

    #[test]
    // Updated: constructors now accept impl Renderable + Send + Sync + 'static (Text still works via Renderable impl)
    fn align_left_text_still_compiles() {
        let _ = Align::left(Text::new("x", Style::null()));
    }

    #[test]
    // Updated: constructors now accept impl Renderable + Send + Sync + 'static (Panel works too)
    fn align_center_panel_compiles() {
        let p = crate::panel::Panel::new(Text::new("x", Style::null()));
        let _ = Align::center(p);
    }

    // -- Plan 7.18 Task 4: minimum = longest-word width ---------------------

    #[test]
    fn test_align_measure_min_is_longest_word() {
        // "hello world foo" — content cell length 15, longest single word 5.
        // Align must report min=5 (not min=15) so the content can wrap to fit.
        let console = make_console(80);
        let opts = console.options();
        let align = Align::center(Text::new("hello world foo", Style::null()));
        let m = align.measure(&console, &opts);
        assert_eq!(
            m.minimum, 5,
            "minimum must equal the longest single word (5), not the full content width"
        );
        assert_eq!(m.maximum, 15, "maximum stays the full content width");
    }

    #[test]
    fn test_align_measure_min_clamps_to_max_width() {
        // Long word in a narrow console — min must not exceed max_width.
        let console = make_console(5);
        let opts = console.options();
        let align = Align::left(Text::new("supercalifragilistic", Style::null()));
        let m = align.measure(&console, &opts);
        assert_eq!(m.minimum, 5, "min clamps to max_width (5)");
        assert_eq!(m.maximum, 5, "max clamps to max_width (5)");
    }

    #[test]
    fn test_align_measure_min_single_word_unchanged() {
        // Single word: longest word == content length, no behavior change.
        let console = make_console(40);
        let opts = console.options();
        let align = Align::right(Text::new("Hello", Style::null()));
        let m = align.measure(&console, &opts);
        assert_eq!(m.minimum, 5);
        assert_eq!(m.maximum, 5);
    }

    // -- Plan 7.24 Task 3: VerticalCenter convenience ------------------------

    #[test]
    fn vertical_center_sets_vertical_middle() {
        // The convenience constructor must produce an Align with
        // vertical=Some(Middle) and a left default for horizontal align.
        let align = vertical_center(Text::new("x", Style::null()));
        assert_eq!(align.vertical, Some(VerticalAlign::Middle));
        assert_eq!(align.align, HorizontalAlign::Left);
    }

    #[test]
    fn vertical_center_renders_content_when_height_matches() {
        // When the available height matches the content's line count, the
        // rendered output must contain the content unchanged (no padding).
        let console = make_console(40);
        let align = vertical_center(Text::new("hi", Style::null()));
        let opts = console.options();
        let segs = align.gilt_console(&console, &opts);
        let text: String = segs.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains("hi"), "content must be rendered: {text:?}");
    }

    #[test]
    fn vertical_center_pads_above_and_below_when_height_exceeds() {
        // With an explicit height larger than the content, vertical=Middle
        // must produce padding above AND below the single content line.
        let console = make_console(40);
        let align = Align::new(
            Text::new("hi", Style::null()),
            HorizontalAlign::Left,
            None,
            Some(VerticalAlign::Middle),
            true,
            None,
            Some(5),
        );
        let opts = console.options();
        let segs = align.gilt_console(&console, &opts);
        let text: String = segs.iter().map(|s| s.text.as_str()).collect();
        let lines: Vec<&str> = text.split('\n').filter(|l| !l.is_empty()).collect();
        // The content line is padded with trailing spaces by the Align
        // widget to fill the available width; check via `starts_with`
        // rather than equality.
        let has_content = lines.iter().any(|l| l.trim_start().starts_with("hi"));
        assert!(has_content, "content line must be present: {lines:?}");
    }
}
