//! Sparkline -- inline Unicode sparkline charts.
//!
//! Renders a sequence of numeric values as a single line of Unicode block
//! characters (`\u{2581}`..`\u{2588}`), ideal for inline visualisation of
//! time-series data, CPU usage, stock prices, and similar metrics.
//!
//! # Example
//!
//! ```
//! use gilt::sparkline::Sparkline;
//!
//! let spark = Sparkline::new(&[1.0, 3.0, 5.0, 7.0, 5.0, 3.0, 1.0]);
//! assert_eq!(spark.to_string(), "\u{2581}\u{2583}\u{2586}\u{2588}\u{2586}\u{2583}\u{2581}");
//! ```

use std::fmt;

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::Style;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Unicode block elements used for sparkline bars, from lowest to highest.
const BARS: [char; 8] = [
    '\u{2581}', // LOWER ONE EIGHTH BLOCK
    '\u{2582}', // LOWER ONE QUARTER BLOCK
    '\u{2583}', // LOWER THREE EIGHTHS BLOCK
    '\u{2584}', // LOWER HALF BLOCK
    '\u{2585}', // LOWER FIVE EIGHTHS BLOCK
    '\u{2586}', // LOWER THREE QUARTERS BLOCK
    '\u{2587}', // LOWER SEVEN EIGHTHS BLOCK
    '\u{2588}', // FULL BLOCK
];

// ---------------------------------------------------------------------------
// Sparkline
// ---------------------------------------------------------------------------

/// An inline sparkline chart rendered with Unicode block characters.
///
/// Each numeric value maps to one of eight block heights (`\u{2581}`..`\u{2588}`),
/// producing a compact, single-line visualisation.
#[derive(Debug, Clone)]
pub struct Sparkline {
    /// The data points to render.
    data: Vec<f64>,
    /// Optional fixed width.  When `Some(n)`, the data is resampled to fit
    /// exactly `n` terminal columns.  When `None`, one column per data point.
    width: Option<usize>,
    /// Explicit minimum value for scaling.  When `None`, derived from data.
    min_value: Option<f64>,
    /// Explicit maximum value for scaling.  When `None`, derived from data.
    max_value: Option<f64>,
    /// Visual style applied to the sparkline output.
    style: Style,
    /// When `true`, the minimum and maximum data cells are drawn with distinct
    /// styles layered over [`style`](Self::style).
    markers: bool,
    /// Style layered over the base style for the minimum data cell (markers on).
    min_style: Style,
    /// Style layered over the base style for the maximum data cell (markers on).
    max_style: Style,
}

impl Sparkline {
    /// Create a new sparkline from a slice of values.
    pub fn new(data: &[f64]) -> Self {
        Self {
            data: data.to_vec(),
            width: None,
            min_value: None,
            max_value: None,
            style: Style::null(),
            markers: false,
            min_style: Style::parse("blue"),
            max_style: Style::parse("bold red"),
        }
    }

    /// Set a fixed output width (builder pattern).
    ///
    /// When specified, the data is resampled via linear interpolation to fill
    /// exactly `width` columns.
    #[must_use]
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Set an explicit minimum value for scaling (builder pattern).
    #[must_use]
    pub fn with_min(mut self, min: f64) -> Self {
        self.min_value = Some(min);
        self
    }

    /// Set an explicit maximum value for scaling (builder pattern).
    #[must_use]
    pub fn with_max(mut self, max: f64) -> Self {
        self.max_value = Some(max);
        self
    }

    /// Set the visual style (builder pattern).
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Enable or disable min/max marker styling (builder pattern).
    ///
    /// When enabled, the cell holding the minimum data point is drawn with
    /// [`with_min_style`](Self::with_min_style) and the cell holding the
    /// maximum with [`with_max_style`](Self::with_max_style), each layered over
    /// the base style. Markers are located from the *original* data's extrema;
    /// when the data is resampled to a different width, each extreme is mapped
    /// to the resampled cell that samples nearest it.
    ///
    /// Defaults to `false`; when disabled the rendered output is byte-identical
    /// to a sparkline without markers. If every data point is equal there are no
    /// distinct extrema, so no markers are drawn even when enabled.
    #[must_use]
    pub fn with_min_max_markers(mut self, enabled: bool) -> Self {
        self.markers = enabled;
        self
    }

    /// Set the style layered over the base style for the minimum data cell
    /// (builder pattern).
    ///
    /// Only takes effect when markers are enabled via
    /// [`with_min_max_markers`](Self::with_min_max_markers). Defaults to `blue`.
    #[must_use]
    pub fn with_min_style(mut self, style: Style) -> Self {
        self.min_style = style;
        self
    }

    /// Set the style layered over the base style for the maximum data cell
    /// (builder pattern).
    ///
    /// Only takes effect when markers are enabled via
    /// [`with_min_max_markers`](Self::with_min_max_markers). Defaults to
    /// `bold red`.
    #[must_use]
    pub fn with_max_style(mut self, style: Style) -> Self {
        self.max_style = style;
        self
    }

    // -- internal helpers ---------------------------------------------------

    /// Resample `data` to `target_len` points using linear interpolation.
    fn resample(data: &[f64], target_len: usize) -> Vec<f64> {
        if data.is_empty() || target_len == 0 {
            return Vec::new();
        }
        if data.len() == 1 {
            return vec![data[0]; target_len];
        }
        let src_len = data.len();
        (0..target_len)
            .map(|i| {
                let t = i as f64 * (src_len - 1) as f64 / (target_len - 1).max(1) as f64;
                let lo = (t.floor() as usize).min(src_len - 1);
                let hi = (lo + 1).min(src_len - 1);
                let frac = t - lo as f64;
                data[lo] * (1.0 - frac) + data[hi] * frac
            })
            .collect()
    }

    /// Render the sparkline data into the per-cell bar characters.
    fn bar_chars(&self) -> Vec<char> {
        if self.data.is_empty() {
            return Vec::new();
        }

        // Width of zero explicitly produces empty output.
        if self.width == Some(0) {
            return Vec::new();
        }

        // Determine the effective data (resample if width differs).
        // Finding #26: use Cow to avoid cloning the Vec when no resampling is needed.
        use std::borrow::Cow;
        let effective: Cow<[f64]> = match self.width {
            Some(w) if w != self.data.len() => Cow::Owned(Self::resample(&self.data, w)),
            _ => Cow::Borrowed(&self.data),
        };

        if effective.is_empty() {
            return Vec::new();
        }

        let min = self
            .min_value
            .unwrap_or_else(|| effective.iter().cloned().fold(f64::INFINITY, f64::min));
        let max = self
            .max_value
            .unwrap_or_else(|| effective.iter().cloned().fold(f64::NEG_INFINITY, f64::max));

        // Edge case: all values identical (or min == max).
        if (max - min).abs() < f64::EPSILON {
            // Single value => full block; all-same => middle block.
            if effective.len() == 1 {
                return vec![BARS[7]];
            }
            return std::iter::repeat_n(BARS[3], effective.len()).collect();
        }

        effective
            .iter()
            .map(|&v| {
                let clamped = v.clamp(min, max);
                let idx = ((clamped - min) / (max - min) * 7.0).round() as usize;
                BARS[idx.min(7)]
            })
            .collect()
    }

    /// Render the sparkline data into a `String` of bar characters.
    fn render_bars(&self) -> String {
        self.bar_chars().into_iter().collect()
    }

    /// Compute the cell indices that should carry the min and max marker styles
    /// for a rendering with `cell_len` cells.
    ///
    /// Markers are located from the *original* data's extrema (the first
    /// occurrence of the minimum and of the maximum). When the data is resampled
    /// to a different width, each extreme source index is mapped to the
    /// resampled cell that samples nearest it (the inverse of the linear
    /// resample map). Returns `None` when markers are disabled, the data is
    /// empty, or every value is equal (no distinct extrema to mark).
    fn marker_cells(&self, cell_len: usize) -> Option<(usize, usize)> {
        if !self.markers || cell_len == 0 || self.data.is_empty() {
            return None;
        }

        let data = &self.data;
        let (mut min_i, mut max_i) = (0usize, 0usize);
        for (i, &v) in data.iter().enumerate() {
            if v < data[min_i] {
                min_i = i;
            }
            if v > data[max_i] {
                max_i = i;
            }
        }

        // All values equal => no distinct extrema, so no markers.
        if (data[max_i] - data[min_i]).abs() < f64::EPSILON {
            return None;
        }

        let src_len = data.len();
        let to_cell = |src: usize| -> usize {
            if cell_len <= 1 || src_len <= 1 {
                0
            } else {
                let cell = (src as f64 * (cell_len - 1) as f64 / (src_len - 1) as f64).round();
                (cell as usize).min(cell_len - 1)
            }
        };

        Some((to_cell(min_i), to_cell(max_i)))
    }

    /// Effective output width.
    fn effective_width(&self) -> usize {
        self.width.unwrap_or(self.data.len())
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl fmt::Display for Sparkline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render_bars())
    }
}

// ---------------------------------------------------------------------------
// Renderable
// ---------------------------------------------------------------------------

impl Renderable for Sparkline {
    fn gilt_console(&self, _console: &Console, _options: &ConsoleOptions) -> Vec<Segment> {
        let chars = self.bar_chars();
        if chars.is_empty() {
            return vec![Segment::line()];
        }

        match self.marker_cells(chars.len()) {
            // No markers: classic single-segment output (byte-identical).
            None => {
                let text: String = chars.iter().collect();
                vec![
                    Segment::new(&text, Some(self.style.clone()), None),
                    Segment::line(),
                ]
            }
            // Markers: split out the min/max cells, layering each marker style
            // over the base style. On collision (both extrema land on the same
            // cell after resampling) the max marker wins.
            Some((min_cell, max_cell)) => {
                let min_style = self.style.clone() + self.min_style.clone();
                let max_style = self.style.clone() + self.max_style.clone();
                let style_at = |i: usize| -> Style {
                    if i == max_cell {
                        max_style.clone()
                    } else if i == min_cell {
                        min_style.clone()
                    } else {
                        self.style.clone()
                    }
                };

                // Coalesce consecutive cells with equal style into one segment.
                let mut segments = Vec::new();
                let mut run = String::new();
                let mut run_style = style_at(0);
                run.push(chars[0]);
                for (i, &ch) in chars.iter().enumerate().skip(1) {
                    let style = style_at(i);
                    if style == run_style {
                        run.push(ch);
                    } else {
                        segments.push(Segment::new(&run, Some(run_style), None));
                        run = String::from(ch);
                        run_style = style;
                    }
                }
                segments.push(Segment::new(&run, Some(run_style), None));
                segments.push(Segment::line());
                segments
            }
        }
    }

    fn gilt_measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        self.measure(console, options)
    }
}

// ---------------------------------------------------------------------------
// Measure
// ---------------------------------------------------------------------------

impl Sparkline {
    /// Return the measurement for this sparkline.
    pub fn measure(&self, _console: &Console, _options: &ConsoleOptions) -> Measurement {
        let w = self.effective_width();
        Measurement::new(1.min(w), w)
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

    // -- gilt_measure override ----------------------------------------------

    #[test]
    fn sparkline_gilt_measure_delegates_to_measure() {
        let spark = Sparkline::new(&[1.0, 2.0, 3.0]);
        let console = Console::builder()
            .width(80)
            .force_terminal(true)
            .no_color(true)
            .build();
        let opts = console.options();
        assert_eq!(
            spark.gilt_measure(&console, &opts),
            spark.measure(&console, &opts),
            "Sparkline::gilt_measure must delegate to Sparkline::measure",
        );
    }

    // 1. Empty data
    #[test]
    fn test_empty_data() {
        let spark = Sparkline::new(&[]);
        assert_eq!(spark.to_string(), "");
    }

    // 2. Single value
    #[test]
    fn test_single_value() {
        let spark = Sparkline::new(&[42.0]);
        assert_eq!(spark.to_string(), "\u{2588}");
    }

    // 3. All same values
    #[test]
    fn test_all_same_values() {
        let spark = Sparkline::new(&[5.0, 5.0, 5.0, 5.0]);
        let text = spark.to_string();
        assert_eq!(text, "\u{2584}\u{2584}\u{2584}\u{2584}");
    }

    // 4. Ascending values
    #[test]
    fn test_ascending_values() {
        let spark = Sparkline::new(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
        let text = spark.to_string();
        let expected: String = BARS.iter().collect();
        assert_eq!(text, expected);
    }

    // 5. Descending values
    #[test]
    fn test_descending_values() {
        let spark = Sparkline::new(&[8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0]);
        let text = spark.to_string();
        let expected: String = BARS.iter().rev().collect();
        assert_eq!(text, expected);
    }

    // 6. Negative values
    #[test]
    fn test_negative_values() {
        let spark = Sparkline::new(&[-10.0, -5.0, 0.0, 5.0, 10.0]);
        let text = spark.to_string();
        assert_eq!(text.chars().count(), 5);
        let chars: Vec<char> = text.chars().collect();
        assert_eq!(chars[0], BARS[0]); // min
        assert_eq!(chars[4], BARS[7]); // max
    }

    // 7. Custom min/max
    #[test]
    fn test_custom_min_max() {
        let spark = Sparkline::new(&[5.0, 5.0, 5.0])
            .with_min(0.0)
            .with_max(10.0);
        let text = spark.to_string();
        // 5 is midpoint of 0..10 => index ~3.5 => rounds to 4
        let expected_char = BARS[4]; // (5/10)*7 = 3.5 => round = 4
        for ch in text.chars() {
            assert_eq!(ch, expected_char);
        }
    }

    // 8. Width resampling (expand)
    #[test]
    fn test_width_resampling_expand() {
        let spark = Sparkline::new(&[0.0, 10.0]).with_width(5);
        let text = spark.to_string();
        assert_eq!(text.chars().count(), 5);
        let chars: Vec<char> = text.chars().collect();
        assert_eq!(chars[0], BARS[0]); // 0.0
        assert_eq!(chars[4], BARS[7]); // 10.0
    }

    // 9. Width resampling (shrink)
    #[test]
    fn test_width_resampling_shrink() {
        let spark = Sparkline::new(&[0.0, 2.5, 5.0, 7.5, 10.0]).with_width(3);
        let text = spark.to_string();
        assert_eq!(text.chars().count(), 3);
    }

    // 10. Float precision
    #[test]
    fn test_float_precision() {
        let spark = Sparkline::new(&[0.001, 0.002, 0.003]);
        let text = spark.to_string();
        assert_eq!(text.chars().count(), 3);
        let chars: Vec<char> = text.chars().collect();
        assert_eq!(chars[0], BARS[0]);
        assert_eq!(chars[2], BARS[7]);
    }

    // 11. Style application
    #[test]
    fn test_style_application() {
        let style = Style::parse("bold red");
        let spark = Sparkline::new(&[1.0, 2.0, 3.0]).with_style(style.clone());
        let console = Console::builder().width(80).build();
        let opts = make_options(80);
        let segments = spark.gilt_console(&console, &opts);
        assert_eq!(segments[0].style.as_ref(), Some(&style));
    }

    // 12. Display trait
    #[test]
    fn test_display_trait() {
        let spark = Sparkline::new(&[1.0, 8.0]);
        let displayed = format!("{spark}");
        assert_eq!(displayed.chars().count(), 2);
        assert_eq!(displayed.chars().next().unwrap(), BARS[0]);
        assert_eq!(displayed.chars().last().unwrap(), BARS[7]);
    }

    // 13. Renderable produces correct segments
    #[test]
    fn test_renderable_segments() {
        let spark = Sparkline::new(&[1.0, 2.0, 3.0]);
        let console = Console::builder().width(80).build();
        let opts = make_options(80);
        let segments = spark.gilt_console(&console, &opts);
        assert_eq!(segments.len(), 2); // content + newline
        assert_eq!(segments[1].text.as_str(), "\n");
    }

    // 14. Renderable for empty data
    #[test]
    fn test_renderable_empty() {
        let spark = Sparkline::new(&[]);
        let console = Console::builder().width(80).build();
        let opts = make_options(80);
        let segments = spark.gilt_console(&console, &opts);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text.as_str(), "\n");
    }

    // 15. Measure
    #[test]
    fn test_measure() {
        let spark = Sparkline::new(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let console = Console::builder().width(80).build();
        let opts = make_options(80);
        let m = spark.measure(&console, &opts);
        assert_eq!(m, Measurement::new(1, 5));
    }

    // 16. Measure with width
    #[test]
    fn test_measure_with_width() {
        let spark = Sparkline::new(&[1.0, 2.0, 3.0]).with_width(10);
        let console = Console::builder().width(80).build();
        let opts = make_options(80);
        let m = spark.measure(&console, &opts);
        assert_eq!(m, Measurement::new(1, 10));
    }

    // 17. Width zero edge case
    #[test]
    fn test_width_zero() {
        let spark = Sparkline::new(&[1.0, 2.0, 3.0]).with_width(0);
        let text = spark.to_string();
        assert_eq!(text, "");
    }

    // 18. Builder chaining
    #[test]
    fn test_builder_chaining() {
        let spark = Sparkline::new(&[1.0, 2.0])
            .with_width(10)
            .with_min(0.0)
            .with_max(10.0)
            .with_style(Style::parse("green"));
        assert_eq!(spark.width, Some(10));
        assert_eq!(spark.min_value, Some(0.0));
        assert_eq!(spark.max_value, Some(10.0));
    }

    // 19. Large data set
    #[test]
    fn test_large_data() {
        let data: Vec<f64> = (0..1000).map(|i| (i as f64).sin()).collect();
        let spark = Sparkline::new(&data);
        let text = spark.to_string();
        assert_eq!(text.chars().count(), 1000);
    }

    // 20. Resample single value to many
    #[test]
    fn test_resample_single_to_many() {
        let spark = Sparkline::new(&[5.0]).with_width(4);
        let text = spark.to_string();
        // Single value resampled to 4 => all same => middle bar
        assert_eq!(text.chars().count(), 4);
    }

    // 21. Min greater than data
    #[test]
    fn test_custom_min_clamping() {
        let spark = Sparkline::new(&[1.0, 2.0, 3.0])
            .with_min(0.0)
            .with_max(100.0);
        let text = spark.to_string();
        assert_eq!(text.chars().count(), 3);
        // All values are near the bottom of the 0-100 range
        for ch in text.chars() {
            assert_eq!(ch, BARS[0]);
        }
    }

    // -- min/max markers ----------------------------------------------------

    /// Expand a sparkline's rendered segments into one `Style` per cell,
    /// ignoring the trailing newline segment. Robust to segment coalescing.
    fn cell_styles(spark: &Sparkline) -> Vec<Style> {
        let console = Console::builder().width(80).build();
        let opts = make_options(80);
        let mut styles = Vec::new();
        for seg in spark.gilt_console(&console, &opts) {
            if seg.text.as_str() == "\n" {
                continue;
            }
            for _ in seg.text.as_str().chars() {
                styles.push(seg.style.clone().unwrap_or_else(Style::null));
            }
        }
        styles
    }

    // 22. Markers default off => output byte-identical to no-marker rendering.
    #[test]
    fn test_markers_off_is_regression_identical() {
        let data = [3.0, 1.0, 5.0, 2.0];
        let console = Console::builder().width(80).build();
        let opts = make_options(80);

        let plain = Sparkline::new(&data);
        let explicit_off = Sparkline::new(&data).with_min_max_markers(false);

        // Default and explicit-off both emit the classic single content segment.
        let plain_segs = plain.gilt_console(&console, &opts);
        let off_segs = explicit_off.gilt_console(&console, &opts);
        assert_eq!(plain_segs.len(), 2, "no markers => content + newline");
        assert_eq!(plain_segs.len(), off_segs.len());
        assert_eq!(plain_segs[0].text.as_str(), off_segs[0].text.as_str());
        assert_eq!(plain_segs[0].style, Some(Style::null()));
        // Display output unaffected by the marker flag.
        assert_eq!(plain.to_string(), explicit_off.to_string());
    }

    // 23. Markers on: min cell carries min_style, max cell carries max_style.
    #[test]
    fn test_markers_apply_default_styles() {
        // min value 1.0 at index 1, max value 5.0 at index 2 (no resampling).
        let spark = Sparkline::new(&[3.0, 1.0, 5.0, 2.0]).with_min_max_markers(true);
        let styles = cell_styles(&spark);
        assert_eq!(styles.len(), 4);
        assert_eq!(styles[0], Style::null());
        assert_eq!(styles[1], Style::null() + Style::parse("blue"));
        assert_eq!(styles[2], Style::null() + Style::parse("bold red"));
        assert_eq!(styles[3], Style::null());
    }

    // 24. Marker styles layer over (do not replace) the base style.
    #[test]
    fn test_markers_layer_over_base_style() {
        let base = Style::parse("on white");
        // min at index 1, max at index 2.
        let spark = Sparkline::new(&[2.0, 0.0, 4.0])
            .with_style(base.clone())
            .with_min_max_markers(true);
        let styles = cell_styles(&spark);
        assert_eq!(styles[0], base.clone());
        assert_eq!(styles[1], base.clone() + Style::parse("blue"));
        assert_eq!(styles[2], base + Style::parse("bold red"));
    }

    // 25. Custom marker styles override the defaults.
    #[test]
    fn test_markers_custom_styles() {
        let spark = Sparkline::new(&[2.0, 0.0, 4.0])
            .with_min_max_markers(true)
            .with_min_style(Style::parse("green"))
            .with_max_style(Style::parse("magenta"));
        let styles = cell_styles(&spark);
        assert_eq!(styles[1], Style::null() + Style::parse("green"));
        assert_eq!(styles[2], Style::null() + Style::parse("magenta"));
    }

    // 26. All-equal data with markers on => no markers, no panic.
    #[test]
    fn test_markers_all_equal_no_markers() {
        let spark = Sparkline::new(&[5.0, 5.0, 5.0, 5.0]).with_min_max_markers(true);
        let styles = cell_styles(&spark);
        assert_eq!(styles.len(), 4);
        for s in &styles {
            assert_eq!(*s, Style::null(), "all-equal data must not be marked");
        }
    }

    // 27. Single element with markers on => no markers, no panic.
    #[test]
    fn test_markers_single_element_no_panic() {
        let spark = Sparkline::new(&[42.0]).with_min_max_markers(true);
        let styles = cell_styles(&spark);
        assert_eq!(styles.len(), 1);
        assert_eq!(styles[0], Style::null());
    }

    // 28. Empty data with markers on => single newline segment, no panic.
    #[test]
    fn test_markers_empty_no_panic() {
        let spark = Sparkline::new(&[]).with_min_max_markers(true);
        let console = Console::builder().width(80).build();
        let opts = make_options(80);
        let segments = spark.gilt_console(&console, &opts);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text.as_str(), "\n");
    }

    // 29. Resampling maps each extreme to the nearest resampled cell.
    #[test]
    fn test_markers_resampled_extreme_cell() {
        // src_len 5: min 1.0 at index 0, max 9.0 at index 1; resampled to 3 cells.
        // to_cell(0) = round(0 * 2/4) = 0; to_cell(1) = round(1 * 2/4) = round(0.5) = 1.
        let spark = Sparkline::new(&[1.0, 9.0, 1.0, 1.0, 1.0])
            .with_width(3)
            .with_min_max_markers(true);
        let styles = cell_styles(&spark);
        assert_eq!(styles.len(), 3);
        assert_eq!(styles[0], Style::null() + Style::parse("blue"));
        assert_eq!(styles[1], Style::null() + Style::parse("bold red"));
        assert_eq!(styles[2], Style::null());
    }

    // 30. Collision (min and max collapse to one cell): max wins.
    #[test]
    fn test_markers_collision_max_wins() {
        // [1, 9] resampled to a single cell: both extrema map to cell 0.
        let spark = Sparkline::new(&[1.0, 9.0])
            .with_width(1)
            .with_min_max_markers(true);
        let styles = cell_styles(&spark);
        assert_eq!(styles.len(), 1);
        assert_eq!(styles[0], Style::null() + Style::parse("bold red"));
    }
}
