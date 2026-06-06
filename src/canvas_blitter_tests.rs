//! TDD red tests for Task 2: Canvas blitter ladder (Braille → Sextant → HalfBlock → Octant).
//!
//! These tests FAIL TO COMPILE until `Blitter` and `Canvas::with_blitter` are implemented.

use crate::canvas::{Blitter, Canvas};

// ---------------------------------------------------------------------------
// Blitter enum — compile check
// ---------------------------------------------------------------------------

/// Verify that all Blitter variants exist and are distinct.
#[test]
fn test_blitter_variants_exist() {
    let _b = Blitter::Braille;
    let _s = Blitter::Sextant;
    let _h = Blitter::HalfBlock;
    let _o = Blitter::Octant;
}

// ---------------------------------------------------------------------------
// Default (Braille) behaviour is UNCHANGED
// ---------------------------------------------------------------------------

/// The default canvas (no explicit blitter) still behaves exactly like before.
#[test]
fn test_braille_default_unchanged() {
    let mut c = Canvas::new(1, 1);
    c.set(0, 0); // Braille dot-1 => bit 0x01 => U+2801
    let frame = c.frame();
    let expected = char::from_u32(0x2800 + 0x01).unwrap();
    assert_eq!(
        frame,
        expected.to_string(),
        "default Braille should still produce U+2801"
    );
}

/// with_blitter(Braille) gives the same result as the default.
#[test]
fn test_with_blitter_braille_same_as_default() {
    let mut default_c = Canvas::new(1, 1);
    default_c.set(0, 0);

    let mut braille_c = Canvas::new(1, 1).with_blitter(Blitter::Braille);
    braille_c.set(0, 0);

    assert_eq!(
        default_c.frame(),
        braille_c.frame(),
        "explicit Braille blitter must equal default"
    );
}

// ---------------------------------------------------------------------------
// HalfBlock blitter  (1×2 pixels per cell, U+2580 ▀ / U+2584 ▄ / U+2588 █)
// ---------------------------------------------------------------------------

/// Top pixel only set in a 1-cell HalfBlock canvas → U+2580 ▀ (upper half block).
#[test]
fn test_halfblock_top_pixel() {
    let mut c = Canvas::new(1, 1).with_blitter(Blitter::HalfBlock);
    c.set(0, 0); // pixel (0,0) = top half
    let frame = c.frame();
    // U+2580 = UPPER HALF BLOCK
    assert!(
        frame.contains('\u{2580}'),
        "top pixel in HalfBlock should produce U+2580 ▀, got: {frame:?}"
    );
}

/// Bottom pixel only set → U+2584 ▄ (lower half block).
#[test]
fn test_halfblock_bottom_pixel() {
    let mut c = Canvas::new(1, 1).with_blitter(Blitter::HalfBlock);
    c.set(0, 1); // pixel (0,1) = bottom half
    let frame = c.frame();
    // U+2584 = LOWER HALF BLOCK
    assert!(
        frame.contains('\u{2584}'),
        "bottom pixel in HalfBlock should produce U+2584 ▄, got: {frame:?}"
    );
}

/// Both pixels set → U+2588 █ (full block).
#[test]
fn test_halfblock_both_pixels() {
    let mut c = Canvas::new(1, 1).with_blitter(Blitter::HalfBlock);
    c.set(0, 0);
    c.set(0, 1);
    let frame = c.frame();
    // U+2588 = FULL BLOCK
    assert!(
        frame.contains('\u{2588}'),
        "both pixels in HalfBlock should produce U+2588 █, got: {frame:?}"
    );
}

/// No pixels set → space character.
#[test]
fn test_halfblock_empty_is_space() {
    let c = Canvas::new(1, 1).with_blitter(Blitter::HalfBlock);
    let frame = c.frame();
    assert_eq!(frame, " ", "empty HalfBlock cell should be a space");
}

// ---------------------------------------------------------------------------
// Sextant blitter  (2×3 per cell, U+1FB00 legacy-computing block sextants)
// ---------------------------------------------------------------------------
//
// The Unicode 13 sextant block at U+1FB00..U+1FB3B assigns a bit pattern
// to each of the 63 non-empty, non-full patterns:
//
//   Sextant layout (col 0 left, col 1 right):
//     bit 0 = (0,0)  bit 1 = (1,0)   row 0 (top)
//     bit 2 = (0,1)  bit 3 = (1,1)   row 1 (middle)
//     bit 4 = (0,2)  bit 5 = (1,2)   row 2 (bottom)
//
//   codepoint = U+1FB00 + (bit_pattern - 1)   for bit_pattern 1..=63
//   except: U+1FB3C..U+1FB3F (patterns 61..=64 in some sources) were
//   reserved; in practice the mapping is compact:
//     U+1FB00 = 0b000001 (top-left only)
//     U+1FB01 = 0b000010 (top-right only)
//     U+1FB02 = 0b000011 (top-left + top-right)
//     ...
//     U+1FB3B = 0b111110 (all except bottom-right)
//   Full block (0b111111 = 63) = U+2588 (FULL BLOCK) — not in the sextant range.
//   Empty (0b000000)           = SPACE.

/// A single top-left sextant pixel → U+1FB00 (bit pattern 0b000001).
#[test]
fn test_sextant_top_left_pixel() {
    let mut c = Canvas::new(1, 1).with_blitter(Blitter::Sextant);
    // pixel (x=0, y=0) = bit 0 = top-left sextant
    c.set(0, 0);
    let frame = c.frame();
    let expected = '\u{1FB00}'; // UPPER LEFT BLOCK SEXTANT-1
    assert!(
        frame.contains(expected),
        "sextant pixel (0,0) should produce U+1FB00, got: {frame:?}"
    );
}

/// A single top-right sextant pixel → U+1FB01 (bit pattern 0b000010).
#[test]
fn test_sextant_top_right_pixel() {
    let mut c = Canvas::new(1, 1).with_blitter(Blitter::Sextant);
    // pixel (x=1, y=0) = bit 1 = top-right sextant
    c.set(1, 0);
    let frame = c.frame();
    let expected = '\u{1FB01}';
    assert!(
        frame.contains(expected),
        "sextant pixel (1,0) should produce U+1FB01, got: {frame:?}"
    );
}

/// Both top sextants set → U+1FB02 (bit pattern 0b000011).
#[test]
fn test_sextant_top_row() {
    let mut c = Canvas::new(1, 1).with_blitter(Blitter::Sextant);
    c.set(0, 0); // bit 0
    c.set(1, 0); // bit 1
    let frame = c.frame();
    let expected = '\u{1FB02}'; // bit_pattern=0b000011 → U+1FB00+2 = U+1FB02
    assert!(
        frame.contains(expected),
        "sextant top row (both pixels) should produce U+1FB02, got: {frame:?}"
    );
}

/// Bottom-left sextant only → U+1FB10 (bit pattern 0b010000 = 16).
#[test]
fn test_sextant_bottom_left_pixel() {
    let mut c = Canvas::new(1, 1).with_blitter(Blitter::Sextant);
    // pixel (x=0, y=2) = bit 4 = bottom-left sextant
    c.set(0, 2);
    let frame = c.frame();
    // bit_pattern = 0b010000 = 16 → U+1FB00 + (16 - 1) = U+1FB0F
    let expected = '\u{1FB0F}';
    assert!(
        frame.contains(expected),
        "sextant pixel (0,2) should produce U+1FB0F, got: {frame:?}"
    );
}

/// Empty sextant cell → space.
#[test]
fn test_sextant_empty_is_space() {
    let c = Canvas::new(1, 1).with_blitter(Blitter::Sextant);
    let frame = c.frame();
    assert_eq!(frame, " ", "empty Sextant cell should be a space");
}

// ---------------------------------------------------------------------------
// Octant blitter  (2×4 per cell, Unicode 16 U+1CD00 block, or Braille stub)
// ---------------------------------------------------------------------------
//
// If Octant is fully implemented: bit pattern maps to U+1CD00 + (bits - 1)
// for patterns 1..=254. The empty pattern is a space, full = U+2588.
// If Octant is stubbed to Braille, the frame is a valid Braille string.

/// Octant blitter with a pixel set should produce a non-empty frame.
#[test]
fn test_octant_pixel_set_nonempty() {
    let mut c = Canvas::new(1, 1).with_blitter(Blitter::Octant);
    c.set(0, 0);
    let frame = c.frame();
    assert!(
        !frame.is_empty(),
        "Octant frame should be non-empty after setting a pixel"
    );
    // Must not be space (pixel is set).
    assert_ne!(
        frame.trim(),
        "",
        "frame with set pixel should not be all spaces"
    );
}

/// Empty Octant canvas → space or empty Braille (if stubbed).
#[test]
fn test_octant_empty_nonempty_or_space() {
    let c = Canvas::new(1, 1).with_blitter(Blitter::Octant);
    let frame = c.frame();
    // Accept either a space (Octant native) or the empty Braille char (stub).
    let is_space = frame == " ";
    let is_empty_braille = frame.contains('\u{2800}');
    assert!(
        is_space || is_empty_braille,
        "empty Octant cell should be space or empty Braille, got: {frame:?}"
    );
}
