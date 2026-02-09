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
//! assert_eq!(spark.to_string(), "\u{2581}\u{2583}\u{2585}\u{2587}\u{2585}\u{2583}\u{2581}");
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

    /// Render the sparkline data into a `String` of bar characters.
    fn render_bars(&self) -> String {
        if self.data.is_empty() {
            return String::new();
        }

        // Width of zero explicitly produces empty output.
        if self.width == Some(0) {
            return String::new();
        }

        // Determine the effective data (resample if width differs).
        let effective: Vec<f64> = match self.width {
            Some(w) if w != self.data.len() => Self::resample(&self.data, w),
            _ => self.data.clone(),
        };

        if effective.is_empty() {
            return String::new();
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
                return String::from(BARS[7]);
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
    fn rich_console(&self, _console: &Console, _options: &ConsoleOptions) -> Vec<Segment> {
        let text = self.render_bars();
        if text.is_empty() {
            return vec![Segment::line()];
        }
        vec![
            Segment::new(&text, Some(self.style.clone()), None),
            Segment::line(),
        ]
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
        let style = Style::parse("bold red").unwrap();
        let spark = Sparkline::new(&[1.0, 2.0, 3.0]).with_style(style.clone());
        let console = Console::builder().width(80).build();
        let opts = make_options(80);
        let segments = spark.rich_console(&console, &opts);
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
        let segments = spark.rich_console(&console, &opts);
        assert_eq!(segments.len(), 2); // content + newline
        assert_eq!(segments[1].text.as_str(), "\n");
    }

    // 14. Renderable for empty data
    #[test]
    fn test_renderable_empty() {
        let spark = Sparkline::new(&[]);
        let console = Console::builder().width(80).build();
        let opts = make_options(80);
        let segments = spark.rich_console(&console, &opts);
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
            .with_style(Style::parse("green").unwrap());
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
}
