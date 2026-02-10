//! Canvas -- Braille dot matrix for high-resolution terminal graphics.
//!
//! Uses Unicode Braille patterns (U+2800..U+28FF) to create a pixel canvas
//! where each terminal character cell contains a 2x4 grid of dots, giving
//! 2x horizontal and 4x vertical sub-character resolution.
//!
//! # Example
//!
//! ```
//! use gilt::canvas::Canvas;
//!
//! let mut c = Canvas::new(4, 2); // 4 cols x 2 rows => 8x8 pixel grid
//! c.set(0, 0);
//! c.set(7, 7);
//! assert!(c.get(0, 0));
//! assert!(c.get(7, 7));
//! assert!(!c.get(1, 1));
//! ```

use std::fmt;

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::Style;

// ---------------------------------------------------------------------------
// Braille pixel mapping
// ---------------------------------------------------------------------------

/// Braille dot-offset lookup.
///
/// Each braille character (U+2800 + bits) encodes a 2x4 dot matrix.
/// `PIXEL_MAP[row][col]` gives the bit that must be set for a pixel at
/// the given row (0..4) and column (0..2) within the character cell.
const PIXEL_MAP: [[u8; 2]; 4] = [
    [0x01, 0x08], // row 0
    [0x02, 0x10], // row 1
    [0x04, 0x20], // row 2
    [0x40, 0x80], // row 3
];

/// The Unicode code point for the empty braille pattern (no dots).
const BRAILLE_BASE: u32 = 0x2800;

// ---------------------------------------------------------------------------
// Canvas
// ---------------------------------------------------------------------------

/// A Braille dot-matrix canvas for terminal graphics.
///
/// The canvas dimensions are specified in terminal columns and rows.  The
/// actual *pixel* resolution is `width * 2` horizontally and `height * 4`
/// vertically, because each braille character encodes a 2x4 dot grid.
#[derive(Debug, Clone)]
pub struct Canvas {
    /// Width in terminal columns.
    width: usize,
    /// Height in terminal rows.
    height: usize,
    /// Dot bits for each character cell, stored row-major: `pixels[row][col]`.
    pixels: Vec<Vec<u8>>,
    /// Visual style applied to the rendered braille text.
    style: Style,
}

impl Canvas {
    /// Create a new empty canvas of the given dimensions (in terminal cells).
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![vec![0u8; width]; height],
            style: Style::null(),
        }
    }

    /// Set the visual style (builder pattern).
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Pixel width (horizontal resolution = terminal columns * 2).
    pub fn pixel_width(&self) -> usize {
        self.width * 2
    }

    /// Pixel height (vertical resolution = terminal rows * 4).
    pub fn pixel_height(&self) -> usize {
        self.height * 4
    }

    // -- pixel operations ---------------------------------------------------

    /// Set a pixel at `(x, y)` in pixel coordinates.
    ///
    /// Out-of-bounds coordinates are silently ignored.
    pub fn set(&mut self, x: usize, y: usize) {
        if x >= self.pixel_width() || y >= self.pixel_height() {
            return;
        }
        let col = x / 2;
        let row = y / 4;
        let bit = PIXEL_MAP[y % 4][x % 2];
        self.pixels[row][col] |= bit;
    }

    /// Clear a pixel at `(x, y)` in pixel coordinates.
    ///
    /// Out-of-bounds coordinates are silently ignored.
    pub fn unset(&mut self, x: usize, y: usize) {
        if x >= self.pixel_width() || y >= self.pixel_height() {
            return;
        }
        let col = x / 2;
        let row = y / 4;
        let bit = PIXEL_MAP[y % 4][x % 2];
        self.pixels[row][col] &= !bit;
    }

    /// Toggle a pixel at `(x, y)` in pixel coordinates.
    ///
    /// Out-of-bounds coordinates are silently ignored.
    pub fn toggle(&mut self, x: usize, y: usize) {
        if x >= self.pixel_width() || y >= self.pixel_height() {
            return;
        }
        let col = x / 2;
        let row = y / 4;
        let bit = PIXEL_MAP[y % 4][x % 2];
        self.pixels[row][col] ^= bit;
    }

    /// Test whether the pixel at `(x, y)` is set.
    ///
    /// Out-of-bounds coordinates return `false`.
    pub fn get(&self, x: usize, y: usize) -> bool {
        if x >= self.pixel_width() || y >= self.pixel_height() {
            return false;
        }
        let col = x / 2;
        let row = y / 4;
        let bit = PIXEL_MAP[y % 4][x % 2];
        self.pixels[row][col] & bit != 0
    }

    // -- shape helpers ------------------------------------------------------

    /// Draw a line from `(x0, y0)` to `(x1, y1)` using Bresenham's algorithm.
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        let mut x0 = x0;
        let mut y0 = y0;
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx: i32 = if x0 < x1 { 1 } else { -1 };
        let sy: i32 = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            if x0 >= 0 && y0 >= 0 {
                self.set(x0 as usize, y0 as usize);
            }
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                if x0 == x1 {
                    break;
                }
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                if y0 == y1 {
                    break;
                }
                err += dx;
                y0 += sy;
            }
        }
    }

    /// Draw a rectangle outline in pixel coordinates.
    pub fn rect(&mut self, x: usize, y: usize, w: usize, h: usize) {
        if w == 0 || h == 0 {
            return;
        }
        let x1 = x + w - 1;
        let y1 = y + h - 1;
        self.line(x as i32, y as i32, x1 as i32, y as i32); // top
        self.line(x as i32, y1 as i32, x1 as i32, y1 as i32); // bottom
        self.line(x as i32, y as i32, x as i32, y1 as i32); // left
        self.line(x1 as i32, y as i32, x1 as i32, y1 as i32); // right
    }

    /// Draw a filled rectangle in pixel coordinates.
    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize) {
        for dy in 0..h {
            for dx in 0..w {
                self.set(x + dx, y + dy);
            }
        }
    }

    /// Draw a circle outline using the midpoint circle algorithm.
    pub fn circle(&mut self, cx: i32, cy: i32, r: i32) {
        if r < 0 {
            return;
        }
        let mut x = r;
        let mut y: i32 = 0;
        let mut err = 1 - r;

        while x >= y {
            // Set pixels in all eight octants.
            self.set_signed(cx + x, cy + y);
            self.set_signed(cx - x, cy + y);
            self.set_signed(cx + x, cy - y);
            self.set_signed(cx - x, cy - y);
            self.set_signed(cx + y, cy + x);
            self.set_signed(cx - y, cy + x);
            self.set_signed(cx + y, cy - x);
            self.set_signed(cx - y, cy - x);

            y += 1;
            if err <= 0 {
                err += 2 * y + 1;
            } else {
                x -= 1;
                err += 2 * (y - x) + 1;
            }
        }
    }

    /// Helper: set a pixel from signed coordinates, ignoring negative values.
    fn set_signed(&mut self, x: i32, y: i32) {
        if x >= 0 && y >= 0 {
            self.set(x as usize, y as usize);
        }
    }

    // -- rendering ----------------------------------------------------------

    /// Render the canvas to a multi-line string of braille characters.
    pub fn frame(&self) -> String {
        let mut lines: Vec<String> = Vec::with_capacity(self.height);
        for row in &self.pixels {
            let line: String = row
                .iter()
                .map(|&bits| {
                    // Safety: BRAILLE_BASE + bits is always a valid Unicode
                    // code point in U+2800..U+28FF.
                    char::from_u32(BRAILLE_BASE + bits as u32).unwrap_or(' ')
                })
                .collect();
            lines.push(line);
        }
        lines.join("\n")
    }

    /// Clear all pixels.
    pub fn clear(&mut self) {
        for row in &mut self.pixels {
            for cell in row.iter_mut() {
                *cell = 0;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl fmt::Display for Canvas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.frame())
    }
}

// ---------------------------------------------------------------------------
// Renderable
// ---------------------------------------------------------------------------

impl Renderable for Canvas {
    fn gilt_console(&self, _console: &Console, _options: &ConsoleOptions) -> Vec<Segment> {
        let mut segments = Vec::new();
        for (i, row) in self.pixels.iter().enumerate() {
            let line: String = row
                .iter()
                .map(|&bits| char::from_u32(BRAILLE_BASE + bits as u32).unwrap_or(' '))
                .collect();
            segments.push(Segment::new(&line, Some(self.style.clone()), None));
            if i < self.height - 1 {
                segments.push(Segment::line());
            }
        }
        segments.push(Segment::line());
        segments
    }
}

// ---------------------------------------------------------------------------
// Measure
// ---------------------------------------------------------------------------

impl Canvas {
    /// Return the measurement for this canvas.
    pub fn measure(&self, _console: &Console, _options: &ConsoleOptions) -> Measurement {
        Measurement::new(self.width, self.width)
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

    // 1. Empty canvas
    #[test]
    fn test_empty_canvas() {
        let c = Canvas::new(3, 2);
        let frame = c.frame();
        // Each row is 3 empty braille chars, separated by newline
        let empty_braille = '\u{2800}';
        let expected = format!(
            "{}{}{}",
            std::iter::repeat(empty_braille).take(3).collect::<String>(),
            "\n",
            std::iter::repeat(empty_braille).take(3).collect::<String>(),
        );
        assert_eq!(frame, expected);
    }

    // 2. Single pixel set
    #[test]
    fn test_single_pixel_set() {
        let mut c = Canvas::new(1, 1);
        c.set(0, 0);
        assert!(c.get(0, 0));
        // Dot 1 (0x01)
        let expected_char = char::from_u32(BRAILLE_BASE + 0x01).unwrap();
        assert_eq!(c.frame(), expected_char.to_string());
    }

    // 3. Pixel unset
    #[test]
    fn test_pixel_unset() {
        let mut c = Canvas::new(1, 1);
        c.set(0, 0);
        assert!(c.get(0, 0));
        c.unset(0, 0);
        assert!(!c.get(0, 0));
    }

    // 4. Pixel toggle
    #[test]
    fn test_pixel_toggle() {
        let mut c = Canvas::new(1, 1);
        assert!(!c.get(0, 0));
        c.toggle(0, 0);
        assert!(c.get(0, 0));
        c.toggle(0, 0);
        assert!(!c.get(0, 0));
    }

    // 5. Pixel get
    #[test]
    fn test_pixel_get_unset() {
        let c = Canvas::new(2, 2);
        assert!(!c.get(0, 0));
        assert!(!c.get(3, 7));
    }

    // 6. Pixel to braille mapping correctness
    #[test]
    fn test_braille_mapping() {
        // Test each of the 8 dot positions in a single cell
        let dots: [(usize, usize, u8); 8] = [
            (0, 0, 0x01), // dot 1
            (0, 1, 0x02), // dot 2
            (0, 2, 0x04), // dot 3
            (0, 3, 0x40), // dot 7
            (1, 0, 0x08), // dot 4
            (1, 1, 0x10), // dot 5
            (1, 2, 0x20), // dot 6
            (1, 3, 0x80), // dot 8
        ];
        for (px, py, expected_bit) in dots {
            let mut c = Canvas::new(1, 1);
            c.set(px, py);
            assert_eq!(
                c.pixels[0][0], expected_bit,
                "pixel ({px},{py}) should set bit 0x{expected_bit:02x}"
            );
        }
    }

    // 7. Horizontal line
    #[test]
    fn test_line_horizontal() {
        let mut c = Canvas::new(5, 1);
        c.line(0, 0, 9, 0); // full width horizontal
        for x in 0..10 {
            assert!(c.get(x, 0), "pixel ({x}, 0) should be set");
        }
    }

    // 8. Vertical line
    #[test]
    fn test_line_vertical() {
        let mut c = Canvas::new(1, 2);
        c.line(0, 0, 0, 7); // full height vertical
        for y in 0..8 {
            assert!(c.get(0, y), "pixel (0, {y}) should be set");
        }
    }

    // 9. Diagonal line
    #[test]
    fn test_line_diagonal() {
        let mut c = Canvas::new(4, 2);
        c.line(0, 0, 7, 7);
        assert!(c.get(0, 0));
        assert!(c.get(7, 7));
    }

    // 10. Rectangle outline
    #[test]
    fn test_rect_outline() {
        let mut c = Canvas::new(4, 2);
        c.rect(0, 0, 8, 8);
        // Corners should be set
        assert!(c.get(0, 0));
        assert!(c.get(7, 0));
        assert!(c.get(0, 7));
        assert!(c.get(7, 7));
        // Interior should be empty
        assert!(!c.get(3, 3));
    }

    // 11. Filled rectangle
    #[test]
    fn test_fill_rect() {
        let mut c = Canvas::new(2, 1);
        c.fill_rect(0, 0, 4, 4);
        for y in 0..4 {
            for x in 0..4 {
                assert!(c.get(x, y), "pixel ({x},{y}) should be set");
            }
        }
    }

    // 12. Circle
    #[test]
    fn test_circle() {
        let mut c = Canvas::new(10, 5);
        c.circle(10, 10, 8);
        // Some pixels on the circle perimeter should be set
        // At angle 0: (10+8, 10) = (18, 10)
        assert!(c.get(18, 10));
        // At angle 90: (10, 10+8) = (10, 18)
        assert!(c.get(10, 18));
    }

    // 13. Out-of-bounds handling (no panic)
    #[test]
    fn test_out_of_bounds() {
        let mut c = Canvas::new(2, 2);
        // These should not panic
        c.set(100, 100);
        c.unset(100, 100);
        c.toggle(100, 100);
        assert!(!c.get(100, 100));
    }

    // 14. Clear
    #[test]
    fn test_clear() {
        let mut c = Canvas::new(3, 2);
        c.fill_rect(0, 0, 6, 8);
        c.clear();
        for row in &c.pixels {
            for &cell in row {
                assert_eq!(cell, 0);
            }
        }
    }

    // 15. Frame output correctness
    #[test]
    fn test_frame_multiline() {
        let c = Canvas::new(2, 3);
        let frame = c.frame();
        let lines: Vec<&str> = frame.split('\n').collect();
        assert_eq!(lines.len(), 3);
        for line in lines {
            assert_eq!(line.chars().count(), 2);
        }
    }

    // 16. Display trait
    #[test]
    fn test_display_trait() {
        let c = Canvas::new(2, 2);
        let displayed = format!("{c}");
        assert_eq!(displayed, c.frame());
    }

    // 17. Renderable output
    #[test]
    fn test_renderable() {
        let c = Canvas::new(3, 2);
        let console = Console::builder().width(80).build();
        let opts = make_options(80);
        let segments = c.gilt_console(&console, &opts);
        // Should have: row0, newline, row1, newline
        assert!(!segments.is_empty());
        assert_eq!(segments.last().unwrap().text.as_str(), "\n");
    }

    // 18. Renderable with style
    #[test]
    fn test_renderable_style() {
        let style = Style::parse("bold green").unwrap();
        let c = Canvas::new(2, 1).with_style(style.clone());
        let console = Console::builder().width(80).build();
        let opts = make_options(80);
        let segments = c.gilt_console(&console, &opts);
        assert_eq!(segments[0].style.as_ref(), Some(&style));
    }

    // 19. Measure
    #[test]
    fn test_measure() {
        let c = Canvas::new(20, 10);
        let console = Console::builder().width(80).build();
        let opts = make_options(80);
        let m = c.measure(&console, &opts);
        assert_eq!(m, Measurement::new(20, 20));
    }

    // 20. Pixel width and height
    #[test]
    fn test_pixel_dimensions() {
        let c = Canvas::new(10, 5);
        assert_eq!(c.pixel_width(), 20);
        assert_eq!(c.pixel_height(), 20);
    }

    // 21. Multiple pixels in same cell
    #[test]
    fn test_multiple_pixels_same_cell() {
        let mut c = Canvas::new(1, 1);
        c.set(0, 0); // bit 0x01
        c.set(1, 0); // bit 0x08
        assert_eq!(c.pixels[0][0], 0x01 | 0x08);
        assert!(c.get(0, 0));
        assert!(c.get(1, 0));
    }

    // 22. Circle with radius 0
    #[test]
    fn test_circle_radius_zero() {
        let mut c = Canvas::new(2, 2);
        c.circle(2, 2, 0);
        assert!(c.get(2, 2));
    }

    // 23. Circle with negative radius
    #[test]
    fn test_circle_negative_radius() {
        let mut c = Canvas::new(2, 2);
        c.circle(2, 2, -5);
        // Should not set any pixels
        for y in 0..c.pixel_height() {
            for x in 0..c.pixel_width() {
                assert!(!c.get(x, y));
            }
        }
    }

    // 24. Rect with zero dimensions
    #[test]
    fn test_rect_zero() {
        let mut c = Canvas::new(2, 2);
        c.rect(0, 0, 0, 0);
        // No pixels set
        for y in 0..c.pixel_height() {
            for x in 0..c.pixel_width() {
                assert!(!c.get(x, y));
            }
        }
    }

    // 25. Full braille char (all 8 dots set)
    #[test]
    fn test_full_braille_char() {
        let mut c = Canvas::new(1, 1);
        // Set all 8 positions
        for y in 0..4 {
            for x in 0..2 {
                c.set(x, y);
            }
        }
        assert_eq!(c.pixels[0][0], 0xFF);
        let ch = char::from_u32(BRAILLE_BASE + 0xFF).unwrap();
        assert_eq!(c.frame(), ch.to_string());
    }
}
