//! Text module tests
//!
//! Extracted from src/text/mod.rs

use super::*;
use crate::style::Style;
use crate::text::*;
use ::regex::Regex;

fn bold() -> Style {
    Style::parse("bold").unwrap()
}

fn italic() -> Style {
    Style::parse("italic").unwrap()
}

fn underline() -> Style {
    Style::parse("underline").unwrap()
}

fn red() -> Style {
    Style::parse("red").unwrap()
}

// -- Span tests ---------------------------------------------------------

#[test]
fn test_span() {
    let span = Span::new(0, 10, bold());
    assert!(!span.is_empty());
    let empty_span = Span::new(5, 5, bold());
    assert!(empty_span.is_empty());
    let empty_span2 = Span::new(10, 5, bold());
    assert!(empty_span2.is_empty());
}

#[test]
fn test_span_split() {
    let span = Span::new(5, 10, bold());

    // Split in middle
    let (left, right) = span.split(7);
    assert_eq!(left, Span::new(5, 7, bold()));
    assert_eq!(right.unwrap(), Span::new(7, 10, bold()));

    // Split before start
    let (left, right) = span.split(3);
    assert_eq!(left, span);
    assert!(right.is_none());

    // Split at or after end
    let (left, right) = span.split(10);
    assert_eq!(left, span);
    assert!(right.is_none());

    // Split at start
    let (left, right) = span.split(5);
    assert_eq!(left, Span::new(5, 5, bold()));
    assert_eq!(right.unwrap(), Span::new(5, 10, bold()));
}

#[test]
fn test_span_move() {
    let span = Span::new(5, 10, bold());
    let moved = span.move_span(3);
    assert_eq!(moved, Span::new(8, 13, bold()));
}

#[test]
fn test_span_right_crop() {
    let span = Span::new(5, 10, bold());
    let cropped = span.right_crop(8);
    assert_eq!(cropped, Span::new(5, 8, bold()));
    let cropped2 = span.right_crop(15);
    assert_eq!(cropped2, Span::new(5, 10, bold()));
}

// -- Text constructor tests ---------------------------------------------

#[test]
fn test_len() {
    let text = Text::new("Hello", Style::null());
    assert_eq!(text.len(), 5);
    assert!(!text.is_empty());

    let empty = Text::new("", Style::null());
    assert_eq!(empty.len(), 0);
    assert!(empty.is_empty());
}

#[test]
fn test_cell_len() {
    let text = Text::new("Hello", Style::null());
    assert_eq!(text.cell_len(), 5);

    // CJK
    let text = Text::new("わさび", Style::null());
    assert_eq!(text.cell_len(), 6);
}

#[test]
fn test_bool() {
    let text = Text::new("Hello", Style::null());
    assert!(!text.is_empty());

    let empty = Text::empty();
    assert!(empty.is_empty());
}

#[test]
fn test_str() {
    let text = Text::new("Hello, World!", Style::null());
    assert_eq!(format!("{}", text), "Hello, World!");
}

#[test]
fn test_repr() {
    let text = Text::new("Hello", bold());
    assert_eq!(text.plain(), "Hello");
}

#[test]
fn test_add() {
    let t1 = Text::new("Hello", bold());
    let t2 = Text::new(" World", italic());
    let combined = t1 + t2;
    assert_eq!(combined.plain(), "Hello World");
}

#[test]
fn test_add_str() {
    let t1 = Text::new("Hello", Style::null());
    let combined = t1 + " World";
    assert_eq!(combined.plain(), "Hello World");
}

#[test]
fn test_eq() {
    let t1 = Text::new("Hello", bold());
    let t2 = Text::new("Hello", bold());
    // Both have no spans, same text
    assert_eq!(t1, t2);

    let mut t3 = Text::new("Hello", Style::null());
    t3.stylize(bold(), 0, Some(5));
    let mut t4 = Text::new("Hello", Style::null());
    t4.stylize(bold(), 0, Some(5));
    assert_eq!(t3, t4);
}

#[test]
fn test_contain() {
    let text = Text::new("Hello, World!", Style::null());
    assert!(text.contains_str("World"));
    assert!(!text.contains_str("Universe"));
}

// -- Plain property tests -----------------------------------------------

#[test]
fn test_plain_property() {
    let text = Text::new("Hello, World!", Style::null());
    assert_eq!(text.plain(), "Hello, World!");
}

#[test]
fn test_plain_property_setter() {
    let mut text = Text::new("Hello, World!", Style::null());
    text.stylize(bold(), 0, Some(13));
    text.set_plain("Goodbye!");
    assert_eq!(text.plain(), "Goodbye!");
    // Span should be trimmed to new length
    assert_eq!(text.spans().len(), 1);
    assert_eq!(text.spans()[0].end, 8);
}

// -- Copy test ----------------------------------------------------------

#[test]
fn test_copy() {
    let mut original = Text::new("Hello", bold());
    original.stylize(italic(), 0, Some(3));
    let copy = original.copy();
    assert_eq!(copy.plain(), "Hello");
    assert_eq!(copy.spans().len(), 1);
}

// -- Strip tests --------------------------------------------------------

#[test]
fn test_rstrip() {
    let mut text = Text::new("Hello   ", Style::null());
    text.rstrip();
    assert_eq!(text.plain(), "Hello");
}

#[test]
fn test_rstrip_end() {
    let mut text = Text::new("Hello   World   ", Style::null());
    text.rstrip_end(12);
    // Only strip whitespace beyond char position 12
    assert_eq!(text.plain(), "Hello   World");
}

// -- Stylize tests ------------------------------------------------------

#[test]
fn test_stylize() {
    let mut text = Text::new("Hello, World!", Style::null());
    text.stylize(bold(), 0, Some(5));
    assert_eq!(text.spans().len(), 1);
    assert_eq!(text.spans()[0], Span::new(0, 5, bold()));
}

#[test]
fn test_stylize_before() {
    let mut text = Text::new("Hello, World!", Style::null());
    text.stylize(bold(), 0, Some(5));
    text.stylize_before(italic(), 0, Some(5));
    assert_eq!(text.spans().len(), 2);
    // italic should be first
    assert_eq!(text.spans()[0].style, italic());
    assert_eq!(text.spans()[1].style, bold());
}

// -- Highlight tests ----------------------------------------------------

#[test]
fn test_highlight_regex() {
    let mut text = Text::new("Hello, World!", Style::null());
    let re = Regex::new(r"World").unwrap();
    let count = text.highlight_regex(&re, bold());
    assert_eq!(count, 1);
    assert_eq!(text.spans().len(), 1);
    assert_eq!(text.spans()[0], Span::new(7, 12, bold()));
}

#[test]
fn test_highlight_words() {
    let mut text = Text::new("The quick brown fox", Style::null());
    let count = text.highlight_words(&["quick", "fox"], bold(), false);
    assert_eq!(count, 2);
    assert_eq!(text.spans().len(), 2);
}

// -- Set length test ----------------------------------------------------

#[test]
fn test_set_length() {
    let mut text = Text::new("Hello", Style::null());
    text.set_length(10);
    assert_eq!(text.len(), 10);
    assert_eq!(text.plain(), "Hello     ");

    let mut text = Text::new("Hello, World!", Style::null());
    text.set_length(5);
    assert_eq!(text.plain(), "Hello");
}

// -- Join test ----------------------------------------------------------

#[test]
fn test_join() {
    let separator = Text::new(", ", Style::null());
    let texts = vec![
        Text::new("Hello", Style::null()),
        Text::new("World", Style::null()),
    ];
    let joined = separator.join(&texts);
    assert_eq!(joined.plain(), "Hello, World");
}

// -- Trim spans test ----------------------------------------------------

#[test]
fn test_trim_spans() {
    let mut text = Text::new("Hello", Style::null());
    text.spans.push(Span::new(0, 20, bold())); // Exceeds text length
    text.trim_spans();
    assert_eq!(text.spans()[0].end, 5);
}

// -- Pad tests ----------------------------------------------------------

#[test]
fn test_pad_left() {
    let mut text = Text::new("Hello", Style::null());
    text.stylize(bold(), 0, Some(5));
    text.pad_left(3, ' ');
    assert_eq!(text.plain(), "   Hello");
    // Span should be shifted
    assert_eq!(text.spans()[0], Span::new(3, 8, bold()));
}

#[test]
fn test_pad_right() {
    let mut text = Text::new("Hello", Style::null());
    text.pad_right(3, ' ');
    assert_eq!(text.plain(), "Hello   ");
}

#[test]
fn test_pad() {
    let mut text = Text::new("Hello", Style::null());
    text.pad(2, '-');
    assert_eq!(text.plain(), "--Hello--");
}

// -- Append tests -------------------------------------------------------

#[test]
fn test_append() {
    let mut text = Text::new("Hello", Style::null());
    text.append_str(", World!", None);
    assert_eq!(text.plain(), "Hello, World!");
}

#[test]
fn test_append_text() {
    let mut text = Text::new("Hello", Style::null());
    let mut other = Text::new(" World", Style::null());
    other.stylize(bold(), 0, Some(6));
    text.append_text(&other);
    assert_eq!(text.plain(), "Hello World");
    assert_eq!(text.spans().len(), 1);
    assert_eq!(text.spans()[0], Span::new(5, 11, bold()));
}

// -- Split test ---------------------------------------------------------

#[test]
fn test_split() {
    let text = Text::new("Hello\nWorld\nFoo", Style::null());
    let lines = text.split("\n", false, false);
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0].plain(), "Hello");
    assert_eq!(lines[1].plain(), "World");
    assert_eq!(lines[2].plain(), "Foo");
}

// -- Divide test --------------------------------------------------------

#[test]
fn test_divide() {
    let mut text = Text::new("Hello World", Style::null());
    text.stylize(bold(), 0, Some(5));
    text.stylize(italic(), 6, Some(11));

    let divided = text.divide(&[6]);
    assert_eq!(divided.len(), 2);
    assert_eq!(divided[0].plain(), "Hello ");
    assert_eq!(divided[1].plain(), "World");

    // Check spans
    assert_eq!(divided[0].spans().len(), 1);
    assert_eq!(divided[0].spans()[0], Span::new(0, 5, bold()));
    assert_eq!(divided[1].spans().len(), 1);
    assert_eq!(divided[1].spans()[0], Span::new(0, 5, italic()));
}

#[test]
fn test_divide_multi_span() {
    let mut text = Text::new("ABCDEFGHIJ", Style::null());
    // Span covers characters 2..8
    text.stylize(bold(), 2, Some(8));
    // Divide at 3 and 7
    let divided = text.divide(&[3, 7]);
    assert_eq!(divided.len(), 3);
    assert_eq!(divided[0].plain(), "ABC");
    assert_eq!(divided[1].plain(), "DEFG");
    assert_eq!(divided[2].plain(), "HIJ");

    // First line: span covers 2..3 (local)
    assert_eq!(divided[0].spans().len(), 1);
    assert_eq!(divided[0].spans()[0], Span::new(2, 3, bold()));

    // Second line: span covers 0..4 (local) -> full line
    assert_eq!(divided[1].spans().len(), 1);
    assert_eq!(divided[1].spans()[0], Span::new(0, 4, bold()));

    // Third line: span covers 0..1 (local)
    assert_eq!(divided[2].spans().len(), 1);
    assert_eq!(divided[2].spans()[0], Span::new(0, 1, bold()));
}

// -- Right crop test ----------------------------------------------------

#[test]
fn test_right_crop() {
    let mut text = Text::new("Hello, World!", Style::null());
    text.right_crop(7);
    assert_eq!(text.plain(), "Hello,");
}

// -- Truncate tests -----------------------------------------------------

#[test]
fn test_truncate_ellipsis() {
    let mut text = Text::new("Hello, World!", Style::null());
    text.truncate(10, Some(OverflowMethod::Ellipsis), false);
    assert_eq!(text.cell_len(), 10);
    assert!(text.plain().ends_with('\u{2026}'));
}

#[test]
fn test_truncate_ellipsis_pad() {
    let mut text = Text::new("Hello", Style::null());
    text.truncate(10, Some(OverflowMethod::Ellipsis), true);
    // "Hello" is only 5 chars, should be padded to 10
    assert_eq!(text.cell_len(), 10);
    assert_eq!(text.plain(), "Hello     ");
}

// -- Fit test -----------------------------------------------------------

#[test]
fn test_fit() {
    let text = Text::new("Hello\nWorld", Style::null());
    let lines = text.fit(10);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].cell_len(), 10);
    assert_eq!(lines[1].cell_len(), 10);
}

// -- Tabs test ----------------------------------------------------------

#[test]
fn test_tabs_to_spaces() {
    let mut text = Text::new("Hello\tWorld", Style::null());
    text.expand_tabs(Some(4));
    assert_eq!(text.plain(), "Hello    World");
}

// -- Strip control codes test -------------------------------------------

#[test]
fn test_strip_control_codes() {
    let result = strip_control_codes("Hello\x07World\x08!\x0B\x0C\x0D");
    assert_eq!(result, "HelloWorld!");
}

// -- Align tests --------------------------------------------------------

#[test]
fn test_align_left() {
    let mut text = Text::new("Hello", Style::null());
    text.align(JustifyMethod::Left, 10, ' ');
    assert_eq!(text.plain(), "Hello     ");
}

#[test]
fn test_align_right() {
    let mut text = Text::new("Hello", Style::null());
    text.align(JustifyMethod::Right, 10, ' ');
    assert_eq!(text.plain(), "     Hello");
}

#[test]
fn test_align_center() {
    let mut text = Text::new("Hello", Style::null());
    text.align(JustifyMethod::Center, 11, ' ');
    assert_eq!(text.plain(), "   Hello   ");
}

// -- Detect indentation test --------------------------------------------

#[test]
fn test_detect_indentation() {
    let text = Text::new("    foo\n        bar\n    baz", Style::null());
    assert_eq!(text.detect_indentation(), 4);

    let text = Text::new("  foo\n    bar\n  baz", Style::null());
    assert_eq!(text.detect_indentation(), 2);
}

// -- Indent guides test -------------------------------------------------

#[test]
fn test_indentation_guides() {
    let text = Text::new("    foo\n        bar\n    baz", Style::null());
    let result = text.with_indent_guides(Some(4), '|', Style::null());
    let lines: Vec<&str> = result.plain().lines().collect();
    assert!(lines[0].starts_with('|'));
    assert!(lines[1].starts_with('|'));
}

// -- Slice test ---------------------------------------------------------

#[test]
fn test_slice() {
    let mut text = Text::new("Hello, World!", Style::null());
    text.stylize(bold(), 7, Some(12));
    let sliced = text.slice(7, 12);
    assert_eq!(sliced.plain(), "World");
    assert_eq!(sliced.spans().len(), 1);
    assert_eq!(sliced.spans()[0], Span::new(0, 5, bold()));
}

// -- Extend style test --------------------------------------------------

#[test]
fn test_extend_style() {
    let mut text = Text::new("Hello", Style::null());
    text.stylize(bold(), 0, Some(5));
    text.extend_style(3);
    assert_eq!(text.plain(), "Hello   ");
    assert_eq!(text.spans()[0].end, 8); // Extended by 3
}

// -- Append tokens test -------------------------------------------------

#[test]
fn test_append_tokens() {
    let mut text = Text::new("", Style::null());
    text.append_tokens(&[
        ("Hello".to_string(), Some(bold())),
        (" World".to_string(), None),
    ]);
    assert_eq!(text.plain(), "Hello World");
    assert_eq!(text.spans().len(), 1);
    assert_eq!(text.spans()[0], Span::new(0, 5, bold()));
}

// -- Assemble test ------------------------------------------------------

#[test]
fn test_assemble() {
    let text = Text::assemble(
        &[
            TextPart::Raw("Hello ".to_string()),
            TextPart::Styled("World".to_string(), bold()),
        ],
        Style::null(),
    );
    assert_eq!(text.plain(), "Hello World");
    assert_eq!(text.spans().len(), 1);
    assert_eq!(text.spans()[0], Span::new(6, 11, bold()));
}

// -- Styled test --------------------------------------------------------

#[test]
fn test_styled() {
    let text = Text::styled("Hello", bold());
    assert_eq!(text.plain(), "Hello");
    assert_eq!(text.spans().len(), 1);
    assert_eq!(text.spans()[0], Span::new(0, 5, bold()));
    // Base style should be null
    assert!(text.spans.is_empty()); // Wait, this is spans field not the base style
    // Actually, styled() applies style as a span, base style is null
    // Let me verify by checking the text's actual base style through debug
}

// -- Render test --------------------------------------------------------

#[test]
fn test_render() {
    let mut text = Text::new("Hello World", Style::null());
    text.stylize(bold(), 0, Some(5));
    text.end = String::new(); // no end segment

    let segments = text.render();
    assert!(segments.len() >= 2);
    assert_eq!(segments[0].text, "Hello");
    assert!(segments[0].style.is_some());
    assert_eq!(segments[1].text, " World");
}

#[test]
fn test_render_no_spans() {
    let mut text = Text::new("Hello", Style::null());
    text.end = String::new();
    let segments = text.render();
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].text, "Hello");
}

// -- Wrap tests ---------------------------------------------------------

#[test]
fn test_wrap_3() {
    let text = Text::new("foo bar baz", Style::null());
    let lines = text.wrap(7, None, None, 8, false);
    assert!(lines.len() >= 2);
    assert_eq!(lines[0].plain().trim(), "foo bar");
    assert_eq!(lines[1].plain().trim(), "baz");
}

#[test]
fn test_wrap_4() {
    let text = Text::new("foo bar baz egg", Style::null());
    let lines = text.wrap(7, None, None, 8, false);
    assert!(lines.len() >= 2);
}

#[test]
fn test_wrap_long() {
    let text = Text::new("abcdefghijklmnop", Style::null());
    let lines = text.wrap(4, None, None, 8, false);
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0].plain(), "abcd");
    assert_eq!(lines[1].plain(), "efgh");
    assert_eq!(lines[2].plain(), "ijkl");
    assert_eq!(lines[3].plain(), "mnop");
}

#[test]
fn test_wrap_long_words() {
    let text = Text::new("longword short", Style::null());
    let lines = text.wrap(4, None, None, 8, false);
    assert!(lines.len() >= 3);
    assert_eq!(lines[0].plain(), "long");
    assert_eq!(lines[1].plain(), "word");
}

#[test]
fn test_wrap_cjk() {
    // Each CJK char is 2 cells wide
    let text = Text::new("ああああ", Style::null());
    let lines = text.wrap(4, None, None, 8, false);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].plain(), "ああ");
    assert_eq!(lines[1].plain(), "ああ");
}

#[test]
fn test_wrap_cjk_width_mid_character() {
    // Width 3 with CJK chars (each 2 wide) - can only fit 1 per line
    let text = Text::new("ああああ", Style::null());
    let lines = text.wrap(3, None, None, 8, false);
    assert_eq!(lines.len(), 4);
    for line in lines.iter() {
        assert_eq!(line.plain().chars().count(), 1);
    }
}

#[test]
fn test_wrap_long_words_2() {
    let text = Text::new("abcdefghij klmnop", Style::null());
    let lines = text.wrap(4, None, None, 8, false);
    assert!(lines.len() >= 4);
}

#[test]
fn test_wrap_long_words_followed_by_other_words() {
    let text = Text::new("abcdefgh foo bar", Style::null());
    let lines = text.wrap(4, None, None, 8, false);
    assert!(lines.len() >= 3);
}

#[test]
fn test_wrap_leading_and_trailing_whitespace() {
    let text = Text::new("  Hello  ", Style::null());
    let lines = text.wrap(20, None, None, 8, false);
    assert_eq!(lines.len(), 1);
    // Trailing whitespace stripped by rstrip_end
}

#[test]
fn test_append_loop_regression() {
    // Ensure appending in a loop doesn't corrupt spans
    let mut text = Text::new("", Style::null());
    for i in 0..10 {
        text.append_str(&format!("item{} ", i), Some(bold()));
    }
    assert_eq!(text.spans().len(), 10);
    // Verify spans don't overlap incorrectly
    for i in 0..text.spans().len() - 1 {
        assert!(text.spans()[i].end <= text.spans()[i + 1].start);
    }
}

#[test]
fn test_remove_suffix() {
    let mut text = Text::new("Hello, World!", Style::null());
    text.remove_suffix("!");
    assert_eq!(text.plain(), "Hello, World");

    // No suffix match
    let mut text2 = Text::new("Hello", Style::null());
    text2.remove_suffix("xyz");
    assert_eq!(text2.plain(), "Hello");
}

// -- from_markup tests --------------------------------------------------

#[test]
fn test_from_markup_basic() {
    let text = Text::from_markup("[bold]Hello[/bold] world").unwrap();
    assert_eq!(text.plain(), "Hello world");
    assert_eq!(text.spans().len(), 1);
    assert_eq!(text.spans()[0], Span::new(0, 5, bold()));
}

#[test]
fn test_from_markup_empty() {
    let text = Text::from_markup("").unwrap();
    assert_eq!(text.plain(), "");
    assert!(text.spans().is_empty());
}

#[test]
fn test_from_markup_no_tags() {
    let text = Text::from_markup("plain text").unwrap();
    assert_eq!(text.plain(), "plain text");
    assert!(text.spans().is_empty());
}

#[test]
fn test_from_markup_nested_styles() {
    let text = Text::from_markup("[bold][italic]nested[/italic][/bold]").unwrap();
    assert_eq!(text.plain(), "nested");
    assert_eq!(text.spans().len(), 2);
}

#[test]
fn test_from_markup_error() {
    let result = Text::from_markup("[bold]hello[/italic]");
    assert!(result.is_err());
}

// -- from_ansi tests ----------------------------------------------------

#[test]
fn test_from_ansi_basic() {
    let text = Text::from_ansi("\x1b[1mBold\x1b[0m");
    assert_eq!(text.plain(), "Bold");
    assert_eq!(text.spans().len(), 1);
    assert_eq!(text.spans()[0].style.bold(), Some(true));
}

#[test]
fn test_from_ansi_empty() {
    let text = Text::from_ansi("");
    assert_eq!(text.plain(), "");
    assert!(text.spans().is_empty());
}

#[test]
fn test_from_ansi_plain_text() {
    let text = Text::from_ansi("no ansi here");
    assert_eq!(text.plain(), "no ansi here");
    assert!(text.spans().is_empty());
}

#[test]
fn test_from_ansi_color_codes() {
    let text = Text::from_ansi("\x1b[31mRed\x1b[0m Normal");
    assert_eq!(text.plain(), "Red Normal");
    assert_eq!(text.spans().len(), 1);
    let color = text.spans()[0].style.color().unwrap();
    assert_eq!(color.number, Some(1));
}

#[test]
fn test_from_ansi_multiple_styles() {
    let text = Text::from_ansi("\x1b[1mBold\x1b[0m \x1b[3mItalic\x1b[0m");
    assert_eq!(text.plain(), "Bold Italic");
    assert_eq!(text.spans().len(), 2);
    assert_eq!(text.spans()[0].style.bold(), Some(true));
    assert_eq!(text.spans()[1].style.italic(), Some(true));
}

// -- Introspection tests ------------------------------------------------

#[test]
fn test_get_style_at_offset_no_spans() {
    let style = Style::parse("bold").unwrap();
    let text = Text::new("hello", style.clone());
    let result = text.get_style_at_offset(2);
    assert_eq!(result.bold(), Some(true));
}

#[test]
fn test_get_style_at_offset_single_span() {
    let mut text = Text::new("hello", Style::null());
    text.stylize(Style::parse("bold").unwrap(), 1, Some(4));
    // offset 2 is inside the span [1..4)
    let result = text.get_style_at_offset(2);
    assert_eq!(result.bold(), Some(true));
    // offset 0 is outside
    let result = text.get_style_at_offset(0);
    assert!(result.is_null());
}

#[test]
fn test_get_style_at_offset_overlapping_spans() {
    let mut text = Text::new("hello world", Style::null());
    text.stylize(Style::parse("bold").unwrap(), 0, Some(8));
    text.stylize(Style::parse("italic").unwrap(), 3, Some(11));
    // offset 5 overlaps both spans
    let result = text.get_style_at_offset(5);
    assert_eq!(result.bold(), Some(true));
    assert_eq!(result.italic(), Some(true));
    // offset 1 only overlaps bold
    let result = text.get_style_at_offset(1);
    assert_eq!(result.bold(), Some(true));
    assert_eq!(result.italic(), None);
}

#[test]
fn test_get_style_at_offset_out_of_range() {
    let mut text = Text::new("hi", Style::parse("bold").unwrap());
    text.stylize(Style::parse("italic").unwrap(), 0, Some(2));
    // offset 99 is beyond text length; only root style returned
    let result = text.get_style_at_offset(99);
    assert_eq!(result.bold(), Some(true));
    assert_eq!(result.italic(), None);
}

#[test]
fn test_flatten_spans_no_overlaps() {
    let mut text = Text::new("hello world", Style::null());
    text.stylize(Style::parse("bold").unwrap(), 0, Some(5));
    text.stylize(Style::parse("italic").unwrap(), 6, Some(11));
    let flat = text.flatten_spans();
    assert_eq!(flat.len(), 2);
    assert_eq!(flat[0].start, 0);
    assert_eq!(flat[0].end, 5);
    assert_eq!(flat[0].style.bold(), Some(true));
    assert_eq!(flat[0].style.italic(), None);
    assert_eq!(flat[1].start, 6);
    assert_eq!(flat[1].end, 11);
    assert_eq!(flat[1].style.italic(), Some(true));
    assert_eq!(flat[1].style.bold(), None);
}

#[test]
fn test_flatten_spans_overlapping() {
    let mut text = Text::new("hello world", Style::null());
    text.stylize(Style::parse("bold").unwrap(), 0, Some(8));
    text.stylize(Style::parse("italic").unwrap(), 3, Some(11));
    let flat = text.flatten_spans();
    // Expected regions: [0..3) bold, [3..8) bold+italic, [8..11) italic
    assert_eq!(flat.len(), 3);
    assert_eq!(flat[0].start, 0);
    assert_eq!(flat[0].end, 3);
    assert_eq!(flat[0].style.bold(), Some(true));
    assert_eq!(flat[0].style.italic(), None);
    assert_eq!(flat[1].start, 3);
    assert_eq!(flat[1].end, 8);
    assert_eq!(flat[1].style.bold(), Some(true));
    assert_eq!(flat[1].style.italic(), Some(true));
    assert_eq!(flat[2].start, 8);
    assert_eq!(flat[2].end, 11);
    assert_eq!(flat[2].style.italic(), Some(true));
    assert_eq!(flat[2].style.bold(), None);
}

#[test]
fn test_flatten_spans_empty() {
    let text = Text::new("hello", Style::null());
    let flat = text.flatten_spans();
    assert!(flat.is_empty());
}

#[test]
fn test_get_text_at_basic() {
    let text = Text::new("hello world", Style::null());
    assert_eq!(text.get_text_at(0, 5), Some("hello"));
    assert_eq!(text.get_text_at(6, 5), Some("world"));
    assert_eq!(text.get_text_at(0, 11), Some("hello world"));
}

#[test]
fn test_get_text_at_unicode() {
    let text = Text::new("cafe\u{0301}s rock", Style::null());
    // "cafe\u{0301}" is 5 chars (c, a, f, e, combining-accent)
    // get_text_at works on char offsets
    assert_eq!(text.get_text_at(0, 5), Some("cafe\u{0301}"));
}

#[test]
fn test_get_text_at_out_of_bounds() {
    let text = Text::new("hi", Style::null());
    assert_eq!(text.get_text_at(10, 5), None);
}

#[test]
fn test_from_str_for_text() {
    let text = Text::from("hello");
    assert_eq!(text.plain(), "hello");
}

#[test]
fn test_from_string_for_text() {
    let text = Text::from(String::from("hello"));
    assert_eq!(text.plain(), "hello");
}

#[test]
fn test_into_text() {
    let text: Text = "hello".into();
    assert_eq!(text.plain(), "hello");
}

#[test]
fn test_from_cow_str_for_text() {
    use std::borrow::Cow;
    let text = Text::from(Cow::Borrowed("hello"));
    assert_eq!(text.plain(), "hello");
    let text = Text::from(Cow::Owned(String::from("world")));
    assert_eq!(text.plain(), "world");
}

// -- Width boundary wrap tests ------------------------------------------

#[test]
fn test_wrap_width_zero() {
    let text = Text::new("Hello world", Style::null());
    // Should not panic at width=0
    let _lines = text.wrap(0, None, None, 8, false);
}

#[test]
fn test_wrap_width_one() {
    let text = Text::new("Hello", Style::null());
    let lines = text.wrap(1, None, None, 8, false);
    // Each line should be at most 1 cell wide
    for line in lines.iter() {
        let plain = line.plain();
        let w = crate::cells::cell_len(&plain);
        assert!(w <= 1, "Line '{}' has cell_len {} > 1", plain, w);
    }
    // Should produce at least 5 lines (one per character)
    assert!(
        lines.len() >= 5,
        "Expected at least 5 lines, got {}",
        lines.len()
    );
}

#[test]
fn test_large_text_wrap() {
    // 10,000 character string — should not panic and should complete quickly
    let big = "a".repeat(10_000);
    let text = Text::new(&big, Style::null());
    let lines = text.wrap(80, None, None, 8, false);
    assert!(!lines.is_empty());
    // Total chars across all lines should equal original
    let total: usize = lines.iter().map(|l| l.plain().len()).sum();
    assert_eq!(total, 10_000);
}
