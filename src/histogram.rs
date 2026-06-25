//! Histogram -- bins raw samples into a vertical block-column distribution chart.
//!
//! A [`Histogram`] takes a slice of `f64` samples, distributes them into
//! equal-width buckets over a value range, and renders the per-bin counts as a
//! row of vertical columns built from the partial-block ladder
//! (`\u{2581}`..`\u{2588}`) -- one column per bin, the same ladder
//! [`Sparkline`] uses, but stacked across several rows for sub-cell precision.
//! Bins are scaled to the tallest bin so the busiest bucket reaches full height.
//!
//! It mirrors the structure of [`Sparkline`] and [`BarChart`] (builder style,
//! [`Renderable`] impl, `gilt_measure` override, inline tests) and is
//! dependency-free, WASM-safe, and `unsafe`-free (pure computation + Unicode
//! block characters).
//!
//! Rendering choice: bins are drawn as **vertical** block columns (one column
//! per bin, the partial-block ladder giving sub-cell height) rather than as
//! horizontal bars -- this reads more like the classic histogram shape.
//!
//! [`Sparkline`]: crate::sparkline::Sparkline
//! [`BarChart`]: crate::barchart::BarChart
//!
//! # Example
//!
//! ```
//! use gilt::histogram::Histogram;
//!
//! // Six samples bucketed into three equal-width bins over [1, 3].
//! let hist = Histogram::new(&[1.0, 2.0, 2.0, 3.0, 3.0, 3.0]).with_bins(3);
//! assert_eq!(hist.counts(), vec![1, 2, 3]);
//! ```

use std::fmt;

use crate::cells::cell_len;
use crate::console::{Console, ConsoleOptions, Renderable};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::Style;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Vertical block elements used for the columns, from one-eighth to full.
/// Index `i` is `(i + 1)` filled eighths; index 7 is the full block.
const BARS: [char; 8] = [
    '\u{2581}', // LOWER ONE EIGHTH BLOCK    ▁
    '\u{2582}', // LOWER ONE QUARTER BLOCK   ▂
    '\u{2583}', // LOWER THREE EIGHTHS BLOCK ▃
    '\u{2584}', // LOWER HALF BLOCK          ▄
    '\u{2585}', // LOWER FIVE EIGHTHS BLOCK  ▅
    '\u{2586}', // LOWER THREE QUARTERS      ▆
    '\u{2587}', // LOWER SEVEN EIGHTHS BLOCK ▇
    '\u{2588}', // FULL BLOCK                █
];

/// Box-drawing glyph used for the optional axis baseline.
const AXIS: char = '\u{2500}'; // ─

/// Default number of bins when not overridden.
const DEFAULT_BINS: usize = 10;

/// Default vertical resolution (rows) when not overridden.
const DEFAULT_HEIGHT: usize = 6;

// ---------------------------------------------------------------------------
// Histogram
// ---------------------------------------------------------------------------

/// A binned-distribution chart rendered as vertical Unicode block columns.
///
/// Construct from a slice of samples with [`Histogram::new`], then tune the bin
/// count, value range, height, style, and axis with the builder methods. Bins
/// are scaled to the tallest bin: the busiest bucket fills the full
/// [`with_height`](Histogram::with_height) rows, and shorter bins use the
/// partial-block ladder for sub-cell precision.
#[derive(Debug, Clone)]
pub struct Histogram {
    /// The raw samples to bucket.
    samples: Vec<f64>,
    /// Number of equal-width bins.
    bins: usize,
    /// Explicit value range `(min, max)`. When `None`, derived from the data.
    range: Option<(f64, f64)>,
    /// Vertical resolution in rows.
    height: usize,
    /// Style applied to the column glyphs.
    style: Style,
    /// Whether to draw a baseline and min/max range labels below the columns.
    show_axis: bool,
}

impl Histogram {
    /// Create a histogram from a slice of samples.
    ///
    /// Defaults: [`DEFAULT_BINS`] bins, a height of [`DEFAULT_HEIGHT`] rows,
    /// an auto-derived value range, no style, and no axis.
    pub fn new(samples: &[f64]) -> Self {
        Self {
            samples: samples.to_vec(),
            bins: DEFAULT_BINS,
            range: None,
            height: DEFAULT_HEIGHT,
            style: Style::null(),
            show_axis: false,
        }
    }

    /// Set the number of equal-width bins (builder pattern).
    ///
    /// A bin count of zero produces an empty render.
    #[must_use]
    pub fn with_bins(mut self, bins: usize) -> Self {
        self.bins = bins;
        self
    }

    /// Set an explicit value range `[min, max]` for bucketing (builder pattern).
    ///
    /// Samples outside the range are clamped into the first/last bin. When
    /// unset, the range is derived from the data's minimum and maximum. A
    /// degenerate range (`min >= max`) places every sample in the first bin.
    #[must_use]
    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.range = Some((min, max));
        self
    }

    /// Set the vertical resolution in rows (builder pattern).
    ///
    /// Each row resolves to eight sub-cell steps via the partial-block ladder,
    /// so the effective vertical precision is `height * 8` steps.
    #[must_use]
    pub fn with_height(mut self, rows: usize) -> Self {
        self.height = rows;
        self
    }

    /// Set the style applied to the column glyphs (builder pattern).
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Enable or disable the axis (builder pattern; default `false`).
    ///
    /// When enabled, a baseline rule and a line of left/right min/max range
    /// labels are drawn beneath the columns.
    #[must_use]
    pub fn with_show_axis(mut self, show: bool) -> Self {
        self.show_axis = show;
        self
    }

    // -- public queries -----------------------------------------------------

    /// Compute the per-bin sample counts.
    ///
    /// Returns an empty vector for empty input or a zero bin count. Otherwise
    /// returns exactly [`bins`](Histogram::with_bins) counts. Each sample is
    /// clamped into the value range and placed in its equal-width bucket; a
    /// value equal to the range maximum lands in the last bin.
    pub fn counts(&self) -> Vec<usize> {
        if self.samples.is_empty() || self.bins == 0 {
            return Vec::new();
        }

        let bins = self.bins;
        let (min, max) = self.effective_range();
        let mut counts = vec![0usize; bins];

        let span = max - min;
        if span <= 0.0 {
            // Degenerate range (all-equal samples or an inverted range): every
            // sample collapses onto a single point -> the first bin. No div-by-zero.
            counts[0] = self.samples.len();
            return counts;
        }

        let bin_width = span / bins as f64;
        for &v in &self.samples {
            let clamped = v.clamp(min, max);
            let idx = if clamped >= max {
                bins - 1
            } else {
                (((clamped - min) / bin_width).floor() as usize).min(bins - 1)
            };
            counts[idx] += 1;
        }
        counts
    }

    // -- internal helpers ---------------------------------------------------

    /// The value range used for bucketing: the explicit
    /// [`with_range`](Histogram::with_range) when set, otherwise the data
    /// extrema. Only meaningful when `samples` is non-empty.
    fn effective_range(&self) -> (f64, f64) {
        if let Some(range) = self.range {
            return range;
        }
        let min = self.samples.iter().copied().fold(f64::INFINITY, f64::min);
        let max = self
            .samples
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        (min, max)
    }

    /// Number of rendered columns (bins) -- the chart's natural cell width.
    fn column_count(&self) -> usize {
        if self.samples.is_empty() {
            0
        } else {
            self.bins
        }
    }

    /// Per-column fill height in eighths (`0..=height*8`), scaled to the
    /// tallest bin. Empty when there is nothing to render.
    fn column_eighths(&self) -> Vec<usize> {
        let counts = self.counts();
        let max_count = counts.iter().copied().max().unwrap_or(0);
        if max_count == 0 {
            // All-empty (or no) bins -> no fill anywhere.
            return vec![0; counts.len()];
        }
        let ceiling = self.height * 8;
        counts
            .iter()
            .map(|&c| {
                let frac = c as f64 / max_count as f64;
                ((frac * ceiling as f64).round() as usize).min(ceiling)
            })
            .collect()
    }

    /// Format a value compactly for an axis label: integers drop the decimal,
    /// others keep up to two decimal places without trailing zeros.
    fn fmt_axis(value: f64) -> String {
        let s = format!("{value:.2}");
        let trimmed = s.trim_end_matches('0').trim_end_matches('.');
        trimmed.to_string()
    }

    /// Push one column row (bottom-anchored index `row`) as coalesced segments,
    /// styling block runs and leaving blank runs unstyled.
    fn push_row(&self, eighths: &[usize], row: usize, segments: &mut Vec<Segment>) {
        // Build the row's glyphs.
        let mut glyphs: Vec<char> = Vec::with_capacity(eighths.len());
        for &e in eighths {
            let full = e / 8;
            let partial = e % 8;
            let ch = if row < full {
                BARS[7] // full block
            } else if row == full && partial > 0 {
                BARS[partial - 1]
            } else {
                ' '
            };
            glyphs.push(ch);
        }

        // Coalesce consecutive cells of the same kind (blank vs block) so blank
        // padding stays unstyled and only the bars carry the chart style.
        let mut run = String::new();
        let mut run_blank = true;
        let mut started = false;
        for &ch in &glyphs {
            let blank = ch == ' ';
            if started && blank == run_blank {
                run.push(ch);
            } else {
                if started {
                    segments.push(Self::run_segment(&run, run_blank, &self.style));
                }
                run.clear();
                run.push(ch);
                run_blank = blank;
                started = true;
            }
        }
        if started {
            segments.push(Self::run_segment(&run, run_blank, &self.style));
        }
        segments.push(Segment::line());
    }

    /// Wrap a run of cells into a segment: blanks unstyled, blocks styled.
    fn run_segment(run: &str, blank: bool, style: &Style) -> Segment {
        if blank {
            Segment::new(run, None, None)
        } else {
            Segment::new(run, Some(style.clone()), None)
        }
    }

    /// Render the columns (and optional axis) to styled segments.
    fn render_columns(&self) -> Vec<Segment> {
        let eighths = self.column_eighths();
        if eighths.is_empty() {
            return vec![Segment::line()];
        }

        let mut segments = Vec::with_capacity(self.height + 2);
        // Top row first: iterate bottom-anchored indices from high to low.
        for row in (0..self.height).rev() {
            self.push_row(&eighths, row, &mut segments);
        }

        if self.show_axis {
            let width = eighths.len();
            // Baseline rule spanning every column.
            let baseline: String = std::iter::repeat_n(AXIS, width).collect();
            segments.push(Segment::new(&baseline, Some(self.style.clone()), None));
            segments.push(Segment::line());

            // Min / max range labels (left / right aligned across the columns).
            let (min, max) = self.effective_range();
            let lo = Self::fmt_axis(min);
            let hi = Self::fmt_axis(max);
            let used = cell_len(&lo) + cell_len(&hi);
            let label = if used < width {
                let mut s = String::with_capacity(width);
                s.push_str(&lo);
                s.push_str(&" ".repeat(width - used));
                s.push_str(&hi);
                s
            } else {
                format!("{lo} {hi}")
            };
            segments.push(Segment::new(&label, None, None));
            segments.push(Segment::line());
        }

        segments
    }

    /// Natural measurement: a fixed `bins`-wide block (0 when empty).
    fn compute_measure(&self) -> Measurement {
        let w = self.column_count();
        Measurement::new(w, w)
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl fmt::Display for Histogram {
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
// Renderable
// ---------------------------------------------------------------------------

impl Renderable for Histogram {
    fn gilt_console(&self, _console: &Console, _options: &ConsoleOptions) -> Vec<Segment> {
        self.render_columns()
    }

    fn gilt_measure(&self, _console: &Console, _options: &ConsoleOptions) -> Measurement {
        self.compute_measure()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::{Console, ConsoleDimensions, ConsoleOptions};

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
            encoding: std::borrow::Cow::Borrowed("utf-8"),
            max_height: 25,
            justify: None,
            overflow: None,
            no_wrap: None,
            highlight: None,
            markup: None,
            height: None,
        }
    }

    /// Render to non-empty text rows (styles ignored), top row first.
    fn render_rows(hist: &Histogram) -> Vec<String> {
        let console = Console::builder().width(80).build();
        let opts = make_options(80);
        let segs = hist.gilt_console(&console, &opts);
        let joined: String = segs.iter().map(|s| s.text.as_str()).collect();
        joined
            .split('\n')
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect()
    }

    // -- binning ------------------------------------------------------------

    // 1. Known samples bucket into the expected counts.
    #[test]
    fn known_samples_bucket_as_expected() {
        let hist = Histogram::new(&[1.0, 2.0, 2.0, 3.0, 3.0, 3.0]).with_bins(3);
        assert_eq!(hist.counts(), vec![1, 2, 3]);
    }

    // 2. with_bins changes the bin count.
    #[test]
    fn with_bins_changes_bin_count() {
        let data = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        assert_eq!(Histogram::new(&data).with_bins(2).counts().len(), 2);
        assert_eq!(Histogram::new(&data).with_bins(5).counts().len(), 5);
        assert_eq!(Histogram::new(&data).with_bins(2).counts(), vec![5, 5]);
    }

    // 3. with_range clamps out-of-range samples into the first/last bin.
    #[test]
    fn with_range_clamps_out_of_range() {
        // Range [0, 10] with 5 bins (width 2). Samples below 0 clamp to bin 0,
        // samples above 10 clamp to the last bin.
        let hist = Histogram::new(&[-5.0, -1.0, 1.0, 100.0])
            .with_bins(5)
            .with_range(0.0, 10.0);
        let counts = hist.counts();
        assert_eq!(counts.len(), 5);
        assert_eq!(counts[0], 3); // -5, -1, 1 all land in [0, 2)
        assert_eq!(counts[4], 1); // 100 clamps into the last bin
    }

    // 4. A value exactly at max lands in the last bin.
    #[test]
    fn value_at_max_lands_in_last_bin() {
        let hist = Histogram::new(&[0.0, 1.0, 2.0, 3.0, 4.0])
            .with_bins(4)
            .with_range(0.0, 4.0);
        let counts = hist.counts();
        assert_eq!(counts.len(), 4);
        // 4.0 == max -> last bin; the last bin holds {3.0, 4.0}.
        assert_eq!(counts[3], 2);
        assert_eq!(counts.iter().sum::<usize>(), 5);
    }

    // 5. Auto-range with all-equal samples: no div-by-zero, all in one bin.
    #[test]
    fn all_equal_samples_no_div_by_zero() {
        let hist = Histogram::new(&[5.0, 5.0, 5.0, 5.0]).with_bins(4);
        let counts = hist.counts();
        assert_eq!(counts, vec![4, 0, 0, 0]);
        // And it renders without panicking.
        let _ = render_rows(&hist);
    }

    // 6. Empty input -> empty counts and a single-newline render (no panic).
    #[test]
    fn empty_input_no_panic() {
        let hist = Histogram::new(&[]);
        assert_eq!(hist.counts(), Vec::<usize>::new());
        let console = Console::builder().width(80).build();
        let opts = make_options(80);
        let segs = hist.gilt_console(&console, &opts);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].text.as_str(), "\n");
    }

    // 7. Zero bins -> empty render.
    #[test]
    fn zero_bins_empty_render() {
        let hist = Histogram::new(&[1.0, 2.0, 3.0]).with_bins(0);
        assert!(hist.counts().is_empty());
        let console = Console::builder().width(80).build();
        let opts = make_options(80);
        let segs = hist.gilt_console(&console, &opts);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].text.as_str(), "\n");
    }

    // -- rendering ----------------------------------------------------------

    // 8. The tallest bin reaches full height (all full blocks).
    #[test]
    fn tallest_bin_reaches_full_height() {
        // Bin 2 is the busiest, so its column must be solid full blocks.
        let hist = Histogram::new(&[1.0, 2.0, 2.0, 3.0, 3.0, 3.0])
            .with_bins(3)
            .with_height(4);
        let rows = render_rows(&hist);
        assert_eq!(rows.len(), 4, "one line per height row");
        for (i, row) in rows.iter().enumerate() {
            let col: Vec<char> = row.chars().collect();
            assert_eq!(col[2], BARS[7], "tallest column full at row {i}");
        }
    }

    // 9. Each rendered row is exactly `bins` cells wide.
    #[test]
    fn rows_are_bins_wide() {
        let hist = Histogram::new(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0])
            .with_bins(6)
            .with_height(3);
        for row in render_rows(&hist) {
            assert_eq!(row.chars().count(), 6);
        }
    }

    // 10. Height controls the number of rows.
    #[test]
    fn height_controls_row_count() {
        let hist = Histogram::new(&[1.0, 2.0, 3.0, 4.0])
            .with_bins(4)
            .with_height(9);
        assert_eq!(render_rows(&hist).len(), 9);
    }

    // 11. Empty bins render as blank (space) columns.
    #[test]
    fn empty_bins_are_blank() {
        // Two clusters with a gap: the middle bins stay empty.
        let hist = Histogram::new(&[0.0, 0.0, 10.0, 10.0])
            .with_bins(5)
            .with_range(0.0, 10.0)
            .with_height(2);
        let rows = render_rows(&hist);
        // Bottom row: first and last columns filled, middle ones blank.
        let bottom: Vec<char> = rows.last().unwrap().chars().collect();
        assert_eq!(bottom[0], BARS[7]);
        assert_eq!(bottom[4], BARS[7]);
        assert_eq!(bottom[2], ' ');
    }

    // -- axis ---------------------------------------------------------------

    // 12. Axis adds a baseline plus a min/max label line.
    #[test]
    fn axis_adds_baseline_and_labels() {
        let hist = Histogram::new(&[0.0, 5.0, 10.0])
            .with_bins(8)
            .with_height(3)
            .with_range(0.0, 10.0)
            .with_show_axis(true);
        let rows = render_rows(&hist);
        // 3 column rows + baseline + labels = 5 lines.
        assert_eq!(rows.len(), 5);
        assert!(rows[3].chars().all(|c| c == AXIS), "baseline is a rule");
        assert!(rows[4].contains('0'));
        assert!(rows[4].contains("10"));
    }

    // 13. fmt_axis is compact for integers and decimals.
    #[test]
    fn fmt_axis_is_compact() {
        assert_eq!(Histogram::fmt_axis(3.0), "3");
        assert_eq!(Histogram::fmt_axis(3.5), "3.5");
        assert_eq!(Histogram::fmt_axis(0.0), "0");
        assert_eq!(Histogram::fmt_axis(10.0), "10");
        assert_eq!(Histogram::fmt_axis(3.14159), "3.14");
    }

    // -- style / segments ---------------------------------------------------

    // 14. The chart style is applied to block runs, not to blank padding.
    #[test]
    fn style_applies_to_blocks_only() {
        let style = Style::parse("bold red");
        let hist = Histogram::new(&[0.0, 0.0, 1.0])
            .with_bins(2)
            .with_range(0.0, 1.0)
            .with_height(2)
            .with_style(style.clone());
        let console = Console::builder().width(80).build();
        let opts = make_options(80);
        let segs = hist.gilt_console(&console, &opts);
        // At least one block segment carries the style; blank segments are unstyled.
        let mut saw_styled_block = false;
        for seg in &segs {
            if seg.text.as_str() == "\n" {
                continue;
            }
            if seg.text.chars().any(|c| c != ' ') {
                assert_eq!(seg.style.as_ref(), Some(&style));
                saw_styled_block = true;
            } else {
                assert!(seg.style.is_none(), "blank padding stays unstyled");
            }
        }
        assert!(saw_styled_block);
    }

    // 15. Renderable ends in a newline.
    #[test]
    fn render_ends_in_newline() {
        let hist = Histogram::new(&[1.0, 2.0, 3.0]).with_bins(3);
        let console = Console::builder().width(80).build();
        let opts = make_options(80);
        let segs = hist.gilt_console(&console, &opts);
        assert_eq!(segs.last().unwrap().text.as_str(), "\n");
    }

    // -- measure ------------------------------------------------------------

    // 16. gilt_measure matches the rendered (bins-wide) chart.
    #[test]
    fn measure_matches_rendered_width() {
        let hist = Histogram::new(&[0.0, 1.0, 2.0, 3.0, 4.0]).with_bins(7);
        let console = Console::builder().width(80).build();
        let opts = make_options(80);
        let rows = render_rows(&hist);
        let rendered = rows.iter().map(|r| cell_len(r)).max().unwrap();
        let m = hist.gilt_measure(&console, &opts);
        assert_eq!(m, Measurement::new(7, 7));
        assert_eq!(m.maximum, rendered);
    }

    // 17. Empty input measures as zero width.
    #[test]
    fn measure_empty_is_zero() {
        let hist = Histogram::new(&[]);
        let console = Console::builder().width(80).build();
        let opts = make_options(80);
        assert_eq!(hist.gilt_measure(&console, &opts), Measurement::new(0, 0));
    }

    // -- builders / display -------------------------------------------------

    // 18. Builder chaining stores every field.
    #[test]
    fn builder_chaining() {
        let hist = Histogram::new(&[1.0, 2.0])
            .with_bins(12)
            .with_range(0.0, 5.0)
            .with_height(8)
            .with_style(Style::parse("green"))
            .with_show_axis(true);
        assert_eq!(hist.bins, 12);
        assert_eq!(hist.range, Some((0.0, 5.0)));
        assert_eq!(hist.height, 8);
        assert!(hist.show_axis);
    }

    // 19. Display renders multiple lines without a trailing newline.
    #[test]
    fn display_is_multiline_trimmed() {
        let hist = Histogram::new(&[0.0, 1.0, 1.0, 2.0, 2.0, 2.0])
            .with_bins(3)
            .with_height(3);
        let s = hist.to_string();
        assert!(s.contains(BARS[7]));
        assert!(!s.ends_with('\n'));
        assert_eq!(s.split('\n').count(), 3);
    }
}
