//! BarChart -- a horizontal bar chart rendered with Unicode block characters.
//!
//! Each row is laid out as `label  ███████▌░░  value`: a right-padded label, a
//! proportional bar built from full blocks (`█`) plus a trailing partial block
//! for sub-cell precision (eighths, `▏▎▍▌▋▊▉`), an unfilled track (`░`), and an
//! optional formatted value.  It mirrors the structure of [`Sparkline`] and
//! [`Bar`] (builder style, [`Renderable`] impl, `gilt_measure` override) and is
//! dependency-free and WASM-safe (pure computation + Unicode block chars).
//!
//! [`Sparkline`]: crate::sparkline::Sparkline
//! [`Bar`]: crate::bar::Bar
//!
//! # Example
//!
//! ```
//! use gilt::barchart::BarChart;
//!
//! let chart = BarChart::from_pairs([("alpha", 3.0_f64), ("beta", 5.0)]);
//! let out = chart.to_string();
//! assert!(out.contains("alpha"));
//! assert!(out.contains('\u{2588}')); // a full block appears in the longest bar
//! ```

use std::fmt;

use crate::cells::cell_len;
use crate::console::{Console, ConsoleOptions, Renderable};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::Style;

// ---------------------------------------------------------------------------
// Block-element constants
// ---------------------------------------------------------------------------

/// Left-anchored partial block elements, indexed by the number of filled
/// eighths (0..8).  Index 0 is the empty placeholder (never emitted); 8 is a
/// full block and is handled separately via [`FULL_BLOCK`].
const PARTIAL_BLOCKS: [&str; 8] = [
    "",         // 0 - empty placeholder (unused)
    "\u{258F}", // 1 - LEFT ONE EIGHTH BLOCK     ▏
    "\u{258E}", // 2 - LEFT ONE QUARTER BLOCK    ▎
    "\u{258D}", // 3 - LEFT THREE EIGHTHS BLOCK  ▍
    "\u{258C}", // 4 - LEFT HALF BLOCK           ▌
    "\u{258B}", // 5 - LEFT FIVE EIGHTHS BLOCK   ▋
    "\u{258A}", // 6 - LEFT THREE QUARTERS BLOCK ▊
    "\u{2589}", // 7 - LEFT SEVEN EIGHTHS BLOCK  ▉
];

/// Full block used for the solid filled body of each bar.
const FULL_BLOCK: &str = "\u{2588}"; // █

/// Light-shade glyph used for the unfilled track of each bar.
///
/// Unlike [`Bar`](crate::bar::Bar) (which leaves the remainder blank), a bar
/// *chart* reads more clearly when the full extent of every row is visible, so
/// the remainder is drawn with `░` -- matching this widget's canonical
/// `███████▌░░` look.
const TRACK: &str = "\u{2591}"; // ░

// ---------------------------------------------------------------------------
// BarChart
// ---------------------------------------------------------------------------

/// A horizontal bar chart.
///
/// Build it from `(label, value)` pairs and render it through the
/// [`Renderable`] pipeline.  Bars are scaled to the largest value (or to an
/// explicit [`with_max`](BarChart::with_max)); each bar fills proportionally
/// using full blocks plus a fractional partial block for sub-cell precision.
#[derive(Debug, Clone)]
pub struct BarChart {
    /// The `(label, value)` rows, in insertion order.
    bars: Vec<(String, f64)>,
    /// Optional total row width in cells.  When `None`, uses
    /// `ConsoleOptions::max_width`.
    width: Option<usize>,
    /// Explicit scale maximum.  When `None`, derived from the data.
    max: Option<f64>,
    /// Style applied to each bar's glyphs (filled body + track).
    bar_style: Style,
    /// Style applied to the labels.
    label_style: Style,
    /// Style applied to the formatted values.
    value_style: Style,
    /// Whether to append the formatted value to each row (default `true`).
    show_values: bool,
}

impl Default for BarChart {
    fn default() -> Self {
        Self::new()
    }
}

impl BarChart {
    /// Create a new, empty bar chart.
    pub fn new() -> Self {
        Self {
            bars: Vec::new(),
            width: None,
            max: None,
            bar_style: Style::null(),
            label_style: Style::null(),
            value_style: Style::null(),
            show_values: true,
        }
    }

    /// Build a chart from an iterator of `(label, value)` pairs.
    pub fn from_pairs<S: Into<String>>(pairs: impl IntoIterator<Item = (S, f64)>) -> Self {
        let mut chart = Self::new();
        for (label, value) in pairs {
            chart.bars.push((label.into(), value));
        }
        chart
    }

    /// Append a bar, returning `&mut Self` for fluent calls on an owned value.
    pub fn push(&mut self, label: impl Into<String>, value: f64) -> &mut Self {
        self.bars.push((label.into(), value));
        self
    }

    /// Append a bar (chainable, consuming builder form).
    #[must_use]
    pub fn with_bar(mut self, label: impl Into<String>, value: f64) -> Self {
        self.bars.push((label.into(), value));
        self
    }

    /// Set the total row width in cells (builder pattern).
    ///
    /// When unset, the chart fills `ConsoleOptions::max_width`.
    #[must_use]
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Set an explicit scale maximum (builder pattern).
    ///
    /// Values are scaled against this instead of the data maximum.  A zero or
    /// negative maximum is guarded: every bar renders empty.
    #[must_use]
    pub fn with_max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self
    }

    /// Set the style applied to the bar glyphs (builder pattern).
    #[must_use]
    pub fn with_bar_style(mut self, style: Style) -> Self {
        self.bar_style = style;
        self
    }

    /// Set the style applied to the labels (builder pattern).
    #[must_use]
    pub fn with_label_style(mut self, style: Style) -> Self {
        self.label_style = style;
        self
    }

    /// Set the style applied to the formatted values (builder pattern).
    #[must_use]
    pub fn with_value_style(mut self, style: Style) -> Self {
        self.value_style = style;
        self
    }

    /// Set whether formatted values are shown (builder pattern; default `true`).
    #[must_use]
    pub fn with_show_values(mut self, show: bool) -> Self {
        self.show_values = show;
        self
    }

    /// Number of bars in the chart.
    pub fn len(&self) -> usize {
        self.bars.len()
    }

    /// Whether the chart has no bars.
    pub fn is_empty(&self) -> bool {
        self.bars.is_empty()
    }

    // -- internal helpers ---------------------------------------------------

    /// The scale maximum used to size bars: the explicit
    /// [`with_max`](BarChart::with_max) when set, otherwise the data maximum.
    fn effective_max(&self) -> f64 {
        self.max.unwrap_or_else(|| {
            self.bars
                .iter()
                .map(|(_, v)| *v)
                .fold(f64::NEG_INFINITY, f64::max)
        })
    }

    /// Format a value for display.  `f64`'s `Display` already drops a trailing
    /// `.0`, so `3.0 -> "3"` while `3.5 -> "3.5"` -- simple and readable.
    fn format_value(value: f64) -> String {
        format!("{value}")
    }

    /// Compute the shared row geometry once, for both rendering and measuring.
    fn layout(&self, options: &ConsoleOptions) -> Layout {
        let label_width = self
            .bars
            .iter()
            .map(|(label, _)| cell_len(label))
            .max()
            .unwrap_or(0);

        let value_strings: Vec<String> = self
            .bars
            .iter()
            .map(|(_, value)| Self::format_value(*value))
            .collect();
        let value_width = if self.show_values {
            value_strings.iter().map(|s| cell_len(s)).max().unwrap_or(0)
        } else {
            0
        };

        // Each populated region is followed (label) or preceded (value) by a
        // single separator space, matching the `gilt_measure` formula:
        // `label + space + bar + space + value`.
        let label_region = if label_width > 0 { label_width + 1 } else { 0 };
        let value_region = if self.show_values && value_width > 0 {
            value_width + 1
        } else {
            0
        };

        let total = self
            .width
            .unwrap_or(options.max_width)
            .min(options.max_width);
        let bar_region = total.saturating_sub(label_region + value_region);
        let total_width = label_region + bar_region + value_region;

        Layout {
            label_width,
            value_strings,
            value_width,
            label_region,
            value_region,
            bar_region,
            total_width,
        }
    }

    /// Append `region` cells of bar glyphs for `value` against `max` to `out`:
    /// full blocks, a fractional partial block, then the unfilled track.
    fn bar_glyphs(value: f64, max: f64, region: usize, out: &mut String) {
        if region == 0 {
            return;
        }
        // Non-positive values or a guarded (zero/negative) max render empty.
        let fraction = if max > 0.0 && value > 0.0 {
            (value / max).min(1.0)
        } else {
            0.0
        };

        let total_eighths = ((fraction * region as f64 * 8.0).round() as usize).min(region * 8);
        let full = total_eighths / 8;
        let partial = total_eighths % 8;

        out.push_str(&FULL_BLOCK.repeat(full));
        let mut filled = full;
        if partial > 0 {
            out.push_str(PARTIAL_BLOCKS[partial]);
            filled += 1;
        }
        out.push_str(&TRACK.repeat(region.saturating_sub(filled)));
    }

    /// Render every row to styled segments (label / bar / value + newline).
    fn render_rows(&self, options: &ConsoleOptions) -> Vec<Segment> {
        if self.bars.is_empty() {
            return vec![Segment::line()];
        }

        let layout = self.layout(options);
        let max = self.effective_max();
        let mut segments = Vec::with_capacity(self.bars.len() * 4);

        for (i, (label, value)) in self.bars.iter().enumerate() {
            // -- label (right-padded to the longest label) + separator -------
            if layout.label_region > 0 {
                let pad = layout.label_width.saturating_sub(cell_len(label));
                let mut text = String::with_capacity(label.len() + pad + 1);
                text.push_str(label);
                text.push_str(&" ".repeat(pad));
                text.push(' ');
                segments.push(Segment::new(&text, Some(self.label_style.clone()), None));
            }

            // -- bar ---------------------------------------------------------
            let mut bar = String::with_capacity(layout.bar_region * 3);
            Self::bar_glyphs(*value, max, layout.bar_region, &mut bar);
            if !bar.is_empty() {
                segments.push(Segment::new(&bar, Some(self.bar_style.clone()), None));
            }

            // -- separator + right-aligned value -----------------------------
            if layout.value_region > 0 {
                let value_str = &layout.value_strings[i];
                let pad = layout.value_width.saturating_sub(cell_len(value_str));
                let mut text = String::with_capacity(pad + value_str.len() + 1);
                text.push(' ');
                text.push_str(&" ".repeat(pad));
                text.push_str(value_str);
                segments.push(Segment::new(&text, Some(self.value_style.clone()), None));
            }

            segments.push(Segment::line());
        }

        segments
    }

    /// Natural measurement: the rendered row width (clamped to `max_width`).
    fn compute_measure(&self, options: &ConsoleOptions) -> Measurement {
        if self.bars.is_empty() {
            return Measurement::new(0, 0);
        }
        let layout = self.layout(options);
        let maximum = layout.total_width.min(options.max_width);
        // At minimum, keep the label/value columns plus one bar cell.
        let minimum =
            (layout.label_region + layout.value_region + layout.bar_region.min(1)).min(maximum);
        Measurement::new(minimum, maximum)
    }
}

/// Shared per-render row geometry (computed once, used by render and measure).
struct Layout {
    /// Maximum label cell width across all rows.
    label_width: usize,
    /// Pre-formatted value strings (one per row).
    value_strings: Vec<String>,
    /// Maximum value cell width (0 when values are hidden).
    value_width: usize,
    /// Cells reserved for the label + its separator (0 when no labels).
    label_region: usize,
    /// Cells reserved for the value + its separator (0 when hidden).
    value_region: usize,
    /// Cells available for the bar itself.
    bar_region: usize,
    /// Actual rendered line width in cells.
    total_width: usize,
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl fmt::Display for BarChart {
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

impl Renderable for BarChart {
    fn gilt_console(&self, _console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        self.render_rows(options)
    }

    fn gilt_measure(&self, _console: &Console, options: &ConsoleOptions) -> Measurement {
        self.compute_measure(options)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cells::cell_len;
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

    /// Render a chart and return its non-empty text lines (styles ignored).
    fn render_lines(chart: &BarChart, max_width: usize) -> Vec<String> {
        let console = Console::builder().width(max_width).build();
        let opts = make_options(max_width);
        let segs = chart.gilt_console(&console, &opts);
        let joined: String = segs.iter().map(|s| s.text.as_str()).collect();
        joined
            .split('\n')
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect()
    }

    // -- empty chart --------------------------------------------------------

    #[test]
    fn empty_chart_renders_single_newline() {
        let chart = BarChart::new();
        let console = Console::builder().width(80).build();
        let opts = make_options(80);
        let segs = chart.gilt_console(&console, &opts);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].text.as_str(), "\n");
        assert!(chart.is_empty());
        assert_eq!(chart.len(), 0);
    }

    // -- single bar ---------------------------------------------------------

    #[test]
    fn single_full_bar_is_all_full_blocks() {
        let chart = BarChart::from_pairs([("", 1.0)])
            .with_max(1.0)
            .with_width(3)
            .with_show_values(false);
        let lines = render_lines(&chart, 80);
        assert_eq!(lines, vec!["\u{2588}\u{2588}\u{2588}".to_string()]);
    }

    // -- partial block at a known fraction ----------------------------------

    #[test]
    fn partial_block_half_cell() {
        // value/max = 0.5 over a 1-cell bar -> 4 eighths -> LEFT HALF BLOCK.
        let chart = BarChart::from_pairs([("", 1.0)])
            .with_max(2.0)
            .with_width(1)
            .with_show_values(false);
        let lines = render_lines(&chart, 80);
        assert_eq!(lines, vec!["\u{258C}".to_string()]); // ▌
    }

    // -- relative scaling ---------------------------------------------------

    #[test]
    fn two_bars_scale_relative_to_larger() {
        let chart = BarChart::from_pairs([("a", 1.0), ("b", 2.0)]).with_show_values(false);
        let lines = render_lines(&chart, 40);
        // The larger value (the scale max) fills completely -> no track glyph.
        assert!(
            !lines[1].contains('\u{2591}'),
            "full bar should have no track"
        );
        // The smaller value is partial -> a track glyph remains.
        assert!(
            lines[0].contains('\u{2591}'),
            "half bar should show a track"
        );
    }

    #[test]
    fn with_max_rescales_bars() {
        let chart = BarChart::from_pairs([("a", 1.0), ("b", 2.0)])
            .with_max(4.0)
            .with_show_values(false);
        let lines = render_lines(&chart, 40);
        // Now even the largest value (2 of 4) is partial -> track remains.
        assert!(lines[1].contains('\u{2591}'));
    }

    // -- label right-alignment ----------------------------------------------

    #[test]
    fn labels_right_padded_to_longest() {
        let chart = BarChart::from_pairs([("a", 1.0), ("bbb", 1.0)]).with_show_values(false);
        let lines = render_lines(&chart, 40);
        // label width = 3, then one separator space => 4-cell label region.
        assert_eq!(lines[0].chars().take(4).collect::<String>(), "a   ");
        assert_eq!(lines[1].chars().take(4).collect::<String>(), "bbb ");
    }

    // -- show_values toggle -------------------------------------------------

    #[test]
    fn show_values_false_omits_value() {
        let hidden = BarChart::from_pairs([("x", 42.0)]).with_show_values(false);
        assert!(!render_lines(&hidden, 40)[0].contains("42"));

        let shown = BarChart::from_pairs([("x", 42.0)]); // default = true
        assert!(render_lines(&shown, 40)[0].contains("42"));
    }

    // -- segment shape ------------------------------------------------------

    #[test]
    fn segments_non_empty_and_end_in_newline() {
        let chart = BarChart::from_pairs([("x", 1.0)]);
        let console = Console::builder().width(80).build();
        let opts = make_options(80);
        let segs = chart.gilt_console(&console, &opts);
        assert!(!segs.is_empty());
        assert_eq!(segs.last().unwrap().text.as_str(), "\n");
    }

    // -- measure matches rendered width -------------------------------------

    #[test]
    fn measure_matches_rendered_width() {
        let chart = BarChart::from_pairs([("alpha", 3.0), ("b", 5.0)]);
        let console = Console::builder().width(80).build();
        let opts = make_options(80);
        let lines = render_lines(&chart, 80);
        let rendered = lines.iter().map(|l| cell_len(l)).max().unwrap();
        let m = chart.gilt_measure(&console, &opts);
        assert_eq!(m.maximum, rendered);
        assert!(m.minimum <= m.maximum);
    }

    // -- guards -------------------------------------------------------------

    #[test]
    fn zero_max_is_guarded() {
        let chart = BarChart::from_pairs([("a", 5.0)]).with_max(0.0);
        let lines = render_lines(&chart, 40);
        assert!(!lines.is_empty());
        assert!(lines[0].contains('\u{2591}')); // all-track bar
        assert!(!lines[0].contains('\u{2588}')); // no full block
    }

    #[test]
    fn negative_max_is_guarded() {
        let chart = BarChart::from_pairs([("a", 5.0)]).with_max(-5.0);
        let lines = render_lines(&chart, 40);
        assert!(!lines[0].contains('\u{2588}'));
    }

    #[test]
    fn negative_value_clamps_to_empty_bar() {
        let chart = BarChart::from_pairs([("a", -5.0), ("b", 10.0)]);
        let lines = render_lines(&chart, 40);
        // The negative row's bar is empty (no full block, has track).
        assert!(!lines[0].contains('\u{2588}'));
        assert!(lines[0].contains('\u{2591}'));
        // ...but the real value is still shown.
        assert!(lines[0].contains("-5"));
        // The max row fills completely.
        assert!(!lines[1].contains('\u{2591}'));
    }

    // -- builders -----------------------------------------------------------

    #[test]
    fn push_is_chainable() {
        let mut chart = BarChart::new();
        chart.push("a", 1.0).push("b", 2.0);
        assert_eq!(chart.len(), 2);
        assert_eq!(render_lines(&chart, 40).len(), 2);
    }

    #[test]
    fn with_bar_chains() {
        let chart = BarChart::new()
            .with_bar("a", 1.0)
            .with_bar("b", 2.0)
            .with_bar("c", 3.0);
        assert_eq!(render_lines(&chart, 40).len(), 3);
    }

    #[test]
    fn from_pairs_collects_all() {
        let chart = BarChart::from_pairs(vec![("x", 1.0), ("y", 2.0)]);
        assert_eq!(chart.len(), 2);
    }

    // -- Display ------------------------------------------------------------

    #[test]
    fn display_renders_labels_and_blocks() {
        let chart = BarChart::from_pairs([("alpha", 3.0), ("beta", 5.0)]);
        let s = chart.to_string();
        assert!(s.contains("alpha"));
        assert!(s.contains('\u{2588}'));
        assert!(!s.ends_with('\n'));
    }
}
