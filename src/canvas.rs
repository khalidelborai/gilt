//! Canvas -- dot-matrix terminal graphics with multiple blitter modes.
//!
//! By default (`Blitter::Braille`) the canvas uses Unicode Braille patterns
//! (U+2800..U+28FF): each terminal character cell encodes a 2×4 pixel grid.
//!
//! Alternative blitters offer different trade-offs between resolution and
//! font support:
//!
//! | Blitter    | Cell (w×h) | Unicode block         |
//! |------------|------------|-----------------------|
//! | `Braille`  | 2 × 4      | U+2800 (Braille)      |
//! | `Sextant`  | 2 × 3      | U+1FB00 (Legacy comp.)   |
//! | `HalfBlock`| 1 × 2      | U+2580..U+2588 (Blocks)  |
//! | `Octant`   | 2 × 4      | U+1CD00 (Octant 16)[^1]  |
//!
//! [^1]: Octant uses Unicode 16.0 "Symbols for Legacy Computing Supplement"
//!       (U+1CD00..U+1CDE5) plus 26 reused legacy glyphs. The mapping is
//!       complete and authoritative; rendering requires a Unicode-16 font.
//!
//! # Example
//!
//! ```
//! use gilt::canvas::{Canvas, Blitter};
//!
//! // Default Braille canvas
//! let mut c = Canvas::new(4, 2); // 4 cols x 2 rows => 8x8 pixel grid
//! c.set(0, 0);
//! c.set(7, 7);
//! assert!(c.get(0, 0));
//! assert!(c.get(7, 7));
//! assert!(!c.get(1, 1));
//!
//! // Sextant canvas (2x3 sub-cell resolution)
//! let mut s = Canvas::new(4, 2).with_blitter(Blitter::Sextant);
//! s.set(0, 0);
//! s.set(7, 5); // pixel_width=8, pixel_height=6 for 4x2 sextant
//! assert!(s.get(0, 0));
//! ```

use std::fmt;

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::Style;

// ---------------------------------------------------------------------------
// Blitter enum
// ---------------------------------------------------------------------------

/// Sub-cell pixel rendering mode for [`Canvas`].
///
/// Each variant controls how pixel bits are mapped to Unicode glyphs in the
/// final rendered output.  The drawing API (`set`, `line`, `rect`, …) is
/// identical across all blitters; only the cell glyph changes.
///
/// # Resolution
///
/// | Variant     | Pixels per cell (w×h) | Unicode block                        |
/// |-------------|------------------------|--------------------------------------|
/// | `Braille`   | 2 × 4                  | U+2800 Braille Patterns              |
/// | `Sextant`   | 2 × 3                  | U+1FB00 Legacy Computing Symbols     |
/// | `HalfBlock` | 1 × 2                  | U+2580..U+2588 Block Elements        |
/// | `Octant`    | 2 × 4                  | U+1CD00 Octant (Unicode 16)          |
///
/// The `Octant` variant offers the same 2×4 resolution as `Braille` but uses
/// solid sub-cell blocks (Unicode 16.0) instead of dots; it requires a
/// Unicode-16 font to render correctly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Blitter {
    /// 2×4 Braille patterns (default). Maximum resolution, universal support.
    #[default]
    Braille,
    /// 2×3 sextants (Unicode 13, U+1FB00). Higher vertical density than HalfBlock;
    /// requires a font with the Legacy Computing block.
    Sextant,
    /// 1×2 half-block characters (▀ ▄ █). Lowest resolution, best compatibility.
    HalfBlock,
    /// 2×4 octant characters (Unicode 16, U+1CD00 block).
    ///
    /// Same 2×4 sub-cell resolution as `Braille`, but renders solid blocks
    /// rather than dots. The full 256-pattern mapping is authoritative
    /// (verified against Unicode 16.0 `UnicodeData.txt`): it combines the new
    /// `U+1CD00..=U+1CDE5` glyphs with 26 reused legacy glyphs (SPACE, the
    /// half/quarter/quadrant blocks, FULL BLOCK). Requires a Unicode-16 font.
    Octant,
}

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
// Sextant pixel mapping
// ---------------------------------------------------------------------------
//
// Unicode 13 Legacy Computing Supplement, block U+1FB00..U+1FB3B.
//
// Each sextant cell is 2 columns × 3 rows = 6 bits:
//
//   col:   0     1
//   row 0: bit0  bit1   (top)
//   row 1: bit2  bit3   (middle)
//   row 2: bit4  bit5   (bottom)
//
// Bit pattern → codepoint:
//   pattern 0        → SPACE  (no glyph)
//   pattern 1..=62   → U+1FB00 + (pattern - 1)
//   pattern 63       → U+2588  FULL BLOCK
//
// The bit to set for pixel at (col, row) within the 2×3 cell:
const SEXTANT_PIXEL_MAP: [[u8; 2]; 3] = [
    [0x01, 0x02], // row 0 (top)
    [0x04, 0x08], // row 1 (middle)
    [0x10, 0x20], // row 2 (bottom)
];

/// Render a 6-bit sextant pattern to its Unicode codepoint.
const SEXTANT_BASE: u32 = 0x1FB00;
const FULL_BLOCK: char = '\u{2588}';

#[inline]
fn sextant_bits_to_char(bits: u8) -> char {
    match bits {
        0 => ' ',
        63 => FULL_BLOCK,
        n => char::from_u32(SEXTANT_BASE + (n as u32) - 1).unwrap_or(' '),
    }
}

// ---------------------------------------------------------------------------
// HalfBlock pixel mapping
// ---------------------------------------------------------------------------
//
// Each half-block cell is 1 column × 2 rows = 2 bits:
//
//   row 0 (top):    bit 0
//   row 1 (bottom): bit 1
//
//   0b00 → SPACE     (U+0020)
//   0b01 → ▀ U+2580  UPPER HALF BLOCK
//   0b10 → ▄ U+2584  LOWER HALF BLOCK
//   0b11 → █ U+2588  FULL BLOCK

#[inline]
fn halfblock_bits_to_char(bits: u8) -> char {
    match bits & 0x03 {
        0b00 => ' ',
        0b01 => '\u{2580}', // ▀ UPPER HALF BLOCK
        0b10 => '\u{2584}', // ▄ LOWER HALF BLOCK
        _ => '\u{2588}',    // █ FULL BLOCK (0b11)
    }
}

// ---------------------------------------------------------------------------
// Octant pixel mapping (Unicode 16.0)
// ---------------------------------------------------------------------------
//
// Each octant cell is 2 columns × 4 rows = 8 bits.  Bit order is row-major:
//
//   bit = row * 2 + col
//
//   col:   0     1
//   row 0: bit0  bit1   (0x01  0x02)
//   row 1: bit2  bit3   (0x04  0x08)
//   row 2: bit4  bit5   (0x10  0x20)
//   row 3: bit6  bit7   (0x40  0x80)
//
// This matches Unicode's `BLOCK OCTANT-N` naming, where cells are numbered
// 1..8 reading the 2×4 grid left-to-right, top-to-bottom — so cell `c`
// corresponds to `bit (c - 1)`:
//
//   cell:  1 2 / 3 4 / 5 6 / 7 8   →   bit: 0 1 / 2 3 / 4 5 / 6 7
//
// The 256 patterns do NOT map to a single contiguous code-point run.  Unicode
// reuses 26 pre-existing glyphs (SPACE, the four half blocks, the sixteen 2×2
// quadrant glyphs, FULL BLOCK, the upper/lower one-quarter and three-quarter
// blocks, the four half×quarter corner blocks, and the two Unicode-16
// MIDDLE-LEFT/RIGHT one-quarter blocks U+1FBE6/U+1FBE7); the remaining 230 are
// encoded contiguously at U+1CD00..=U+1CDE5 in *name* order (which is not
// pattern order).  `OCTANT_CHARS` below is the verified, fully-covered table.
//
// Authoritatively derived from Unicode 16.0 `UnicodeData.txt`
// (https://www.unicode.org/Public/16.0.0/ucd/UnicodeData.txt): the 230
// `BLOCK OCTANT-N` names were parsed to (cell-set → code-point) pairs, and the
// 26 patterns absent from that block were mapped to their reused glyphs by
// matching the documented geometry of the corresponding Block-Elements /
// Symbols-for-Legacy-Computing names. Every pattern maps to a distinct glyph
// (asserted by `test_octant_table_all_distinct`).

/// Row-major bit lookup for the [`Blitter::Octant`] 2×4 cell.
///
/// `OCTANT_PIXEL_MAP[row][col]` is the bit set for the pixel at that position,
/// where `bit = row * 2 + col` (bit0 = top-left … bit7 = bottom-right).
const OCTANT_PIXEL_MAP: [[u8; 2]; 4] = [
    [0x01, 0x02], // row 0
    [0x04, 0x08], // row 1
    [0x10, 0x20], // row 2
    [0x40, 0x80], // row 3
];

/// Pattern (`0..=255`) → Unicode glyph for the octant blitter.
///
/// Index by the 8-bit cell value (`bit = row * 2 + col`).  Reused pre-existing
/// glyphs (SPACE, half/quarter/quadrant blocks, FULL BLOCK) sit alongside the
/// new U+1CD00 block characters; see the module-level note for the derivation.
#[rustfmt::skip]
const OCTANT_CHARS: [char; 256] = [
    '\u{0020}', '\u{1CEA8}', '\u{1CEAB}', '\u{1FB82}', '\u{1CD00}', '\u{2598}', '\u{1CD01}', '\u{1CD02}', // 0x00..=0x07
    '\u{1CD03}', '\u{1CD04}', '\u{259D}', '\u{1CD05}', '\u{1CD06}', '\u{1CD07}', '\u{1CD08}', '\u{2580}', // 0x08..=0x0F
    '\u{1CD09}', '\u{1CD0A}', '\u{1CD0B}', '\u{1CD0C}', '\u{1FBE6}', '\u{1CD0D}', '\u{1CD0E}', '\u{1CD0F}', // 0x10..=0x17
    '\u{1CD10}', '\u{1CD11}', '\u{1CD12}', '\u{1CD13}', '\u{1CD14}', '\u{1CD15}', '\u{1CD16}', '\u{1CD17}', // 0x18..=0x1F
    '\u{1CD18}', '\u{1CD19}', '\u{1CD1A}', '\u{1CD1B}', '\u{1CD1C}', '\u{1CD1D}', '\u{1CD1E}', '\u{1CD1F}', // 0x20..=0x27
    '\u{1FBE7}', '\u{1CD20}', '\u{1CD21}', '\u{1CD22}', '\u{1CD23}', '\u{1CD24}', '\u{1CD25}', '\u{1CD26}', // 0x28..=0x2F
    '\u{1CD27}', '\u{1CD28}', '\u{1CD29}', '\u{1CD2A}', '\u{1CD2B}', '\u{1CD2C}', '\u{1CD2D}', '\u{1CD2E}', // 0x30..=0x37
    '\u{1CD2F}', '\u{1CD30}', '\u{1CD31}', '\u{1CD32}', '\u{1CD33}', '\u{1CD34}', '\u{1CD35}', '\u{1FB85}', // 0x38..=0x3F
    '\u{1CEA3}', '\u{1CD36}', '\u{1CD37}', '\u{1CD38}', '\u{1CD39}', '\u{1CD3A}', '\u{1CD3B}', '\u{1CD3C}', // 0x40..=0x47
    '\u{1CD3D}', '\u{1CD3E}', '\u{1CD3F}', '\u{1CD40}', '\u{1CD41}', '\u{1CD42}', '\u{1CD43}', '\u{1CD44}', // 0x48..=0x4F
    '\u{2596}', '\u{1CD45}', '\u{1CD46}', '\u{1CD47}', '\u{1CD48}', '\u{258C}', '\u{1CD49}', '\u{1CD4A}', // 0x50..=0x57
    '\u{1CD4B}', '\u{1CD4C}', '\u{259E}', '\u{1CD4D}', '\u{1CD4E}', '\u{1CD4F}', '\u{1CD50}', '\u{259B}', // 0x58..=0x5F
    '\u{1CD51}', '\u{1CD52}', '\u{1CD53}', '\u{1CD54}', '\u{1CD55}', '\u{1CD56}', '\u{1CD57}', '\u{1CD58}', // 0x60..=0x67
    '\u{1CD59}', '\u{1CD5A}', '\u{1CD5B}', '\u{1CD5C}', '\u{1CD5D}', '\u{1CD5E}', '\u{1CD5F}', '\u{1CD60}', // 0x68..=0x6F
    '\u{1CD61}', '\u{1CD62}', '\u{1CD63}', '\u{1CD64}', '\u{1CD65}', '\u{1CD66}', '\u{1CD67}', '\u{1CD68}', // 0x70..=0x77
    '\u{1CD69}', '\u{1CD6A}', '\u{1CD6B}', '\u{1CD6C}', '\u{1CD6D}', '\u{1CD6E}', '\u{1CD6F}', '\u{1CD70}', // 0x78..=0x7F
    '\u{1CEA0}', '\u{1CD71}', '\u{1CD72}', '\u{1CD73}', '\u{1CD74}', '\u{1CD75}', '\u{1CD76}', '\u{1CD77}', // 0x80..=0x87
    '\u{1CD78}', '\u{1CD79}', '\u{1CD7A}', '\u{1CD7B}', '\u{1CD7C}', '\u{1CD7D}', '\u{1CD7E}', '\u{1CD7F}', // 0x88..=0x8F
    '\u{1CD80}', '\u{1CD81}', '\u{1CD82}', '\u{1CD83}', '\u{1CD84}', '\u{1CD85}', '\u{1CD86}', '\u{1CD87}', // 0x90..=0x97
    '\u{1CD88}', '\u{1CD89}', '\u{1CD8A}', '\u{1CD8B}', '\u{1CD8C}', '\u{1CD8D}', '\u{1CD8E}', '\u{1CD8F}', // 0x98..=0x9F
    '\u{2597}', '\u{1CD90}', '\u{1CD91}', '\u{1CD92}', '\u{1CD93}', '\u{259A}', '\u{1CD94}', '\u{1CD95}', // 0xA0..=0xA7
    '\u{1CD96}', '\u{1CD97}', '\u{2590}', '\u{1CD98}', '\u{1CD99}', '\u{1CD9A}', '\u{1CD9B}', '\u{259C}', // 0xA8..=0xAF
    '\u{1CD9C}', '\u{1CD9D}', '\u{1CD9E}', '\u{1CD9F}', '\u{1CDA0}', '\u{1CDA1}', '\u{1CDA2}', '\u{1CDA3}', // 0xB0..=0xB7
    '\u{1CDA4}', '\u{1CDA5}', '\u{1CDA6}', '\u{1CDA7}', '\u{1CDA8}', '\u{1CDA9}', '\u{1CDAA}', '\u{1CDAB}', // 0xB8..=0xBF
    '\u{2582}', '\u{1CDAC}', '\u{1CDAD}', '\u{1CDAE}', '\u{1CDAF}', '\u{1CDB0}', '\u{1CDB1}', '\u{1CDB2}', // 0xC0..=0xC7
    '\u{1CDB3}', '\u{1CDB4}', '\u{1CDB5}', '\u{1CDB6}', '\u{1CDB7}', '\u{1CDB8}', '\u{1CDB9}', '\u{1CDBA}', // 0xC8..=0xCF
    '\u{1CDBB}', '\u{1CDBC}', '\u{1CDBD}', '\u{1CDBE}', '\u{1CDBF}', '\u{1CDC0}', '\u{1CDC1}', '\u{1CDC2}', // 0xD0..=0xD7
    '\u{1CDC3}', '\u{1CDC4}', '\u{1CDC5}', '\u{1CDC6}', '\u{1CDC7}', '\u{1CDC8}', '\u{1CDC9}', '\u{1CDCA}', // 0xD8..=0xDF
    '\u{1CDCB}', '\u{1CDCC}', '\u{1CDCD}', '\u{1CDCE}', '\u{1CDCF}', '\u{1CDD0}', '\u{1CDD1}', '\u{1CDD2}', // 0xE0..=0xE7
    '\u{1CDD3}', '\u{1CDD4}', '\u{1CDD5}', '\u{1CDD6}', '\u{1CDD7}', '\u{1CDD8}', '\u{1CDD9}', '\u{1CDDA}', // 0xE8..=0xEF
    '\u{2584}', '\u{1CDDB}', '\u{1CDDC}', '\u{1CDDD}', '\u{1CDDE}', '\u{2599}', '\u{1CDDF}', '\u{1CDE0}', // 0xF0..=0xF7
    '\u{1CDE1}', '\u{1CDE2}', '\u{259F}', '\u{1CDE3}', '\u{2586}', '\u{1CDE4}', '\u{1CDE5}', '\u{2588}', // 0xF8..=0xFF
];

/// Render an 8-bit octant pattern to its Unicode glyph (see [`OCTANT_CHARS`]).
#[inline]
fn octant_bits_to_char(bits: u8) -> char {
    OCTANT_CHARS[bits as usize]
}

// ---------------------------------------------------------------------------
// Canvas
// ---------------------------------------------------------------------------

/// A dot-matrix canvas for terminal graphics.
///
/// The canvas dimensions are specified in terminal columns and rows.  The
/// actual *pixel* resolution depends on the active [`Blitter`]:
///
/// | Blitter    | pixel_width        | pixel_height        |
/// |------------|--------------------|--------------------|
/// | `Braille`  | `width * 2`        | `height * 4`        |
/// | `Sextant`  | `width * 2`        | `height * 3`        |
/// | `HalfBlock`| `width * 1`        | `height * 2`        |
/// | `Octant`   | `width * 2`        | `height * 4`        |
///
/// The default blitter is [`Blitter::Braille`], so existing code continues
/// to work without any changes.
#[derive(Debug, Clone)]
pub struct Canvas {
    /// Width in terminal columns.
    width: usize,
    /// Height in terminal rows.
    height: usize,
    /// Dot bits for each character cell, stored row-major: `pixels[row][col]`.
    pixels: Vec<Vec<u8>>,
    /// Visual style applied to the rendered text.
    style: Style,
    /// The blitter used to map pixel bits to Unicode glyphs.
    blitter: Blitter,
}

impl Canvas {
    /// Create a new empty canvas of the given dimensions (in terminal cells).
    ///
    /// Defaults to the [`Blitter::Braille`] blitter (unchanged from before v1.10).
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![vec![0u8; width]; height],
            style: Style::null(),
            blitter: Blitter::Braille,
        }
    }

    /// Set the blitter used to convert pixel bits to Unicode glyphs (builder pattern).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::canvas::{Canvas, Blitter};
    ///
    /// let mut c = Canvas::new(4, 2).with_blitter(Blitter::Sextant);
    /// c.set(0, 0);
    /// // Sextant: pixel_height = 2 * 3 = 6, pixel_width = 4 * 2 = 8
    /// assert!(c.get(0, 0));
    /// ```
    #[must_use]
    pub fn with_blitter(mut self, blitter: Blitter) -> Self {
        // Changing the blitter changes the pixel resolution of the cells, so
        // we must also re-size the pixel buffer.
        self.blitter = blitter;
        // No change to the `pixels` storage layout — always `height × width`
        // cells, each holding `u8` bits. Only the bit-layout interpretation
        // changes between Braille / Sextant / HalfBlock.
        self
    }

    /// Set the visual style (builder pattern).
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Pixel width (horizontal resolution = terminal columns × cols-per-cell).
    ///
    /// | Blitter              | cols-per-cell |
    /// |----------------------|---------------|
    /// | Braille / Sextant / Octant | 2       |
    /// | HalfBlock            | 1             |
    pub fn pixel_width(&self) -> usize {
        match self.blitter {
            Blitter::HalfBlock => self.width,
            _ => self.width * 2,
        }
    }

    /// Pixel height (vertical resolution = terminal rows × rows-per-cell).
    ///
    /// | Blitter    | rows-per-cell |
    /// |------------|---------------|
    /// | Braille / Octant | 4      |
    /// | Sextant    | 3             |
    /// | HalfBlock  | 2             |
    pub fn pixel_height(&self) -> usize {
        match self.blitter {
            Blitter::Braille | Blitter::Octant => self.height * 4,
            Blitter::Sextant => self.height * 3,
            Blitter::HalfBlock => self.height * 2,
        }
    }

    // -- pixel operations ---------------------------------------------------

    /// Compute `(cell_col, cell_row, bit_mask)` for a pixel at `(x, y)`.
    ///
    /// Returns `None` when the coordinates are out of bounds.
    fn pixel_address(&self, x: usize, y: usize) -> Option<(usize, usize, u8)> {
        if x >= self.pixel_width() || y >= self.pixel_height() {
            return None;
        }
        let (cell_col, cell_row, bit) = match self.blitter {
            Blitter::Braille => {
                // 2 columns × 4 rows per cell (Braille dot bit-order).
                let col = x / 2;
                let row = y / 4;
                let bit = PIXEL_MAP[y % 4][x % 2];
                (col, row, bit)
            }
            Blitter::Octant => {
                // 2 columns × 4 rows per cell (row-major bit-order).
                let col = x / 2;
                let row = y / 4;
                let bit = OCTANT_PIXEL_MAP[y % 4][x % 2];
                (col, row, bit)
            }
            Blitter::Sextant => {
                // 2 columns × 3 rows per cell.
                let col = x / 2;
                let row = y / 3;
                let bit = SEXTANT_PIXEL_MAP[y % 3][x % 2];
                (col, row, bit)
            }
            Blitter::HalfBlock => {
                // 1 column × 2 rows per cell.
                let col = x; // one pixel per cell column
                let row = y / 2;
                // bit 0 = top (y even), bit 1 = bottom (y odd)
                let bit: u8 = if y % 2 == 0 { 0x01 } else { 0x02 };
                (col, row, bit)
            }
        };
        Some((cell_col, cell_row, bit))
    }

    /// Set a pixel at `(x, y)` in pixel coordinates.
    ///
    /// Out-of-bounds coordinates are silently ignored.
    pub fn set(&mut self, x: usize, y: usize) {
        if let Some((col, row, bit)) = self.pixel_address(x, y) {
            self.pixels[row][col] |= bit;
        }
    }

    /// Clear a pixel at `(x, y)` in pixel coordinates.
    ///
    /// Out-of-bounds coordinates are silently ignored.
    pub fn unset(&mut self, x: usize, y: usize) {
        if let Some((col, row, bit)) = self.pixel_address(x, y) {
            self.pixels[row][col] &= !bit;
        }
    }

    /// Toggle a pixel at `(x, y)` in pixel coordinates.
    ///
    /// Out-of-bounds coordinates are silently ignored.
    pub fn toggle(&mut self, x: usize, y: usize) {
        if let Some((col, row, bit)) = self.pixel_address(x, y) {
            self.pixels[row][col] ^= bit;
        }
    }

    /// Test whether the pixel at `(x, y)` is set.
    ///
    /// Out-of-bounds coordinates return `false`.
    pub fn get(&self, x: usize, y: usize) -> bool {
        match self.pixel_address(x, y) {
            Some((col, row, bit)) => self.pixels[row][col] & bit != 0,
            None => false,
        }
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

    /// Render the canvas to a multi-line string using the active [`Blitter`].
    ///
    /// Each character cell is mapped to its Unicode glyph according to the
    /// blitter's bit-to-codepoint table.  Rows are joined with `'\n'`.
    pub fn frame(&self) -> String {
        let mut lines: Vec<String> = Vec::with_capacity(self.height);
        for row in &self.pixels {
            let line: String = match self.blitter {
                Blitter::Braille => row
                    .iter()
                    .map(|&bits| char::from_u32(BRAILLE_BASE + bits as u32).unwrap_or(' '))
                    .collect(),
                Blitter::Octant => row.iter().map(|&bits| octant_bits_to_char(bits)).collect(),
                Blitter::Sextant => row.iter().map(|&bits| sextant_bits_to_char(bits)).collect(),
                Blitter::HalfBlock => row
                    .iter()
                    .map(|&bits| halfblock_bits_to_char(bits))
                    .collect(),
            };
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
            let line: String = match self.blitter {
                Blitter::Braille => row
                    .iter()
                    .map(|&bits| char::from_u32(BRAILLE_BASE + bits as u32).unwrap_or(' '))
                    .collect(),
                Blitter::Octant => row.iter().map(|&bits| octant_bits_to_char(bits)).collect(),
                Blitter::Sextant => row.iter().map(|&bits| sextant_bits_to_char(bits)).collect(),
                Blitter::HalfBlock => row
                    .iter()
                    .map(|&bits| halfblock_bits_to_char(bits))
                    .collect(),
            };
            segments.push(Segment::new(&line, Some(self.style.clone()), None));
            if i < self.height - 1 {
                segments.push(Segment::line());
            }
        }
        segments.push(Segment::line());
        segments
    }

    fn gilt_measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        self.measure(console, options)
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
    fn canvas_gilt_measure_delegates_to_measure() {
        let canvas = Canvas::new(10, 4);
        let console = Console::builder()
            .width(80)
            .force_terminal(true)
            .no_color(true)
            .build();
        let opts = console.options();
        assert_eq!(
            canvas.gilt_measure(&console, &opts),
            canvas.measure(&console, &opts),
            "Canvas::gilt_measure must delegate to Canvas::measure",
        );
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
            std::iter::repeat_n(empty_braille, 3).collect::<String>(),
            "\n",
            std::iter::repeat_n(empty_braille, 3).collect::<String>(),
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
        let style = Style::parse("bold green");
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

    // -- Octant blitter (Unicode 16) ----------------------------------------
    //
    // The 256 2×4 octant patterns were verified against the authoritative
    // Unicode 16.0 `UnicodeData.txt` (`BLOCK OCTANT-N` names).  Bit order:
    // `bit = row * 2 + col`, so cell `c` (numbered 1..8 left-to-right,
    // top-to-bottom) corresponds to `bit (c - 1)`.

    // 26. Octant anchors: empty → SPACE, all set → FULL BLOCK.
    #[test]
    fn test_octant_anchors() {
        assert_eq!(octant_bits_to_char(0x00), ' ');
        assert_eq!(octant_bits_to_char(0xFF), '\u{2588}');
    }

    // 27. The four half-block patterns reuse existing Block Elements glyphs.
    #[test]
    fn test_octant_half_blocks() {
        assert_eq!(octant_bits_to_char(0x0F), '\u{2580}'); // ▀ upper (cells 1-4)
        assert_eq!(octant_bits_to_char(0xF0), '\u{2584}'); // ▄ lower (cells 5-8)
        assert_eq!(octant_bits_to_char(0x55), '\u{258C}'); // ▌ left  (cells 1,3,5,7)
        assert_eq!(octant_bits_to_char(0xAA), '\u{2590}'); // ▐ right (cells 2,4,6,8)
    }

    // 28. New U+1CD00 block glyphs — codepoints verified against Unicode 16.0
    //     UnicodeData.txt:
    //       U+1CD00 BLOCK OCTANT-3       (cell 3        = 0x04)
    //       U+1CD01 BLOCK OCTANT-23      (cells 2,3     = 0x06)
    //       U+1CD02 BLOCK OCTANT-123     (cells 1,2,3   = 0x07)
    //       U+1CDE5 BLOCK OCTANT-2345678 (cells 2-8     = 0xFE)
    #[test]
    fn test_octant_new_block_codepoints() {
        assert_eq!(octant_bits_to_char(0x04), '\u{1CD00}');
        assert_eq!(octant_bits_to_char(0x06), '\u{1CD01}');
        assert_eq!(octant_bits_to_char(0x07), '\u{1CD02}');
        assert_eq!(octant_bits_to_char(0xFE), '\u{1CDE5}');
    }

    // 29. The two new Unicode 16 "middle quarter" glyphs + corner quarter
    //     blocks (also reused, not in the contiguous block).
    #[test]
    fn test_octant_reused_quarter_blocks() {
        assert_eq!(octant_bits_to_char(0x14), '\u{1FBE6}'); // MIDDLE LEFT ¼ (cells 3,5)
        assert_eq!(octant_bits_to_char(0x28), '\u{1FBE7}'); // MIDDLE RIGHT ¼ (cells 4,6)
        assert_eq!(octant_bits_to_char(0x03), '\u{1FB82}'); // UPPER ¼ (cells 1,2)
        assert_eq!(octant_bits_to_char(0xC0), '\u{2582}'); // LOWER ¼ (cells 7,8)
        assert_eq!(octant_bits_to_char(0x3F), '\u{1FB85}'); // UPPER ¾ (cells 1-6)
        assert_eq!(octant_bits_to_char(0xFC), '\u{2586}'); // LOWER ¾ (cells 3-8)
        assert_eq!(octant_bits_to_char(0x01), '\u{1CEA8}'); // top-left cell only
        assert_eq!(octant_bits_to_char(0x80), '\u{1CEA0}'); // bottom-right cell only
    }

    // 30. Each of the 8 sub-cell positions sets exactly its row-major bit.
    #[test]
    fn test_octant_pixel_bit_mapping() {
        let cases: [(usize, usize, u8); 8] = [
            (0, 0, 0x01), // cell 1 (top-left)
            (1, 0, 0x02), // cell 2 (top-right)
            (0, 1, 0x04), // cell 3
            (1, 1, 0x08), // cell 4
            (0, 2, 0x10), // cell 5
            (1, 2, 0x20), // cell 6
            (0, 3, 0x40), // cell 7 (bottom-left)
            (1, 3, 0x80), // cell 8 (bottom-right)
        ];
        for (px, py, expected_bit) in cases {
            let mut c = Canvas::new(1, 1).with_blitter(Blitter::Octant);
            c.set(px, py);
            assert_eq!(
                c.pixels[0][0], expected_bit,
                "octant pixel ({px},{py}) should set bit 0x{expected_bit:02X}"
            );
        }
    }

    // 31. Octant frame: a fully-set cell renders FULL BLOCK.
    #[test]
    fn test_octant_frame_full_cell() {
        let mut c = Canvas::new(1, 1).with_blitter(Blitter::Octant);
        for y in 0..4 {
            for x in 0..2 {
                c.set(x, y);
            }
        }
        assert_eq!(c.pixels[0][0], 0xFF);
        assert_eq!(c.frame(), '\u{2588}'.to_string());
    }

    // 32. An empty octant canvas renders SPACEs (not braille blanks) — proves
    //     the Octant arm no longer falls back to Braille.
    #[test]
    fn test_octant_empty_frame_is_space() {
        let c = Canvas::new(3, 1).with_blitter(Blitter::Octant);
        assert_eq!(c.frame(), "   ");
    }

    // 33. gilt_console renders octant glyphs (not braille) for the Octant arm.
    #[test]
    fn test_octant_gilt_console_uses_octant_glyphs() {
        let mut c = Canvas::new(1, 1).with_blitter(Blitter::Octant);
        c.set(0, 1); // cell 3 → bit 0x04 → U+1CD00 BLOCK OCTANT-3
        let console = Console::builder().width(80).build();
        let opts = make_options(80);
        let segs = c.gilt_console(&console, &opts);
        assert_eq!(segs[0].text.as_str(), "\u{1CD00}");
    }

    // 34. Invariant: every one of the 256 patterns maps to a distinct glyph
    //     (guards against typos in the table).
    #[test]
    fn test_octant_table_all_distinct() {
        use std::collections::HashSet;
        let set: HashSet<char> = OCTANT_CHARS.iter().copied().collect();
        assert_eq!(
            set.len(),
            256,
            "every octant pattern must map to a distinct glyph"
        );
    }
}
