//! Pretty-printed, syntax-highlighted JSON display.
//!
//! This module provides the [`Json`] type which parses JSON data and produces
//! highlighted [`Text`] for rich terminal rendering. It is a port of Python's
//! `rich/json.py`.
//!
//! # Examples
//!
//! ```rust
//! use gilt::json::{Json, JsonOptions};
//!
//! let json = Json::new(r#"{"name": "world", "count": 42}"#, JsonOptions::default()).unwrap();
//! assert!(json.text.plain().contains("name"));
//! ```

use serde_json::Value;

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::highlighter::{Highlighter, JSONHighlighter, NullHighlighter};
use crate::segment::Segment;
use crate::text::Text;

// ---------------------------------------------------------------------------
// JsonError
// ---------------------------------------------------------------------------

/// Errors that can occur when constructing a [`Json`] from a string.
#[derive(Debug, thiserror::Error)]
pub enum JsonError {
    /// The input string is not valid JSON.
    #[error("invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
}

// ---------------------------------------------------------------------------
// JsonOptions
// ---------------------------------------------------------------------------

/// Configuration for JSON rendering.
///
/// Use the builder methods or `Default` to construct. The defaults match
/// Python rich's behaviour: 2-space indent, highlighting enabled, keys not
/// explicitly sorted (serde_json's default ordering is used).
#[derive(Debug, Clone)]
pub struct JsonOptions {
    /// Number of spaces per indent level. `None` produces compact (single-line)
    /// output. Defaults to `Some(2)`.
    pub indent: Option<usize>,
    /// Whether to apply syntax highlighting via [`JSONHighlighter`].
    /// Defaults to `true`.
    pub highlight: bool,
    /// Whether to sort object keys alphabetically. Defaults to `false`.
    ///
    /// Note: `serde_json::Value::Object` uses a `BTreeMap` by default (keys
    /// sorted), unless the `preserve_order` feature is enabled. When
    /// `sort_keys` is `false` and the source is a parsed `Value`, the key
    /// order depends on the `serde_json` feature flags in use.  When
    /// `sort_keys` is `true`, keys are always guaranteed to be sorted.
    pub sort_keys: bool,
}

impl Default for JsonOptions {
    fn default() -> Self {
        JsonOptions {
            indent: Some(2),
            highlight: true,
            sort_keys: false,
        }
    }
}

impl JsonOptions {
    /// Create options for compact (single-line) JSON.
    pub fn compact() -> Self {
        JsonOptions {
            indent: None,
            ..Default::default()
        }
    }

    /// Builder: set the indent level.
    #[must_use]
    pub fn with_indent(mut self, indent: Option<usize>) -> Self {
        self.indent = indent;
        self
    }

    /// Builder: enable or disable highlighting.
    #[must_use]
    pub fn with_highlight(mut self, highlight: bool) -> Self {
        self.highlight = highlight;
        self
    }

    /// Builder: enable or disable key sorting.
    #[must_use]
    pub fn with_sort_keys(mut self, sort_keys: bool) -> Self {
        self.sort_keys = sort_keys;
        self
    }
}

// ---------------------------------------------------------------------------
// Json
// ---------------------------------------------------------------------------

/// A renderable which pretty-prints JSON with syntax highlighting.
///
/// Construct via [`Json::new`] (from a JSON string) or [`Json::from_value`]
/// (from a pre-parsed [`serde_json::Value`]).
#[derive(Debug)]
pub struct Json {
    /// The highlighted text representation of the JSON data.
    pub text: Text,
}

impl Json {
    /// Parse a JSON string and produce highlighted [`Text`].
    ///
    /// # Errors
    ///
    /// Returns [`JsonError::InvalidJson`] if the input is not valid JSON.
    pub fn new(json_str: &str, options: JsonOptions) -> Result<Self, JsonError> {
        let value: Value = serde_json::from_str(json_str)?;
        Ok(Self::from_value(&value, options))
    }

    /// Create a `Json` from a pre-parsed [`serde_json::Value`].
    pub fn from_value(value: &Value, options: JsonOptions) -> Self {
        let pretty = format_value(value, &options);

        let text = if options.highlight {
            let hl = JSONHighlighter::new();
            let mut t = hl.apply(&pretty);
            t.no_wrap = Some(true);
            t.overflow = None;
            t
        } else {
            let hl = NullHighlighter;
            let mut t = hl.apply(&pretty);
            t.no_wrap = Some(true);
            t.overflow = None;
            t
        };

        Json { text }
    }
}

impl Renderable for Json {
    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        self.text.gilt_console(console, options)
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Sort a `serde_json::Value` recursively so that all object keys are in
/// alphabetical order.
fn sort_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sorted: Vec<(String, Value)> = map
                .iter()
                .map(|(k, v)| (k.clone(), sort_value(v)))
                .collect();
            sorted.sort_by(|a, b| a.0.cmp(&b.0));
            Value::Object(sorted.into_iter().collect())
        }
        Value::Array(arr) => Value::Array(arr.iter().map(sort_value).collect()),
        other => other.clone(),
    }
}

/// Format a `Value` as a JSON string respecting indent and sort_keys options.
fn format_value(value: &Value, options: &JsonOptions) -> String {
    let value = if options.sort_keys {
        sort_value(value)
    } else {
        value.clone()
    };

    match options.indent {
        None => serde_json::to_string(&value).unwrap_or_default(),
        Some(indent) => {
            let mut buf = Vec::new();
            let indent_str: Vec<u8> = vec![b' '; indent];
            let formatter = serde_json::ser::PrettyFormatter::with_indent(&indent_str);
            let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
            serde::Serialize::serialize(&value, &mut ser)
                .expect("serialization of Value should not fail");
            String::from_utf8(buf).unwrap_or_default()
        }
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl std::fmt::Display for Json {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut console = Console::builder()
            .width(f.width().unwrap_or(80))
            .force_terminal(true)
            .no_color(true)
            .build();
        console.begin_capture();
        console.print(self);
        let output = console.end_capture();
        write!(f, "{}", output.trim_end_matches('\n'))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Simple object --------------------------------------------------

    #[test]
    fn test_simple_object() {
        let json = Json::new(r#"{"name": "Alice", "age": 30}"#, JsonOptions::default()).unwrap();
        let plain = json.text.plain().to_string();
        assert!(plain.contains("name"));
        assert!(plain.contains("Alice"));
        assert!(plain.contains("30"));
    }

    // -- Simple array ---------------------------------------------------

    #[test]
    fn test_simple_array() {
        let json = Json::new(r#"[1, 2, 3]"#, JsonOptions::default()).unwrap();
        let plain = json.text.plain().to_string();
        assert!(plain.contains('1'));
        assert!(plain.contains('2'));
        assert!(plain.contains('3'));
    }

    // -- Nested structures ----------------------------------------------

    #[test]
    fn test_nested_structures() {
        let input = r#"{"users": [{"name": "Alice"}, {"name": "Bob"}]}"#;
        let json = Json::new(input, JsonOptions::default()).unwrap();
        let plain = json.text.plain().to_string();
        assert!(plain.contains("users"));
        assert!(plain.contains("Alice"));
        assert!(plain.contains("Bob"));
    }

    // -- Sorted keys ----------------------------------------------------

    #[test]
    fn test_sorted_keys() {
        let input = r#"{"zebra": 1, "apple": 2, "mango": 3}"#;
        let json = Json::new(input, JsonOptions::default().with_sort_keys(true)).unwrap();
        let plain = json.text.plain().to_string();
        let apple_pos = plain.find("apple").expect("apple not found");
        let mango_pos = plain.find("mango").expect("mango not found");
        let zebra_pos = plain.find("zebra").expect("zebra not found");
        assert!(apple_pos < mango_pos, "apple should come before mango");
        assert!(mango_pos < zebra_pos, "mango should come before zebra");
    }

    // -- Compact (no indent) --------------------------------------------

    #[test]
    fn test_compact_json() {
        let input = r#"{"a": 1, "b": 2}"#;
        let json = Json::new(input, JsonOptions::compact()).unwrap();
        let plain = json.text.plain().to_string();
        // Compact output should not contain newlines
        assert!(!plain.contains('\n'), "compact JSON should be single-line");
    }

    // -- Custom indent --------------------------------------------------

    #[test]
    fn test_custom_indent_4() {
        let input = r#"{"key": "value"}"#;
        let json = Json::new(input, JsonOptions::default().with_indent(Some(4))).unwrap();
        let plain = json.text.plain().to_string();
        // With indent=4, should have 4-space indentation
        assert!(
            plain.contains("    \"key\""),
            "expected 4-space indent, got:\n{}",
            plain
        );
    }

    #[test]
    fn test_custom_indent_1() {
        let input = r#"{"key": "value"}"#;
        let json = Json::new(input, JsonOptions::default().with_indent(Some(1))).unwrap();
        let plain = json.text.plain().to_string();
        let lines: Vec<&str> = plain.lines().collect();
        // Line with key should start with single space
        assert!(lines.len() >= 2);
        assert!(
            lines[1].starts_with(" \"key\""),
            "expected 1-space indent, got: {:?}",
            lines[1]
        );
    }

    // -- Highlighting enabled/disabled ----------------------------------

    #[test]
    fn test_highlight_enabled() {
        let input = r#"{"key": true}"#;
        let json = Json::new(input, JsonOptions::default().with_highlight(true)).unwrap();
        // Highlighted text should have spans
        assert!(
            !json.text.spans().is_empty(),
            "highlighting enabled should produce spans"
        );
    }

    #[test]
    fn test_highlight_disabled() {
        let input = r#"{"key": true}"#;
        let json = Json::new(input, JsonOptions::default().with_highlight(false)).unwrap();
        // No highlighting should produce no spans
        assert!(
            json.text.spans().is_empty(),
            "highlighting disabled should produce no spans"
        );
    }

    // -- Invalid JSON returns error -------------------------------------

    #[test]
    fn test_invalid_json() {
        let result = Json::new("not valid json {{{", JsonOptions::default());
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("invalid JSON"), "error message: {}", msg);
    }

    // -- from_value constructor -----------------------------------------

    #[test]
    fn test_from_value() {
        let value: Value = serde_json::json!({
            "name": "Alice",
            "age": 30,
            "active": true
        });
        let json = Json::from_value(&value, JsonOptions::default());
        let plain = json.text.plain().to_string();
        assert!(plain.contains("Alice"));
        assert!(plain.contains("30"));
        assert!(plain.contains("true"));
    }

    #[test]
    fn test_from_value_array() {
        let value: Value = serde_json::json!([1, "two", null, false]);
        let json = Json::from_value(&value, JsonOptions::default());
        let plain = json.text.plain().to_string();
        assert!(plain.contains('1'));
        assert!(plain.contains("two"));
        assert!(plain.contains("null"));
        assert!(plain.contains("false"));
    }

    // -- Renderable trait integration -----------------------------------

    #[test]
    fn test_renderable_integration() {
        let json = Json::new(r#"{"x": 1}"#, JsonOptions::default()).unwrap();
        let console = Console::builder().width(80).build();
        let opts = console.options();
        let segments = json.gilt_console(&console, &opts);
        let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(combined.contains('x'));
        assert!(combined.contains('1'));
    }

    // -- Numbers, booleans, null highlighted ----------------------------

    #[test]
    fn test_numbers_highlighted() {
        let input = r#"{"int": 42, "float": 3.14, "neg": -1}"#;
        let json = Json::new(input, JsonOptions::default()).unwrap();
        let plain = json.text.plain();
        // Numbers should be present
        assert!(plain.contains("42"));
        assert!(plain.contains("3.14"));
        assert!(plain.contains("-1"));
        // Should have spans for highlighting
        assert!(!json.text.spans().is_empty());
    }

    #[test]
    fn test_booleans_highlighted() {
        let input = r#"{"yes": true, "no": false}"#;
        let json = Json::new(input, JsonOptions::default()).unwrap();
        let plain = json.text.plain();
        assert!(plain.contains("true"));
        assert!(plain.contains("false"));
        assert!(!json.text.spans().is_empty());
    }

    #[test]
    fn test_null_highlighted() {
        let input = r#"{"nothing": null}"#;
        let json = Json::new(input, JsonOptions::default()).unwrap();
        let plain = json.text.plain();
        assert!(plain.contains("null"));
        assert!(!json.text.spans().is_empty());
    }

    // -- Keys highlighted differently from values -----------------------

    #[test]
    fn test_keys_vs_values_spans() {
        let input = r#"{"key": "value"}"#;
        let json = Json::new(input, JsonOptions::default()).unwrap();
        let plain = json.text.plain();

        // Helper to find spans covering a substring
        fn spans_covering<'a>(
            text: &'a Text,
            substr: &str,
            plain: &str,
        ) -> Vec<&'a crate::text::Span> {
            let start = plain.find(substr).unwrap();
            let char_start = plain[..start].chars().count();
            let char_end = char_start + substr.chars().count();
            text.spans()
                .iter()
                .filter(|s| s.start <= char_start && s.end >= char_end)
                .collect()
        }

        let key_spans = spans_covering(&json.text, "\"key\"", plain);
        let value_spans = spans_covering(&json.text, "\"value\"", plain);

        // Key should have at least one span (the key style from JSONHighlighter)
        assert!(!key_spans.is_empty(), "key should have highlighting spans");
        // Value should have at least one span (the str style)
        assert!(
            !value_spans.is_empty(),
            "value should have highlighting spans"
        );

        // Key spans should include a different style than value-only spans.
        // Specifically, the key gets an additional "json.key" span that the
        // value does not. We verify this by checking that the key has more
        // spans than the value (key gets both "json.str" and "json.key").
        assert!(
            key_spans.len() > value_spans.len(),
            "key should have more spans (str + key) than value (str only): key={}, value={}",
            key_spans.len(),
            value_spans.len()
        );
    }

    // -- Empty object/array ---------------------------------------------

    #[test]
    fn test_empty_object() {
        let json = Json::new("{}", JsonOptions::default()).unwrap();
        let plain = json.text.plain().to_string();
        assert!(plain.contains('{'));
        assert!(plain.contains('}'));
    }

    #[test]
    fn test_empty_array() {
        let json = Json::new("[]", JsonOptions::default()).unwrap();
        let plain = json.text.plain().to_string();
        assert!(plain.contains('['));
        assert!(plain.contains(']'));
    }

    // -- no_wrap is set -------------------------------------------------

    #[test]
    fn test_no_wrap_set() {
        let json = Json::new(r#"{"a": 1}"#, JsonOptions::default()).unwrap();
        assert_eq!(json.text.no_wrap, Some(true));
    }

    // -- overflow is None -----------------------------------------------

    #[test]
    fn test_overflow_is_none() {
        let json = Json::new(r#"{"a": 1}"#, JsonOptions::default()).unwrap();
        assert_eq!(json.text.overflow, None);
    }

    // -- Sort keys with nested objects ----------------------------------

    #[test]
    fn test_sorted_keys_nested() {
        let input = r#"{"z": {"b": 2, "a": 1}, "a": {"y": 3, "x": 4}}"#;
        let json = Json::new(input, JsonOptions::default().with_sort_keys(true)).unwrap();
        let plain = json.text.plain().to_string();

        // Top-level: "a" before "z"
        let a_pos = plain.find("\"a\"").expect("'a' not found");
        let z_pos = plain.find("\"z\"").expect("'z' not found");
        assert!(a_pos < z_pos, "top-level 'a' should come before 'z'");

        // Nested under "a": "x" before "y"
        // Find from after the first "a" key
        let nested_start = a_pos + 3;
        let remaining = &plain[nested_start..];
        let x_pos = remaining.find("\"x\"").expect("'x' not found in nested");
        let y_pos = remaining.find("\"y\"").expect("'y' not found in nested");
        assert!(x_pos < y_pos, "nested 'x' should come before 'y'");
    }

    // -- Large/complex JSON ---------------------------------------------

    #[test]
    fn test_complex_json() {
        let input = r#"{
            "string": "hello",
            "number": 42,
            "float": 3.14,
            "bool_true": true,
            "bool_false": false,
            "null_val": null,
            "array": [1, "two", null],
            "nested": {"inner": "value"}
        }"#;
        let json = Json::new(input, JsonOptions::default()).unwrap();
        let plain = json.text.plain();
        assert!(plain.contains("hello"));
        assert!(plain.contains("42"));
        assert!(plain.contains("3.14"));
        assert!(plain.contains("true"));
        assert!(plain.contains("false"));
        assert!(plain.contains("null"));
        assert!(plain.contains("two"));
        assert!(plain.contains("inner"));
        // Should have many highlight spans
        assert!(
            json.text.spans().len() > 5,
            "complex JSON should have many highlight spans, got {}",
            json.text.spans().len()
        );
    }

    // -- String with escape sequences -----------------------------------

    #[test]
    fn test_json_string_escapes() {
        let input = r#"{"msg": "hello\nworld"}"#;
        let json = Json::new(input, JsonOptions::default()).unwrap();
        let plain = json.text.plain();
        // serde_json preserves the escaped form in pretty output
        assert!(plain.contains("hello\\nworld"));
    }

    // -- Unicode in JSON ------------------------------------------------

    #[test]
    fn test_json_unicode() {
        let input = r#"{"greeting": "こんにちは"}"#;
        let json = Json::new(input, JsonOptions::default()).unwrap();
        let plain = json.text.plain();
        assert!(plain.contains("こんにちは"));
    }

    // -- JsonOptions builder chain --------------------------------------

    #[test]
    fn test_options_builder_chain() {
        let opts = JsonOptions::default()
            .with_indent(Some(4))
            .with_highlight(false)
            .with_sort_keys(true);
        assert_eq!(opts.indent, Some(4));
        assert!(!opts.highlight);
        assert!(opts.sort_keys);
    }

    // -- JsonError display ----------------------------------------------

    #[test]
    fn test_json_error_display() {
        let result = Json::new("{bad", JsonOptions::default());
        let err = result.unwrap_err();
        let display = format!("{}", err);
        assert!(display.starts_with("invalid JSON:"));
    }

    // -- Default indent produces pretty output --------------------------

    #[test]
    fn test_default_indent_is_pretty() {
        let input = r#"{"a":1}"#;
        let json = Json::new(input, JsonOptions::default()).unwrap();
        let plain = json.text.plain().to_string();
        // Pretty output should have newlines and spaces
        assert!(
            plain.contains('\n'),
            "default indent should produce multi-line output"
        );
        assert!(
            plain.contains("  "),
            "default indent should use 2-space indentation"
        );
    }

    #[test]
    fn test_display_trait() {
        let json = Json::new(r#"{"name": "world"}"#, JsonOptions::default()).unwrap();
        let s = format!("{}", json);
        assert!(!s.is_empty());
        assert!(s.contains("name"));
        assert!(s.contains("world"));
    }
}
