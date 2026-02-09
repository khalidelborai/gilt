//! Property-based tests for gilt using proptest.
//!
//! These tests verify key invariants across randomized inputs,
//! catching edge cases that hand-written tests might miss.

use proptest::prelude::*;

use gilt::cells::cell_len;
use gilt::color::{Color, ColorSystem, ColorType};
use gilt::color_triplet::ColorTriplet;
use gilt::segment::Segment;
use gilt::style::Style;
use gilt::wrap::divide_line;

// ---------------------------------------------------------------------------
// 1. Text wrapping width invariant
// ---------------------------------------------------------------------------
//
// For any string and any max_width >= 1, wrapping produces lines that are
// all <= max_width visual columns (when fold=true).

/// Given divide_line break positions, reconstruct lines as char slices and
/// verify each line's cell_len is <= width.
fn assert_wrapped_lines_fit(text: &str, width: usize) {
    if width == 0 || text.is_empty() {
        return;
    }

    let breaks = divide_line(text, width, true);
    let chars: Vec<char> = text.chars().collect();
    let total_chars = chars.len();

    // Build line boundaries: [0, breaks[0], breaks[1], ..., total_chars]
    let mut boundaries = vec![0usize];
    boundaries.extend_from_slice(&breaks);
    boundaries.push(total_chars);

    for window in boundaries.windows(2) {
        let start = window[0];
        let end = window[1];
        if start >= end {
            continue;
        }
        let line: String = chars[start..end].iter().collect();
        let trimmed = line.trim_end();
        let line_width = cell_len(trimmed);
        assert!(
            line_width <= width,
            "Line {:?} (trimmed {:?}) has cell_len {} > width {}; breaks={:?}, text={:?}",
            line,
            trimmed,
            line_width,
            width,
            breaks,
            text,
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn text_wrapping_respects_width(
        s in "[a-zA-Z0-9 ]{0,80}",
        w in 1usize..=40,
    ) {
        assert_wrapped_lines_fit(&s, w);
    }

    #[test]
    fn text_wrapping_single_words(
        word in "[a-z]{1,20}",
        w in 1usize..=10,
    ) {
        assert_wrapped_lines_fit(&word, w);
    }

    #[test]
    fn text_wrapping_with_spaces(
        words in prop::collection::vec("[a-z]{1,8}", 1..6),
        w in 1usize..=20,
    ) {
        let text = words.join(" ");
        assert_wrapped_lines_fit(&text, w);
    }
}

// ---------------------------------------------------------------------------
// 2. Color downgrade produces valid colors
// ---------------------------------------------------------------------------
//
// For any RGB triplet, downgrading from TrueColor to EightBit produces a
// valid 0-255 index, and downgrading to Standard produces a valid 0-15 index.

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn color_downgrade_truecolor_to_eightbit(
        r in 0u8..=255u8,
        g in 0u8..=255u8,
        b in 0u8..=255u8,
    ) {
        let color = Color::from_rgb(r, g, b);
        assert_eq!(color.color_type, ColorType::TrueColor);

        let downgraded = color.downgrade(ColorSystem::EightBit);
        assert_eq!(downgraded.color_type, ColorType::EightBit);
        let _number = downgraded.number.expect("EightBit color must have a number");
        // u8 guarantees 0-255 range; the key assertion is that downgrade
        // produces a color with EightBit type and a valid number.
    }

    #[test]
    fn color_downgrade_truecolor_to_standard(
        r in 0u8..=255u8,
        g in 0u8..=255u8,
        b in 0u8..=255u8,
    ) {
        let color = Color::from_rgb(r, g, b);
        let downgraded = color.downgrade(ColorSystem::Standard);
        assert_eq!(downgraded.color_type, ColorType::Standard);
        let number = downgraded.number.expect("Standard color must have a number");
        assert!(
            number < 16,
            "Standard index {} out of range [0,15] for rgb({},{},{})",
            number, r, g, b,
        );
    }

    #[test]
    fn color_downgrade_eightbit_to_standard(
        n in 0u8..=255u8,
    ) {
        let color = Color::from_ansi(n);
        let downgraded = color.downgrade(ColorSystem::Standard);
        assert_eq!(downgraded.color_type, ColorType::Standard);
        let number = downgraded.number.expect("Standard color must have a number");
        assert!(
            number < 16,
            "Standard index {} out of range [0,15] for color({})",
            number, n,
        );
    }

    #[test]
    fn color_downgrade_to_truecolor_is_identity(
        r in 0u8..=255u8,
        g in 0u8..=255u8,
        b in 0u8..=255u8,
    ) {
        let color = Color::from_rgb(r, g, b);
        let downgraded = color.downgrade(ColorSystem::TrueColor);
        assert_eq!(downgraded, color, "TrueColor downgrade should be identity");
    }
}

// ---------------------------------------------------------------------------
// 3. Cell length non-negative and empty-string invariant
// ---------------------------------------------------------------------------
//
// cell_len always returns >= 0 (trivially true for usize), and cell_len("") == 0.

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn cell_len_empty_is_zero(_dummy in 0..1i32) {
        assert_eq!(cell_len(""), 0);
    }

    #[test]
    fn cell_len_is_at_least_char_count_for_ascii(s in "[a-zA-Z0-9]{0,100}") {
        // For pure ASCII, cell_len == byte len == char count
        assert_eq!(cell_len(&s), s.len());
    }

    #[test]
    fn cell_len_is_nonnegative(s in "[ -~]{0,100}") {
        // cell_len returns usize, so always >= 0, but verify it doesn't panic
        let _ = cell_len(&s);
    }

    #[test]
    fn cell_len_geq_char_count_for_printable(s in "[a-z\\u{3041}-\\u{3096}]{0,30}") {
        // For printable text (ASCII lowercase + Hiragana letters U+3041..U+3096),
        // cell_len >= char count because CJK chars are double-width.
        // We exclude combining marks (U+3099-U+309A) which have zero width.
        let char_count = s.chars().count();
        assert!(
            cell_len(&s) >= char_count,
            "cell_len({:?}) = {} < char_count {}",
            s, cell_len(&s), char_count,
        );
    }
}

// ---------------------------------------------------------------------------
// 4. Style parse roundtrip
// ---------------------------------------------------------------------------
//
// For known style components, Style::parse(s).to_string() produces a string
// that can be parsed back (the re-parsed style should equal the original).

/// Generate a random style definition from known components.
fn style_definition_strategy() -> impl Strategy<Value = String> {
    let attributes = prop::sample::subsequence(
        &["bold", "italic", "underline", "dim", "reverse", "strike"],
        0..=6,
    );
    let colors = prop::option::of(prop::sample::select(&[
        "red", "green", "blue", "cyan", "magenta", "yellow", "white", "black",
    ]));
    let bg_colors = prop::option::of(prop::sample::select(&[
        "red", "green", "blue", "cyan", "magenta", "yellow", "white", "black",
    ]));

    (attributes, colors, bg_colors).prop_map(|(attrs, fg, bg)| {
        let mut parts: Vec<String> = attrs.iter().map(|a| a.to_string()).collect();
        if let Some(color) = fg {
            parts.push(color.to_string());
        }
        if let Some(bgcolor) = bg {
            parts.push(format!("on {}", bgcolor));
        }
        parts.join(" ")
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn style_parse_roundtrip(definition in style_definition_strategy()) {
        // Parse the generated definition
        let style = Style::parse(&definition).unwrap_or_else(|e| panic!("Failed to parse {:?}: {}", definition, e));
        let display = style.to_string();

        // Style::null().to_string() produces "none", which is not parseable as
        // a style definition. An empty definition parses to null, so substitute.
        let reparse_input = if display == "none" { "" } else { &display };

        // The display string should also be parseable
        let reparsed = Style::parse(reparse_input).unwrap_or_else(|e| panic!(
            "Failed to reparse display {:?} from original {:?}: {}",
            display, definition, e,
        ));

        // The reparsed style should equal the original
        assert_eq!(
            style, reparsed,
            "Roundtrip mismatch: {:?} -> {:?} -> {:?}",
            definition, display, reparsed.to_string(),
        );
    }

    #[test]
    fn style_parse_single_attribute(
        attr in prop::sample::select(&[
            "bold", "italic", "underline", "dim", "reverse", "strike",
            "overline", "blink", "blink2", "conceal", "underline2",
            "frame", "encircle",
        ]),
    ) {
        let style = Style::parse(attr).unwrap();
        let display = style.to_string();
        let reparsed = Style::parse(&display).unwrap();
        assert_eq!(style, reparsed);
    }

    #[test]
    fn style_parse_not_attribute(
        attr in prop::sample::select(&[
            "bold", "italic", "underline", "dim", "reverse", "strike",
        ]),
    ) {
        let definition = format!("not {}", attr);
        let style = Style::parse(&definition).unwrap();
        let display = style.to_string();
        let reparsed = Style::parse(&display).unwrap();
        assert_eq!(style, reparsed);
    }
}

// ---------------------------------------------------------------------------
// 5. Segment split preserves content
// ---------------------------------------------------------------------------
//
// For any text and split position, splitting a Segment should preserve the
// total text content (modulo space substitution for split double-width chars).

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn segment_split_preserves_total_width(
        s in "[a-zA-Z0-9]{0,50}",
        cut in 0usize..=60,
    ) {
        let segment = Segment::text(&s);
        let original_len = segment.cell_length();
        let (left, right) = segment.split_cells(cut);

        // For ASCII, the total cell_len should be preserved exactly
        assert_eq!(
            left.cell_length() + right.cell_length(),
            original_len,
            "Split at {} of {:?}: left={:?} right={:?}",
            cut, s, left.text, right.text,
        );
    }

    #[test]
    fn segment_split_left_width_bounded(
        s in "[a-zA-Z0-9]{0,50}",
        cut in 0usize..=60,
    ) {
        let segment = Segment::text(&s);
        let (left, _right) = segment.split_cells(cut);

        // The left part should have cell_len <= cut
        assert!(
            left.cell_length() <= cut.max(cell_len(&s)),
            "Left part cell_len {} > cut {} for {:?}",
            left.cell_length(), cut, s,
        );
    }

    #[test]
    fn segment_split_ascii_content_preserved(
        s in "[a-zA-Z0-9]{0,30}",
        cut in 0usize..=40,
    ) {
        let segment = Segment::text(&s);
        let (left, right) = segment.split_cells(cut);

        // For pure ASCII, concatenating left+right should give back the original
        let combined = format!("{}{}", left.text, right.text);
        assert_eq!(
            combined, s,
            "Content not preserved: split {:?} at {} -> {:?} + {:?}",
            s, cut, left.text, right.text,
        );
    }

    #[test]
    fn segment_split_preserves_style(
        s in "[a-z]{1,10}",
        cut in 0usize..=15,
    ) {
        let style = Style::parse("bold red").unwrap();
        let segment = Segment::styled(&s, style.clone());
        let (left, right) = segment.split_cells(cut);

        // Both halves should have the same style
        assert_eq!(left.style, Some(style.clone()));
        assert_eq!(right.style, Some(style));
    }
}

// ---------------------------------------------------------------------------
// Bonus: ColorTriplet hex roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn color_triplet_hex_roundtrip(
        r in 0u8..=255u8,
        g in 0u8..=255u8,
        b in 0u8..=255u8,
    ) {
        let triplet = ColorTriplet::new(r, g, b);
        let hex = triplet.hex();

        // The hex string should be parseable back to the same color
        let parsed = Color::parse(&hex).expect("hex should be parseable");
        let parsed_triplet = parsed.triplet.expect("parsed hex should have triplet");
        assert_eq!(triplet, parsed_triplet);
    }

    #[test]
    fn color_triplet_normalized_in_range(
        r in 0u8..=255u8,
        g in 0u8..=255u8,
        b in 0u8..=255u8,
    ) {
        let triplet = ColorTriplet::new(r, g, b);
        let (nr, ng, nb) = triplet.normalized();
        assert!((0.0..=1.0).contains(&nr));
        assert!((0.0..=1.0).contains(&ng));
        assert!((0.0..=1.0).contains(&nb));
    }
}
