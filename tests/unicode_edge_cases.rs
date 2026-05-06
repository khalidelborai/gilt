//! End-to-end Unicode-correctness tests for v1.4.0.
//!
//! Pre-v1.4 `set_cell_size` (and the `Text::truncate` / `Text::right_crop`
//! that route through it) iterated codepoints, which could leave a
//! dangling ZWJ joiner or split a flag emoji's regional-indicator pair.
//! Post-v1.4 the iteration is grapheme-cluster-based.

use gilt::cells::{cell_len, set_cell_size};
use gilt::style::Style;
use gilt::text::{OverflowMethod, Text};

// -- Visible width assertions (regression guards on PR2 fixes) -------------

#[test]
fn cell_len_zwj_family_emoji_is_2() {
    let s = "👨\u{200d}👩\u{200d}👧";
    assert_eq!(cell_len(s), 2);
}

#[test]
fn cell_len_us_flag_is_2() {
    let s = "\u{1F1FA}\u{1F1F8}";
    assert_eq!(cell_len(s), 2);
}

#[test]
fn cell_len_combining_acute_is_4() {
    let s = "cafe\u{0301}";
    assert_eq!(cell_len(s), 4);
}

#[test]
fn cell_len_hangul_composed_is_4() {
    // 한글 = U+D55C U+AE00 (precomposed Hangul syllables, 2 cells each)
    let s = "한글";
    assert_eq!(cell_len(s), 4);
}

#[test]
fn cell_len_variation_selector_heart_is_2() {
    // ❤️ = U+2764 (heavy black heart) U+FE0F (VS-16, emoji presentation).
    // unicode-width 0.2 returns 2 for the sequence — VS-16 promotes the
    // base char to its emoji presentation, which terminals render at
    // 2 cells. (Pre-0.2 returned 1.)
    let s = "\u{2764}\u{FE0F}";
    assert_eq!(cell_len(s), 2);
}

// -- set_cell_size: grapheme-safe truncation -------------------------------

#[test]
fn set_cell_size_truncates_before_zwj_cluster() {
    // "👨‍👩‍👧 family" — family glyph is 2 cells, then space, then "family".
    // Truncating to 4 cells should keep the family glyph (2) + " f" (2)
    // and NOT leave a dangling ZWJ orphan.
    let s = "👨\u{200d}👩\u{200d}👧 family";
    let cropped = set_cell_size(s, 4);
    // Must contain the full family glyph, not just "👨\u{200d}👩\u{200d}"
    // (which the pre-v1.4 codepoint-iterating crop would emit).
    assert!(
        cropped.contains("👨\u{200d}👩\u{200d}👧") || !cropped.contains("\u{200d}"),
        "expected full family or no orphan ZWJ, got {:?}",
        cropped
    );
    // Width invariant: result fills exactly 4 cells.
    assert_eq!(cell_len(&cropped), 4);
}

#[test]
fn set_cell_size_truncates_before_flag() {
    // "🇺🇸 USA" — flag is 2 cells, " USA" is 4 cells = 6 total.
    // Crop to 1 cell: should NOT emit only the first regional indicator
    // (which would visually show "🇺" as a broken half-flag in some
    // terminals). Replace with a space.
    let s = "\u{1F1FA}\u{1F1F8} USA";
    let cropped = set_cell_size(s, 1);
    // Width invariant: exactly 1 cell.
    assert_eq!(cell_len(&cropped), 1);
    // Must not contain the lone first regional indicator.
    assert!(!cropped.contains('\u{1F1FA}') || cropped.contains('\u{1F1F8}'));
}

#[test]
fn set_cell_size_keeps_combining_marks_with_their_base() {
    // "café" with combining acute = "cafe\u{0301}".
    // Truncate to 4 cells should keep the full sequence (cafe + combining).
    let s = "cafe\u{0301}";
    let cropped = set_cell_size(s, 4);
    assert_eq!(cell_len(&cropped), 4);
    // The combining acute (zero width) must travel with its base 'e'.
    assert!(cropped.contains("e\u{0301}") || !cropped.contains('\u{0301}'));
}

#[test]
fn set_cell_size_pure_ascii_unchanged() {
    // The grapheme path must not regress the ASCII fast path.
    assert_eq!(set_cell_size("hello", 5), "hello");
    assert_eq!(set_cell_size("hello", 3), "hel");
    assert_eq!(set_cell_size("hi", 5), "hi   ");
}

// -- Text::truncate end-to-end --------------------------------------------

#[test]
fn text_truncate_keeps_zwj_family_intact_when_it_fits() {
    let mut t = Text::new("👨\u{200d}👩\u{200d}👧 family", Style::null());
    t.truncate(4, Some(OverflowMethod::Crop), false);
    // 4-cell crop = family (2) + " f" (2). Plain text must contain the
    // full family glyph.
    let plain = t.plain().to_string();
    assert!(
        plain.contains("👨\u{200d}👩\u{200d}👧"),
        "truncate dropped or split the ZWJ family: {:?}",
        plain
    );
}

#[test]
fn text_truncate_no_dangling_zwj() {
    // A 3-cell crop puts the boundary inside the ZWJ cluster.
    // Pre-v1.4: would emit "👨\u{200d}👩" with dangling ZWJ.
    // Post-v1.4: replaces the partial cluster with space.
    let mut t = Text::new("👨\u{200d}👩\u{200d}👧 family", Style::null());
    t.truncate(3, Some(OverflowMethod::Crop), false);
    let plain = t.plain().to_string();
    // Either the full family glyph is in the output (cell budget allowed),
    // or it's been replaced with whitespace — but there must be no orphan
    // ZWJ codepoint.
    let has_family = plain.contains("👨\u{200d}👩\u{200d}👧");
    let has_orphan_zwj = !has_family && plain.contains('\u{200d}');
    assert!(!has_orphan_zwj, "truncate left an orphan ZWJ: {:?}", plain);
}
