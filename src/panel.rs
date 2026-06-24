//! Panel widget -- a bordered box around content with optional title/subtitle.
//!

use crate::align_widget::HorizontalAlign;
use crate::box_chars::{BoxChars, ROUNDED};
use crate::console::{Console, ConsoleOptions, Renderable, RenderableArc};
use crate::highlighter::Highlighter;
use crate::measure::Measurement;
use crate::padding::PaddingDimensions;
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;

// ---------------------------------------------------------------------------
// Panel
// ---------------------------------------------------------------------------

/// A bordered box around content, with optional title and subtitle in the border.
///
/// The content can be any type that implements the [`Renderable`] trait, such as
/// [`Text`], [`Table`](crate::table::Table), [`Tree`](crate::tree::Tree),
/// [`Columns`](crate::columns::Columns), or even another [`Panel`].
///
/// # Examples
///
/// ```
/// use gilt::prelude::*;
///
/// // Panel with Text content
/// let panel = Panel::new(Text::new("Hello, world!", Style::null()));
///
/// // Panel with styled border and title
/// let panel = Panel::new(Text::new("Important message", Style::null()))
///     .with_title("Notice")
///     .with_border_style(Style::parse("red"));
///
/// // Panel wrapping a Table
/// let mut table = Table::new(&["Name", "Value"]);
/// table.add_row(&["Key", "Value"]);
/// let panel = Panel::new(table).with_title("Data");
/// ```
#[derive(Clone)]
pub struct Panel {
    /// The inner content — any renderable widget (Text, Table, Tree, Panel, …).
    pub content: RenderableArc,
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
    /// Override safe-box substitution. `None` inherits from the console;
    /// `Some(true)` forces Unicode→ASCII substitution on legacy terminals.
    pub safe_box: Option<bool>,
}

// Manual Debug — RenderableArc (Arc<dyn Renderable + Send + Sync>) doesn't
// implement Debug, so we print a placeholder for the content field.
impl std::fmt::Debug for Panel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Panel")
            .field("content", &"<renderable>")
            .field("box_chars", &self.box_chars)
            .field("title", &self.title)
            .field("title_align", &self.title_align)
            .field("subtitle", &self.subtitle)
            .field("subtitle_align", &self.subtitle_align)
            .field("expand", &self.expand)
            .field("style", &self.style)
            .field("border_style", &self.border_style)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("padding", &self.padding)
            .field("highlight", &self.highlight)
            .field("safe_box", &self.safe_box)
            .finish()
    }
}

impl Panel {
    /// Create a new expanding `Panel` with ROUNDED box and default padding.
    ///
    /// Accepts any type that implements [`Renderable`] — [`Text`], [`Table`],
    /// [`Tree`], another [`Panel`], etc.  The value is stored as a
    /// [`RenderableArc`] (reference-counted, cheaply cloned).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::prelude::*;
    ///
    /// // Panel with Text
    /// let panel = Panel::new(Text::new("Hello", Style::null()));
    ///
    /// // Panel with Table
    /// let mut table = Table::new(&["Name", "Value"]);
    /// table.add_row(&["Key", "Value"]);
    /// let panel = Panel::new(table);
    /// ```
    pub fn new(content: impl Renderable + Send + Sync + 'static) -> Self {
        Panel {
            content: std::sync::Arc::new(content),
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
            safe_box: None,
        }
    }

    /// Create a non-expanding (fit-to-content) `Panel`.
    pub fn fit(content: impl Renderable + Send + Sync + 'static) -> Self {
        let mut panel = Panel::new(content);
        panel.expand = false;
        panel
    }

    /// Wrap any [`Renderable`] in a `Panel`.
    ///
    /// This is a thin wrapper around [`Panel::new`] — the renderable is stored
    /// directly as a [`RenderableArc`] with no pre-rendering.  Mirror of
    /// [`Live::from_renderable`](crate::live::Live::from_renderable).
    pub fn from_renderable<R: Renderable + Send + Sync + 'static>(renderable: R) -> Self {
        Self::new(renderable)
    }

    // -- Builder methods ----------------------------------------------------

    /// Set the box-drawing character set.
    #[must_use]
    pub fn with_box_chars(mut self, box_chars: &'static BoxChars) -> Self {
        self.box_chars = box_chars;
        self
    }

    /// Set the title text.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<Text>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the title alignment.
    #[must_use]
    pub fn with_title_align(mut self, align: HorizontalAlign) -> Self {
        self.title_align = align;
        self
    }

    /// Set the subtitle text.
    #[must_use]
    pub fn with_subtitle(mut self, subtitle: impl Into<Text>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Set the subtitle alignment.
    #[must_use]
    pub fn with_subtitle_align(mut self, align: HorizontalAlign) -> Self {
        self.subtitle_align = align;
        self
    }

    /// Set whether the panel expands to fill available width.
    #[must_use]
    pub fn with_expand(mut self, expand: bool) -> Self {
        self.expand = expand;
        self
    }

    /// Set the content style.
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the border style.
    #[must_use]
    pub fn with_border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }

    /// Set a fixed width.
    #[must_use]
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Set a fixed height for the content area.
    #[must_use]
    pub fn with_height(mut self, height: usize) -> Self {
        self.height = Some(height);
        self
    }

    /// Set the inner padding.
    #[must_use]
    pub fn with_padding(mut self, padding: PaddingDimensions) -> Self {
        self.padding = padding;
        self
    }

    /// Enable or disable `ReprHighlighter` on the content.
    #[must_use]
    pub fn with_highlight(mut self, highlight: bool) -> Self {
        self.highlight = highlight;
        self
    }

    /// Override safe-box substitution.
    ///
    /// When `true`, non-ASCII box characters are substituted with their
    /// ASCII equivalents when the console is in ascii-only mode (rich parity).
    /// `None` (default) inherits the console setting.
    #[must_use]
    pub fn with_safe_box(mut self, safe_box: bool) -> Self {
        self.safe_box = Some(safe_box);
        self
    }

    /// Measure the minimum and maximum width requirements.
    ///
    /// Uses the content's [`Measurement::maximum`] (longest line) rather than
    /// `cell_len()` which sums all characters across all lines. Also accounts
    /// for the title width so the panel is always wide enough to display its
    /// title (rich parity).
    pub fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        let (_, right, _, left) = self.padding.unpack();
        let padding = left + right;
        let w = if let Some(fixed) = self.width {
            fixed
        } else {
            // Use the content's true maximum (longest line) via the Renderable
            // trait so any widget type (Text, Table, Tree, nested Panel…) is
            // measured correctly.
            let content_max = self.content.gilt_measure(console, options).maximum;
            let mut w = content_max + padding + 2;

            // Panel must be wide enough to display its title.
            if let Some(ref title) = self.title {
                let mut title_text = title.clone();
                let plain = title_text.plain().replace('\n', " ");
                title_text.set_plain(&plain);
                title_text.expand_tabs(None);
                title_text.pad(1, ' ');
                // title needs: border(1) + fill(1) + title + fill(1) + border(1) = title + 4
                let title_min = title_text.cell_len() + 4;
                w = w.max(title_min);
            }

            w
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
    console: &Console,
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
        .render_themed(console)
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
    fn gilt_measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        self.measure(console, options)
    }

    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        // Apply box substitution (ascii_only / safe_box), matching rich behaviour.
        let safe = self.safe_box.unwrap_or(true);
        let ascii_only = options.ascii_only();
        let bx = if ascii_only || safe {
            self.box_chars.substitute(ascii_only)
        } else {
            self.box_chars
        };
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
            // Fit mode: size to the longest rendered line via the Renderable trait
            // so any content type (Text, Table, Tree, nested Panel…) is measured.
            let content_width = self.content.gilt_measure(console, options).maximum;
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

        // Inner width = the space available to content (child_width minus h-padding).
        let inner_width = child_width.saturating_sub(horizontal_padding).max(1);

        // Build child ConsoleOptions at inner_width so the child widget wraps
        // and measures correctly at panel interior size.
        let child_opts = options.update_width(inner_width);

        // Render the child widget.
        //
        // When `highlight` is set (a Text-oriented feature), first render the
        // content to plain text (collecting the text of each segment), then
        // apply the ReprHighlighter to a Text built from that plain string, and
        // finally re-render the highlighted Text at inner_width.
        //
        // For all other content (and when highlight=false), render directly via
        // `render_lines` so complex widgets (Table, Tree, nested Panel…) render
        // at full fidelity.
        let raw_lines: Vec<Vec<Segment>> = if self.highlight {
            // Collect plain text from the content's segments at inner_width,
            // joining lines with "\n" so multi-line content is preserved as
            // distinct lines rather than fused into one run-on string.
            let plain_segments =
                console.render_lines(self.content.as_ref(), Some(&child_opts), None, false, false);
            let flat: String = plain_segments
                .iter()
                .map(|line| line.iter().map(|s| s.text.as_str()).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n");
            // Build a Text from the joined string and apply the highlighter.
            let mut content_text = Text::new(&flat, Style::null());
            crate::highlighter::ReprHighlighter.highlight(&mut content_text);
            // Re-render the highlighted text at inner_width (reuse child_opts).
            console.render_lines(&content_text, Some(&child_opts), None, false, false)
        } else {
            console.render_lines(self.content.as_ref(), Some(&child_opts), None, false, false)
        };

        // Pad / trim each rendered line to exactly `inner_width` cells and
        // apply the content style.
        let mut lines: Vec<Vec<Segment>> = raw_lines
            .into_iter()
            .map(|line_segs| {
                let styled = if !self.style.is_null() {
                    Segment::apply_style(&line_segs, Some(self.style.clone()), None)
                } else {
                    line_segs
                };
                Segment::adjust_line_length(&styled, inner_width, &self.style, true)
            })
            .collect();

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
                    console,
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

        // ── Shared border strings (hoisted out of all loops) ─────────
        let left_pad_str = " ".repeat(pad_left);
        let right_pad_str = " ".repeat(pad_right);
        let mid_l_str = bx.mid_left.to_string();
        let mid_r_str = bx.mid_right.to_string();

        // ── Top padding rows ──────────────────────────────────────────
        for _ in 0..pad_top {
            segments.push(Segment::styled(&mid_l_str, self.border_style.clone()));
            let blank = " ".repeat(child_width);
            segments.push(Segment::styled(&blank, self.style.clone()));
            segments.push(Segment::styled(&mid_r_str, self.border_style.clone()));
            segments.push(Segment::line());
        }

        // ── Content rows ──────────────────────────────────────────────
        for line in &lines {
            // Left border
            segments.push(Segment::styled(&mid_l_str, self.border_style.clone()));

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
            segments.push(Segment::styled(&mid_r_str, self.border_style.clone()));
            segments.push(Segment::line());
        }

        // ── Bottom padding rows ───────────────────────────────────────
        for _ in 0..pad_bottom {
            segments.push(Segment::styled(&mid_l_str, self.border_style.clone()));
            let blank = " ".repeat(child_width);
            segments.push(Segment::styled(&blank, self.style.clone()));
            segments.push(Segment::styled(&mid_r_str, self.border_style.clone()));
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
                    console,
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
        let segments = panel.gilt_console(console, &opts);
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
            .with_title(Text::new("Title", Style::null()));
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
        let panel =
            Panel::new(Text::new("X", Style::null())).with_title(Text::new("T", Style::null()));
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
            .with_title(Text::new("Left", Style::null()))
            .with_title_align(HorizontalAlign::Left);
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
            .with_title(Text::new("Right", Style::null()))
            .with_title_align(HorizontalAlign::Right);
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
            .with_subtitle(Text::new("Sub", Style::null()));
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
            .with_subtitle(Text::new("SubLeft", Style::null()))
            .with_subtitle_align(HorizontalAlign::Left);
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
            .with_title(Text::new("A Longer Title", Style::null()));
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
        let panel = Panel::new(Text::new("X", Style::null())).with_box_chars(&DOUBLE);
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
        let panel = Panel::new(Text::new("X", Style::null())).with_box_chars(&HEAVY);
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
        let panel = Panel::new(Text::new("X", Style::null())).with_box_chars(&ASCII);
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
        let panel = Panel::new(Text::new("X", Style::null())).with_box_chars(&SQUARE);
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        assert!(lines[0].starts_with('┌'));
        assert!(lines[0].ends_with('┐'));
    }

    // ── 7. Panel with padding ─────────────────────────────────────────

    #[test]
    fn test_custom_padding() {
        let console = make_console(30);
        let panel = Panel::new(Text::new("X", Style::null()))
            .with_padding(PaddingDimensions::Full(1, 2, 1, 2));
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
        let panel = Panel::new(Text::new("Hello", Style::null()))
            .with_padding(PaddingDimensions::Uniform(0));
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
        let panel = Panel::new(Text::new("X", Style::null())).with_width(25);
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
        let panel = Panel::new(Text::new("X", Style::null())).with_width(50);
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
            .with_box_chars(&DOUBLE)
            .with_title(Text::new("T", Style::null()))
            .with_title_align(HorizontalAlign::Left)
            .with_subtitle(Text::new("S", Style::null()))
            .with_subtitle_align(HorizontalAlign::Right)
            .with_expand(false)
            .with_style(Style::parse("bold"))
            .with_border_style(Style::parse("red"))
            .with_width(40)
            .with_height(5)
            .with_padding(PaddingDimensions::Uniform(2));

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
        let panel = Panel::new(Text::new("Hello", Style::null())).with_width(30);
        let opts = console.options();
        let m = panel.measure(&console, &opts);

        assert_eq!(m.minimum, 30);
        assert_eq!(m.maximum, 30);
    }

    #[test]
    fn test_measure_with_padding() {
        let console = make_console(80);
        let panel = Panel::new(Text::new("Hi", Style::null()))
            .with_padding(PaddingDimensions::Full(0, 3, 0, 3));
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
            .with_title(Text::new("This Is A Very Long Title", Style::null()));
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
            .with_title(Text::new("Top", Style::null()))
            .with_subtitle(Text::new("Bottom", Style::null()));
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        assert!(lines[0].contains(" Top "));
        assert!(lines.last().unwrap().contains(" Bottom "));
    }

    #[test]
    fn test_fixed_height() {
        let console = make_console(20);
        let panel = Panel::new(Text::new("Short", Style::null())).with_height(5);
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);

        // top border + 5 content lines + bottom border = 7
        assert_eq!(lines.len(), 7);
    }

    #[test]
    fn test_panel_consistency_all_lines_same_width() {
        let console = make_console(40);
        let panel = Panel::new(Text::new("Hello, World!", Style::null()))
            .with_title(Text::new("Title", Style::null()))
            .with_subtitle(Text::new("Subtitle", Style::null()));
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
        let border_style = Style::parse("bold");
        let panel =
            Panel::new(Text::new("X", Style::null())).with_border_style(border_style.clone());
        let opts = console.options();
        let segments = panel.gilt_console(&console, &opts);

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
                seg.style().is_some(),
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
        let panel = Panel::new(Text::new("hello 123", Style::null())).with_highlight(true);
        assert!(panel.highlight);

        let panel2 = Panel::new(Text::new("hello 123", Style::null())).with_highlight(false);
        assert!(!panel2.highlight);
    }

    #[test]
    fn test_panel_highlight_renders() {
        // When highlight is true, the rendered output should contain styled
        // segments (the ReprHighlighter adds styles to numbers, strings, etc.)
        let console = make_console(40);
        let panel =
            Panel::new(Text::new("value=42 name='hello'", Style::null())).with_highlight(true);
        let opts = console.options();
        let segments = panel.gilt_console(&console, &opts);
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

    // -- CJK / emoji content tests ------------------------------------------

    #[test]
    fn test_panel_cjk_content() {
        let console = make_console(40);
        let panel = Panel::new(Text::new("こんにちは世界", Style::null()));
        let output = render_panel(&console, &panel);
        assert!(output.contains("こんにちは世界"));
        let lines = content_lines(&output);
        assert!(lines.len() >= 3); // top border, content, bottom border
    }

    #[test]
    fn test_panel_emoji_title() {
        let console = make_console(40);
        let panel = Panel::new(Text::new("Body text", Style::null()))
            .with_title(Text::new("🎉 Title", Style::null()));
        let output = render_panel(&console, &panel);
        assert!(output.contains("🎉"));
        assert!(output.contains("Title"));
    }

    // -- Extreme width boundary tests ---------------------------------------

    #[test]
    fn test_panel_width_one() {
        let console = make_console(1);
        let panel = Panel::new(Text::new("Hello", Style::null()));
        // Should not panic at width=1
        let _output = render_panel(&console, &panel);
    }

    #[test]
    fn test_panel_width_zero() {
        let console = make_console(0);
        let panel = Panel::new(Text::new("Hello", Style::null()));
        // Should not panic at width=0 (may produce empty output)
        let _output = render_panel(&console, &panel);
    }

    // ── Audit #17: fit mode must size to the longest line, not total chars ──

    #[test]
    fn panel_fit_uses_longest_line_not_total_chars() {
        // Content: "abc\nde" — longest line = 3 cells, total chars = 5 (+ newline = 6).
        // Fit panel interior = longest_line (3) + left_pad (1) + right_pad (1) = 5.
        // Total panel width  = interior (5) + left_border (1) + right_border (1) = 7.
        let console = make_console(80);
        let panel = Panel::fit(Text::new("abc\nde", Style::null()));
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);
        let expected_width = 3 + 2 + 2; // longest_line + padding(1+1) + borders(1+1)
        for line in &lines {
            assert_eq!(
                cell_len(line),
                expected_width,
                "fit panel width must track longest line (3)+padding+borders={}; got {} for '{}'",
                expected_width,
                cell_len(line),
                line
            );
        }
    }

    // -- gilt_measure override ------------------------------------------

    #[test]
    fn panel_gilt_measure_matches_standalone() {
        let console = make_console(80);
        let panel = Panel::new(Text::new("Hello World", Style::null()));
        let opts = console.options();
        assert_eq!(
            panel.gilt_measure(&console, &opts),
            panel.measure(&console, &opts),
            "Panel::gilt_measure must delegate to Panel::measure"
        );
    }

    #[test]
    fn panel_gilt_measure_with_title_matches_standalone() {
        let console = make_console(80);
        let mut panel = Panel::new(Text::new("content", Style::null()));
        panel.title = Some(Text::new("My Title", Style::null()));
        let opts = console.options();
        assert_eq!(
            panel.gilt_measure(&console, &opts),
            panel.measure(&console, &opts),
            "Panel::gilt_measure with title must delegate to Panel::measure"
        );
    }

    // ── Task 4.2: RenderableArc content tests ────────────────────────────

    /// Panel::new still compiles and renders correctly for Text content (no regression).
    #[test]
    fn panel_text_content_compiles_and_renders() {
        let console = make_console(30);
        let panel = Panel::new(Text::new("Hello, world!", Style::null()));
        let output = render_panel(&console, &panel);
        let lines = content_lines(&output);
        // Standard 3-line panel: top border, content, bottom border.
        assert_eq!(lines.len(), 3);
        assert!(
            lines[1].contains("Hello, world!"),
            "Text content must appear in output"
        );
        // All lines must be 30 cells wide (expand mode).
        for line in &lines {
            assert_eq!(cell_len(line), 30, "Line '{}' should be 30 cells", line);
        }
    }

    /// Panel wrapping a Table renders the table's headers and rows inside the border.
    #[test]
    fn panel_table_content_renders_headers_and_rows() {
        use crate::table::Table;
        let console = make_console(60);
        let mut table = Table::new(&["Name", "Score"]);
        table.add_row(&["Alice", "42"]);
        let panel = Panel::new(table).with_title(Text::new("Results", Style::null()));
        let output = render_panel(&console, &panel);
        // The border/title must be present.
        let lines = content_lines(&output);
        assert!(
            lines[0].contains("Results"),
            "Title must appear in top border"
        );
        assert!(lines[0].starts_with('╭'), "Top border must use rounded box");
        assert!(
            lines.last().unwrap().starts_with('╰'),
            "Bottom border must use rounded box"
        );
        // Table headers and row content must appear somewhere in the output.
        assert!(output.contains("Name"), "Table header 'Name' must appear");
        assert!(output.contains("Score"), "Table header 'Score' must appear");
        assert!(output.contains("Alice"), "Row value 'Alice' must appear");
        assert!(output.contains("42"), "Row value '42' must appear");
        // All lines must have equal width (panel geometry must be consistent).
        let w = cell_len(lines[0]);
        for (i, line) in lines.iter().enumerate() {
            assert_eq!(
                cell_len(line),
                w,
                "Line {} has width {}, expected {}",
                i,
                cell_len(line),
                w
            );
        }
    }

    /// Panel wrapping another Panel renders the inner panel's border inside the outer border.
    #[test]
    fn panel_nested_panel_content_renders() {
        let console = make_console(40);
        let inner = Panel::new(Text::new("inner content", Style::null()))
            .with_title(Text::new("Inner", Style::null()));
        let outer = Panel::new(inner).with_title(Text::new("Outer", Style::null()));
        let output = render_panel(&console, &outer);
        // Both titles must appear.
        assert!(output.contains("Outer"), "Outer panel title must appear");
        assert!(output.contains("Inner"), "Inner panel title must appear");
        // The inner panel's border characters must be present in the content area.
        assert!(
            output.contains("inner content"),
            "Inner content must appear"
        );
        // Outer panel geometry: all non-empty lines same width.
        let lines = content_lines(&output);
        let w = cell_len(lines[0]);
        for (i, line) in lines.iter().enumerate() {
            assert_eq!(
                cell_len(line),
                w,
                "Outer line {} has width {}, expected {}",
                i,
                cell_len(line),
                w
            );
        }
    }

    /// from_renderable is a thin wrapper — no pre-render, same result as new().
    #[test]
    fn panel_from_renderable_same_as_new() {
        let console = make_console(30);
        let panel1 = Panel::new(Text::new("test content", Style::null()));
        let panel2 = Panel::from_renderable(Text::new("test content", Style::null()));
        let out1 = render_panel(&console, &panel1);
        let out2 = render_panel(&console, &panel2);
        assert_eq!(
            out1, out2,
            "from_renderable must produce identical output to new()"
        );
    }

    /// Panel Debug impl must not panic and must print "<renderable>" for content.
    #[test]
    fn panel_debug_impl() {
        let panel = Panel::new(Text::new("debug me", Style::null()));
        let s = format!("{:?}", panel);
        assert!(
            s.contains("<renderable>"),
            "Debug output must contain '<renderable>'"
        );
        assert!(
            s.contains("Panel"),
            "Debug output must contain struct name 'Panel'"
        );
    }

    /// highlight=true on multi-line Text must preserve line breaks — both
    /// "line1" and "line2" must appear on SEPARATE rendered lines, not fused.
    #[test]
    fn panel_highlight_multiline_preserves_line_breaks() {
        let console = make_console(40);
        let panel = Panel::new(Text::new("line1\nline2", Style::null())).with_highlight(true);
        let output = render_panel(&console, &panel);
        // Both strings must appear in the output.
        assert!(
            output.contains("line1"),
            "highlight multiline: 'line1' must appear"
        );
        assert!(
            output.contains("line2"),
            "highlight multiline: 'line2' must appear"
        );
        // They must be on different rendered lines (not fused like "line1line2").
        let lines = content_lines(&output);
        let has_line1 = lines
            .iter()
            .any(|l| l.contains("line1") && !l.contains("line2"));
        let has_line2 = lines
            .iter()
            .any(|l| l.contains("line2") && !l.contains("line1"));
        assert!(
            has_line1,
            "highlight multiline: 'line1' must appear on its own line (not fused with line2)"
        );
        assert!(
            has_line2,
            "highlight multiline: 'line2' must appear on its own line (not fused with line1)"
        );
    }
}
