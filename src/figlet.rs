//! Large ASCII art text using Unicode block characters.
//!
//! Renders banner text using a built-in 5-wide x 7-tall pixel font where each
//! set pixel is drawn as a full block character (`\u{2588}`).
//!
//! # Example
//!
//! ```rust
//! use gilt::figlet::Figlet;
//!
//! let banner = Figlet::new("HI");
//! let output = format!("{}", banner);
//! assert!(!output.is_empty());
//! ```

use std::fmt;

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::Style;

// ---------------------------------------------------------------------------
// Built-in 5x7 block font
// ---------------------------------------------------------------------------

/// Each glyph is 7 rows of `u8`. Within each `u8`, bits 4..0 represent
/// pixels left-to-right (bit 4 = leftmost column, bit 0 = rightmost).
/// A set bit renders as `\u{2588}` (full block); an unset bit renders as space.
const CHAR_WIDTH: usize = 5;
const CHAR_HEIGHT: usize = 7;

/// Full block character used for "on" pixels.
const BLOCK: char = '\u{2588}';

/// Gap between characters (in columns).
const CHAR_GAP: usize = 1;

/// Width of a space character (in columns).
const SPACE_WIDTH: usize = 3;

/// Return the 7-row bitmap for an ASCII character, or `None` if not defined.
fn glyph(ch: char) -> Option<[u8; CHAR_HEIGHT]> {
    match ch {
        // Uppercase letters
        'A' => Some([
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ]),
        'B' => Some([
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ]),
        'C' => Some([
            0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
        ]),
        'D' => Some([
            0b11100, 0b10010, 0b10001, 0b10001, 0b10001, 0b10010, 0b11100,
        ]),
        'E' => Some([
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ]),
        'F' => Some([
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ]),
        'G' => Some([
            0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110,
        ]),
        'H' => Some([
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ]),
        'I' => Some([
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
        ]),
        'J' => Some([
            0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100,
        ]),
        'K' => Some([
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ]),
        'L' => Some([
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ]),
        'M' => Some([
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ]),
        'N' => Some([
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ]),
        'O' => Some([
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ]),
        'P' => Some([
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ]),
        'Q' => Some([
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ]),
        'R' => Some([
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ]),
        'S' => Some([
            0b01110, 0b10001, 0b10000, 0b01110, 0b00001, 0b10001, 0b01110,
        ]),
        'T' => Some([
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ]),
        'U' => Some([
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ]),
        'V' => Some([
            0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b01010, 0b00100,
        ]),
        'W' => Some([
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001,
        ]),
        'X' => Some([
            0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
        ]),
        'Y' => Some([
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
        ]),
        'Z' => Some([
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
        ]),

        // Lowercase letters (same glyphs as uppercase for this block font)
        'a'..='z' => glyph(ch.to_ascii_uppercase()),

        // Digits
        '0' => Some([
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ]),
        '1' => Some([
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
        ]),
        '2' => Some([
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ]),
        '3' => Some([
            0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110,
        ]),
        '4' => Some([
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ]),
        '5' => Some([
            0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110,
        ]),
        '6' => Some([
            0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ]),
        '7' => Some([
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ]),
        '8' => Some([
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ]),
        '9' => Some([
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110,
        ]),

        // Common punctuation
        '!' => Some([
            0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100,
        ]),
        '?' => Some([
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b00000, 0b00100,
        ]),
        '.' => Some([
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00100,
        ]),
        ',' => Some([
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b01000,
        ]),
        ':' => Some([
            0b00000, 0b00000, 0b00100, 0b00000, 0b00000, 0b00100, 0b00000,
        ]),
        ';' => Some([
            0b00000, 0b00000, 0b00100, 0b00000, 0b00000, 0b00100, 0b01000,
        ]),
        '-' => Some([
            0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ]),
        '+' => Some([
            0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000,
        ]),
        '=' => Some([
            0b00000, 0b00000, 0b11111, 0b00000, 0b11111, 0b00000, 0b00000,
        ]),
        '/' => Some([
            0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000,
        ]),
        '(' => Some([
            0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010,
        ]),
        ')' => Some([
            0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000,
        ]),
        '[' => Some([
            0b01110, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01110,
        ]),
        ']' => Some([
            0b01110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b01110,
        ]),
        '#' => Some([
            0b01010, 0b01010, 0b11111, 0b01010, 0b11111, 0b01010, 0b01010,
        ]),
        '@' => Some([
            0b01110, 0b10001, 0b10111, 0b10101, 0b10110, 0b10000, 0b01110,
        ]),
        '*' => Some([
            0b00000, 0b10101, 0b01110, 0b11111, 0b01110, 0b10101, 0b00000,
        ]),
        '_' => Some([
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111,
        ]),
        '\'' => Some([
            0b00100, 0b00100, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
        ]),
        '"' => Some([
            0b01010, 0b01010, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
        ]),
        '<' => Some([
            0b00010, 0b00100, 0b01000, 0b10000, 0b01000, 0b00100, 0b00010,
        ]),
        '>' => Some([
            0b01000, 0b00100, 0b00010, 0b00001, 0b00010, 0b00100, 0b01000,
        ]),
        '&' => Some([
            0b01100, 0b10010, 0b10100, 0b01000, 0b10101, 0b10010, 0b01101,
        ]),
        '%' => Some([
            0b11001, 0b11010, 0b00010, 0b00100, 0b01000, 0b01011, 0b10011,
        ]),
        '$' => Some([
            0b00100, 0b01111, 0b10100, 0b01110, 0b00101, 0b11110, 0b00100,
        ]),
        '^' => Some([
            0b00100, 0b01010, 0b10001, 0b00000, 0b00000, 0b00000, 0b00000,
        ]),
        '~' => Some([
            0b00000, 0b00000, 0b01000, 0b10101, 0b00010, 0b00000, 0b00000,
        ]),
        '`' => Some([
            0b01000, 0b00100, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
        ]),
        '{' => Some([
            0b00110, 0b00100, 0b00100, 0b01000, 0b00100, 0b00100, 0b00110,
        ]),
        '}' => Some([
            0b01100, 0b00100, 0b00100, 0b00010, 0b00100, 0b00100, 0b01100,
        ]),
        '|' => Some([
            0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ]),
        '\\' => Some([
            0b10000, 0b01000, 0b01000, 0b00100, 0b00010, 0b00010, 0b00001,
        ]),

        _ => None,
    }
}

/// Compute the rendered width for a sequence of characters.
///
/// Each printable character takes `CHAR_WIDTH` columns, with `CHAR_GAP`
/// columns between adjacent characters. Spaces take `SPACE_WIDTH` columns.
fn rendered_width(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    let mut width = 0;
    let mut first = true;
    for ch in text.chars() {
        if !first {
            width += CHAR_GAP;
        }
        first = false;
        if ch == ' ' {
            width += SPACE_WIDTH;
        } else {
            width += CHAR_WIDTH;
        }
    }
    width
}

// ---------------------------------------------------------------------------
// Figlet struct
// ---------------------------------------------------------------------------

/// Large ASCII art banner text rendered using Unicode full-block characters.
///
/// Each character is drawn on a 5-wide by 7-tall pixel grid. Set pixels are
/// rendered as `\u{2588}` (full block) and unset pixels as spaces. Characters
/// are separated by a 1-column gap.
///
/// # Example
///
/// ```rust
/// use gilt::figlet::Figlet;
///
/// let banner = Figlet::new("OK");
/// println!("{}", banner);
/// ```
#[derive(Debug, Clone)]
pub struct Figlet {
    /// The text to render.
    text: String,
    /// Style applied to the block characters.
    style: Style,
    /// Optional maximum width; if the rendered text exceeds this, it wraps
    /// to the next "line" of banner rows.
    width: Option<usize>,
}

impl Figlet {
    /// Create a new `Figlet` with default (null) style and no width constraint.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            style: Style::null(),
            width: None,
        }
    }

    /// Set the style applied to block characters.
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set a maximum width constraint. If the rendered banner exceeds this
    /// width, characters wrap to subsequent banner rows.
    #[must_use]
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Render the text into lines of block characters.
    ///
    /// Returns a `Vec<String>` where each element is one row of the output.
    /// If a width constraint is set, the text is split into chunks that fit.
    fn render_lines(&self) -> Vec<String> {
        if self.text.is_empty() {
            return Vec::new();
        }

        // Split text into "visual lines" that respect the width constraint
        let chunks = self.split_into_chunks();

        let mut all_rows: Vec<String> = Vec::new();

        for chunk in &chunks {
            let mut rows = vec![String::new(); CHAR_HEIGHT];

            let mut first_in_chunk = true;
            for &ch in chunk {
                if ch == ' ' {
                    // Space: add SPACE_WIDTH spaces + gap
                    let gap = if first_in_chunk { 0 } else { CHAR_GAP };
                    for row in rows.iter_mut() {
                        row.push_str(&" ".repeat(gap + SPACE_WIDTH));
                    }
                } else if let Some(bitmap) = glyph(ch) {
                    let gap = if first_in_chunk { 0 } else { CHAR_GAP };
                    for (r, &bits) in bitmap.iter().enumerate() {
                        if gap > 0 {
                            rows[r].push_str(&" ".repeat(gap));
                        }
                        for col in (0..CHAR_WIDTH).rev() {
                            if bits & (1 << col) != 0 {
                                rows[r].push(BLOCK);
                            } else {
                                rows[r].push(' ');
                            }
                        }
                    }
                } else {
                    // Unknown character: blank space of CHAR_WIDTH
                    let gap = if first_in_chunk { 0 } else { CHAR_GAP };
                    for row in rows.iter_mut() {
                        row.push_str(&" ".repeat(gap + CHAR_WIDTH));
                    }
                }
                first_in_chunk = false;
            }

            all_rows.extend(rows);
        }

        all_rows
    }

    /// Split the input text into chunks that fit within the width constraint.
    fn split_into_chunks(&self) -> Vec<Vec<char>> {
        let chars: Vec<char> = self.text.chars().collect();
        let max_width = match self.width {
            Some(w) => w,
            None => return vec![chars],
        };

        if chars.is_empty() {
            return vec![];
        }

        let mut chunks: Vec<Vec<char>> = Vec::new();
        let mut current: Vec<char> = Vec::new();
        let mut current_width: usize = 0;

        for &ch in &chars {
            let ch_width = if ch == ' ' { SPACE_WIDTH } else { CHAR_WIDTH };
            let needed = if current.is_empty() {
                ch_width
            } else {
                CHAR_GAP + ch_width
            };

            if !current.is_empty() && current_width + needed > max_width {
                chunks.push(std::mem::take(&mut current));
                current_width = 0;
            }

            if current.is_empty() {
                current_width = ch_width;
            } else {
                current_width += CHAR_GAP + ch_width;
            }
            current.push(ch);
        }

        if !current.is_empty() {
            chunks.push(current);
        }

        chunks
    }

    /// Return the measurement (width) of this figlet text.
    pub fn measure(&self, _console: &Console, options: &ConsoleOptions) -> Measurement {
        let natural = rendered_width(&self.text);
        let max = match self.width {
            Some(w) => w.min(options.max_width),
            None => natural.min(options.max_width),
        };
        // Minimum is one character width
        let min = if self.text.is_empty() { 0 } else { CHAR_WIDTH };
        Measurement::new(min.min(max), max)
    }
}

// ---------------------------------------------------------------------------
// Renderable
// ---------------------------------------------------------------------------

impl Renderable for Figlet {
    fn rich_console(&self, _console: &Console, _options: &ConsoleOptions) -> Vec<Segment> {
        let lines = self.render_lines();
        let mut segments = Vec::new();
        for line in &lines {
            if self.style.is_null() {
                segments.push(Segment::text(line));
            } else {
                segments.push(Segment::styled(line, self.style.clone()));
            }
            segments.push(Segment::line());
        }
        segments
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl fmt::Display for Figlet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lines = self.render_lines();
        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{}", line)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_console(width: usize) -> Console {
        Console::builder()
            .width(width)
            .force_terminal(true)
            .no_color(true)
            .markup(false)
            .build()
    }

    // -- Single character rendering -----------------------------------------

    #[test]
    fn test_single_character() {
        let f = Figlet::new("A");
        let lines = f.render_lines();
        assert_eq!(lines.len(), CHAR_HEIGHT);
        // Each row should be exactly CHAR_WIDTH characters
        for line in &lines {
            assert_eq!(line.chars().count(), CHAR_WIDTH);
        }
    }

    #[test]
    fn test_single_character_pixels() {
        let f = Figlet::new("I");
        let lines = f.render_lines();
        // 'I' top row is 0b11111 => all blocks
        assert_eq!(lines.len(), 7);
        let top = &lines[0];
        let block_count = top.chars().filter(|&c| c == BLOCK).count();
        assert_eq!(block_count, 5);
    }

    // -- Multiple characters ------------------------------------------------

    #[test]
    fn test_multiple_characters() {
        let f = Figlet::new("AB");
        let lines = f.render_lines();
        assert_eq!(lines.len(), CHAR_HEIGHT);
        // Width: 5 + 1 (gap) + 5 = 11
        for line in &lines {
            assert_eq!(line.chars().count(), 11);
        }
    }

    // -- Full alphabet ------------------------------------------------------

    #[test]
    fn test_full_uppercase_alphabet() {
        for ch in 'A'..='Z' {
            assert!(glyph(ch).is_some(), "Missing glyph for '{}'", ch);
        }
    }

    #[test]
    fn test_lowercase_maps_to_uppercase() {
        for ch in 'a'..='z' {
            let lower = glyph(ch);
            let upper = glyph(ch.to_ascii_uppercase());
            assert_eq!(lower, upper);
        }
    }

    // -- Numbers ------------------------------------------------------------

    #[test]
    fn test_digits() {
        for ch in '0'..='9' {
            assert!(glyph(ch).is_some(), "Missing glyph for '{}'", ch);
        }
        let f = Figlet::new("0123456789");
        let lines = f.render_lines();
        assert_eq!(lines.len(), CHAR_HEIGHT);
    }

    // -- Unknown character fallback -----------------------------------------

    #[test]
    fn test_unknown_character_fallback() {
        // Unicode chars not in the font should render as blank
        let f = Figlet::new("\u{1F600}"); // emoji
        let lines = f.render_lines();
        assert_eq!(lines.len(), CHAR_HEIGHT);
        for line in &lines {
            // Should be blank (all spaces)
            assert!(line.chars().all(|c| c == ' '));
            assert_eq!(line.chars().count(), CHAR_WIDTH);
        }
    }

    // -- Empty string -------------------------------------------------------

    #[test]
    fn test_empty_string() {
        let f = Figlet::new("");
        let lines = f.render_lines();
        assert!(lines.is_empty());
    }

    // -- Space handling -----------------------------------------------------

    #[test]
    fn test_space_handling() {
        let f = Figlet::new("A B");
        let lines = f.render_lines();
        assert_eq!(lines.len(), CHAR_HEIGHT);
        // Width: 5 + 1(gap) + 3(space) + 1(gap) + 5 = 15
        for line in &lines {
            assert_eq!(line.chars().count(), 15);
        }
    }

    #[test]
    fn test_space_is_blank() {
        let f = Figlet::new(" ");
        let lines = f.render_lines();
        assert_eq!(lines.len(), CHAR_HEIGHT);
        for line in &lines {
            assert!(line.chars().all(|c| c == ' '));
            assert_eq!(line.chars().count(), SPACE_WIDTH);
        }
    }

    // -- Style application --------------------------------------------------

    #[test]
    fn test_style_application() {
        let style = Style::parse("bold").unwrap();
        let f = Figlet::new("A").with_style(style);
        let console = make_console(80);
        let opts = console.options();
        let segments = f.rich_console(&console, &opts);
        // Non-newline segments should carry the style
        let styled_segs: Vec<_> = segments.iter().filter(|s| s.text != "\n").collect();
        assert!(!styled_segs.is_empty());
        for seg in styled_segs {
            assert!(seg.style.is_some());
        }
    }

    // -- Display trait ------------------------------------------------------

    #[test]
    fn test_display_trait() {
        let f = Figlet::new("X");
        let s = format!("{}", f);
        assert!(!s.is_empty());
        // Should have 7 lines
        let line_count = s.lines().count();
        assert_eq!(line_count, CHAR_HEIGHT);
    }

    #[test]
    fn test_display_empty() {
        let f = Figlet::new("");
        let s = format!("{}", f);
        assert!(s.is_empty());
    }

    // -- Measure correctness ------------------------------------------------

    #[test]
    fn test_measure_single_char() {
        let f = Figlet::new("A");
        let console = make_console(80);
        let opts = console.options();
        let m = f.measure(&console, &opts);
        assert_eq!(m.maximum, CHAR_WIDTH);
    }

    #[test]
    fn test_measure_multiple_chars() {
        let f = Figlet::new("AB");
        let console = make_console(80);
        let opts = console.options();
        let m = f.measure(&console, &opts);
        // 5 + 1 + 5 = 11
        assert_eq!(m.maximum, 11);
    }

    #[test]
    fn test_measure_empty() {
        let f = Figlet::new("");
        let console = make_console(80);
        let opts = console.options();
        let m = f.measure(&console, &opts);
        assert_eq!(m.minimum, 0);
        assert_eq!(m.maximum, 0);
    }

    // -- Width constraint ---------------------------------------------------

    #[test]
    fn test_width_constraint() {
        // "ABCD" = 5+1+5+1+5+1+5 = 23 wide
        // With width 12, should split into two chunks
        let f = Figlet::new("ABCD").with_width(12);
        let lines = f.render_lines();
        // Two chunks of 7 rows each = 14 rows
        assert_eq!(lines.len(), CHAR_HEIGHT * 2);
    }

    #[test]
    fn test_width_constraint_no_split() {
        let f = Figlet::new("AB").with_width(80);
        let lines = f.render_lines();
        assert_eq!(lines.len(), CHAR_HEIGHT);
    }

    // -- Renderable trait ---------------------------------------------------

    #[test]
    fn test_renderable_produces_segments() {
        let f = Figlet::new("A");
        let console = make_console(80);
        let opts = console.options();
        let segments = f.rich_console(&console, &opts);
        // Should have 7 content segments + 7 newline segments = 14
        assert_eq!(segments.len(), CHAR_HEIGHT * 2);
    }

    // -- Punctuation --------------------------------------------------------

    #[test]
    fn test_punctuation() {
        let puncts = "!?.,-:;+=/()[]#@*_'\"<>&%$^~`{}|\\";
        for ch in puncts.chars() {
            assert!(glyph(ch).is_some(), "Missing glyph for '{}'", ch);
        }
    }

    // -- rendered_width helper ----------------------------------------------

    #[test]
    fn test_rendered_width() {
        assert_eq!(rendered_width(""), 0);
        assert_eq!(rendered_width("A"), 5);
        assert_eq!(rendered_width("AB"), 11);
        assert_eq!(rendered_width("A B"), 15); // 5+1+3+1+5
        assert_eq!(rendered_width(" "), 3);
    }
}
