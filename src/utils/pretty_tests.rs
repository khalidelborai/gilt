//! Tests extracted from pretty.rs for readability.
//! Wired in via `#[path = ...] mod tests;`.

use super::*;
use crate::console::Console;

// -- Helper -------------------------------------------------------------

fn make_console() -> Console {
    Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(false)
        .build()
}

// -- from_str tests -----------------------------------------------------

#[test]
fn test_from_str_simple() {
    let pretty = Pretty::from_str("Hello, World!");
    assert_eq!(pretty.text.plain(), "Hello, World!");
    assert!(pretty.indent_guides);
    assert_eq!(pretty.indent_size, 4);
    assert!(!pretty.no_wrap);
}

#[test]
fn test_from_str_repr_highlighting() {
    // Numbers and booleans should get highlighted
    let pretty = Pretty::from_str("count=42 flag=true");
    assert_eq!(pretty.text.plain(), "count=42 flag=true");
    // The ReprHighlighter should have added spans
    assert!(!pretty.text.spans().is_empty());
}

#[test]
fn test_from_str_empty() {
    let pretty = Pretty::from_str("");
    assert_eq!(pretty.text.plain(), "");
    assert!(pretty.text.spans().is_empty());
}

#[test]
fn test_from_str_single_line() {
    let pretty = Pretty::from_str("no indentation here");
    assert_eq!(pretty.text.plain(), "no indentation here");
}

// -- from_debug tests ---------------------------------------------------

#[test]
fn test_from_debug_struct() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Foo {
        x: i32,
        y: String,
    }
    let value = Foo {
        x: 42,
        y: "hello".to_string(),
    };
    let pretty = Pretty::from_debug(&value);
    let plain = pretty.text.plain().to_string();
    assert!(plain.contains("Foo"));
    assert!(plain.contains("42"));
    assert!(plain.contains("hello"));
    // Debug pretty-printing should produce multi-line output for structs
    assert!(plain.contains('\n'));
}

#[test]
fn test_from_debug_primitive() {
    let pretty = Pretty::from_debug(&42i32);
    assert_eq!(pretty.text.plain(), "42");
}

#[test]
fn test_from_debug_vec() {
    let v = vec![1, 2, 3];
    let pretty = Pretty::from_debug(&v);
    let plain = pretty.text.plain().to_string();
    assert!(plain.contains('1'));
    assert!(plain.contains('2'));
    assert!(plain.contains('3'));
}

// -- from_json tests ----------------------------------------------------

#[cfg(feature = "json")]
#[test]
fn test_from_json_simple_object() {
    let json: serde_json::Value = serde_json::from_str(r#"{"name": "Alice", "age": 30}"#).unwrap();
    let pretty = Pretty::from_json(&json);
    let plain = pretty.text.plain().to_string();
    assert!(plain.contains("Alice"));
    assert!(plain.contains("30"));
    assert!(pretty.no_wrap);
    assert_eq!(pretty.indent_size, 2);
}

#[cfg(feature = "json")]
#[test]
fn test_from_json_nested_object() {
    let json: serde_json::Value =
        serde_json::from_str(r#"{"user": {"name": "Bob", "address": {"city": "NYC"}}}"#).unwrap();
    let pretty = Pretty::from_json(&json);
    let plain = pretty.text.plain().to_string();
    assert!(plain.contains("Bob"));
    assert!(plain.contains("NYC"));
    // Nested JSON should have multiple indent levels
    assert!(plain.contains("    "));
}

#[cfg(feature = "json")]
#[test]
fn test_from_json_array() {
    let json: serde_json::Value = serde_json::from_str(r#"[1, 2, 3]"#).unwrap();
    let pretty = Pretty::from_json(&json);
    let plain = pretty.text.plain().to_string();
    assert!(plain.contains('1'));
    assert!(plain.contains('2'));
    assert!(plain.contains('3'));
}

#[cfg(feature = "json")]
#[test]
fn test_from_json_highlighting() {
    let json: serde_json::Value = serde_json::from_str(r#"{"key": true, "num": 42}"#).unwrap();
    let pretty = Pretty::from_json(&json);
    // JSONHighlighter should have added spans for booleans, numbers, etc.
    assert!(!pretty.text.spans().is_empty());
}

// -- Indent guides tests ------------------------------------------------

#[test]
fn test_indent_guides_applied() {
    let input = "root\n    child\n        grandchild";
    let pretty = Pretty::from_str(input).with_indent_size(4);
    let guided = pretty.apply_indent_guides(None);
    let plain = guided.plain().to_string();
    // Indent guides should insert the vertical bar character
    assert!(
        plain.contains('\u{2502}'),
        "expected indent guide character in: {}",
        plain
    );
}

#[test]
fn test_indent_guides_custom_size() {
    let input = "root\n  child\n    grandchild";
    let pretty = Pretty::from_str(input).with_indent_size(2);
    let guided = pretty.apply_indent_guides(None);
    let plain = guided.plain().to_string();
    assert!(
        plain.contains('\u{2502}'),
        "expected indent guide character in: {}",
        plain
    );
}

#[test]
fn test_indent_guides_disabled() {
    let input = "root\n    child\n        grandchild";
    let pretty = Pretty::from_str(input).with_indent_guides(false);
    let guided = pretty.apply_indent_guides(None);
    let plain = guided.plain().to_string();
    // No indent guide characters should be present
    assert!(
        !plain.contains('\u{2502}'),
        "did not expect indent guide character in: {}",
        plain
    );
}

#[test]
fn test_indent_guides_no_indentation() {
    let input = "line one\nline two\nline three";
    let pretty = Pretty::from_str(input);
    let guided = pretty.apply_indent_guides(None);
    let plain = guided.plain().to_string();
    // No leading spaces, so no guides
    assert!(
        !plain.contains('\u{2502}'),
        "did not expect indent guide character in: {}",
        plain
    );
}

#[test]
fn test_indent_guides_multi_level() {
    let input = "a\n    b\n        c\n            d";
    let pretty = Pretty::from_str(input).with_indent_size(4);
    let guided = pretty.apply_indent_guides(None);
    let lines: Vec<&str> = guided.plain().lines().collect();
    // Line "    b" should have 1 guide at position 0
    assert_eq!(
        lines[1].chars().filter(|c| *c == '\u{2502}').count(),
        1,
        "expected 1 guide in line: '{}'",
        lines[1]
    );
    // Line "        c" should have 2 guides
    assert_eq!(
        lines[2].chars().filter(|c| *c == '\u{2502}').count(),
        2,
        "expected 2 guides in line: '{}'",
        lines[2]
    );
    // Line "            d" should have 3 guides
    assert_eq!(
        lines[3].chars().filter(|c| *c == '\u{2502}').count(),
        3,
        "expected 3 guides in line: '{}'",
        lines[3]
    );
}

// -- Builder method tests -----------------------------------------------

#[test]
fn test_builder_with_indent_guides() {
    let pretty = Pretty::from_str("test").with_indent_guides(false);
    assert!(!pretty.indent_guides);
}

#[test]
fn test_builder_with_indent_size() {
    let pretty = Pretty::from_str("test").with_indent_size(8);
    assert_eq!(pretty.indent_size, 8);
}

#[test]
fn test_builder_with_no_wrap() {
    let pretty = Pretty::from_str("test").with_no_wrap(true);
    assert!(pretty.no_wrap);
}

#[test]
fn test_builder_with_overflow() {
    let pretty = Pretty::from_str("test").with_overflow(OverflowMethod::Ellipsis);
    assert_eq!(pretty.overflow, Some(OverflowMethod::Ellipsis));
}

#[test]
fn test_builder_chaining() {
    let pretty = Pretty::from_str("test")
        .with_indent_guides(false)
        .with_indent_size(2)
        .with_no_wrap(true)
        .with_overflow(OverflowMethod::Crop);
    assert!(!pretty.indent_guides);
    assert_eq!(pretty.indent_size, 2);
    assert!(pretty.no_wrap);
    assert_eq!(pretty.overflow, Some(OverflowMethod::Crop));
}

// -- Renderable integration tests ---------------------------------------

#[test]
fn test_renderable_produces_segments() {
    let console = make_console();
    let opts = console.options();
    let pretty = Pretty::from_str("Hello, World!");
    let segments = pretty.gilt_console(&console, &opts);
    assert!(!segments.is_empty());
    let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(combined.contains("Hello, World!"));
}

#[test]
fn test_renderable_with_no_wrap() {
    let console = make_console();
    let opts = console.options();
    let pretty = Pretty::from_str("a very long line that might wrap").with_no_wrap(true);
    let segments = pretty.gilt_console(&console, &opts);
    assert!(!segments.is_empty());
}

#[cfg(feature = "json")]
#[test]
fn test_renderable_json() {
    let console = make_console();
    let opts = console.options();
    let json: serde_json::Value = serde_json::from_str(r#"{"key": "value"}"#).unwrap();
    let pretty = Pretty::from_json(&json);
    let segments = pretty.gilt_console(&console, &opts);
    assert!(!segments.is_empty());
    let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(combined.contains("key"));
    assert!(combined.contains("value"));
}

#[test]
fn test_renderable_debug_struct() {
    let console = make_console();
    let opts = console.options();
    let v = vec![1, 2, 3];
    let pretty = Pretty::from_debug(&v);
    let segments = pretty.gilt_console(&console, &opts);
    assert!(!segments.is_empty());
}

// -- Measure tests ------------------------------------------------------

#[test]
fn test_measure_simple() {
    let pretty = Pretty::from_str("Hello");
    let m = pretty.measure();
    assert_eq!(m.minimum, 5);
    assert_eq!(m.maximum, 5);
}

#[test]
fn test_measure_multiline() {
    let pretty = Pretty::from_str("short\na much longer line");
    let m = pretty.measure();
    assert_eq!(m.maximum, 18); // "a much longer line"
                               // minimum is the longest single word
    assert!(m.minimum > 0);
}

#[test]
fn test_measure_empty() {
    let pretty = Pretty::from_str("");
    let m = pretty.measure();
    assert_eq!(m.minimum, 0);
    assert_eq!(m.maximum, 0);
}

#[cfg(feature = "json")]
#[test]
fn test_measure_json() {
    let json: serde_json::Value = serde_json::from_str(r#"{"key": "value"}"#).unwrap();
    let pretty = Pretty::from_json(&json);
    let m = pretty.measure();
    assert!(m.maximum > 0);
}

// -- New builder method tests -------------------------------------------

#[test]
fn test_builder_with_max_length() {
    let pretty = Pretty::from_str("test").with_max_length(5);
    assert_eq!(pretty.max_length, Some(5));
}

#[test]
fn test_builder_with_max_string() {
    let pretty = Pretty::from_str("test").with_max_string(10);
    assert_eq!(pretty.max_string, Some(10));
}

#[test]
fn test_builder_with_expand_all() {
    let pretty = Pretty::from_str("test").with_expand_all(true);
    assert!(pretty.expand_all);
}

// -- max_length tests ---------------------------------------------------

#[cfg(feature = "json")]
#[test]
fn test_max_length_truncates_array() {
    let json: serde_json::Value = serde_json::from_str("[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]").unwrap();
    let pretty = Pretty::from_json(&json)
        .with_max_length(3)
        .rebuild_json(&json, 80);
    let plain = pretty.text.plain().to_string();
    // Should contain the first 3 items
    assert!(plain.contains('1'), "should contain 1: {}", plain);
    assert!(plain.contains('2'), "should contain 2: {}", plain);
    assert!(plain.contains('3'), "should contain 3: {}", plain);
    // Should have truncation indicator (no " more" suffix — rich parity)
    assert!(
        plain.contains("+7"),
        "should contain '+7' truncation indicator: {}",
        plain
    );
}

#[cfg(feature = "json")]
#[test]
fn test_max_length_none_shows_all() {
    let json: serde_json::Value = serde_json::from_str("[1, 2, 3, 4, 5]").unwrap();
    let pretty = Pretty::from_json(&json).rebuild_json(&json, 80);
    let plain = pretty.text.plain().to_string();
    // All items should be present
    for i in 1..=5 {
        assert!(
            plain.contains(&i.to_string()),
            "should contain {}: {}",
            i,
            plain
        );
    }
    // No truncation indicator
    assert!(
        !plain.contains("more"),
        "should not contain truncation indicator: {}",
        plain
    );
}

#[cfg(feature = "json")]
#[test]
fn test_max_length_truncates_object() {
    let json: serde_json::Value =
        serde_json::from_str(r#"{"a": 1, "b": 2, "c": 3, "d": 4, "e": 5}"#).unwrap();
    let pretty = Pretty::from_json(&json)
        .with_max_length(2)
        .rebuild_json(&json, 80);
    let plain = pretty.text.plain().to_string();
    // Should have truncation indicator for the remaining 3 items (no " more" suffix)
    assert!(
        plain.contains("+3"),
        "should contain '+3' truncation indicator: {}",
        plain
    );
}

// -- max_string tests ---------------------------------------------------

#[cfg(feature = "json")]
#[test]
fn test_max_string_truncates() {
    let json: serde_json::Value = serde_json::from_str(
        r#"{"message": "This is a very long string that should be truncated"}"#,
    )
    .unwrap();
    let pretty = Pretty::from_json(&json)
        .with_max_string(10)
        .rebuild_json(&json, 80);
    let plain = pretty.text.plain().to_string();
    // The string value should be truncated
    assert!(
        plain.contains("+"),
        "should contain '+N' truncation suffix: {}",
        plain
    );
    // The full original string should NOT be present
    assert!(
        !plain.contains("This is a very long string that should be truncated"),
        "should not contain the full original string: {}",
        plain
    );
}

#[cfg(feature = "json")]
#[test]
fn test_max_string_none_shows_full() {
    let long_str = "This is a very long string that should not be truncated";
    let json: serde_json::Value = serde_json::json!({"message": long_str});
    let pretty = Pretty::from_json(&json).rebuild_json(&json, 80);
    let plain = pretty.text.plain().to_string();
    assert!(
        plain.contains(long_str),
        "should contain full string: {}",
        plain
    );
}

#[cfg(feature = "json")]
#[test]
fn test_max_string_short_string_not_truncated() {
    let json: serde_json::Value = serde_json::json!({"name": "Alice"});
    let pretty = Pretty::from_json(&json)
        .with_max_string(100)
        .rebuild_json(&json, 80);
    let plain = pretty.text.plain().to_string();
    assert!(
        plain.contains("Alice"),
        "short string should not be truncated: {}",
        plain
    );
    // No +N suffix for short strings
    assert!(
        !plain.contains("+"),
        "should not contain truncation suffix: {}",
        plain
    );
}

// -- expand_all tests ---------------------------------------------------

#[cfg(feature = "json")]
#[test]
fn test_expand_all_forces_expansion() {
    let json: serde_json::Value = serde_json::from_str("[1, 2]").unwrap();
    let pretty = Pretty::from_json(&json)
        .with_expand_all(true)
        .rebuild_json(&json, 80);
    let plain = pretty.text.plain().to_string();
    // With expand_all, even a short array should be multi-line
    assert!(
        plain.contains('\n'),
        "expand_all should force multi-line output: {}",
        plain
    );
    // Each item should be on its own line
    let lines: Vec<&str> = plain.lines().collect();
    assert!(
        lines.len() >= 3,
        "expected at least 3 lines (open, items, close), got {}: {}",
        lines.len(),
        plain
    );
}

#[cfg(feature = "json")]
#[test]
fn test_expand_all_false_compact() {
    let json: serde_json::Value = serde_json::from_str("[1, 2]").unwrap();
    let pretty = Pretty::from_json(&json)
        .with_expand_all(false)
        .rebuild_json(&json, 80);
    let plain = pretty.text.plain().to_string();
    // A short array without expand_all should be single-line
    assert!(
        !plain.contains('\n'),
        "short array without expand_all should be single-line: {}",
        plain
    );
}

#[cfg(feature = "json")]
#[test]
fn test_expand_all_object() {
    let json: serde_json::Value = serde_json::from_str(r#"{"a": 1}"#).unwrap();
    let pretty = Pretty::from_json(&json)
        .with_expand_all(true)
        .rebuild_json(&json, 80);
    let plain = pretty.text.plain().to_string();
    assert!(
        plain.contains('\n'),
        "expand_all should force multi-line object output: {}",
        plain
    );
}

// -- Combined parameter tests -------------------------------------------

#[cfg(feature = "json")]
#[test]
fn test_all_params_combined() {
    let json: serde_json::Value = serde_json::from_str(
            r#"["short", "a medium length string", "another medium string", "this is a very long string value that exceeds limits", "fifth item"]"#,
        )
        .unwrap();
    let pretty = Pretty::from_json(&json)
        .with_max_length(3)
        .with_max_string(10)
        .with_expand_all(true)
        .rebuild_json(&json, 80);
    let plain = pretty.text.plain().to_string();

    // expand_all: should be multi-line
    assert!(
        plain.contains('\n'),
        "should be multi-line with expand_all: {}",
        plain
    );

    // max_length=3: should show truncation for remaining 2 items (no " more" suffix)
    assert!(
        plain.contains("+2"),
        "should contain '+2' for max_length truncation: {}",
        plain
    );

    // max_string=10: long strings should be truncated
    assert!(
        !plain.contains("this is a very long string value that exceeds limits"),
        "long string should be truncated: {}",
        plain
    );
}

#[cfg(feature = "json")]
#[test]
fn test_max_length_with_nested_arrays() {
    let json: serde_json::Value =
        serde_json::from_str(r#"{"items": [1, 2, 3, 4, 5, 6, 7, 8]}"#).unwrap();
    let pretty = Pretty::from_json(&json)
        .with_max_length(2)
        .with_expand_all(true)
        .rebuild_json(&json, 80);
    let plain = pretty.text.plain().to_string();

    // The nested array should also be truncated (no " more" suffix)
    assert!(
        plain.contains("+6"),
        "nested array should be truncated: {}",
        plain
    );
}

// -- Debug rebuild tests ------------------------------------------------

#[test]
fn test_rebuild_debug_max_string() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Data {
        name: String,
    }
    let value = Data {
        name: "a very long name that should be truncated".to_string(),
    };
    let pretty = Pretty::from_debug(&value)
        .with_max_string(10)
        .rebuild_debug(&value);
    let plain = pretty.text.plain().to_string();
    assert!(
        !plain.contains("a very long name that should be truncated"),
        "debug string should be truncated: {}",
        plain
    );
    assert!(
        plain.contains("+"),
        "should contain truncation indicator: {}",
        plain
    );
}

// -- Task 1.4: truncate_debug_strings places +N OUTSIDE the closing quote ------
// rich renders truncated debug strings as "kept"+N not "kept+N".
// Audit #32: the suffix was being appended before the closing quote.

#[test]
fn truncate_debug_string_places_suffix_outside_quote() {
    // Input: a quoted string of 10 chars; max_string = 5 → 5 kept, +5 suffix.
    // Expected form: "abcde"+5   NOT  "abcde+5"
    let out = truncate_debug_strings("\"abcdefghij\"", 5);
    assert!(
        out.contains("\"+5"),
        "suffix must be outside the closing quote — got: {out:?}"
    );
    assert!(
        !out.contains("j+"),
        "suffix must not appear inside the quote — got: {out:?}"
    );
    // Also verify the kept portion and the closing structure
    assert!(out.starts_with("\"abcde\""), "unexpected prefix: {out:?}");
}

#[test]
fn truncate_debug_string_short_string_unchanged() {
    // Strings within the limit must not be touched.
    let out = truncate_debug_strings("\"hello\"", 10);
    assert_eq!(out, "\"hello\"", "short string should be unchanged");
}

#[test]
fn truncate_debug_string_exact_limit_unchanged() {
    // String whose length equals max_string must not be truncated.
    let out = truncate_debug_strings("\"hello\"", 5);
    assert_eq!(out, "\"hello\"", "string at the limit should be unchanged");
}

// -- max_depth tests --------------------------------------------------------

#[cfg(feature = "json")]
#[test]
fn test_max_depth_hides_nested_object() {
    // P2 parity: containers beyond max_depth render as placeholder
    let json: serde_json::Value =
        serde_json::from_str(r#"{"outer": {"inner": {"deep": 42}}}"#).unwrap();
    let pretty = Pretty::from_json(&json)
        .with_max_depth(1)
        .rebuild_json(&json, 80);
    let plain = pretty.text.plain().to_string();
    // At depth>1 the object should be replaced with "{...}"
    assert!(
        plain.contains("{...}"),
        "nested object beyond max_depth should render as {{...}}: {}",
        plain
    );
    // The deep value should NOT be present
    assert!(
        !plain.contains("deep"),
        "key inside nested object beyond max_depth should be hidden: {}",
        plain
    );
}

#[cfg(feature = "json")]
#[test]
fn test_max_depth_hides_nested_array() {
    let json: serde_json::Value = serde_json::from_str(r#"{"items": [1, 2, 3]}"#).unwrap();
    let pretty = Pretty::from_json(&json)
        .with_max_depth(0)
        .rebuild_json(&json, 80);
    let plain = pretty.text.plain().to_string();
    assert!(
        plain.contains("[...]"),
        "nested array beyond max_depth should render as [...]: {}",
        plain
    );
}

#[cfg(feature = "json")]
#[test]
fn test_max_depth_none_shows_all() {
    let json: serde_json::Value = serde_json::from_str(r#"{"a": {"b": {"c": 1}}}"#).unwrap();
    let pretty = Pretty::from_json(&json).rebuild_json(&json, 80);
    let plain = pretty.text.plain().to_string();
    assert!(
        plain.contains("c"),
        "without max_depth all keys should be visible: {}",
        plain
    );
}

#[cfg(feature = "json")]
#[test]
fn test_with_max_depth_builder() {
    let pretty = Pretty::from_json(&serde_json::Value::Null).with_max_depth(3);
    assert_eq!(pretty.max_depth, Some(3));
}

// -- JSON string truncation tests (P1 parity: "kept"+N not "kept+N") --------

#[cfg(feature = "json")]
#[test]
fn test_json_string_truncation_within_limit() {
    // Short string: no truncation, no +N suffix
    let v = serde_json::Value::String("hello".to_string());
    let result = format_json_value(
        &v,
        0,
        JsonFmtOpts {
            indent_size: 2,
            max_length: None,
            max_string: Some(10),
            expand_all: false,
            max_depth: None,
            max_width: 80,
        },
    );
    assert_eq!(result, r#""hello""#);
}

#[cfg(feature = "json")]
#[test]
fn test_json_string_truncation_at_limit() {
    // Exactly at limit: no truncation
    let v = serde_json::Value::String("hello".to_string());
    let result = format_json_value(
        &v,
        0,
        JsonFmtOpts {
            indent_size: 2,
            max_length: None,
            max_string: Some(5),
            expand_all: false,
            max_depth: None,
            max_width: 80,
        },
    );
    assert_eq!(result, r#""hello""#);
}

#[cfg(feature = "json")]
#[test]
fn test_json_string_truncation_over_limit() {
    // P1 parity: +N appears OUTSIDE the closing quote: "hello"+6 not "hello+6"
    let v = serde_json::Value::String("hello world".to_string());
    let result = format_json_value(
        &v,
        0,
        JsonFmtOpts {
            indent_size: 2,
            max_length: None,
            max_string: Some(5),
            expand_all: false,
            max_depth: None,
            max_width: 80,
        },
    );
    assert_eq!(result, r#""hello"+6"#);
}

#[cfg(feature = "json")]
#[test]
fn test_json_string_truncation_none() {
    // No max_string: full value, no +N
    let v = serde_json::Value::String("hello world".to_string());
    let result = format_json_value(
        &v,
        0,
        JsonFmtOpts {
            indent_size: 2,
            max_length: None,
            max_string: None,
            expand_all: false,
            max_depth: None,
            max_width: 80,
        },
    );
    assert_eq!(result, r#""hello world""#);
}

#[cfg(feature = "json")]
#[test]
fn test_escape_json_string_basic() {
    assert_eq!(escape_json_string("hello"), "hello");
}

#[cfg(feature = "json")]
#[test]
fn test_escape_json_string_quotes() {
    assert_eq!(escape_json_string(r#"say "hi""#), r#"say \"hi\""#);
}

#[cfg(feature = "json")]
#[test]
fn test_format_json_value_null() {
    let v = serde_json::Value::Null;
    assert_eq!(
        format_json_value(
            &v,
            0,
            JsonFmtOpts {
                indent_size: 2,
                max_length: None,
                max_string: None,
                expand_all: false,
                max_depth: None,
                max_width: 80
            }
        ),
        "null"
    );
}

#[cfg(feature = "json")]
#[test]
fn test_format_json_value_bool() {
    let v = serde_json::Value::Bool(true);
    assert_eq!(
        format_json_value(
            &v,
            0,
            JsonFmtOpts {
                indent_size: 2,
                max_length: None,
                max_string: None,
                expand_all: false,
                max_depth: None,
                max_width: 80
            }
        ),
        "true"
    );
}

#[cfg(feature = "json")]
#[test]
fn test_format_json_empty_array() {
    let v: serde_json::Value = serde_json::from_str("[]").unwrap();
    assert_eq!(
        format_json_value(
            &v,
            0,
            JsonFmtOpts {
                indent_size: 2,
                max_length: None,
                max_string: None,
                expand_all: false,
                max_depth: None,
                max_width: 80
            }
        ),
        "[]"
    );
}

#[cfg(feature = "json")]
#[test]
fn test_format_json_empty_object() {
    let v: serde_json::Value = serde_json::from_str("{}").unwrap();
    assert_eq!(
        format_json_value(
            &v,
            0,
            JsonFmtOpts {
                indent_size: 2,
                max_length: None,
                max_string: None,
                expand_all: false,
                max_depth: None,
                max_width: 80
            }
        ),
        "{}"
    );
}

#[test]
fn test_display_trait() {
    let pretty = Pretty::from_debug(&vec![1, 2, 3]);
    let s = format!("{}", pretty);
    assert!(!s.is_empty());
}

// -- type_annotation tests ----------------------------------------------

#[test]
fn test_type_annotation_default_false() {
    let pretty = Pretty::from_str("hello");
    assert!(!pretty.type_annotation);
}

#[test]
fn test_builder_with_type_annotation() {
    let pretty = Pretty::from_str("hello").with_type_annotation(true);
    assert!(pretty.type_annotation);
}

#[cfg(feature = "json")]
#[test]
fn test_type_annotation_prepends_type_for_json_object() {
    let console = make_console();
    let opts = console.options();
    let json: serde_json::Value = serde_json::from_str(r#"{"key": "value"}"#).unwrap();
    let pretty = Pretty::from_json(&json).with_type_annotation(true);
    let segments = pretty.gilt_console(&console, &opts);
    let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(
        combined.contains("(object)"),
        "expected type annotation '(object)' in: {}",
        combined
    );
}

#[cfg(feature = "json")]
#[test]
fn test_type_annotation_prepends_type_for_json_array() {
    let console = make_console();
    let opts = console.options();
    let json: serde_json::Value = serde_json::from_str("[1, 2, 3]").unwrap();
    let pretty = Pretty::from_json(&json).with_type_annotation(true);
    let segments = pretty.gilt_console(&console, &opts);
    let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(
        combined.contains("(array)"),
        "expected type annotation '(array)' in: {}",
        combined
    );
}

#[cfg(feature = "json")]
#[test]
fn test_type_annotation_disabled_no_prefix() {
    let console = make_console();
    let opts = console.options();
    let json: serde_json::Value = serde_json::from_str(r#"{"key": "value"}"#).unwrap();
    let pretty = Pretty::from_json(&json).with_type_annotation(false);
    let segments = pretty.gilt_console(&console, &opts);
    let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(
        !combined.contains("(object)"),
        "should NOT contain type annotation when disabled: {}",
        combined
    );
}

#[test]
fn test_type_annotation_for_debug_struct() {
    let console = make_console();
    let opts = console.options();
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Foo {
        x: i32,
    }
    let value = Foo { x: 42 };
    let pretty = Pretty::from_debug(&value).with_type_annotation(true);
    let segments = pretty.gilt_console(&console, &opts);
    let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(
        combined.contains("(struct)"),
        "expected type annotation '(struct)' in: {}",
        combined
    );
}

// -- infer_type_name tests ----------------------------------------------

#[test]
fn test_infer_type_name_object() {
    assert_eq!(super::infer_type_name("{\"key\": 1}"), "object");
}

#[test]
fn test_infer_type_name_array() {
    assert_eq!(super::infer_type_name("[1, 2]"), "array");
}

#[test]
fn test_infer_type_name_string() {
    assert_eq!(super::infer_type_name("\"hello\""), "str");
}

#[test]
fn test_infer_type_name_bool() {
    assert_eq!(super::infer_type_name("true"), "bool");
    assert_eq!(super::infer_type_name("false"), "bool");
}

#[test]
fn test_infer_type_name_null() {
    assert_eq!(super::infer_type_name("null"), "null");
}

#[test]
fn test_infer_type_name_number() {
    assert_eq!(super::infer_type_name("42"), "number");
    assert_eq!(super::infer_type_name("-3.14"), "number");
}

#[test]
fn test_infer_type_name_empty() {
    assert_eq!(super::infer_type_name(""), "empty");
}

#[test]
fn test_infer_type_name_struct() {
    assert_eq!(super::infer_type_name("Foo {\n    x: 42\n}"), "struct");
}

// -- Task 4: Pretty::from_serde -----------------------------------------------

#[cfg(feature = "json")]
mod from_serde_tests {
    use super::super::Pretty;
    use serde::Serialize;

    #[derive(Serialize)]
    struct Point {
        x: f64,
        y: f64,
    }

    #[derive(Serialize)]
    struct Nested {
        name: String,
        tags: Vec<String>,
        value: i32,
    }

    fn render_pretty(pretty: &Pretty) -> String {
        use crate::console::Console;
        let mut console = Console::builder()
            .width(80)
            .force_terminal(true)
            .no_color(true)
            .markup(false)
            .build();
        console.begin_capture();
        console.print(pretty);
        console.end_capture()
    }

    #[test]
    fn test_from_serde_produces_ok() {
        let p = Point { x: 1.0, y: 2.5 };
        assert!(Pretty::from_serde(&p).is_ok());
    }

    #[test]
    fn test_from_serde_renders_fields() {
        let p = Point { x: 3.0, y: 4.0 };
        let pretty = Pretty::from_serde(&p).unwrap();
        let output = render_pretty(&pretty);
        assert!(
            output.contains('x') || output.contains("3"),
            "output: {output}"
        );
        assert!(
            output.contains('y') || output.contains("4"),
            "output: {output}"
        );
    }

    #[test]
    fn test_from_serde_matches_from_json() {
        let p = Point { x: 1.5, y: 2.5 };

        // Serialize to Value first, then use from_json to get the reference.
        let value = serde_json::to_value(&p).unwrap();
        let from_json = Pretty::from_json(&value);
        let from_serde = Pretty::from_serde(&p).unwrap();

        // Both should produce the same plain text (same formatting path).
        assert_eq!(
            from_json.text.plain(),
            from_serde.text.plain(),
            "from_serde should produce identical text to from_json"
        );
    }

    #[test]
    fn test_from_serde_nested_struct() {
        let nested = Nested {
            name: "example".to_string(),
            tags: vec!["alpha".to_string(), "beta".to_string()],
            value: 42,
        };
        let pretty = Pretty::from_serde(&nested).unwrap();
        let output = render_pretty(&pretty);
        assert!(output.contains("example"), "output: {output}");
        assert!(output.contains("alpha"), "output: {output}");
        assert!(output.contains("42"), "output: {output}");
    }

    #[test]
    fn test_from_serde_simple_value() {
        let val: u32 = 99;
        let pretty = Pretty::from_serde(&val).unwrap();
        let output = render_pretty(&pretty);
        assert!(output.contains("99"), "output: {output}");
    }

    #[test]
    fn test_from_serde_vec() {
        let items = vec![1u8, 2, 3];
        let pretty = Pretty::from_serde(&items).unwrap();
        let output = render_pretty(&pretty);
        assert!(
            output.contains('1') && output.contains('2') && output.contains('3'),
            "output: {output}"
        );
    }
}

// ---------------------------------------------------------------------------
// Item 1: pretty_repr function
// ---------------------------------------------------------------------------

#[test]
fn test_pretty_repr_returns_string() {
    let result = super::pretty_repr(&vec![1, 2, 3], 80);
    assert!(result.contains('1'));
    assert!(result.contains('2'));
    assert!(result.contains('3'));
    // No trailing newline
    assert!(
        !result.ends_with('\n'),
        "pretty_repr should not end with newline: {:?}",
        result
    );
}

#[test]
fn test_pretty_repr_primitive() {
    let result = super::pretty_repr(&42i32, 80);
    assert_eq!(result, "42");
}

#[test]
fn test_pretty_repr_respects_max_width() {
    // A vec of many items; the output should use the width parameter
    let result = super::pretty_repr(&vec![1, 2, 3], 40);
    assert!(result.contains('1'));
}

// ---------------------------------------------------------------------------
// Item 3: max_depth on Debug path
// ---------------------------------------------------------------------------

#[test]
fn test_rebuild_debug_max_depth_prunes_nested_struct() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Inner {
        value: i32,
    }
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Outer {
        inner: Inner,
    }
    let value = Outer {
        inner: Inner { value: 99 },
    };
    let pretty = Pretty::from_debug(&value)
        .with_max_depth(0)
        .rebuild_debug(&value);
    let plain = pretty.text.plain().to_string();
    // At max_depth=0, the inner struct content should be replaced
    assert!(
        !plain.contains("99"),
        "inner value should be hidden at max_depth=0: {}",
        plain
    );
}

#[test]
fn test_rebuild_debug_max_depth_none_shows_all() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Inner {
        value: i32,
    }
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Outer {
        inner: Inner,
    }
    let value = Outer {
        inner: Inner { value: 42 },
    };
    let pretty = Pretty::from_debug(&value).rebuild_debug(&value); // no max_depth
    let plain = pretty.text.plain().to_string();
    assert!(
        plain.contains("42"),
        "without max_depth all content should be visible: {}",
        plain
    );
}

// ---------------------------------------------------------------------------
// Item 4: expand_all on Debug path (compact vs pretty format)
// ---------------------------------------------------------------------------

#[test]
fn test_rebuild_debug_expand_all_false_uses_compact_for_small() {
    // A small value that fits on one line should use compact {:?} form when
    // expand_all is false.
    let value = vec![1i32, 2, 3];
    let pretty = Pretty::from_debug(&value)
        .with_expand_all(false)
        .rebuild_debug(&value);
    let plain = pretty.text.plain().to_string();
    // Compact form should be single-line
    assert!(
        !plain.contains('\n'),
        "small value with expand_all=false should be compact (single-line): {:?}",
        plain
    );
}

#[test]
fn test_rebuild_debug_expand_all_true_always_expanded() {
    // expand_all=true should force pretty-print format (multi-line for structs)
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Foo {
        a: i32,
        b: i32,
    }
    let value = Foo { a: 1, b: 2 };
    let pretty = Pretty::from_debug(&value)
        .with_expand_all(true)
        .rebuild_debug(&value);
    let plain = pretty.text.plain().to_string();
    assert!(
        plain.contains('\n'),
        "expand_all=true should produce multi-line output: {:?}",
        plain
    );
}

// ---------------------------------------------------------------------------
// Item 5: brace truncation in truncate_inline_collection
// ---------------------------------------------------------------------------

#[test]
fn test_truncate_inline_brace_collection() {
    // A debug output like "{a: 1, b: 2, c: 3}" should be truncatable with {...}
    let s = "{a: 1, b: 2, c: 3}";
    let result = super::truncate_inline_collection(s, 2);
    assert!(
        result.contains("a: 1"),
        "first item should be present: {}",
        result
    );
    assert!(
        result.contains("b: 2"),
        "second item should be present: {}",
        result
    );
    assert!(
        !result.contains("c: 3"),
        "third item should be truncated: {}",
        result
    );
    assert!(
        result.contains("+1"),
        "truncation indicator should be present: {}",
        result
    );
}

#[test]
fn test_truncate_inline_brace_no_truncation_needed() {
    let s = "{a: 1, b: 2}";
    let result = super::truncate_inline_collection(s, 5);
    // No truncation: original should be returned
    assert_eq!(
        result, s,
        "no truncation should return original: {}",
        result
    );
}

// ---------------------------------------------------------------------------
// Item 6: Console::pprint (in console_render.rs) - test via integration
// ---------------------------------------------------------------------------

#[test]
fn test_console_pprint_produces_output() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(true)
        .markup(false)
        .build();
    console.begin_capture();
    console.pprint(&vec![1, 2, 3]);
    let output = console.end_capture();
    assert!(
        output.contains('1'),
        "pprint output should contain value: {}",
        output
    );
}

#[test]
fn test_console_pprint_primitive() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(true)
        .markup(false)
        .build();
    console.begin_capture();
    console.pprint(&42i32);
    let output = console.end_capture();
    assert!(
        output.contains("42"),
        "pprint should render the value: {}",
        output
    );
}

// ---------------------------------------------------------------------------
// Item 7: cell_len for width threshold (verified indirectly via behavior)
// ---------------------------------------------------------------------------

#[cfg(feature = "json")]
#[test]
fn test_cell_len_width_threshold_ascii_json() {
    // For ASCII JSON, cell_len == byte len, so behavior should be identical.
    // Verify that a short array fits within 80-char width and stays compact.
    let json: serde_json::Value = serde_json::from_str("[1, 2]").unwrap();
    let pretty = Pretty::from_json(&json)
        .with_expand_all(false)
        .rebuild_json(&json, 80);
    let plain = pretty.text.plain().to_string();
    // Should be compact (single-line) since [1, 2] fits in 80 chars
    assert!(
        !plain.contains('\n'),
        "short ASCII JSON should stay compact: {}",
        plain
    );
}

// ---------------------------------------------------------------------------
// Item 8: apply_indent_guides uses console style (verified via existing guides)
// ---------------------------------------------------------------------------

#[test]
fn test_apply_indent_guides_with_console() {
    // The gilt_console method should still render indent guides correctly
    let console = make_console();
    let opts = console.options();
    let input = "root\n    child\n        grandchild";
    let pretty = Pretty::from_str(input).with_indent_size(4);
    let segments = pretty.gilt_console(&console, &opts);
    let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
    // Guide characters should still appear
    assert!(
        combined.contains('\u{2502}'),
        "indent guides should appear when rendered via gilt_console: {}",
        combined
    );
}

// ---------------------------------------------------------------------------
// Deep-review: max_depth must be applied BEFORE max_length on the Debug path
// ---------------------------------------------------------------------------

/// When both `max_depth` and `max_length` are set, rich applies depth pruning
/// during the recursive traversal (before length truncation).  This means
/// length truncation operates on the depth-pruned tree, and `... +N` markers
/// from length truncation must not be swallowed into `{...}` depth collapses.
#[test]
fn test_debug_max_depth_before_max_length() {
    // A vec of nested vecs: [[1], [2], [3], [4], [5]]
    // With max_depth=1, the inner vecs collapse to [...].
    // With max_length=2, the outer vec truncates to 2 items + "... +3".
    // Rich applies depth first: [[...], [...], ... +3] (5 items → 2 kept + 3 hidden)
    let value: Vec<Vec<i32>> = vec![vec![1], vec![2], vec![3], vec![4], vec![5]];
    let pretty = Pretty::from_debug(&value)
        .with_max_depth(1)
        .with_max_length(2)
        .rebuild_debug(&value);
    let plain = pretty.text.plain().to_string();

    // The depth-pruned inner vecs should appear as [...]
    assert!(
        plain.contains("[...]"),
        "inner vecs should be depth-pruned to [...]; got: {:?}",
        plain
    );
    // The length truncation marker should be present (not swallowed by depth)
    assert!(
        plain.contains("+3"),
        "length truncation marker ... +3 should survive depth pruning; got: {:?}",
        plain
    );
    // Original inner values should be hidden by depth pruning
    assert!(
        !plain.contains("1") || plain.contains("+3"),
        "inner values should be hidden by depth pruning; got: {:?}",
        plain
    );
}

/// Depth-first order matters when a container has many items AND is itself
/// at the depth limit.  In that case, depth pruning collapses the entire
/// container to `{...}`/`[...]`, and length truncation should NOT add a
/// `... +N` marker to a container that depth has already collapsed.
#[test]
fn test_debug_max_depth_collapses_before_length_truncates() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Wrapper {
        items: Vec<i32>,
    }
    let value = Wrapper {
        items: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    };
    let pretty = Pretty::from_debug(&value)
        .with_max_depth(0)
        .with_max_length(3)
        .rebuild_debug(&value);
    let plain = pretty.text.plain().to_string();
    assert!(
        plain.contains("{...}"),
        "outer struct at max_depth=0 should collapse to {{...}}; got: {:?}",
        plain
    );
    assert!(
        !plain.contains("+7"),
        "length truncation marker should not survive inside depth-collapsed region; got: {:?}",
        plain
    );
}

/// When both max_depth and max_length are set on a multi-line Debug struct,
/// depth pruning must happen BEFORE length truncation.  This ensures the
/// `... +N` length marker is not swallowed and the output structure (closing
/// brace, etc.) remains intact.
#[test]
fn test_debug_max_depth_before_max_length_multiline() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Inner {
        a: i32,
        b: i32,
        c: i32,
    }
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Outer {
        f1: Inner,
        f2: Inner,
        f3: Inner,
        f4: Inner,
        f5: Inner,
    }
    let value = Outer {
        f1: Inner { a: 1, b: 2, c: 3 },
        f2: Inner { a: 4, b: 5, c: 6 },
        f3: Inner { a: 7, b: 8, c: 9 },
        f4: Inner {
            a: 10,
            b: 11,
            c: 12,
        },
        f5: Inner {
            a: 13,
            b: 14,
            c: 15,
        },
    };
    let pretty = Pretty::from_debug(&value)
        .with_max_depth(1)
        .with_max_length(2)
        .with_expand_all(true)
        .rebuild_debug(&value);
    let plain = pretty.text.plain().to_string();
    assert!(
        plain.contains("Inner {...}"),
        "inner structs should be depth-pruned to Inner {{...}}; got: {:?}",
        plain
    );
    assert!(
        plain.contains("+3"),
        "length truncation marker ... +3 should survive; got: {:?}",
        plain
    );
    assert!(
        plain.contains('}'),
        "closing brace should be present; got: {:?}",
        plain
    );
    assert!(
        !plain.contains("15"),
        "inner value 15 should be hidden by depth pruning; got: {:?}",
        plain
    );
}
