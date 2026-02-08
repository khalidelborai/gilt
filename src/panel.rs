//! Panel widget -- a bordered box around content with optional title/subtitle.
//!
//! Port of Python's `rich/panel.py`.

use crate::align_widget::HorizontalAlign;
use crate::box_chars::{BoxChars, ROUNDED};
use crate::console::{Console, ConsoleOptions, Renderable};
use crate::highlighter::{Highlighter, ReprHighlighter};
use crate::measure::Measurement;
use crate::padding::PaddingDimensions;
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;

// ---------------------------------------------------------------------------
// Panel
// ---------------------------------------------------------------------------

/// A bordered box around content, with optional title and subtitle in the border.
#[derive(Debug, Clone)]
pub struct Panel {
    /// The inner content.
    pub content: Text,
    /// Box-drawing character set (reference to one of the 19 static constants).
    pub box_chars: &'static BoxChars,
    /// Optional title rendered in the top border.
    pub title: Option<Text>,
    /// Alignment of the title within the top border.
    pub title_align: HorizontalAlign,
    /// Optional subtitle rendered in the bottom border.
    pub subtitle: Option<Text>,
    /// Alignment of the subtitle within the bottom border.
    pub subtitle_align: HorizontalAlign,
    /// If true, expand to fill available width.
    pub expand: bool,
    /// Style applied to the content area.
    pub style: Style,
    /// Style applied to the border characters.
    pub border_style: Style,
    /// Optional fixed width for the panel.
    pub width: Option<usize>,
    /// Optional fixed height for the content area.
    pub height: Option<usize>,
    /// Inner padding (default `Pair(0, 1)` = 1 space each side horizontally).
    pub padding: PaddingDimensions,
    /// If true, apply `ReprHighlighter` to the content before rendering.
    pub highlight: bool,
}

impl Panel {
    /// Create a new expanding `Panel` with ROUNDED box and default padding.
    pub fn new(content: Text) -> Self {
        Panel {
            content,
            box_chars: &ROUNDED,
            title: None,
            title_align: HorizontalAlign::Center,
            subtitle: None,
            subtitle_align: HorizontalAlign::Center,
            expand: true,
            style: Style::null(),
            border_style: Style::null(),
            width: None,
            height: None,
            padding: PaddingDimensions::Pair(0, 1),
            highlight: false,
        }
    }

    /// Create a non-expanding (fit-to-content) `Panel`.
    pub fn fit(content: Text) -> Self {
        let mut panel = Panel::new(content);
        panel.expand = false;
        panel
    }

    // -- Builder methods ----------------------------------------------------

    /// Set the box-drawing character set.
    #[must_use]
    pub fn box_chars(mut self, box_chars: &'static BoxChars) -> Self {
        self.box_chars = box_chars;
        self
    }

    /// Set the title text.
    #[must_use]
    pub fn title(mut self, title: Text) -> Self {
        self.title = Some(title);
        self
    }

    /// Set the title alignment.
    #[must_use]
    pub fn title_align(mut self, align: HorizontalAlign) -> Self {
        self.title_align = align;
        self
    }

    /// Set the subtitle text.
    #[must_use]
    pub fn subtitle(mut self, subtitle: Text) -> Self {
        self.subtitle = Some(subtitle);
        self
    }

    /// Set the subtitle alignment.
    #[must_use]
    pub fn subtitle_align(mut self, align: HorizontalAlign) -> Self {
        self.subtitle_align = align;
        self
    }

    /// Set whether the panel expands to fill available width.
    #[must_use]
    pub fn expand(mut self, expand: bool) -> Self {
        self.expand = expand;
        self
    }

    /// Set the content style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the border style.
    #[must_use]
    pub fn border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }

    /// Set a fixed width.
    #[must_use]
    pub fn width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Set a fixed height for the content area.
    #[must_use]
    pub fn height(mut self, height: usize) -> Self {
        self.height = Some(height);
        self
    }

    /// Set the inner padding.
    #[must_use]
    pub fn padding(mut self, padding: PaddingDimensions) -> Self {
        self.padding = padding;
        self
    }

    /// Enable or disable `ReprHighlighter` on the content.
    #[must_use]
    pub fn highlight(mut self, highlight: bool) -> Self {
        self.highlight = highlight;
        self
    }

    /// Measure the minimum and maximum width requirements.
    pub fn measure(&self, _console: &Console, _options: &ConsoleOptions) -> Measurement {
        let (_, right, _, left) = self.padding.unpack();
        let padding = left + right;
        let content_width = self.content.cell_len();
        let w = if let Some(fixed) = self.width {
            fixed
        } else {
            content_width + padding + 2
        };
        Measurement::new(w, w)
    }
}

// ---------------------------------------------------------------------------
// Helper: align title/subtitle text within the border
// ---------------------------------------------------------------------------

/// Render a title (or subtitle) aligned within border fill characters.
///
/// Returns a list of segments representing: `fill_char...title...fill_char`.
/// `available_width` is the space between the two anchor `top`/`bottom` chars
/// that flank the title area (i.e. total_width - 4, since we have
/// `border_char + fill_char` on each side).
fn align_title_segments(
    title: &Text,
    available_width: usize,
    align: HorizontalAlign,
    fill_char: char,
    border_style: &Style,
) -> Vec<Segment> {
    let mut title_text = title.clone();

    // Prepare the title: replace newlines, expand tabs, pad with 1 space each side
    let plain = title_text.plain().replace('\n', " ");
    title_text.set_plain(&plain);
    title_text.expand_tabs(None);
    title_text.pad(1, ' ');

    // Truncate to fit
    let title_cell_len = title_text.cell_len();
    if title_cell_len > available_width {
        title_text.truncate(available_width, None, false);
    }

    let title_width = title_text.cell_len();
    let fill_remaining = available_width.saturating_sub(title_width);

    // Render the title into segments (strip trailing newline from Text::render)
    let title_segments: Vec<Segment> = title_text
        .render()
        .into_iter()
        .filter(|s| s.text != "\n")
        .collect();

    let mut result = Vec::new();

    let (left_fill, right_fill) = match align {
        HorizontalAlign::Left => (0, fill_remaining),
        HorizontalAlign::Right => (fill_remaining, 0),
        HorizontalAlign::Center => {
            let left = fill_remaining / 2;
            let right = fill_remaining - left;
            (left, right)
        }
    };

    if left_fill > 0 {
        let fill: String = std::iter::repeat_n(fill_char, left_fill).collect();
        result.push(Segment::styled(&fill, border_style.clone()));
    }

    result.extend(title_segments);

    if right_fill > 0 {
        let fill: String = std::iter::repeat_n(fill_char, right_fill).collect();
        result.push(Segment::styled(&fill, border_style.clone()));
    }

    result
}

// ---------------------------------------------------------------------------
// Renderable
// ---------------------------------------------------------------------------

impl Renderable for Panel {
    fn rich_console(&self, _console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let bx = self.box_chars;
        let (pad_top, pad_right, pad_bottom, pad_left) = self.padding.unpack();
        let horizontal_padding = pad_left + pad_right;

        // Determine the panel width
        let max_width = if let Some(w) = self.width {
            w.min(options.max_width)
        } else {
            options.max_width
        };

        // Calculate child_width (interior width, excluding the two border columns)
        let mut child_width = if self.expand {
            max_width.saturating_sub(2)
        } else {
            // Fit mode: measure the content
            let content_width = self.content.cell_len();
            content_width + horizontal_padding
        };

        // If there's a title, ensure child_width is wide enough.
        // child_width must be >= padded_title_len + 2 so the title fits
        // between the two anchor fill chars (top_left + fill ... fill + top_right).
        if let Some(ref title) = self.title {
            let mut title_text = title.clone();
            let plain = title_text.plain().replace('\n', " ");
            title_text.set_plain(&plain);
            title_text.expand_tabs(None);
            title_text.pad(1, ' ');
            let title_cell_len = title_text.cell_len();
            child_width = child_width.max(title_cell_len + 2);
        }

        // If there's a subtitle, ensure child_width is wide enough.
        if let Some(ref subtitle) = self.subtitle {
            let mut sub_text = subtitle.clone();
            let plain = sub_text.plain().replace('\n', " ");
            sub_text.set_plain(&plain);
            sub_text.expand_tabs(None);
            sub_text.pad(1, ' ');
            let sub_cell_len = sub_text.cell_len();
            child_width = child_width.max(sub_cell_len + 2);
        }

        // Clamp child_width to max_width - 2
        child_width = child_width.min(max_width.saturating_sub(2));

        // The total panel width
        let width = child_width + 2;

        // Render content lines.
        // We wrap the content text ourselves and render each line individually
        // to avoid the double-newline issue that occurs when Text.rich_console's
        // wrap (which includes separators in line text) combines with the
        // between-lines Segment::line().
        let inner_width = child_width.saturating_sub(horizontal_padding).max(1);
        let mut content_copy = self.content.clone();
        content_copy.end = String::new();
        let tab_size = content_copy.tab_size.unwrap_or(8);

        // Apply ReprHighlighter if highlight is enabled
        if self.highlight {
            ReprHighlighter.highlight(&mut content_copy);
        }
        let wrapped = content_copy.wrap(
            inner_width,
            content_copy.justify,
            content_copy.overflow,
            tab_size,
            content_copy.no_wrap.unwrap_or(false),
        );
        let mut lines: Vec<Vec<Segment>> = Vec::new();
        for mut line in wrapped.lines {
            line.end = String::new();
            // Strip trailing newline that Text::split("\n", true, true) embeds
            // in each line's plain text during wrap().
            line.remove_suffix("\n");
            let line_segments = line.render();
            // Apply content style if set
            let styled = if !self.style.is_null() {
                Segment::apply_style(&line_segments, Some(self.style.clone()), None)
            } else {
                line_segments
            };
            let adjusted = Segment::adjust_line_length(&styled, inner_width, &self.style, true);
            lines.push(adjusted);
        }

        // Apply fixed height if specified
        if let Some(h) = self.height {
            lines = Segment::set_shape(&lines, inner_width, Some(h), Some(&self.style), false);
        }

        let mut segments = Vec::new();

        // ── Top border ────────────────────────────────────────────────
        match self.title.as_ref() {
            Some(title) if width > 4 => {
                let available = width.saturating_sub(4); // minus border_char + fill_char on each side

                // top_left + top_fill_char
                let mut left_anchor = String::new();
                left_anchor.push(bx.top_left);
                left_anchor.push(bx.top);
                segments.push(Segment::styled(&left_anchor, self.border_style.clone()));

                // Aligned title within fill chars
                let title_segs = align_title_segments(
                    title,
                    available,
                    self.title_align,
                    bx.top,
                    &self.border_style,
                );
                segments.extend(title_segs);

                // top_fill_char + top_right
                let mut right_anchor = String::new();
                right_anchor.push(bx.top);
                right_anchor.push(bx.top_right);
                segments.push(Segment::styled(&right_anchor, self.border_style.clone()));
            }
            _ => {
                // No title or too narrow: full border line
                let top = bx.get_top(&[child_width]);
                segments.push(Segment::styled(&top, self.border_style.clone()));
            }
        }
        segments.push(Segment::line());

        // ── Top padding rows ──────────────────────────────────────────
        let left_pad_str = " ".repeat(pad_left);
        let right_pad_str = " ".repeat(pad_right);

        for _ in 0..pad_top {
            let mid_l = String::from(bx.mid_left);
            segments.push(Segment::styled(&mid_l, self.border_style.clone()));
            let blank = " ".repeat(child_width);
            segments.push(Segment::styled(&blank, self.style.clone()));
            let mid_r = String::from(bx.mid_right);
            segments.push(Segment::styled(&mid_r, self.border_style.clone()));
            segments.push(Segment::line());
        }

        // ── Content rows ──────────────────────────────────────────────
        for line in &lines {
            // Left border
            let mid_l = String::from(bx.mid_left);
            segments.push(Segment::styled(&mid_l, self.border_style.clone()));

            // Left padding
            if pad_left > 0 {
                segments.push(Segment::styled(&left_pad_str, self.style.clone()));
            }

            // Content segments
            segments.extend(line.iter().cloned());

            // Right padding
            if pad_right > 0 {
                segments.push(Segment::styled(&right_pad_str, self.style.clone()));
            }

            // Right border
            let mid_r = String::from(bx.mid_right);
            segments.push(Segment::styled(&mid_r, self.border_style.clone()));
            segments.push(Segment::line());
        }

        // ── Bottom padding rows ───────────────────────────────────────
        for _ in 0..pad_bottom {
            let mid_l = String::from(bx.mid_left);
            segments.push(Segment::styled(&mid_l, self.border_style.clone()));
            let blank = " ".repeat(child_width);
            segments.push(Segment::styled(&blank, self.style.clone()));
            let mid_r = String::from(bx.mid_right);
            segments.push(Segment::styled(&mid_r, self.border_style.clone()));
            segments.push(Segment::line());
        }

        // ── Bottom border ─────────────────────────────────────────────
        match self.subtitle.as_ref() {
            Some(subtitle) if width > 4 => {
                let available = width.saturating_sub(4);

                let mut left_anchor = String::new();
                left_anchor.push(bx.bottom_left);
                left_anchor.push(bx.bottom_char);
                segments.push(Segment::styled(&left_anchor, self.border_style.clone()));

                let sub_segs = align_title_segments(
                    subtitle,
                    available,
                    self.subtitle_align,
                    bx.bottom_char,
                    &self.border_style,
                );
                segments.extend(sub_segs);

                let mut right_anchor = String::new();
                right_anchor.push(bx.bottom_char);
                right_anchor.push(bx.bottom_right);
                segments.push(Segment::styled(&right_anchor, self.border_style.clone()));
            }
            _ => {
                let bottom = bx.get_bottom(&[child_width]);
                segments.push(Segment::styled(&bottom, self.border_style.clone()));
            }
        }
        segments.push(Segment::line());

        segments
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl std::fmt::Display for Panel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut console = Console::builder()
            .width(f.width().unwrap_or(80))
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
    use crate::box_chars::{ASCII, DOUBLE, HEAVY, SQUARE};
    use crate::cells::cell_len;

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

    fn render_panel(console: &Console, panel: &Panel) -> String {
        let opts = console.options();
        let segments = panel.rich_console(console, &opts);
        segments_to_text(&segments)
    }

    fn content_lines(output: &str) -> Vec<&str> {
        output.split('\n').filter(|l| !l.is_empty()).collect()
    }

    // ── 1. Panel with no title (just border around content) ───────────

    #[test]
    fn test_no_title() {
        let console = make_console(20);
        let panel = Panel::new(Text::new("Hello", Style::null()));
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        // Should have 3 lines: top border, content, bottom border
        assert_eq!(lines.len(), 3);
        // Top border starts with top_left
        assert!(lines[0].starts_with('╭'));
        assert!(lines[0].ends_with('╮'));
        // Content line has mid_left and mid_right
        assert!(lines[1].starts_with('│'));
        assert!(lines[1].ends_with('│'));
        assert!(lines[1].contains("Hello"));
        // Bottom border
        assert!(lines[2].starts_with('╰'));
        assert!(lines[2].ends_with('╯'));
    }

    #[test]
    fn test_no_title_width_fills() {
        let console = make_console(20);
        let panel = Panel::new(Text::new("Hi", Style::null()));
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        // All lines should be 20 cells wide
        for line in &lines {
            assert_eq!(
                cell_len(line),
                20,
                "Line '{}' is {} cells, expected 20",
                line,
                cell_len(line)
            );
        }
    }

    // ── 2. Panel with centered title ──────────────────────────────────

    #[test]
    fn test_centered_title() {
        let console = make_console(30);
        let panel = Panel::new(Text::new("Content", Style::null()))
            .title(Text::new("Title", Style::null()));
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        // Top border should contain the title
        assert!(lines[0].contains("Title"));
        // Should start with top_left
        assert!(lines[0].starts_with('╭'));
        assert!(lines[0].ends_with('╮'));
        assert_eq!(cell_len(lines[0]), 30);
    }

    #[test]
    fn test_centered_title_padded_with_spaces() {
        let console = make_console(30);
        let panel = Panel::new(Text::new("X", Style::null())).title(Text::new("T", Style::null()));
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        // Title should appear with spaces on either side: " T "
        assert!(lines[0].contains(" T "));
    }

    // ── 3. Panel with left/right-aligned title ────────────────────────

    #[test]
    fn test_left_aligned_title() {
        let console = make_console(30);
        let panel = Panel::new(Text::new("Content", Style::null()))
            .title(Text::new("Left", Style::null()))
            .title_align(HorizontalAlign::Left);
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        // The title should be near the left side
        // Pattern: ╭─ Left ─────...─╮
        assert!(lines[0].starts_with('╭'));
        let _after_anchor = &lines[0][3..]; // skip "╭─" (two chars, but unicode)
                                            // The title " Left " should appear early
        assert!(lines[0].contains(" Left "));
        assert_eq!(cell_len(lines[0]), 30);
    }

    #[test]
    fn test_right_aligned_title() {
        let console = make_console(30);
        let panel = Panel::new(Text::new("Content", Style::null()))
            .title(Text::new("Right", Style::null()))
            .title_align(HorizontalAlign::Right);
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        assert!(lines[0].contains(" Right "));
        assert_eq!(cell_len(lines[0]), 30);
    }

    // ── 4. Panel with subtitle ────────────────────────────────────────

    #[test]
    fn test_subtitle() {
        let console = make_console(30);
        let panel = Panel::new(Text::new("Content", Style::null()))
            .subtitle(Text::new("Sub", Style::null()));
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        let bottom = lines.last().unwrap();
        assert!(bottom.contains(" Sub "));
        assert!(bottom.starts_with('╰'));
        assert!(bottom.ends_with('╯'));
        assert_eq!(cell_len(bottom), 30);
    }

    #[test]
    fn test_subtitle_left_aligned() {
        let console = make_console(30);
        let panel = Panel::new(Text::new("X", Style::null()))
            .subtitle(Text::new("SubLeft", Style::null()))
            .subtitle_align(HorizontalAlign::Left);
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        let bottom = lines.last().unwrap();
        assert!(bottom.contains(" SubLeft "));
    }

    // ── 5. Panel.fit() (non-expanding) ────────────────────────────────

    #[test]
    fn test_fit_panel() {
        let console = make_console(80);
        let panel = Panel::fit(Text::new("Hi", Style::null()));
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        // "Hi" is 2 cells + padding(1+1) + border(2) = 6 cells wide
        let expected_width = 6;
        for line in &lines {
            assert_eq!(
                cell_len(line),
                expected_width,
                "Line '{}' is {} cells, expected {}",
                line,
                cell_len(line),
                expected_width
            );
        }
    }

    #[test]
    fn test_fit_panel_wider_title() {
        let console = make_console(80);
        let panel = Panel::fit(Text::new("Hi", Style::null()))
            .title(Text::new("A Longer Title", Style::null()));
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        // The panel should be wide enough for the title
        assert!(lines[0].contains(" A Longer Title "));
        // All lines should have the same width
        let w = cell_len(lines[0]);
        for line in &lines {
            assert_eq!(cell_len(line), w);
        }
    }

    // ── 6. Panel with custom box ──────────────────────────────────────

    #[test]
    fn test_double_box() {
        let console = make_console(20);
        let panel = Panel::new(Text::new("X", Style::null())).box_chars(&DOUBLE);
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        assert!(lines[0].starts_with('╔'));
        assert!(lines[0].ends_with('╗'));
        assert!(lines[1].starts_with('║'));
        assert!(lines[1].ends_with('║'));
        assert!(lines[2].starts_with('╚'));
        assert!(lines[2].ends_with('╝'));
    }

    #[test]
    fn test_heavy_box() {
        let console = make_console(20);
        let panel = Panel::new(Text::new("X", Style::null())).box_chars(&HEAVY);
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        assert!(lines[0].starts_with('┏'));
        assert!(lines[0].ends_with('┓'));
        assert!(lines[1].starts_with('┃'));
        assert!(lines[1].ends_with('┃'));
        assert!(lines[2].starts_with('┗'));
        assert!(lines[2].ends_with('┛'));
    }

    #[test]
    fn test_ascii_box() {
        let console = make_console(20);
        let panel = Panel::new(Text::new("X", Style::null())).box_chars(&ASCII);
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        assert!(lines[0].starts_with('+'));
        assert!(lines[0].ends_with('+'));
        assert!(lines[1].starts_with('|'));
        assert!(lines[1].ends_with('|'));
        assert!(lines[2].starts_with('+'));
        assert!(lines[2].ends_with('+'));
    }

    #[test]
    fn test_square_box() {
        let console = make_console(20);
        let panel = Panel::new(Text::new("X", Style::null())).box_chars(&SQUARE);
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        assert!(lines[0].starts_with('┌'));
        assert!(lines[0].ends_with('┐'));
    }

    // ── 7. Panel with padding ─────────────────────────────────────────

    #[test]
    fn test_custom_padding() {
        let console = make_console(30);
        let panel =
            Panel::new(Text::new("X", Style::null())).padding(PaddingDimensions::Full(1, 2, 1, 2));
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        // 1 top border + 1 pad_top + 1 content + 1 pad_bottom + 1 bottom border = 5
        assert_eq!(lines.len(), 5);
        // Second line (pad_top) should be all spaces inside borders
        assert!(lines[1].starts_with('│'));
        assert!(lines[1].ends_with('│'));
        // Should be only whitespace between borders
        let inner = &lines[1][3..lines[1].len() - 3]; // skip border chars
        assert!(inner.trim().is_empty());
    }

    #[test]
    fn test_zero_padding() {
        let console = make_console(20);
        let panel =
            Panel::new(Text::new("Hello", Style::null())).padding(PaddingDimensions::Uniform(0));
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        // Should be 3 lines: top, content, bottom
        assert_eq!(lines.len(), 3);
        // Content line should have Hello right next to borders
        assert!(lines[1].starts_with('│'));
        // With zero padding, the content starts right after the border
        let inner_start = lines[1].chars().nth(1).unwrap();
        assert_eq!(inner_start, 'H');
    }

    // ── 8. Panel with custom width ────────────────────────────────────

    #[test]
    fn test_custom_width() {
        let console = make_console(80);
        let panel = Panel::new(Text::new("X", Style::null())).width(25);
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        for line in &lines {
            assert_eq!(
                cell_len(line),
                25,
                "Expected width 25, got {}",
                cell_len(line)
            );
        }
    }

    #[test]
    fn test_custom_width_clamped() {
        // width larger than console should be clamped
        let console = make_console(20);
        let panel = Panel::new(Text::new("X", Style::null())).width(50);
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        for line in &lines {
            assert_eq!(cell_len(line), 20);
        }
    }

    // ── 9. Builder pattern chain ──────────────────────────────────────

    #[test]
    fn test_builder_chain() {
        let panel = Panel::new(Text::new("X", Style::null()))
            .box_chars(&DOUBLE)
            .title(Text::new("T", Style::null()))
            .title_align(HorizontalAlign::Left)
            .subtitle(Text::new("S", Style::null()))
            .subtitle_align(HorizontalAlign::Right)
            .expand(false)
            .style(Style::parse("bold").unwrap())
            .border_style(Style::parse("red").unwrap())
            .width(40)
            .height(5)
            .padding(PaddingDimensions::Uniform(2));

        assert_eq!(panel.box_chars.top_left, '╔');
        assert!(panel.title.is_some());
        assert_eq!(panel.title_align, HorizontalAlign::Left);
        assert!(panel.subtitle.is_some());
        assert_eq!(panel.subtitle_align, HorizontalAlign::Right);
        assert!(!panel.expand);
        assert!(panel.style.bold() == Some(true));
        assert!(panel.border_style.color().is_some());
        assert_eq!(panel.width, Some(40));
        assert_eq!(panel.height, Some(5));
        assert_eq!(panel.padding, PaddingDimensions::Uniform(2));
    }

    // ── 10. Measure ───────────────────────────────────────────────────

    #[test]
    fn test_measure_default() {
        let console = make_console(80);
        let panel = Panel::new(Text::new("Hello", Style::null()));
        let opts = console.options();
        let m = panel.measure(&console, &opts);

        // "Hello" is 5 cells + padding(1+1) + border(2) = 9
        assert_eq!(m.minimum, 9);
        assert_eq!(m.maximum, 9);
    }

    #[test]
    fn test_measure_with_fixed_width() {
        let console = make_console(80);
        let panel = Panel::new(Text::new("Hello", Style::null())).width(30);
        let opts = console.options();
        let m = panel.measure(&console, &opts);

        assert_eq!(m.minimum, 30);
        assert_eq!(m.maximum, 30);
    }

    #[test]
    fn test_measure_with_padding() {
        let console = make_console(80);
        let panel =
            Panel::new(Text::new("Hi", Style::null())).padding(PaddingDimensions::Full(0, 3, 0, 3));
        let opts = console.options();
        let m = panel.measure(&console, &opts);

        // "Hi" is 2 cells + padding(3+3) + border(2) = 10
        assert_eq!(m.minimum, 10);
        assert_eq!(m.maximum, 10);
    }

    // ── 11. Wide content truncation ───────────────────────────────────

    #[test]
    fn test_wide_content_truncation() {
        let console = make_console(15);
        let panel = Panel::new(Text::new("This is a very long string", Style::null()));
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        // All lines should be exactly 15 cells wide
        for line in &lines {
            assert_eq!(
                cell_len(line),
                15,
                "Line width should be 15, got {}",
                cell_len(line)
            );
        }
    }

    // ── 12. Title truncation when too long ────────────────────────────

    #[test]
    fn test_title_truncation() {
        let console = make_console(15);
        let panel = Panel::new(Text::new("X", Style::null()))
            .title(Text::new("This Is A Very Long Title", Style::null()));
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        // Top border should still be exactly 15 cells
        assert_eq!(cell_len(lines[0]), 15);
        // Title should be truncated
        assert!(!lines[0].contains("This Is A Very Long Title"));
    }

    // ── Additional edge case tests ────────────────────────────────────

    #[test]
    fn test_multiline_content() {
        let console = make_console(20);
        let panel = Panel::new(Text::new("Line 1\nLine 2\nLine 3", Style::null()));
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        // top border + 3 content lines + bottom border = 5
        assert_eq!(lines.len(), 5);
        assert!(lines[1].contains("Line 1"));
        assert!(lines[2].contains("Line 2"));
        assert!(lines[3].contains("Line 3"));
    }

    #[test]
    fn test_empty_content() {
        let console = make_console(20);
        let panel = Panel::new(Text::new("", Style::null()));
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        // top border + at least 1 content line + bottom border
        assert!(lines.len() >= 3);
        // All lines should be 20 cells
        for line in &lines {
            assert_eq!(cell_len(line), 20);
        }
    }

    #[test]
    fn test_title_and_subtitle_together() {
        let console = make_console(30);
        let panel = Panel::new(Text::new("Body", Style::null()))
            .title(Text::new("Top", Style::null()))
            .subtitle(Text::new("Bottom", Style::null()));
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        assert!(lines[0].contains(" Top "));
        assert!(lines.last().unwrap().contains(" Bottom "));
    }

    #[test]
    fn test_fixed_height() {
        let console = make_console(20);
        let panel = Panel::new(Text::new("Short", Style::null())).height(5);
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        // top border + 5 content lines + bottom border = 7
        assert_eq!(lines.len(), 7);
    }

    #[test]
    fn test_panel_consistency_all_lines_same_width() {
        let console = make_console(40);
        let panel = Panel::new(Text::new("Hello, World!", Style::null()))
            .title(Text::new("Title", Style::null()))
            .subtitle(Text::new("Subtitle", Style::null()));
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        let expected_width = 40;
        for (i, line) in lines.iter().enumerate() {
            assert_eq!(
                cell_len(line),
                expected_width,
                "Line {} has width {}, expected {}",
                i,
                cell_len(line),
                expected_width
            );
        }
    }

    #[test]
    fn test_fit_panel_no_title() {
        let console = make_console(80);
        let panel = Panel::fit(Text::new("Test", Style::null()));
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        // "Test" is 4 cells + padding(1+1) + border(2) = 8
        for line in &lines {
            assert_eq!(cell_len(line), 8);
        }
    }

    #[test]
    fn test_border_style_applied() {
        let console = Console::builder()
            .width(20)
            .force_terminal(true)
            .no_color(false)
            .markup(false)
            .build();
        let border_style = Style::parse("bold").unwrap();
        let panel = Panel::new(Text::new("X", Style::null())).border_style(border_style.clone());
        let opts = console.options();
        let segments = panel.rich_console(&console, &opts);

        // The border segments should carry the border_style
        let border_segs: Vec<&Segment> = segments
            .iter()
            .filter(|s| {
                let t = s.text.trim();
                !t.is_empty()
                    && s.text != "\n"
                    && (t.contains('╭')
                        || t.contains('╮')
                        || t.contains('│')
                        || t.contains('╰')
                        || t.contains('╯'))
            })
            .collect();
        assert!(!border_segs.is_empty());
        for seg in border_segs {
            assert!(
                seg.style.is_some(),
                "Border segment '{}' should have a style",
                seg.text
            );
        }
    }

    // -- Highlight feature ---------------------------------------------------

    #[test]
    fn test_panel_highlight_flag() {
        let panel = Panel::new(Text::new("hello 123", Style::null()));
        assert!(!panel.highlight, "highlight should default to false");

        let panel2 = Panel {
            highlight: true,
            ..Panel::new(Text::new("hello 123", Style::null()))
        };
        assert!(panel2.highlight);
    }

    #[test]
    fn test_panel_highlight_builder() {
        let panel = Panel::new(Text::new("hello 123", Style::null())).highlight(true);
        assert!(panel.highlight);

        let panel2 = Panel::new(Text::new("hello 123", Style::null())).highlight(false);
        assert!(!panel2.highlight);
    }

    #[test]
    fn test_panel_highlight_renders() {
        // When highlight is true, the rendered output should contain styled
        // segments (the ReprHighlighter adds styles to numbers, strings, etc.)
        let console = make_console(40);
        let panel = Panel::new(Text::new("value=42 name='hello'", Style::null())).highlight(true);
        let opts = console.options();
        let segments = panel.rich_console(&console, &opts);
        // The content should still contain the text
        let text = segments_to_text(&segments);
        assert!(text.contains("42"));
        assert!(text.contains("hello"));
        // With highlight on, some segments should have non-null styles
        // (from ReprHighlighter matching numbers/strings)
        let content_segments: Vec<&Segment> = segments
            .iter()
            .filter(|s| {
                let t = s.text.trim();
                !t.is_empty()
                    && s.text != "\n"
                    && !t.contains('╭')
                    && !t.contains('╮')
                    && !t.contains('│')
                    && !t.contains('╰')
                    && !t.contains('╯')
                    && !t.contains('─')
            })
            .collect();
        let has_styled = content_segments.iter().any(|s| s.style.is_some());
        assert!(
            has_styled,
            "highlight=true should produce styled segments for repr patterns"
        );
    }

    #[test]
    fn test_display_trait() {
        let panel = Panel::new(Text::new("Hello, World!", Style::null()));
        let s = format!("{}", panel);
        assert!(!s.is_empty());
        assert!(s.contains("Hello, World!"));
    }

    #[test]
    fn test_display_with_width() {
        let panel = Panel::new(Text::new("content", Style::null()));
        let s = format!("{:60}", panel);
        assert!(s.contains("content"));
    }
}
