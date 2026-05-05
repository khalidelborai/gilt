//! Pretty-printing module for structured data.
//!
//! Provides the [`Pretty`] renderable widget that pretty-prints strings,
//! `Debug` values, and `serde_json::Value` objects with syntax highlighting
//! and optional indent guides.
//!
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

        // Cache the parsed style across calls — was re-parsing on every render.
        static GUIDE_STYLE: std::sync::LazyLock<Style> =
            std::sync::LazyLock::new(|| Style::parse("dim green"));
        self.text
            .with_indent_guides(Some(self.indent_size), '\u{2502}', GUIDE_STYLE.clone())
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
            let annotation_style = Style::parse("dim italic");
            use crate::text::TextPart;
            text = Text::assemble(
                &[
                    TextPart::Styled(format!("({}) ", type_name), annotation_style),
                    TextPart::Inner(text),
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
#[path = "pretty_tests.rs"]
mod tests;
