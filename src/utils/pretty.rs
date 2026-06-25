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
    /// Maximum nesting depth before rendering a placeholder (`{...}` / `[...]`).
    /// `None` means no depth limit.
    pub max_depth: Option<usize>,
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
            max_depth: None,
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
            max_depth: None,
        }
    }

    /// Create a `Pretty` from any value implementing [`serde::Serialize`].
    ///
    /// Serializes `value` to a [`serde_json::Value`] and delegates to
    /// [`from_json`](Self::from_json), inheriting all its formatting and
    /// highlighting behaviour.
    ///
    /// Returns `Err` if serialization fails (e.g. the type contains a
    /// non-string map key that `serde_json` cannot serialize).
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "json")] {
    /// use gilt::utils::pretty::Pretty;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Point { x: f64, y: f64 }
    ///
    /// let p = Point { x: 1.0, y: 2.5 };
    /// let pretty = Pretty::from_serde(&p).unwrap();
    /// let rendered = format!("{}", pretty);
    /// assert!(rendered.contains("1.0") || rendered.contains("1"));
    /// # }
    /// ```
    #[cfg(feature = "json")]
    pub fn from_serde<T: serde::Serialize>(value: &T) -> Result<Self, serde_json::Error> {
        let json_value = serde_json::to_value(value)?;
        Ok(Self::from_json(&json_value))
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
            max_depth: None,
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
    /// `... +N` indicator appended.
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

    /// Set the maximum nesting depth before rendering a placeholder.
    ///
    /// When a container (array or object) is nested deeper than `max_depth`,
    /// a placeholder (`{...}` for objects, `[...]` for arrays) is rendered
    /// instead of expanding the contents. Matches rich's `max_depth` parameter.
    #[must_use]
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = Some(max_depth);
        self
    }

    // -- Rebuild from JSON with parameters ----------------------------------

    /// Re-format the Pretty from a JSON value, applying `max_length`,
    /// `max_string`, `expand_all`, and `max_depth` parameters.
    ///
    /// This is the primary way to use the new parameters with JSON data:
    /// ```ignore
    /// let pretty = Pretty::from_json(&value)
    ///     .with_max_length(3)
    ///     .with_max_string(20)
    ///     .with_expand_all(true)
    ///     .with_max_depth(2)
    ///     .rebuild_json(&value, 80);
    /// ```
    ///
    /// The `max_width` argument controls the cell-width threshold above which a
    /// container is expanded to multi-line form.  Pass the render context's
    /// `max_width` (e.g. `options.max_width`) for accurate results; use `80` as
    /// a default when no context is available.
    #[cfg(feature = "json")]
    #[must_use]
    pub fn rebuild_json(mut self, value: &serde_json::Value, max_width: usize) -> Self {
        let opts = JsonFmtOpts {
            indent_size: self.indent_size,
            max_length: self.max_length,
            max_string: self.max_string,
            expand_all: self.expand_all,
            max_depth: self.max_depth,
            max_width,
        };
        let formatted = format_json_value(value, 0, opts);
        let hl = JSONHighlighter::new();
        self.text = hl.apply(&formatted);
        self
    }

    /// Re-format the Pretty from a Debug value, applying `max_length`,
    /// `max_string`, `max_depth`, and `expand_all` parameters.
    ///
    /// When `expand_all` is `false` (the default), the compact single-line
    /// `{:?}` form is used if the result fits within 88 characters; otherwise
    /// the alternate pretty-print `{:#?}` form is used.  When `expand_all` is
    /// `true`, `{:#?}` is always used regardless of length.
    #[must_use]
    pub fn rebuild_debug<T: std::fmt::Debug>(mut self, value: &T) -> Self {
        let formatted = if self.expand_all {
            format!("{:#?}", value)
        } else {
            // Try compact form first; fall back to pretty if it's too wide.
            let compact = format!("{:?}", value);
            if compact.len() <= 88 {
                compact
            } else {
                format!("{:#?}", value)
            }
        };
        let processed =
            apply_debug_params(&formatted, self.max_length, self.max_string, self.max_depth);
        let hl = ReprHighlighter::new();
        self.text = hl.apply(&processed);
        self
    }

    // -- Indent guides ------------------------------------------------------

    /// Apply indent guides to the underlying text.
    ///
    /// For each line, leading spaces are inspected. At every `indent_size`
    /// boundary within the leading whitespace, the space character is replaced
    /// with a vertical bar (`│`) styled with the `"repr.indent"` theme style
    /// (or `dim green` as the fallback).
    fn apply_indent_guides(&self, console: Option<&Console>) -> Text {
        if !self.indent_guides {
            return self.text.clone();
        }

        // Cache the fallback style across calls — was re-parsing on every render.
        static GUIDE_STYLE_DEFAULT: std::sync::LazyLock<Style> =
            std::sync::LazyLock::new(|| Style::parse("dim green"));
        let guide_style = console
            .and_then(|c| c.get_style("repr.indent").ok())
            .unwrap_or_else(|| GUIDE_STYLE_DEFAULT.clone());
        self.text
            .with_indent_guides(Some(self.indent_size), '\u{2502}', guide_style)
    }

    // -- Measurement --------------------------------------------------------

    /// Measure the minimum and maximum widths required to render this widget.
    pub fn measure(&self) -> Measurement {
        self.text.measure()
    }
}

// ---------------------------------------------------------------------------
// pretty_repr free function
// ---------------------------------------------------------------------------

/// Render any [`std::fmt::Debug`] value as a pretty-printed string.
///
/// Constructs a [`Pretty`] widget from the debug representation and renders it
/// at the given `max_width`. Returns the rendered string without a trailing
/// newline.
///
/// # Examples
///
/// ```
/// use gilt::utils::pretty::pretty_repr;
///
/// let s = pretty_repr(&vec![1, 2, 3], 80);
/// assert!(s.contains('1'));
/// assert!(!s.ends_with('\n'));
/// ```
pub fn pretty_repr<T: std::fmt::Debug>(value: &T, max_width: usize) -> String {
    let mut console = Console::builder()
        .width(max_width)
        .force_terminal(true)
        .no_color(true)
        .build();
    console.begin_capture();
    console.print(&Pretty::from_debug(value));
    let output = console.end_capture();
    output.trim_end_matches('\n').to_string()
}

// -- Renderable implementation ----------------------------------------------

impl Renderable for Pretty {
    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let mut text = self.apply_indent_guides(Some(console));

        if self.no_wrap {
            text.no_wrap = Some(true);
        }
        if let Some(overflow) = self.overflow {
            text.overflow = Some(overflow);
        }

        if self.type_annotation {
            let type_name = infer_type_name(self.text.plain());
            // P2 perf: parse once, reuse on every render
            static ANNOTATION_STYLE: std::sync::LazyLock<Style> =
                std::sync::LazyLock::new(|| Style::parse("dim italic"));
            use crate::text::TextPart;
            text = Text::assemble(
                &[
                    TextPart::Styled(format!("({}) ", type_name), ANNOTATION_STYLE.clone()),
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

/// Formatting options threaded through the recursive JSON formatter.
#[cfg(feature = "json")]
#[derive(Clone, Copy)]
struct JsonFmtOpts {
    indent_size: usize,
    max_length: Option<usize>,
    max_string: Option<usize>,
    expand_all: bool,
    max_depth: Option<usize>,
    /// Terminal cell width used for expand/collapse threshold.
    max_width: usize,
}

/// Format a JSON value as a pretty-printed string, respecting `max_length`,
/// `max_string`, `expand_all`, `max_depth`, and `max_width` parameters.
#[cfg(feature = "json")]
fn format_json_value(value: &serde_json::Value, depth: usize, opts: JsonFmtOpts) -> String {
    let JsonFmtOpts {
        max_depth,
        max_string,
        ..
    } = opts;
    // P2 parity: when max_depth is exceeded, render placeholder instead of contents
    if let Some(max_d) = max_depth {
        if depth > max_d {
            match value {
                serde_json::Value::Array(_) => return "[...]".to_string(),
                serde_json::Value::Object(_) => return "{...}".to_string(),
                _ => {} // scalars render normally even at depth > max_d
            }
        }
    }

    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => {
            // P1 parity: rich closes the quote BEFORE the +N indicator:
            //   "kept"+N  not  "kept+N"
            match max_string {
                Some(max) if s.chars().count() > max => {
                    let kept: String = s.chars().take(max).collect();
                    let remaining = s.chars().count() - max;
                    format!("\"{}\"+{}", escape_json_string(&kept), remaining)
                }
                _ => format!("\"{}\"", escape_json_string(s)),
            }
        }
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                return "[]".to_string();
            }
            format_json_array(arr, depth, opts)
        }
        serde_json::Value::Object(obj) => {
            if obj.is_empty() {
                return "{}".to_string();
            }
            format_json_object(obj, depth, opts)
        }
    }
}

/// Format a JSON array with optional truncation and forced expansion.
#[cfg(feature = "json")]
fn format_json_array(arr: &[serde_json::Value], depth: usize, opts: JsonFmtOpts) -> String {
    let JsonFmtOpts {
        indent_size,
        max_length,
        expand_all,
        max_width,
        ..
    } = opts;
    let total = arr.len();
    let display_count = match max_length {
        Some(max) => max.min(total),
        None => total,
    };
    let truncated_count = total - display_count;

    let items: Vec<String> = arr[..display_count]
        .iter()
        .map(|v| format_json_value(v, depth + 1, opts))
        .collect();

    let should_expand = if expand_all {
        true
    } else {
        // P2/P3 parity+perf: compare cell width (not raw byte count) against
        // render context max_width (not a hardcoded 80).
        let compact_len: usize = items
            .iter()
            .map(|s| crate::utils::cells::cell_len(s))
            .sum::<usize>()
            + items.len().saturating_sub(1) * 2; // ", " separators
        compact_len > max_width || items.iter().any(|s| s.contains('\n'))
    };

    if should_expand {
        let indent = " ".repeat(indent_size * (depth + 1));
        let closing_indent = " ".repeat(indent_size * depth);
        let mut parts: Vec<String> = items
            .iter()
            .map(|item| format!("{}{}", indent, item))
            .collect();
        if truncated_count > 0 {
            parts.push(format!("{}... +{}", indent, truncated_count));
        }
        format!("[\n{}\n{}]", parts.join(",\n"), closing_indent)
    } else {
        let mut result = items.join(", ");
        if truncated_count > 0 {
            result.push_str(&format!(", ... +{}", truncated_count));
        }
        format!("[{}]", result)
    }
}

/// Format a JSON object with optional truncation and forced expansion.
#[cfg(feature = "json")]
fn format_json_object(
    obj: &serde_json::Map<String, serde_json::Value>,
    depth: usize,
    opts: JsonFmtOpts,
) -> String {
    let JsonFmtOpts {
        indent_size,
        max_length,
        expand_all,
        max_width,
        ..
    } = opts;
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
            let val_str = format_json_value(v, depth + 1, opts);
            format!("{}: {}", key_str, val_str)
        })
        .collect();

    let should_expand = if expand_all {
        true
    } else {
        // P2/P3 parity+perf: compare cell width against render context max_width
        let compact_len: usize = items
            .iter()
            .map(|s| crate::utils::cells::cell_len(s))
            .sum::<usize>()
            + items.len().saturating_sub(1) * 2; // ", " separators
        compact_len > max_width || items.iter().any(|s| s.contains('\n'))
    };

    if should_expand {
        let indent = " ".repeat(indent_size * (depth + 1));
        let closing_indent = " ".repeat(indent_size * depth);
        let mut parts: Vec<String> = items
            .iter()
            .map(|item| format!("{}{}", indent, item))
            .collect();
        if truncated_count > 0 {
            parts.push(format!("{}... +{}", indent, truncated_count));
        }
        format!("{{\n{}\n{}}}", parts.join(",\n"), closing_indent)
    } else {
        let mut result = items.join(", ");
        if truncated_count > 0 {
            result.push_str(&format!(", ... +{}", truncated_count));
        }
        format!("{{{}}}", result)
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

/// Apply `max_length`, `max_string`, `max_depth`, and `expand_all` to a
/// Debug-formatted string.
///
/// This works by post-processing the already-formatted debug string:
/// - `max_string`: truncates quoted string literals
/// - `max_length`: truncates items in bracket/brace-delimited collections
/// - `max_depth`: replaces nested content beyond depth limit with `{...}`/`[...]`
fn apply_debug_params(
    formatted: &str,
    max_length: Option<usize>,
    max_string: Option<usize>,
    max_depth: Option<usize>,
) -> String {
    let mut result = formatted.to_string();
    if let Some(max_s) = max_string {
        result = truncate_debug_strings(&result, max_s);
    }
    // Deep-review fix: apply max_depth BEFORE max_length, matching rich's
    // traversal order where depth pruning happens during the recursive walk
    // (before length truncation operates on the tree).  This ensures length
    // truncation sees the depth-pruned tree and `... +N` markers aren't
    // swallowed into `{...}` collapses.
    if let Some(max_d) = max_depth {
        result = apply_max_depth_debug(&result, max_d);
    }
    if let Some(max_l) = max_length {
        result = truncate_debug_collections(&result, max_l);
    }
    result
}

/// Prune nested braces/brackets beyond `max_depth` in a Debug-formatted string.
///
/// Scans the string tracking brace/bracket nesting depth. When depth exceeds
/// `max_depth`, the contents of that bracket are replaced with `...`.
fn apply_max_depth_debug(s: &str, max_depth: usize) -> String {
    let mut result = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut depth = 0usize;

    while i < len {
        let ch = chars[i];
        match ch {
            '{' | '[' => {
                result.push(ch);
                if depth >= max_depth {
                    // Replace the contents with '...' and find the matching close
                    result.push_str("...");
                    let close = if ch == '{' { '}' } else { ']' };
                    let mut inner_depth = 1usize;
                    i += 1;
                    while i < len && inner_depth > 0 {
                        if chars[i] == '{' || chars[i] == '[' {
                            inner_depth += 1;
                        } else if chars[i] == '}' || chars[i] == ']' {
                            inner_depth -= 1;
                        }
                        i += 1;
                    }
                    result.push(close);
                    // Adjust: we already incremented i past the close above.
                    // But depth doesn't change since we consumed without nesting.
                    continue;
                } else {
                    depth += 1;
                }
            }
            '}' | ']' => {
                result.push(ch);
                depth = depth.saturating_sub(1);
            }
            _ => {
                result.push(ch);
            }
        }
        i += 1;
    }
    result
}

/// Truncate quoted string literals in a Debug-formatted string.
///
/// P3 perf: uses `char_indices()` + a counter instead of materialising
/// a full `Vec<char>` per call.
fn truncate_debug_strings(s: &str, max_string: usize) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.char_indices().peekable();
    while let Some((_, ch)) = chars.next() {
        if ch == '"' {
            // Found start of a string literal — collect content
            result.push('"');
            let mut content = String::new();
            loop {
                match chars.next() {
                    None => break,
                    Some((_, '"')) => {
                        // Closing quote: truncate content if needed, then close
                        let char_count = content.chars().count();
                        if char_count > max_string {
                            let kept: String = content.chars().take(max_string).collect();
                            let remaining = char_count - max_string;
                            result.push_str(&kept);
                            result.push('"');
                            result.push_str(&format!("+{}", remaining));
                        } else {
                            result.push_str(&content);
                            result.push('"');
                        }
                        break;
                    }
                    Some((_, '\\')) => {
                        // Escape sequence: consume the next char too
                        content.push('\\');
                        if let Some((_, escaped)) = chars.next() {
                            content.push(escaped);
                        }
                    }
                    Some((_, c)) => {
                        content.push(c);
                    }
                }
            }
        } else {
            result.push(ch);
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

/// Truncate items in a single-line collection like `[1, 2, 3, 4, 5]` or
/// `{a: 1, b: 2, c: 3}` (brace collections from Debug output).
fn truncate_inline_collection(s: &str, max_length: usize) -> String {
    // Try bracket `[...]` first
    if let Some(start) = s.find('[') {
        if let Some(end) = s.rfind(']') {
            if start < end {
                let inner = &s[start + 1..end];
                let truncated = truncate_comma_items(inner, max_length);
                return format!("{}[{}]{}", &s[..start], truncated, &s[end + 1..]);
            }
        }
    }
    // Try brace `{...}` (Debug collections like `{field: val, ...}`)
    if let Some(start) = s.find('{') {
        if let Some(end) = s.rfind('}') {
            if start < end {
                let inner = &s[start + 1..end];
                let truncated = truncate_comma_items(inner, max_length);
                return format!("{}{{{}}}{}", &s[..start], truncated, &s[end + 1..]);
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
    format!("{}, ... +{}", kept.join(", "), remaining)
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
                result.push(format!("{}... +{},", pad, skipped_count));
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
