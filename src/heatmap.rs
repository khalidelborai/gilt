//! Heatmap -- a 2-D grid of values rendered as coloured cell blocks.
//!
//! Each cell is a solid block of background colour whose hue encodes the cell's
//! value: values are normalised to `[0, 1]` and mapped through a multi-stop
//! colour gradient (dark-blue → cyan → yellow → red by default).  This is the
//! classic "heatmap" / matrix visualisation — ideal for correlation matrices,
//! density grids, confusion matrices, and similar tabular numeric data.
//!
//! Cells are rendered as `cell_width` spaces (default `2`, so cells look roughly
//! square) carrying the colour as a [`Style`] *background*.  One grid row is
//! emitted per line, terminated by a trailing newline.
//!
//! Because the colour lives in the background, a [`Heatmap`] must be rendered
//! through a [`Console`] (which serialises styles to ANSI) to be meaningful —
//! the visible glyphs are all spaces.
//!
//! # Example
//!
//! ```
//! use gilt::heatmap::Heatmap;
//! use gilt::console::Console;
//!
//! let grid = vec![
//!     vec![0.0, 0.5, 1.0],
//!     vec![1.0, 0.5, 0.0],
//! ];
//! let heatmap = Heatmap::new(grid);
//!
//! let mut console = Console::builder().width(80).force_terminal(true).build();
//! console.print(&heatmap);
//! ```

use crate::color::Color;
use crate::console::{Console, ConsoleOptions, Renderable};
use crate::gradient::interpolate_color;
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::Style;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default number of character columns per cell (cells look roughly square).
const DEFAULT_CELL_WIDTH: usize = 2;

// ---------------------------------------------------------------------------
// Heatmap
// ---------------------------------------------------------------------------

/// A 2-D grid of `f64` values rendered as a colour-coded cell grid.
///
/// Construct from a `Vec<Vec<f64>>` (or `&[Vec<f64>]`), optionally override the
/// value range and gradient, then render through a [`Console`].
///
/// Ragged rows (rows of differing length) are right-padded to the width of the
/// widest row using the effective minimum value, so the rendered grid is always
/// rectangular; padded cells therefore take the first gradient stop.
#[derive(Debug, Clone)]
pub struct Heatmap {
    /// The grid of values, one inner `Vec` per row.
    rows: Vec<Vec<f64>>,
    /// Explicit minimum for normalisation.  When `None`, derived from the data.
    min_value: Option<f64>,
    /// Explicit maximum for normalisation.  When `None`, derived from the data.
    max_value: Option<f64>,
    /// Colour gradient stops (≥2 for a visible gradient).
    gradient: Vec<Color>,
    /// Character columns per cell.
    cell_width: usize,
}

impl Heatmap {
    /// Create a new heatmap from a grid of values.
    ///
    /// Accepts anything convertible into `Vec<Vec<f64>>` — both an owned
    /// `Vec<Vec<f64>>` and a borrowed `&[Vec<f64>]` work ergonomically:
    ///
    /// ```
    /// use gilt::heatmap::Heatmap;
    ///
    /// let owned = Heatmap::new(vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
    /// let slice: &[Vec<f64>] = &[vec![1.0, 2.0], vec![3.0, 4.0]];
    /// let borrowed = Heatmap::new(slice);
    /// ```
    pub fn new(rows: impl Into<Vec<Vec<f64>>>) -> Self {
        Self {
            rows: rows.into(),
            min_value: None,
            max_value: None,
            gradient: default_gradient(),
            cell_width: DEFAULT_CELL_WIDTH,
        }
    }

    /// Set an explicit minimum value for normalisation (builder pattern).
    #[must_use]
    pub fn with_min(mut self, min: f64) -> Self {
        self.min_value = Some(min);
        self
    }

    /// Set an explicit maximum value for normalisation (builder pattern).
    #[must_use]
    pub fn with_max(mut self, max: f64) -> Self {
        self.max_value = Some(max);
        self
    }

    /// Set the colour gradient stops (builder pattern).
    ///
    /// At least two stops are expected; with a single stop every cell renders in
    /// that colour, and with no stops the terminal default colour is used.
    #[must_use]
    pub fn with_gradient(mut self, gradient: Vec<Color>) -> Self {
        self.gradient = gradient;
        self
    }

    /// Set the number of character columns per cell (builder pattern, default `2`).
    #[must_use]
    pub fn with_cell_width(mut self, cell_width: usize) -> Self {
        self.cell_width = cell_width;
        self
    }

    // -- internal helpers ---------------------------------------------------

    /// Number of columns = width of the widest row (`0` for an empty grid).
    fn cols(&self) -> usize {
        self.rows.iter().map(Vec::len).max().unwrap_or(0)
    }

    /// Effective `(min, max)` used for normalisation.
    ///
    /// Auto-derived from the data when not overridden; for an empty grid the
    /// bounds are unused (rendering short-circuits) but are returned as
    /// `(0.0, 1.0)` to stay well-defined.
    fn bounds(&self) -> (f64, f64) {
        let auto_min = || {
            self.rows
                .iter()
                .flatten()
                .copied()
                .fold(f64::INFINITY, f64::min)
        };
        let auto_max = || {
            self.rows
                .iter()
                .flatten()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max)
        };
        let min = self.min_value.unwrap_or_else(auto_min);
        let max = self.max_value.unwrap_or_else(auto_max);
        if min.is_finite() && max.is_finite() {
            (min, max)
        } else {
            (0.0, 1.0)
        }
    }

    /// Normalise `v` into `[0, 1]`, guarding against a zero (or NaN) range.
    fn normalize(v: f64, min: f64, max: f64) -> f64 {
        let range = max - min;
        if !range.is_finite() || range.abs() < f64::EPSILON {
            return 0.0;
        }
        ((v - min) / range).clamp(0.0, 1.0)
    }

    /// Map a normalised parameter `t ∈ [0, 1]` to a colour on the gradient.
    fn gradient_color(&self, t: f64) -> Color {
        match self.gradient.len() {
            0 => Color::default_color(),
            1 => self.gradient[0],
            n => {
                let t = t.clamp(0.0, 1.0);
                let segments = n - 1;
                let scaled = t * segments as f64;
                let seg = (scaled.floor() as usize).min(segments - 1);
                let local = scaled - seg as f64;
                interpolate_color(&self.gradient[seg], &self.gradient[seg + 1], local)
            }
        }
    }
}

/// The default heatmap gradient: dark-blue → cyan → yellow → red.
fn default_gradient() -> Vec<Color> {
    vec![
        Color::from_rgb(0, 0, 128),   // dark blue
        Color::from_rgb(0, 255, 255), // cyan
        Color::from_rgb(255, 255, 0), // yellow
        Color::from_rgb(255, 0, 0),   // red
    ]
}

// ---------------------------------------------------------------------------
// Renderable
// ---------------------------------------------------------------------------

impl Renderable for Heatmap {
    fn gilt_console(&self, _console: &Console, _options: &ConsoleOptions) -> Vec<Segment> {
        let cols = self.cols();
        // A grid with no rows (or no columns) has nothing to draw.
        if cols == 0 {
            return Vec::new();
        }

        let (min, max) = self.bounds();
        let cell = " ".repeat(self.cell_width);

        let mut segments = Vec::with_capacity(self.rows.len() * (cols + 1));
        for row in &self.rows {
            for c in 0..cols {
                // Ragged rows are padded to `cols` with the effective minimum.
                let v = row.get(c).copied().unwrap_or(min);
                let t = Self::normalize(v, min, max);
                let color = self.gradient_color(t);
                segments.push(Segment::new(&cell, Some(Style::null().bg(color)), None));
            }
            segments.push(Segment::line());
        }
        segments
    }

    fn gilt_measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        self.measure(console, options)
    }
}

// ---------------------------------------------------------------------------
// Measure
// ---------------------------------------------------------------------------

impl Heatmap {
    /// Return the measurement for this heatmap.
    ///
    /// The rendered width is fixed at `cols * cell_width`, so the minimum and
    /// maximum widths are equal.
    pub fn measure(&self, _console: &Console, _options: &ConsoleOptions) -> Measurement {
        let w = self.cols() * self.cell_width;
        Measurement::new(w, w)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color_triplet::ColorTriplet;
    use crate::console::Console;

    fn console() -> Console {
        Console::builder()
            .width(80)
            .force_terminal(true)
            .no_color(false)
            .build()
    }

    /// Split rendered segments into lines (by newline control segments) and
    /// return, per line, the total visible cell-text width (in chars).
    fn line_widths(segments: &[Segment]) -> Vec<usize> {
        let mut widths = vec![0usize];
        for seg in segments {
            if seg.text.as_str() == "\n" {
                widths.push(0);
            } else {
                *widths.last_mut().unwrap() += seg.text.chars().count();
            }
        }
        // Drop the trailing empty bucket created by the final newline.
        if widths.last() == Some(&0) {
            widths.pop();
        }
        widths
    }

    /// Background colour triplet of the `idx`-th cell segment (ignoring newlines).
    fn cell_bg(segments: &[Segment], idx: usize) -> ColorTriplet {
        let cell = segments
            .iter()
            .filter(|s| s.text.as_str() != "\n")
            .nth(idx)
            .expect("cell exists");
        cell.style
            .as_ref()
            .expect("cell has style")
            .bgcolor()
            .expect("cell has bg colour")
            .get_truecolor(None, false)
    }

    // -- gilt_measure delegates ---------------------------------------------

    #[test]
    fn heatmap_gilt_measure_delegates_to_measure() {
        let h = Heatmap::new(vec![vec![1.0, 2.0, 3.0]]);
        let c = console();
        let opts = c.options();
        assert_eq!(h.gilt_measure(&c, &opts), h.measure(&c, &opts));
    }

    // 1. Grid dimensions reflected in output.
    #[test]
    fn test_grid_dimensions() {
        let h = Heatmap::new(vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]);
        let c = console();
        let segs = h.gilt_console(&c, &c.options());
        // 2 rows -> 2 lines, each 3 cols * 2 cell_width = 6 chars wide.
        let widths = line_widths(&segs);
        assert_eq!(widths, vec![6, 6]);
    }

    // 2. Custom cell width changes line width.
    #[test]
    fn test_cell_width() {
        let h = Heatmap::new(vec![vec![1.0, 2.0]]).with_cell_width(4);
        let c = console();
        let segs = h.gilt_console(&c, &c.options());
        assert_eq!(line_widths(&segs), vec![8]); // 2 cols * 4
    }

    // 3. Value -> colour at t = 0 (first stop).
    #[test]
    fn test_color_at_t0_first_stop() {
        let h = Heatmap::new(vec![vec![0.0, 1.0]]).with_gradient(vec![
            Color::from_rgb(0, 0, 0),
            Color::from_rgb(255, 255, 255),
        ]);
        let c = console();
        let segs = h.gilt_console(&c, &c.options());
        assert_eq!(cell_bg(&segs, 0), ColorTriplet::new(0, 0, 0));
    }

    // 4. Value -> colour at t = 1 (last stop).
    #[test]
    fn test_color_at_t1_last_stop() {
        let h = Heatmap::new(vec![vec![0.0, 1.0]]).with_gradient(vec![
            Color::from_rgb(0, 0, 0),
            Color::from_rgb(255, 255, 255),
        ]);
        let c = console();
        let segs = h.gilt_console(&c, &c.options());
        assert_eq!(cell_bg(&segs, 1), ColorTriplet::new(255, 255, 255));
    }

    // 5. Value -> colour at a midpoint (t = 0.5).
    #[test]
    fn test_color_at_midpoint() {
        let h = Heatmap::new(vec![vec![0.0, 0.5, 1.0]]).with_gradient(vec![
            Color::from_rgb(0, 0, 0),
            Color::from_rgb(255, 255, 255),
        ]);
        let c = console();
        let segs = h.gilt_console(&c, &c.options());
        // interpolate(0, 255, 0.5) = round(127.5) = 128.
        assert_eq!(cell_bg(&segs, 1), ColorTriplet::new(128, 128, 128));
    }

    // 6. Default gradient: first/last stops.
    #[test]
    fn test_default_gradient_endpoints() {
        let h = Heatmap::new(vec![vec![0.0, 1.0]]);
        let c = console();
        let segs = h.gilt_console(&c, &c.options());
        assert_eq!(cell_bg(&segs, 0), ColorTriplet::new(0, 0, 128)); // dark blue
        assert_eq!(cell_bg(&segs, 1), ColorTriplet::new(255, 0, 0)); // red
    }

    // 7. with_min / with_max override auto bounds.
    #[test]
    fn test_with_min_max_override() {
        // Data 0..10 but bounds forced to 0..20 -> value 10 maps to t = 0.5.
        let h = Heatmap::new(vec![vec![0.0, 10.0]])
            .with_min(0.0)
            .with_max(20.0)
            .with_gradient(vec![
                Color::from_rgb(0, 0, 0),
                Color::from_rgb(200, 200, 200),
            ]);
        let c = console();
        let segs = h.gilt_console(&c, &c.options());
        // value 0 -> t 0 -> black.
        assert_eq!(cell_bg(&segs, 0), ColorTriplet::new(0, 0, 0));
        // value 10 -> t 0.5 -> round(100) = 100.
        assert_eq!(cell_bg(&segs, 1), ColorTriplet::new(100, 100, 100));
    }

    // 8. max == min guarded (no panic / NaN), maps to first stop.
    #[test]
    fn test_max_equals_min_guard() {
        let h = Heatmap::new(vec![vec![5.0, 5.0, 5.0]]).with_gradient(vec![
            Color::from_rgb(10, 20, 30),
            Color::from_rgb(200, 100, 50),
        ]);
        let c = console();
        let segs = h.gilt_console(&c, &c.options());
        // All equal -> t = 0 -> first stop, for every cell.
        for i in 0..3 {
            assert_eq!(cell_bg(&segs, i), ColorTriplet::new(10, 20, 30));
        }
    }

    // 9. Empty grid produces no output and does not panic.
    #[test]
    fn test_empty_grid() {
        let h = Heatmap::new(Vec::<Vec<f64>>::new());
        let c = console();
        let segs = h.gilt_console(&c, &c.options());
        assert!(segs.is_empty());
    }

    // 10. Ragged rows handled without panic (padded to widest row).
    #[test]
    fn test_ragged_rows() {
        let h = Heatmap::new(vec![vec![1.0, 2.0, 3.0], vec![4.0]]);
        let c = console();
        let segs = h.gilt_console(&c, &c.options());
        // Both rows padded to 3 cols * 2 = 6 wide.
        assert_eq!(line_widths(&segs), vec![6, 6]);
    }

    // 11. gilt_console ends in a newline.
    #[test]
    fn test_ends_in_newline() {
        let h = Heatmap::new(vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
        let c = console();
        let segs = h.gilt_console(&c, &c.options());
        assert_eq!(segs.last().unwrap().text.as_str(), "\n");
    }

    // 12. gilt_measure matches rendered width.
    #[test]
    fn test_measure_matches_rendered_width() {
        let h = Heatmap::new(vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]).with_cell_width(3);
        let c = console();
        let opts = c.options();
        let m = h.measure(&c, &opts);
        let widths = line_widths(&h.gilt_console(&c, &opts));
        assert_eq!(m, Measurement::new(9, 9)); // 3 cols * 3
        assert!(widths.iter().all(|&w| w == 9));
    }

    // 13. &[Vec<f64>] constructor ergonomics.
    #[test]
    fn test_slice_constructor() {
        let slice: &[Vec<f64>] = &[vec![1.0, 2.0], vec![3.0, 4.0]];
        let h = Heatmap::new(slice);
        let c = console();
        let widths = line_widths(&h.gilt_console(&c, &c.options()));
        assert_eq!(widths, vec![4, 4]);
    }

    // 14. Cells carry a background colour (not a foreground).
    #[test]
    fn test_cells_use_background() {
        let h = Heatmap::new(vec![vec![0.0, 1.0]]);
        let c = console();
        let segs = h.gilt_console(&c, &c.options());
        let first = segs.iter().find(|s| s.text.as_str() != "\n").unwrap();
        let style = first.style.as_ref().unwrap();
        assert!(style.bgcolor().is_some(), "cell must set a bg colour");
        assert!(style.color().is_none(), "cell must not set a fg colour");
        // The visible glyphs are spaces.
        assert!(first.text.chars().all(|ch| ch == ' '));
    }

    // 15. Builder fields stored.
    #[test]
    fn test_builder_fields() {
        let h = Heatmap::new(vec![vec![1.0]])
            .with_min(-1.0)
            .with_max(1.0)
            .with_cell_width(5)
            .with_gradient(vec![Color::from_rgb(1, 2, 3), Color::from_rgb(4, 5, 6)]);
        assert_eq!(h.min_value, Some(-1.0));
        assert_eq!(h.max_value, Some(1.0));
        assert_eq!(h.cell_width, 5);
        assert_eq!(h.gradient.len(), 2);
    }

    // 16. Grid with only empty rows produces no output.
    #[test]
    fn test_all_empty_rows() {
        let h = Heatmap::new(vec![Vec::<f64>::new(), Vec::<f64>::new()]);
        let c = console();
        let segs = h.gilt_console(&c, &c.options());
        assert!(segs.is_empty());
    }
}
