//! LineChart -- a Braille line plot with axes and multiple series.
//!
//! A `LineChart` plots one or more numeric y-series over an integer index
//! (`x = 0..n`) as connected lines drawn on a [`Canvas`] with the default
//! [`Blitter::Braille`](crate::canvas::Blitter::Braille) (2×4 sub-cell
//! resolution).  It is the "complex" sibling of [`Sparkline`] and
//! [`BarChart`]: it has a real plot area, a left y-axis gutter with
//! min/mid/max labels, a bottom axis line, and an optional colour legend.
//!
//! [`Sparkline`]: crate::sparkline::Sparkline
//! [`BarChart`]: crate::barchart::BarChart
//! [`Canvas`]: crate::canvas::Canvas
//!
//! # Multi-series colouring
//!
//! Every series is mapped onto its own Braille [`Canvas`] (so lines are drawn
//! with [`Canvas::line`](crate::canvas::Canvas::line)'s Bresenham routine — the
//! dot matrix is reused, not reinvented).  For the final overlay the per-cell
//! *glyph* is the union of every series' dots (read from a single composite
//! canvas), while the per-cell *colour* is taken from the top-most series that
//! lit any pixel in that cell (later series win on overlap, a simple z-order).
//! This keeps each series in its own colour wherever lines are separated and
//! degrades gracefully where they cross.
//!
//! # Example
//!
//! ```
//! use gilt::linechart::LineChart;
//! use gilt::style::Style;
//!
//! let chart = LineChart::new()
//!     .with_width(40)
//!     .with_height(8)
//!     .series("ramp", &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0], Style::parse("green"));
//!
//! let out = chart.to_string();
//! assert!(out.contains('\u{25CF}')); // ● legend marker
//! ```

use std::fmt;

use crate::canvas::Canvas;
use crate::cells::cell_len;
use crate::console::{Console, ConsoleOptions, Renderable};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::Style;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default plot-area width in terminal columns.
const DEFAULT_WIDTH: usize = 60;
/// Default plot-area height in terminal rows.
const DEFAULT_HEIGHT: usize = 12;

/// Empty Braille pattern (no dots) — the blank plot cell.
const BRAILLE_BLANK: char = '\u{2800}';
/// Vertical axis glyph for un-labelled gutter rows.
const AXIS_VERTICAL: char = '\u{2502}'; // │
/// Vertical axis glyph for labelled gutter rows (reads as a tick).
const AXIS_TICK: char = '\u{2524}'; // ┤
/// Bottom-left corner where the y-axis meets the x-axis.
const AXIS_CORNER: char = '\u{2514}'; // └
/// Horizontal axis glyph.
const AXIS_HORIZONTAL: char = '\u{2500}'; // ─
/// Legend bullet marker.
const LEGEND_MARKER: char = '\u{25CF}'; // ●

// ---------------------------------------------------------------------------
// Series
// ---------------------------------------------------------------------------

/// A single labelled y-series with its own draw [`Style`].
#[derive(Debug, Clone)]
struct Series {
    label: String,
    data: Vec<f64>,
    style: Style,
}

// ---------------------------------------------------------------------------
// LineChart
// ---------------------------------------------------------------------------

/// A multi-series Braille line chart with axes and a legend.
///
/// Build it by chaining [`series`](LineChart::series) calls, then render it
/// through the [`Renderable`] pipeline (or via [`Display`](fmt::Display)).
#[derive(Debug, Clone)]
pub struct LineChart {
    /// The y-series, in insertion (and z-) order.
    series: Vec<Series>,
    /// Plot-area width in terminal columns.
    width: usize,
    /// Plot-area height in terminal rows.
    height: usize,
    /// Explicit y-range `(min, max)`.  When `None`, auto-derived (with pad).
    y_range: Option<(f64, f64)>,
    /// Whether to draw the y-axis gutter (labels) and the bottom axis line.
    axes: bool,
    /// Whether to draw the per-series legend line below the chart.
    legend: bool,
    /// Style applied to the axis glyphs and y-axis labels.
    axis_style: Style,
}

impl Default for LineChart {
    fn default() -> Self {
        Self::new()
    }
}

impl LineChart {
    /// Create a new, empty line chart with default geometry (60×12).
    pub fn new() -> Self {
        Self {
            series: Vec::new(),
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            y_range: None,
            axes: true,
            legend: true,
            axis_style: Style::parse("dim"),
        }
    }

    /// Append a labelled series with its own draw style (chainable).
    #[must_use]
    pub fn series(mut self, label: impl Into<String>, data: &[f64], style: Style) -> Self {
        self.series.push(Series {
            label: label.into(),
            data: data.to_vec(),
            style,
        });
        self
    }

    /// Set the plot-area width in terminal columns (builder pattern).
    #[must_use]
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Set the plot-area height in terminal rows (builder pattern).
    #[must_use]
    pub fn with_height(mut self, height: usize) -> Self {
        self.height = height;
        self
    }

    /// Set an explicit y-range (builder pattern).
    ///
    /// Values outside the range are clamped onto the nearest edge.  A degenerate
    /// range (`min == max`, or inverted) is guarded so rendering never divides
    /// by zero.
    #[must_use]
    pub fn with_y_range(mut self, min: f64, max: f64) -> Self {
        self.y_range = Some((min, max));
        self
    }

    /// Enable or disable the axes — the y-axis gutter (min/mid/max labels) and
    /// the bottom axis line (builder pattern; default `true`).
    #[must_use]
    pub fn with_axes(mut self, axes: bool) -> Self {
        self.axes = axes;
        self
    }

    /// Enable or disable the per-series legend line (builder pattern; default
    /// `true`).  The legend lists each series' label in its own colour and is
    /// only drawn for series that have a non-empty label.
    #[must_use]
    pub fn with_legend(mut self, legend: bool) -> Self {
        self.legend = legend;
        self
    }

    /// Set the style applied to the axis glyphs and labels (builder pattern;
    /// default `dim`).
    #[must_use]
    pub fn with_axis_style(mut self, style: Style) -> Self {
        self.axis_style = style;
        self
    }

    /// Number of series in the chart.
    pub fn len(&self) -> usize {
        self.series.len()
    }

    /// Whether the chart has no series.
    pub fn is_empty(&self) -> bool {
        self.series.is_empty()
    }

    // -- range -------------------------------------------------------------

    /// The effective `(min, max)` y-range used for mapping.
    ///
    /// Uses [`with_y_range`](LineChart::with_y_range) when set (guarded), else
    /// auto-derives from every finite data point with a small 5% pad.  Falls
    /// back to `(0.0, 1.0)` when there is no finite data, and expands an
    /// all-equal range so the divisor is never zero.
    fn effective_range(&self) -> (f64, f64) {
        if let Some((mn, mx)) = self.y_range {
            return Self::guard_range(mn, mx);
        }

        let mut mn = f64::INFINITY;
        let mut mx = f64::NEG_INFINITY;
        for s in &self.series {
            for &v in &s.data {
                if v.is_finite() {
                    if v < mn {
                        mn = v;
                    }
                    if v > mx {
                        mx = v;
                    }
                }
            }
        }

        if !mn.is_finite() || !mx.is_finite() {
            return (0.0, 1.0);
        }
        if (mx - mn).abs() < f64::EPSILON {
            // All values equal: expand symmetrically (never zero-width).
            let pad = mn.abs().max(1.0);
            return (mn - pad, mx + pad);
        }
        let pad = (mx - mn) * 0.05;
        (mn - pad, mx + pad)
    }

    /// Normalise a possibly-degenerate explicit range into a strictly-positive
    /// span (sorting and padding as needed).
    fn guard_range(mn: f64, mx: f64) -> (f64, f64) {
        let (mut lo, mut hi) = if mn <= mx { (mn, mx) } else { (mx, mn) };
        if !lo.is_finite() || !hi.is_finite() {
            return (0.0, 1.0);
        }
        if (hi - lo).abs() < f64::EPSILON {
            let pad = lo.abs().max(1.0);
            lo -= pad;
            hi += pad;
        }
        (lo, hi)
    }

    // -- drawing -----------------------------------------------------------

    /// Draw one series onto `canvas` as a connected Braille line.
    ///
    /// Each point `(i, y)` maps to pixel `(x, y')` where `x` is scaled across
    /// the pixel width and `y'` is inverted (larger values sit higher).  Values
    /// outside the range are clamped onto the top/bottom edge; non-finite values
    /// break the line into separate segments.
    fn draw_series(&self, canvas: &mut Canvas, series: &Series, ymin: f64, ymax: f64) {
        let pw = canvas.pixel_width() as f64;
        let ph = canvas.pixel_height() as f64;
        let n = series.data.len();
        let span = ymax - ymin;

        let mut prev: Option<(i32, i32)> = None;
        for (i, &y) in series.data.iter().enumerate() {
            if !y.is_finite() {
                prev = None;
                continue;
            }
            let px = if n <= 1 {
                0
            } else {
                (i as f64 / (n - 1) as f64 * (pw - 1.0)).round() as i32
            };
            let t = if span > 0.0 {
                ((y - ymin) / span).clamp(0.0, 1.0)
            } else {
                0.5
            };
            let py = ((1.0 - t) * (ph - 1.0)).round() as i32;

            match prev {
                Some((ppx, ppy)) => canvas.line(ppx, ppy, px, py),
                None => {
                    if px >= 0 && py >= 0 {
                        canvas.set(px as usize, py as usize);
                    }
                }
            }
            prev = Some((px, py));
        }
    }

    /// Build one Braille canvas per series (used for per-cell colour ownership).
    fn build_canvases(&self, ymin: f64, ymax: f64) -> Vec<Canvas> {
        self.series
            .iter()
            .map(|s| {
                let mut c = Canvas::new(self.width, self.height);
                self.draw_series(&mut c, s, ymin, ymax);
                c
            })
            .collect()
    }

    /// The index of the top-most series lighting any pixel in cell `(row, col)`,
    /// or `None` when the cell is empty.  Braille cells are 2×4 pixels, so cell
    /// `(row, col)` spans pixels `x ∈ [col*2, col*2+1]`, `y ∈ [row*4, row*4+3]`.
    fn cell_owner(canvases: &[Canvas], row: usize, col: usize) -> Option<usize> {
        let mut owner = None;
        for (s, cv) in canvases.iter().enumerate() {
            let mut touched = false;
            'scan: for dy in 0..4 {
                for dx in 0..2 {
                    if cv.get(col * 2 + dx, row * 4 + dy) {
                        touched = true;
                        break 'scan;
                    }
                }
            }
            if touched {
                owner = Some(s);
            }
        }
        owner
    }

    // -- axis labels -------------------------------------------------------

    /// Per-row y-axis labels: max at the top row, min at the bottom row, and the
    /// midpoint at the middle row (when there is room).  Length == `height`.
    fn y_label_rows(&self, ymin: f64, ymax: f64) -> Vec<Option<String>> {
        let h = self.height;
        let mut rows = vec![None; h];
        if h == 0 {
            return rows;
        }
        rows[0] = Some(Self::fmt_label(ymax));
        if h == 1 {
            return rows;
        }
        rows[h - 1] = Some(Self::fmt_label(ymin));
        if h >= 3 {
            let mid = h / 2;
            if mid != 0 && mid != h - 1 {
                rows[mid] = Some(Self::fmt_label((ymin + ymax) / 2.0));
            }
        }
        rows
    }

    /// Maximum cell-width across the non-empty y-axis labels.
    fn label_width(labels: &[Option<String>]) -> usize {
        labels
            .iter()
            .flatten()
            .map(|s| cell_len(s))
            .max()
            .unwrap_or(0)
    }

    /// Format a y-value compactly: integers print without a decimal, otherwise
    /// two decimal places.
    fn fmt_label(v: f64) -> String {
        if !v.is_finite() {
            return "·".to_string();
        }
        if (v - v.round()).abs() < 1e-9 && v.abs() < 1e15 {
            format!("{}", v.round() as i64)
        } else {
            format!("{v:.2}")
        }
    }

    // -- legend ------------------------------------------------------------

    /// The legend as `(text, style)` pieces (markers + labels + separators).
    /// Empty when the legend is disabled or no series has a label.
    fn legend_pieces(&self) -> Vec<(String, Style)> {
        if !self.legend {
            return Vec::new();
        }
        let mut pieces = Vec::new();
        for s in &self.series {
            if s.label.is_empty() {
                continue;
            }
            if !pieces.is_empty() {
                pieces.push(("  ".to_string(), Style::null()));
            }
            pieces.push((format!("{LEGEND_MARKER} {}", s.label), s.style.clone()));
        }
        pieces
    }

    // -- measure -----------------------------------------------------------

    /// The fixed rendered width (max line width across plot, axis, and legend).
    fn content_width(&self) -> usize {
        let (ymin, ymax) = self.effective_range();
        let labels = self.y_label_rows(ymin, ymax);
        let gutter = if self.axes {
            Self::label_width(&labels) + 1
        } else {
            0
        };
        let plot_w = gutter + self.width;
        let legend_w: usize = self.legend_pieces().iter().map(|(t, _)| cell_len(t)).sum();
        plot_w.max(legend_w)
    }
}

// ---------------------------------------------------------------------------
// Renderable
// ---------------------------------------------------------------------------

impl Renderable for LineChart {
    fn gilt_console(&self, _console: &Console, _options: &ConsoleOptions) -> Vec<Segment> {
        let (ymin, ymax) = self.effective_range();
        let labels = self.y_label_rows(ymin, ymax);
        let label_width = Self::label_width(&labels);

        // Per-series canvases (for colour ownership) + one composite (for glyphs).
        let canvases = self.build_canvases(ymin, ymax);
        let mut composite = Canvas::new(self.width, self.height);
        for s in &self.series {
            self.draw_series(&mut composite, s, ymin, ymax);
        }
        let frame = composite.frame();
        let glyph_rows: Vec<Vec<char>> = if self.height == 0 {
            Vec::new()
        } else {
            frame.split('\n').map(|l| l.chars().collect()).collect()
        };

        let mut segments = Vec::new();

        // -- plot rows (with optional gutter) ------------------------------
        // `labels` has exactly `height` entries, so iterating it covers every row.
        for (row, label_cell) in labels.iter().enumerate() {
            if self.axes {
                let (label, tick) = match label_cell {
                    Some(l) => (l.as_str(), AXIS_TICK),
                    None => ("", AXIS_VERTICAL),
                };
                let field = format!("{label:>label_width$}{tick}");
                segments.push(Segment::new(&field, Some(self.axis_style.clone()), None));
            }

            // Plot cells, coalesced into runs of equal style.
            let mut run = String::new();
            let mut run_style: Option<Style> = None;
            for col in 0..self.width {
                let ch = glyph_rows
                    .get(row)
                    .and_then(|r| r.get(col))
                    .copied()
                    .unwrap_or(BRAILLE_BLANK);
                let style = match Self::cell_owner(&canvases, row, col) {
                    Some(s) => self.series[s].style.clone(),
                    None => Style::null(),
                };
                match &run_style {
                    Some(rs) if *rs == style => run.push(ch),
                    Some(rs) => {
                        segments.push(Segment::new(&run, Some(rs.clone()), None));
                        run.clear();
                        run.push(ch);
                        run_style = Some(style);
                    }
                    None => {
                        run.push(ch);
                        run_style = Some(style);
                    }
                }
            }
            if let Some(rs) = run_style {
                segments.push(Segment::new(&run, Some(rs), None));
            }
            segments.push(Segment::line());
        }

        // -- bottom axis line ----------------------------------------------
        if self.axes {
            let mut line = String::with_capacity(label_width + 1 + self.width);
            line.push_str(&" ".repeat(label_width));
            line.push(AXIS_CORNER);
            line.push_str(&AXIS_HORIZONTAL.to_string().repeat(self.width));
            segments.push(Segment::new(&line, Some(self.axis_style.clone()), None));
            segments.push(Segment::line());
        }

        // -- legend ---------------------------------------------------------
        let pieces = self.legend_pieces();
        if !pieces.is_empty() {
            for (text, style) in &pieces {
                segments.push(Segment::new(text, Some(style.clone()), None));
            }
            segments.push(Segment::line());
        }

        // Always terminate with a newline so output is well-formed.
        if !matches!(segments.last(), Some(s) if s.text.as_str() == "\n") {
            segments.push(Segment::line());
        }
        segments
    }

    fn gilt_measure(&self, _console: &Console, _options: &ConsoleOptions) -> Measurement {
        let w = self.content_width();
        Measurement::new(w, w)
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl fmt::Display for LineChart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    use crate::console::Console;

    fn console() -> Console {
        Console::builder()
            .width(120)
            .force_terminal(true)
            .no_color(false)
            .build()
    }

    /// The first series' rendered canvas (for pixel-level assertions).
    fn series_canvas(chart: &LineChart, idx: usize) -> Canvas {
        let (lo, hi) = chart.effective_range();
        let mut canvases = chart.build_canvases(lo, hi);
        canvases.remove(idx)
    }

    /// Render and split into lines by newline control segments, returning the
    /// visible cell width of each line.
    fn line_widths(segments: &[Segment]) -> Vec<usize> {
        let mut widths = vec![0usize];
        for seg in segments {
            if seg.text.as_str() == "\n" {
                widths.push(0);
            } else {
                *widths.last_mut().unwrap() += cell_len(seg.text.as_str());
            }
        }
        if widths.last() == Some(&0) {
            widths.pop();
        }
        widths
    }

    /// True if any pixel is set within the inclusive-exclusive box.
    fn any_set(c: &Canvas, x0: usize, x1: usize, y0: usize, y1: usize) -> bool {
        for y in y0..y1 {
            for x in x0..x1 {
                if c.get(x, y) {
                    return true;
                }
            }
        }
        false
    }

    // -- gilt_measure delegates / matches -----------------------------------

    #[test]
    fn gilt_measure_matches_rendered_width() {
        let chart = LineChart::new()
            .with_width(30)
            .with_height(8)
            .series("a", &[0.0, 1.0, 2.0, 3.0], Style::parse("red"))
            .series("beta", &[3.0, 2.0, 1.0, 0.0], Style::parse("blue"));
        let c = console();
        let opts = c.options();
        let segs = chart.gilt_console(&c, &opts);
        let rendered = line_widths(&segs).into_iter().max().unwrap();
        let m = chart.gilt_measure(&c, &opts);
        assert_eq!(m.maximum, rendered);
        assert_eq!(m.minimum, m.maximum);
    }

    #[test]
    fn measure_without_axes_is_just_width() {
        // No axes, no legend => the plot cells are the only content.
        let chart = LineChart::new()
            .with_width(25)
            .with_height(6)
            .with_axes(false)
            .with_legend(false)
            .series("x", &[0.0, 1.0, 2.0], Style::parse("green"));
        let c = console();
        let opts = c.options();
        let m = chart.gilt_measure(&c, &opts);
        assert_eq!(m.maximum, 25);
        let rendered = line_widths(&chart.gilt_console(&c, &opts))
            .into_iter()
            .max()
            .unwrap();
        assert_eq!(m.maximum, rendered);
    }

    // -- flat series is roughly horizontal ----------------------------------

    #[test]
    fn flat_series_is_roughly_horizontal() {
        let chart = LineChart::new().with_width(20).with_height(8).series(
            "flat",
            &[5.0; 10],
            Style::parse("white"),
        );
        let c = series_canvas(&chart, 0);
        let pw = c.pixel_width();
        let ph = c.pixel_height();
        // All-equal data maps to the vertical middle (t = 0.5).
        let mid = ph / 2;
        // Something is set in the middle band...
        assert!(
            any_set(&c, 0, pw, mid.saturating_sub(2), mid + 2),
            "flat line should sit near the vertical middle"
        );
        // ...and nothing near the very top or very bottom.
        assert!(!any_set(&c, 0, pw, 0, 3), "no pixels near the top");
        assert!(!any_set(&c, 0, pw, ph - 3, ph), "no pixels near the bottom");
        // The line spans the full width (left and right thirds both lit).
        assert!(any_set(&c, 0, pw / 3, 0, ph), "left of line is drawn");
        assert!(any_set(&c, 2 * pw / 3, pw, 0, ph), "right of line is drawn");
    }

    // -- rising ramp trends upward ------------------------------------------

    #[test]
    fn rising_ramp_trends_upward() {
        let data: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let chart = LineChart::new().with_width(20).with_height(8).series(
            "ramp",
            &data,
            Style::parse("green"),
        );
        let c = series_canvas(&chart, 0);
        let pw = c.pixel_width();
        let ph = c.pixel_height();
        // Near bottom-left: low x, high y (y grows downward).
        assert!(
            any_set(&c, 0, pw / 4, 3 * ph / 4, ph),
            "ramp starts low (bottom-left)"
        );
        // Near top-right: high x, low y.
        assert!(
            any_set(&c, 3 * pw / 4, pw, 0, ph / 4),
            "ramp ends high (top-right)"
        );
    }

    // -- with_y_range clamps and scales -------------------------------------

    #[test]
    fn with_y_range_clamps_out_of_range_points() {
        // Range 0..10, but the points are 20 (above) and -5 (below).
        let chart = LineChart::new()
            .with_width(20)
            .with_height(8)
            .with_y_range(0.0, 10.0)
            .series("clamped", &[20.0, -5.0], Style::parse("yellow"));
        let c = series_canvas(&chart, 0);
        let pw = c.pixel_width();
        let ph = c.pixel_height();
        // First point (i=0, value 20) clamps to the top edge at x=0.
        assert!(c.get(0, 0), "above-range value clamps to the top edge");
        // Last point (i=1, value -5) clamps to the bottom edge at x=pw-1.
        assert!(
            c.get(pw - 1, ph - 1),
            "below-range value clamps to the bottom edge"
        );
    }

    #[test]
    fn with_y_range_scales_within_range() {
        // A value at the midpoint of an explicit range lands in the middle row.
        let chart = LineChart::new()
            .with_width(10)
            .with_height(8)
            .with_y_range(0.0, 10.0)
            .series("mid", &[5.0, 5.0], Style::parse("cyan"));
        let c = series_canvas(&chart, 0);
        let ph = c.pixel_height();
        let mid = ph / 2;
        assert!(
            any_set(&c, 0, c.pixel_width(), mid.saturating_sub(2), mid + 2),
            "midpoint value renders in the middle band"
        );
    }

    // -- auto-range handles all-equal data ----------------------------------

    #[test]
    fn auto_range_all_equal_no_div_by_zero() {
        let chart = LineChart::new().series("c", &[7.0, 7.0, 7.0], Style::null());
        let (lo, hi) = chart.effective_range();
        assert!(lo.is_finite() && hi.is_finite());
        assert!(hi > lo, "all-equal range must be expanded, not zero-width");
        // Rendering must not panic.
        let c = console();
        let _ = chart.gilt_console(&c, &c.options());
    }

    #[test]
    fn auto_range_no_data_falls_back() {
        let chart = LineChart::new();
        assert_eq!(chart.effective_range(), (0.0, 1.0));
    }

    // -- empty chart / empty series no panic --------------------------------

    #[test]
    fn empty_chart_no_panic_ends_in_newline() {
        let chart = LineChart::new();
        let c = console();
        let segs = chart.gilt_console(&c, &c.options());
        assert!(!segs.is_empty());
        assert_eq!(segs.last().unwrap().text.as_str(), "\n");
        assert!(chart.is_empty());
        assert_eq!(chart.len(), 0);
    }

    #[test]
    fn empty_series_data_no_panic() {
        let chart = LineChart::new()
            .with_width(10)
            .with_height(4)
            .series("empty", &[], Style::parse("red"))
            .series("one", &[1.0], Style::parse("blue"));
        let c = console();
        let segs = chart.gilt_console(&c, &c.options());
        assert_eq!(segs.last().unwrap().text.as_str(), "\n");
    }

    // -- multiple series each appear (distinct styles) ----------------------

    #[test]
    fn multiple_series_each_appear_with_distinct_styles() {
        let red = Style::parse("red");
        let blue = Style::parse("blue");
        // Two clearly separated flat lines so neither fully overlaps the other.
        let chart = LineChart::new()
            .with_width(20)
            .with_height(8)
            .with_y_range(0.0, 10.0)
            .with_axes(false)
            .with_legend(false)
            .series("low", &[1.0, 1.0, 1.0], red.clone())
            .series("high", &[9.0, 9.0, 9.0], blue.clone());
        let c = console();
        let segs = chart.gilt_console(&c, &c.options());
        let styles: Vec<Style> = segs
            .iter()
            .filter(|s| s.text.as_str() != "\n")
            .filter_map(|s| s.style.clone())
            .collect();
        assert!(styles.contains(&red), "red series must appear in segments");
        assert!(
            styles.contains(&blue),
            "blue series must appear in segments"
        );
    }

    // -- gilt_console ends in a newline -------------------------------------

    #[test]
    fn gilt_console_ends_in_newline() {
        let chart = LineChart::new().series("s", &[1.0, 2.0, 3.0], Style::parse("magenta"));
        let c = console();
        let segs = chart.gilt_console(&c, &c.options());
        assert_eq!(segs.last().unwrap().text.as_str(), "\n");
    }

    // -- legend -------------------------------------------------------------

    #[test]
    fn legend_lists_labelled_series_in_colour() {
        let green = Style::parse("green");
        let chart = LineChart::new().with_width(20).with_height(6).series(
            "alpha",
            &[0.0, 1.0],
            green.clone(),
        );
        let c = console();
        let segs = chart.gilt_console(&c, &c.options());
        // A segment carries the marker + label in the series style.
        let legend = segs
            .iter()
            .find(|s| s.text.as_str().contains("alpha"))
            .expect("legend entry for alpha");
        assert!(legend.text.as_str().contains(LEGEND_MARKER));
        assert_eq!(legend.style.as_ref(), Some(&green));
    }

    #[test]
    fn legend_disabled_omits_labels() {
        let chart =
            LineChart::new()
                .with_legend(false)
                .series("hidden", &[0.0, 1.0], Style::parse("red"));
        let c = console();
        let segs = chart.gilt_console(&c, &c.options());
        assert!(segs.iter().all(|s| !s.text.as_str().contains("hidden")));
    }

    // -- axes / gutter ------------------------------------------------------

    #[test]
    fn axes_draw_gutter_labels_and_bottom_line() {
        let chart = LineChart::new()
            .with_width(15)
            .with_height(6)
            .with_y_range(0.0, 100.0)
            .with_legend(false)
            .series("s", &[0.0, 50.0, 100.0], Style::parse("white"));
        let c = console();
        let segs = chart.gilt_console(&c, &c.options());
        let text: String = segs.iter().map(|s| s.text.as_str()).collect();
        // y-axis labels (max / mid / min).
        assert!(text.contains("100"), "top label present");
        assert!(text.contains('0'), "bottom label present");
        // bottom axis corner + horizontal run.
        assert!(text.contains(AXIS_CORNER), "bottom corner present");
        assert!(text.contains(AXIS_HORIZONTAL), "horizontal axis present");
        // every plot row carries the vertical axis or a tick.
        assert!(
            text.contains(AXIS_VERTICAL) || text.contains(AXIS_TICK),
            "vertical axis present"
        );
    }

    #[test]
    fn axes_disabled_has_no_axis_glyphs() {
        let chart = LineChart::new()
            .with_width(15)
            .with_height(6)
            .with_axes(false)
            .with_legend(false)
            .series("s", &[0.0, 1.0, 2.0], Style::parse("white"));
        let c = console();
        let segs = chart.gilt_console(&c, &c.options());
        let text: String = segs.iter().map(|s| s.text.as_str()).collect();
        assert!(!text.contains(AXIS_CORNER));
        assert!(!text.contains(AXIS_VERTICAL));
        assert!(!text.contains(AXIS_TICK));
    }

    // -- builders -----------------------------------------------------------

    #[test]
    fn builders_store_fields() {
        let chart = LineChart::new()
            .with_width(42)
            .with_height(9)
            .with_y_range(-1.0, 1.0)
            .with_axes(false)
            .with_legend(false)
            .with_axis_style(Style::parse("blue"))
            .series("a", &[1.0], Style::null());
        assert_eq!(chart.width, 42);
        assert_eq!(chart.height, 9);
        assert_eq!(chart.y_range, Some((-1.0, 1.0)));
        assert!(!chart.axes);
        assert!(!chart.legend);
        assert_eq!(chart.len(), 1);
    }

    #[test]
    fn display_renders_braille_and_legend() {
        let chart = LineChart::new().with_width(20).with_height(6).series(
            "wave",
            &[0.0, 1.0, 0.0, 1.0],
            Style::parse("green"),
        );
        let out = chart.to_string();
        assert!(out.contains(LEGEND_MARKER), "legend marker present");
        assert!(!out.ends_with('\n'), "Display trims the trailing newline");
    }

    // -- inverted explicit range is guarded ---------------------------------

    #[test]
    fn inverted_y_range_is_sorted() {
        let chart = LineChart::new().with_y_range(10.0, 0.0);
        let (lo, hi) = chart.effective_range();
        assert!(lo < hi, "inverted range must be normalised");
        assert_eq!((lo, hi), (0.0, 10.0));
    }
}
