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
fn cell_len_zwj_family_emoji_is_6_per_codepoint() {
    // v1.4.1: cell_len reports per-codepoint sum (6) not cluster
    // width (2). Reflects what most terminals actually render when
    // they lack ZWJ-aware emoji fonts.
    let s = "👨\u{200d}👩\u{200d}👧";
    assert_eq!(cell_len(s), 6);
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
fn cell_len_variation_selector_heart_is_1() {
    // ❤️ = U+2764 (heavy black heart) U+FE0F (VS-16). Per-codepoint:
    // U+2764 is width=1 (text-presentation default) and U+FE0F is
    // width=0. v1.4.1's per-codepoint sum returns 1.
    //
    // On modern terminals VS-16 promotes the base to a 2-cell emoji,
    // so this is one of the cases where gilt under-reports vs visual
    // reality. The over/under tradeoff: ZWJ over-reports (good for
    // tables); VS-16 under-reports (would clip emoji-presentation
    // hearts). Documented in cell_len's docstring.
    let s = "\u{2764}\u{FE0F}";
    assert_eq!(cell_len(s), 1);
}

// -- set_cell_size: grapheme-safe truncation -------------------------------

#[test]
fn set_cell_size_truncates_around_zwj_cluster() {
    // "👨‍👩‍👧 family" — per-codepoint widths are 2+0+2+0+2 + 1 + 6 = 13.
    // Truncate to 6 cells: the entire ZWJ cluster fits exactly. Must
    // not be split mid-cluster.
    let s = "👨\u{200d}👩\u{200d}👧 family";
    let cropped = set_cell_size(s, 6);
    assert!(
        cropped.contains("👨\u{200d}👩\u{200d}👧") || !cropped.contains("\u{200d}"),
        "expected full family or no orphan ZWJ, got {:?}",
        cropped
    );
    // Width invariant: result fills exactly 6 cells.
    assert_eq!(cell_len(&cropped), 6);
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
    // v1.4.1: family is 6 cells (per-codepoint). Budget 6 fits it exactly.
    let mut t = Text::new("👨\u{200d}👩\u{200d}👧 family", Style::null());
    t.truncate(6, Some(OverflowMethod::Crop), false);
    let plain = t.plain().to_string();
    assert!(
        plain.contains("👨\u{200d}👩\u{200d}👧"),
        "truncate dropped or split the ZWJ family: {:?}",
        plain
    );
}

#[test]
fn text_truncate_no_dangling_zwj() {
    // A 3-cell crop puts the boundary inside the ZWJ cluster
    // (which is 6 cells per the v1.4.1 per-codepoint width).
    // Pre-v1.4: would emit "👨\u{200d}👩" with dangling ZWJ.
    // v1.4+: replaces the partial cluster with spaces.
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
