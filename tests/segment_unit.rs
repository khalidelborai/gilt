//! Unit tests for `Segment::divide` and `Segment::split_lines_terminator`.
//!
//! These are ported from rich/tests/test_segment.py.

use gilt::segment::Segment;
use gilt::style::Style;

// ---------------------------------------------------------------------------
// divide tests
// ---------------------------------------------------------------------------

/// Divide a multi-segment list at several cut points that span across segment
/// boundaries.  Ported from `test_divide` (the [1, 2, 12] and [4, 20] cases).
#[test]
fn divide_complex_multi_segment() {
    let bold = Style::parse("bold").unwrap();
    let italic = Style::parse("italic").unwrap();
    let segments = vec![
        Segment::styled("Hello", bold.clone()),
        Segment::styled(" World!", italic.clone()),
    ];

    // Cut at [1, 2, 12]: spans H | e | llo World!
    let parts = Segment::divide(&segments, &[1, 2, 12]);
    assert_eq!(parts.len(), 3, "expected 3 portions");
    assert_eq!(parts[0], vec![Segment::styled("H", bold.clone())]);
    assert_eq!(parts[1], vec![Segment::styled("e", bold.clone())]);
    assert_eq!(
        parts[2],
        vec![
            Segment::styled("llo", bold.clone()),
            Segment::styled(" World!", italic.clone()),
        ]
    );

    // Cut at [4, 20]: Hell | o World!
    let parts2 = Segment::divide(&segments, &[4, 20]);
    assert_eq!(parts2.len(), 2, "expected 2 portions");
    assert_eq!(parts2[0], vec![Segment::styled("Hell", bold.clone())]);
    assert_eq!(
        parts2[1],
        vec![
            Segment::styled("o", bold.clone()),
            Segment::styled(" World!", italic.clone()),
        ]
    );
}

/// Dividing at a position that falls *inside* a 2-cell-wide emoji replaces the
/// straddled emoji with spaces on both sides.  Ported from `test_divide_emoji`.
///
/// Segment layout: "Hello" (bold, 5 cells) + "💩💩💩" (italic, 6 cells = 2 each)
///
/// Cut at cell 8 falls inside the second emoji (cells 7-8): the before slice
/// should contain the first emoji followed by a replacement space, and the
/// after slice of the next portion should start with a replacement space.
#[test]
fn divide_emoji_at_2_cell_boundary() {
    let bold = Style::parse("bold").unwrap();
    let italic = Style::parse("italic").unwrap();
    let segments = vec![
        Segment::styled("Hello", bold.clone()),
        Segment::styled("💩💩💩", italic.clone()),
    ];

    // Cut at 7 — exact boundary between emoji 1 and emoji 2.
    let parts = Segment::divide(&segments, &[7]);
    assert_eq!(parts.len(), 1);
    assert_eq!(
        parts[0],
        vec![
            Segment::styled("Hello", bold.clone()),
            Segment::styled("💩", italic.clone()),
        ]
    );

    // Cut at 8 — mid-way through the second emoji (2-cell wide): replace with spaces.
    let parts = Segment::divide(&segments, &[8]);
    assert_eq!(parts.len(), 1);
    // "💩 " — first emoji intact, second emoji replaced on the left side by a space
    assert_eq!(
        parts[0],
        vec![
            Segment::styled("Hello", bold.clone()),
            Segment::styled("💩 ", italic.clone()),
        ]
    );

    // Two cuts at [8, 11]: first portion ends mid-emoji; second gets the right half.
    let parts = Segment::divide(&segments, &[8, 11]);
    assert_eq!(parts.len(), 2);
    assert_eq!(
        parts[0],
        vec![
            Segment::styled("Hello", bold.clone()),
            Segment::styled("💩 ", italic.clone()),
        ]
    );
    assert_eq!(parts[1], vec![Segment::styled(" 💩", italic.clone())]);
}

/// When the cut position equals the end of one segment, no empty trailing
/// segment should appear in that portion.  Ported from `test_divide_edge`.
#[test]
fn divide_at_exact_segment_end() {
    let segments = vec![
        Segment::text("foo"),
        Segment::text("bar"),
        Segment::text("baz"),
    ];
    let parts = Segment::divide(&segments, &[1, 3, 9]);
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0], vec![Segment::text("f")]);
    assert_eq!(parts[1], vec![Segment::text("oo")]);
    assert_eq!(parts[2], vec![Segment::text("bar"), Segment::text("baz")]);
}

/// Cut position equals the *start* of the next segment (i.e. exactly at a
/// segment boundary on the right side).  Ported from the [1] and [1, 2] cases
/// of `test_divide`.
#[test]
fn divide_at_exact_segment_start() {
    let bold = Style::parse("bold").unwrap();
    let italic = Style::parse("italic").unwrap();
    let segments = vec![
        Segment::styled("Hello", bold.clone()),
        Segment::styled(" World!", italic.clone()),
    ];

    // Single cut at 1: only "H" in the result.
    let parts = Segment::divide(&segments, &[1]);
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0], vec![Segment::styled("H", bold.clone())]);

    // Two cuts at [1, 2]: "H" then "e".
    let parts = Segment::divide(&segments, &[1, 2]);
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0], vec![Segment::styled("H", bold.clone())]);
    assert_eq!(parts[1], vec![Segment::styled("e", bold.clone())]);
}

// ---------------------------------------------------------------------------
// split_lines_terminator tests
// ---------------------------------------------------------------------------

/// Lines that end with `\n` must have the terminator flag set to `true`;
/// the last line (no trailing newline) must have it set to `false`.
/// Ported from `test_split_lines_terminator`.
#[test]
fn split_lines_terminator_keeps_newline() {
    // Basic two-line case.
    let segments = vec![Segment::text("Hello\nWorld")];
    let lines = Segment::split_lines_terminator(&segments);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].0, vec![Segment::text("Hello")]);
    assert!(lines[0].1, "first line should be marked as terminated");
    assert_eq!(lines[1].0, vec![Segment::text("World")]);
    assert!(!lines[1].1, "last line should NOT be marked as terminated");

    // Multi-segment version: terminator still works across segment boundaries.
    let styled = Style::parse("bold").unwrap();
    let segments2 = vec![
        Segment::styled("Line1\n", styled.clone()),
        Segment::text("Line2"),
    ];
    let lines2 = Segment::split_lines_terminator(&segments2);
    assert_eq!(lines2.len(), 2);
    assert!(
        lines2[0].1,
        "first line (bold) should be marked as terminated"
    );
    assert!(
        !lines2[1].1,
        "second line should NOT be marked as terminated"
    );
    assert_eq!(lines2[1].0, vec![Segment::text("Line2")]);
}
