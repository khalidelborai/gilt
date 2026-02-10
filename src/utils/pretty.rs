//! Pretty-printing module for structured data.
//!
//! Provides the [`Pretty`] renderable widget that pretty-prints strings,
//! `Debug` values, and `serde_json::Value` objects with syntax highlighting
//! and optional indent guides.
//!
//! Rust port of Python's `rich/pretty.py`, adapted to use `Debug` and
//! `serde_json` instead of Python's runtime introspection.

use crate::console::{Console, ConsoleOptions, Renderable};
#[cfg(feature = "json")]
use crate::highlighter::JSONHighlighter;
use crate::highlighter::{Highlighter, ReprHighlighter};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::Style;
use crate::text::{OverflowMethod, Text};

// ---------------------------------------------------------------------------
// Pretty
// ---------------------------------------------------------------------------

/// A renderable widget that pretty-prints text with highlighting and optional
/// indent guides.
///
/// `Pretty` wraps a [`Text`] object and can be constructed from plain strings,
/// `Debug` values, or `serde_json::Value` instances. Each constructor applies
/// the appropriate highlighter automatically.
#[derive(Clone, Debug)]
pub struct Pretty {
    /// The underlying styled text.
    pub text: Text,
    /// Whether to disable word-wrapping.
    pub no_wrap: bool,
    /// Overflow handling method.
    pub overflow: Option<OverflowMethod>,
    /// Whether to render indent guides (vertical lines at indent boundaries).
    pub indent_guides: bool,
    /// Number of spaces per indent level (default 4).
    pub indent_size: usize,
    /// Maximum number of elements shown in a container (array/object).
    /// `None` means show all elements.
    pub max_length: Option<usize>,
    /// Maximum length of string values before truncation.
    /// `None` means show full strings.
    pub max_string: Option<usize>,
    /// When `true`, always expand containers (one item per line) even if they
    /// would fit on a single line.
    pub expand_all: bool,
    /// When `true`, prepend the type name (e.g. `"String"`, `"Object"`) to the output.
    pub type_annotation: bool,
}

impl Pretty {
    // -- Constructors -------------------------------------------------------

    /// Create a `Pretty` from a plain string.
    ///
    /// Applies [`ReprHighlighter`] to the text and enables indent guides.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Self {
        let hl = ReprHighlighter::new();
        let text = hl.apply(value);
        Pretty {
            text,
            no_wrap: false,
            overflow: None,
            indent_guides: true,
            indent_size: 4,
            max_length: None,
            max_string: None,
            expand_all: false,
            type_annotation: false,
        }
    }

    /// Create a `Pretty` from any value implementing [`std::fmt::Debug`].
    ///
    /// Uses the alternate pretty-print format (`{:#?}`) and applies
    /// [`ReprHighlighter`].
    pub fn from_debug<T: std::fmt::Debug>(value: &T) -> Self {
        let formatted = format!("{:#?}", value);
        let hl = ReprHighlighter::new();
        let text = hl.apply(&formatted);
        Pretty {
            text,
            no_wrap: false,
            overflow: None,
            indent_guides: true,
            indent_size: 4,
            max_length: None,
            max_string: None,
            expand_all: false,
            type_annotation: false,
        }
    }

    /// Create a `Pretty` from a [`serde_json::Value`].
    ///
    /// Formats the JSON with `serde_json::to_string_pretty` and applies
    /// [`JSONHighlighter`]. Sets `no_wrap` to `true` since pretty-printed JSON
    /// already contains newlines at appropriate positions.
    #[cfg(feature = "json")]
    pub fn from_json(value: &serde_json::Value) -> Self {
        let formatted = serde_json::to_string_pretty(value).unwrap_or_default();
        let hl = JSONHighlighter::new();
        let text = hl.apply(&formatted);
        Pretty {
            text,
            no_wrap: true,
            overflow: None,
            indent_guides: true,
            indent_size: 2, // JSON convention: 2-space indent
            max_length: None,
            max_string: None,
            expand_all: false,
            type_annotation: false,
        }
    }

    // -- Builder methods ----------------------------------------------------

    /// Set whether indent guides are rendered.
    #[must_use]
    pub fn with_indent_guides(mut self, guides: bool) -> Self {
        self.indent_guides = guides;
        self
    }

    /// Set the indent size (number of spaces per level).
    #[must_use]
    pub fn with_indent_size(mut self, size: usize) -> Self {
        self.indent_size = size;
        self
    }

    /// Set whether word-wrapping is disabled.
    #[must_use]
    pub fn with_no_wrap(mut self, no_wrap: bool) -> Self {
        self.no_wrap = no_wrap;
        self
    }

    /// Set the overflow handling method.
    #[must_use]
    pub fn with_overflow(mut self, overflow: OverflowMethod) -> Self {
        self.overflow = Some(overflow);
        self
    }

    /// Set the maximum number of elements shown in a container.
    ///
    /// When set, arrays and objects in JSON (or collection items in Debug
    /// output) are truncated after `max_length` items, with a
    /// `... +N more` indicator appended.
    #[must_use]
    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// Set the maximum length of string values before truncation.
    ///
    /// String values exceeding this length are truncated and a `+N` suffix
    /// is appended to indicate the number of hidden characters.
    #[must_use]
    pub fn with_max_string(mut self, max_string: usize) -> Self {
        self.max_string = Some(max_string);
        self
    }

    /// Set whether containers are always expanded (one item per line).
    ///
    /// When `true`, even short containers that would fit on a single line
    /// are formatted with each item on its own line.
    #[must_use]
    pub fn with_expand_all(mut self, expand_all: bool) -> Self {
        self.expand_all = expand_all;
        self
    }

    /// Set whether a type annotation is prepended to the output.
    ///
    /// When enabled, the output is prefixed with the data type (e.g.
    /// `"(str) ..."`  for strings, `"(object) ..."` for JSON objects).
    #[must_use]
    pub fn with_type_annotation(mut self, annotation: bool) -> Self {
        self.type_annotation = annotation;
        self
    }

    // -- Rebuild from JSON with parameters ----------------------------------

    /// Re-format the Pretty from a JSON value, applying `max_length`,
    /// `max_string`, and `expand_all` parameters.
    ///
    /// This is the primary way to use the new parameters with JSON data:
    /// ```ignore
    /// let pretty = Pretty::from_json(&value)
    ///     .with_max_length(3)
    ///     .with_max_string(20)
    ///     .with_expand_all(true)
    ///     .rebuild_json(&value);
    /// ```
    #[cfg(feature = "json")]
    #[must_use]
    pub fn rebuild_json(mut self, value: &serde_json::Value) -> Self {
        let formatted = format_json_value(
            value,
            0,
            self.indent_size,
            self.max_length,
            self.max_string,
            self.expand_all,
        );
        let hl = JSONHighlighter::new();
        self.text = hl.apply(&formatted);
        self
    }

    /// Re-format the Pretty from a Debug value, applying `max_length` and
    /// `max_string` parameters.
    #[must_use]
    pub fn rebuild_debug<T: std::fmt::Debug>(mut self, value: &T) -> Self {
        let formatted = format!("{:#?}", value);
        let processed = apply_debug_params(&formatted, self.max_length, self.max_string);
        let hl = ReprHighlighter::new();
        self.text = hl.apply(&processed);
        self
    }

    // -- Indent guides ------------------------------------------------------

    /// Apply indent guides to the underlying text.
    ///
    /// For each line, leading spaces are inspected. At every `indent_size`
    /// boundary within the leading whitespace, the space character is replaced
    /// with a vertical bar (`│`) styled with dim text.
    fn apply_indent_guides(&self) -> Text {
        if !self.indent_guides {
            return self.text.clone();
        }

        let guide_style = Style::parse("dim green").unwrap_or_else(|_| Style::null());
        self.text
            .with_indent_guides(Some(self.indent_size), '\u{2502}', guide_style)
    }

    // -- Measurement --------------------------------------------------------

    /// Measure the minimum and maximum widths required to render this widget.
    pub fn measure(&self) -> Measurement {
        self.text.measure()
    }
}

// -- Renderable implementation ----------------------------------------------

impl Renderable for Pretty {
    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let mut text = self.apply_indent_guides();

        if self.no_wrap {
            text.no_wrap = Some(true);
        }
        if let Some(overflow) = self.overflow {
            text.overflow = Some(overflow);
        }

        if self.type_annotation {
            let type_name = infer_type_name(self.text.plain());
            let annotation_style = Style::parse("dim italic").unwrap_or_else(|_| Style::null());
            use crate::text::TextPart;
            text = Text::assemble(
                &[
                    TextPart::Styled(format!("({}) ", type_name), annotation_style),
                    TextPart::Rich(text),
                ],
                Style::null(),
            );
        }
        text.gilt_console(console, options)
    }
}

// ---------------------------------------------------------------------------
// JSON formatting with parameters
// ---------------------------------------------------------------------------

/// Format a JSON value as a pretty-printed string, respecting `max_length`,
/// `max_string`, and `expand_all` parameters.
#[cfg(feature = "json")]
fn format_json_value(
    value: &serde_json::Value,
    depth: usize,
    indent_size: usize,
    max_length: Option<usize>,
    max_string: Option<usize>,
    expand_all: bool,
) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => {
            let truncated = truncate_string(s, max_string);
            format!("\"{}\"", escape_json_string(&truncated))
        }
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                return "[]".to_string();
            }
            format_json_array(arr, depth, indent_size, max_length, max_string, expand_all)
        }
        serde_json::Value::Object(obj) => {
            if obj.is_empty() {
                return "{}".to_string();
            }
            format_json_object(obj, depth, indent_size, max_length, max_string, expand_all)
        }
    }
}

/// Format a JSON array with optional truncation and forced expansion.
#[cfg(feature = "json")]
fn format_json_array(
    arr: &[serde_json::Value],
    depth: usize,
    indent_size: usize,
    max_length: Option<usize>,
    max_string: Option<usize>,
    expand_all: bool,
) -> String {
    let total = arr.len();
    let display_count = match max_length {
        Some(max) => max.min(total),
        None => total,
    };
    let truncated_count = total - display_count;

    let items: Vec<String> = arr[..display_count]
        .iter()
        .map(|v| {
            format_json_value(
                v,
                depth + 1,
                indent_size,
                max_length,
                max_string,
                expand_all,
            )
        })
        .collect();

    let should_expand = if expand_all {
        true
    } else {
        // Check if the compact representation would be too long (> 80 chars)
        // or if any item contains newlines
        let compact = items.join(", ");
        compact.len() > 80 || items.iter().any(|s| s.contains('\n'))
    };

    if should_expand {
        let indent = " ".repeat(indent_size * (depth + 1));
        let closing_indent = " ".repeat(indent_size * depth);
        let mut parts: Vec<String> = items
            .iter()
            .map(|item| format!("{}{}", indent, item))
            .collect();
        if truncated_count > 0 {
            parts.push(format!("{}... +{} more", indent, truncated_count));
        }
        format!("[\n{}\n{}]", parts.join(",\n"), closing_indent)
    } else {
        let mut result = items.join(", ");
        if truncated_count > 0 {
            result.push_str(&format!(", ... +{} more", truncated_count));
        }
        format!("[{}]", result)
    }
}

/// Format a JSON object with optional truncation and forced expansion.
#[cfg(feature = "json")]
fn format_json_object(
    obj: &serde_json::Map<String, serde_json::Value>,
    depth: usize,
    indent_size: usize,
    max_length: Option<usize>,
    max_string: Option<usize>,
    expand_all: bool,
) -> String {
    let entries: Vec<(&String, &serde_json::Value)> = obj.iter().collect();
    let total = entries.len();
    let display_count = match max_length {
        Some(max) => max.min(total),
        None => total,
    };
    let truncated_count = total - display_count;

    let items: Vec<String> = entries[..display_count]
        .iter()
        .map(|(k, v)| {
            let key_str = format!("\"{}\"", escape_json_string(k));
            let val_str = format_json_value(
                v,
                depth + 1,
                indent_size,
                max_length,
                max_string,
                expand_all,
            );
            format!("{}: {}", key_str, val_str)
        })
        .collect();

    let should_expand = if expand_all {
        true
    } else {
        let compact = items.join(", ");
        compact.len() > 80 || items.iter().any(|s| s.contains('\n'))
    };

    if should_expand {
        let indent = " ".repeat(indent_size * (depth + 1));
        let closing_indent = " ".repeat(indent_size * depth);
        let mut parts: Vec<String> = items
            .iter()
            .map(|item| format!("{}{}", indent, item))
            .collect();
        if truncated_count > 0 {
            parts.push(format!("{}... +{} more", indent, truncated_count));
        }
        format!("{{\n{}\n{}}}", parts.join(",\n"), closing_indent)
    } else {
        let mut result = items.join(", ");
        if truncated_count > 0 {
            result.push_str(&format!(", ... +{} more", truncated_count));
        }
        format!("{{{}}}", result)
    }
}

/// Truncate a string if it exceeds `max_string` characters.
/// Appends `+N` to indicate hidden characters.
#[cfg(feature = "json")]
fn truncate_string(s: &str, max_string: Option<usize>) -> String {
    match max_string {
        Some(max) if s.chars().count() > max => {
            let truncated: String = s.chars().take(max).collect();
            let remaining = s.chars().count() - max;
            format!("{}+{}", truncated, remaining)
        }
        _ => s.to_string(),
    }
}

/// Infer a human-readable type name from the text content.
///
/// Examines the first non-whitespace character(s) to determine the likely data
/// type for annotation purposes.
fn infer_type_name(text: &str) -> &'static str {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return "empty";
    }
    match trimmed.as_bytes()[0] {
        b'{' => "object",
        b'[' => "array",
        b'"' => "str",
        b't' | b'f' if trimmed == "true" || trimmed == "false" => "bool",
        b'n' if trimmed == "null" => "null",
        b'0'..=b'9' | b'-' => "number",
        _ => {
            // Check if it looks like a Rust Debug struct (e.g. "Foo {")
            if trimmed.contains(' ') && trimmed.contains('{') {
                "struct"
            } else {
                "str"
            }
        }
    }
}

/// Escape special JSON characters in a string.
#[cfg(feature = "json")]
fn escape_json_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            _ => result.push(c),
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Debug formatting with parameters
// ---------------------------------------------------------------------------

/// Apply `max_length` and `max_string` to a Debug-formatted string.
///
/// This works by post-processing the already-formatted debug string:
/// - `max_string`: truncates quoted string literals
/// - `max_length`: truncates items in bracket/brace-delimited collections
fn apply_debug_params(
    formatted: &str,
    max_length: Option<usize>,
    max_string: Option<usize>,
) -> String {
    let mut result = formatted.to_string();
    if let Some(max_s) = max_string {
        result = truncate_debug_strings(&result, max_s);
    }
    if let Some(max_l) = max_length {
        result = truncate_debug_collections(&result, max_l);
    }
    result
}

/// Truncate quoted string literals in a Debug-formatted string.
fn truncate_debug_strings(s: &str, max_string: usize) -> String {
    let mut result = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '"' {
            // Found start of a string literal -- collect its contents
            result.push('"');
            i += 1;
            let mut content = String::new();
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    content.push(chars[i]);
                    content.push(chars[i + 1]);
                    i += 2;
                } else {
                    content.push(chars[i]);
                    i += 1;
                }
            }
            // Truncate the content if needed
            let char_count = content.chars().count();
            if char_count > max_string {
                let truncated: String = content.chars().take(max_string).collect();
                let remaining = char_count - max_string;
                result.push_str(&truncated);
                result.push_str(&format!("+{}", remaining));
            } else {
                result.push_str(&content);
            }
            if i < chars.len() {
                result.push('"'); // closing quote
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

/// Truncate collections (items in `[...]` or `{...}`) in a Debug-formatted
/// string.
///
/// This scans for top-level comma-separated items within bracket/brace pairs
/// and truncates after `max_length` items.
fn truncate_debug_collections(s: &str, max_length: usize) -> String {
    let lines: Vec<&str> = s.lines().collect();
    if lines.len() <= 1 {
        // Single-line: try inline truncation
        return truncate_inline_collection(s, max_length);
    }

    // Multi-line: find collection boundaries and truncate
    truncate_multiline_collection(&lines, max_length)
}

/// Truncate items in a single-line collection like `[1, 2, 3, 4, 5]`.
fn truncate_inline_collection(s: &str, max_length: usize) -> String {
    if let Some(start) = s.find('[') {
        if let Some(end) = s.rfind(']') {
            if start < end {
                let inner = &s[start + 1..end];
                let truncated = truncate_comma_items(inner, max_length);
                return format!("{}[{}]{}", &s[..start], truncated, &s[end + 1..]);
            }
        }
    }
    s.to_string()
}

/// Truncate comma-separated items in a string.
fn truncate_comma_items(inner: &str, max_length: usize) -> String {
    let items: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
    let total = items.len();
    if total <= max_length {
        return inner.to_string();
    }
    let kept: Vec<&str> = items[..max_length].to_vec();
    let remaining = total - max_length;
    format!("{}, ... +{} more", kept.join(", "), remaining)
}

/// Truncate items in a multi-line Debug collection.
fn truncate_multiline_collection(lines: &[&str], max_length: usize) -> String {
    let mut result = Vec::new();
    let mut depth = 0i32;
    let mut item_count = 0usize;
    let mut truncated = false;
    let mut skipped_count = 0usize;
    let mut inside_collection = false;

    for &line in lines {
        let trimmed = line.trim();

        // Track collection depth
        let opens = trimmed.chars().filter(|&c| c == '[' || c == '{').count() as i32;
        let closes = trimmed.chars().filter(|&c| c == ']' || c == '}').count() as i32;

        if depth == 0 && opens > 0 {
            inside_collection = true;
            item_count = 0;
            truncated = false;
            skipped_count = 0;
            depth += opens - closes;
            result.push(line.to_string());
            continue;
        }

        if inside_collection && depth == 1 && (closes > 0 && opens == 0) {
            // Closing bracket at top level
            if skipped_count > 0 {
                let indent_len = line.len() - line.trim_start().len();
                let pad = " ".repeat(indent_len + 4);
                result.push(format!("{}... +{} more,", pad, skipped_count));
            }
            depth += opens - closes;
            if depth <= 0 {
                inside_collection = false;
            }
            result.push(line.to_string());
            continue;
        }

        depth += opens - closes;

        if inside_collection && !truncated {
            if trimmed.ends_with(',') || closes > 0 {
                item_count += 1;
            }
            if item_count > max_length {
                truncated = true;
                skipped_count += 1;
                continue;
            }
            result.push(line.to_string());
        } else if truncated {
            skipped_count += 1;
        } else {
            result.push(line.to_string());
        }
    }
    result.join("\n")
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl std::fmt::Display for Pretty {
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
        let json: serde_json::Value =
            serde_json::from_str(r#"{"name": "Alice", "age": 30}"#).unwrap();
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
            serde_json::from_str(r#"{"user": {"name": "Bob", "address": {"city": "NYC"}}}"#)
                .unwrap();
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
        let guided = pretty.apply_indent_guides();
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
        let guided = pretty.apply_indent_guides();
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
        let guided = pretty.apply_indent_guides();
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
        let guided = pretty.apply_indent_guides();
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
        let guided = pretty.apply_indent_guides();
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
        let json: serde_json::Value =
            serde_json::from_str("[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]").unwrap();
        let pretty = Pretty::from_json(&json)
            .with_max_length(3)
            .rebuild_json(&json);
        let plain = pretty.text.plain().to_string();
        // Should contain the first 3 items
        assert!(plain.contains('1'), "should contain 1: {}", plain);
        assert!(plain.contains('2'), "should contain 2: {}", plain);
        assert!(plain.contains('3'), "should contain 3: {}", plain);
        // Should have truncation indicator
        assert!(
            plain.contains("+7 more"),
            "should contain '+7 more' truncation indicator: {}",
            plain
        );
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_max_length_none_shows_all() {
        let json: serde_json::Value = serde_json::from_str("[1, 2, 3, 4, 5]").unwrap();
        let pretty = Pretty::from_json(&json).rebuild_json(&json);
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
            .rebuild_json(&json);
        let plain = pretty.text.plain().to_string();
        // Should have truncation indicator for the remaining 3 items
        assert!(
            plain.contains("+3 more"),
            "should contain '+3 more' truncation indicator: {}",
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
            .rebuild_json(&json);
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
        let pretty = Pretty::from_json(&json).rebuild_json(&json);
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
            .rebuild_json(&json);
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
            .rebuild_json(&json);
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
            .rebuild_json(&json);
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
            .rebuild_json(&json);
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
            .rebuild_json(&json);
        let plain = pretty.text.plain().to_string();

        // expand_all: should be multi-line
        assert!(
            plain.contains('\n'),
            "should be multi-line with expand_all: {}",
            plain
        );

        // max_length=3: should show truncation for remaining 2 items
        assert!(
            plain.contains("+2 more"),
            "should contain '+2 more' for max_length truncation: {}",
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
            .rebuild_json(&json);
        let plain = pretty.text.plain().to_string();

        // The nested array should also be truncated
        assert!(
            plain.contains("+6 more"),
            "nested array should be truncated: {}",
            plain
        );
    }

    // -- Debug rebuild tests ------------------------------------------------

    #[test]
    fn test_rebuild_debug_max_string() {
        #[derive(Debug)]
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

    // -- Helper function unit tests -----------------------------------------

    #[cfg(feature = "json")]
    #[test]
    fn test_truncate_string_within_limit() {
        assert_eq!(truncate_string("hello", Some(10)), "hello");
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_truncate_string_at_limit() {
        assert_eq!(truncate_string("hello", Some(5)), "hello");
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_truncate_string_over_limit() {
        assert_eq!(truncate_string("hello world", Some(5)), "hello+6");
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_truncate_string_none() {
        assert_eq!(truncate_string("hello world", None), "hello world");
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
        assert_eq!(format_json_value(&v, 0, 2, None, None, false), "null");
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_format_json_value_bool() {
        let v = serde_json::Value::Bool(true);
        assert_eq!(format_json_value(&v, 0, 2, None, None, false), "true");
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_format_json_empty_array() {
        let v: serde_json::Value = serde_json::from_str("[]").unwrap();
        assert_eq!(format_json_value(&v, 0, 2, None, None, false), "[]");
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_format_json_empty_object() {
        let v: serde_json::Value = serde_json::from_str("{}").unwrap();
        assert_eq!(format_json_value(&v, 0, 2, None, None, false), "{}");
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
}
