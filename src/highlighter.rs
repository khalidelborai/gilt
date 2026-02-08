//! Regex-based text highlighting.
//!
//! This module provides a trait hierarchy for applying regex-based highlighting
//! to [`Text`] objects. It is a port of Python's `rich/highlighter.py`.
//!
//! The core abstraction is the [`Highlighter`] trait, which defines how to
//! apply highlighting in-place to a [`Text`] instance. Several pre-configured
//! highlighters are provided:
//!
//! - [`NullHighlighter`] — does nothing (disables highlighting).
//! - [`RegexHighlighter`] — applies highlighting from a list of compiled regexes.
//! - [`ReprHighlighter`] — patterns for repr-style output (numbers, strings, booleans, etc.).
//! - [`JSONHighlighter`] — patterns for JSON (braces, strings, numbers, keys).
//! - [`ISO8601Highlighter`] — patterns for ISO 8601 date/time strings.

use once_cell::sync::Lazy;
use regex::Regex;

use crate::default_styles::DEFAULT_STYLES;
use crate::style::Style;
use crate::text::{Span, Text};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Combine multiple regex pattern strings into a single pattern using `|`.
fn combine_regex(patterns: &[&str]) -> String {
    patterns.join("|")
}

/// Apply a regex with named capture groups to a Text, looking up styles from
/// `DEFAULT_STYLES` using `"{prefix}{group_name}"` as the key.
///
/// This is equivalent to `Text::highlight_regex_with_groups` but uses the
/// default styles map for name resolution instead of `Style::parse()`.
fn highlight_with_groups(text: &mut Text, pattern: &Regex, style_prefix: &str) -> usize {
    let plain = text.plain().to_string();
    let mut count = 0;
    for captures in pattern.captures_iter(&plain) {
        for name in pattern.capture_names().flatten() {
            if let Some(mat) = captures.name(name) {
                let style_name = format!("{}{}", style_prefix, name);
                if let Some(style) = DEFAULT_STYLES.get(&style_name) {
                    let byte_start = mat.start();
                    let byte_end = mat.end();
                    let char_start = plain[..byte_start].chars().count();
                    let char_end = plain[..byte_end].chars().count();
                    text.stylize(style.clone(), char_start, Some(char_end));
                    count += 1;
                }
            }
        }
    }
    count
}

// ---------------------------------------------------------------------------
// Highlighter trait
// ---------------------------------------------------------------------------

/// Trait for objects that apply highlighting to [`Text`].
pub trait Highlighter {
    /// Apply highlighting in-place to `text`.
    fn highlight(&self, text: &mut Text);

    /// Highlight a plain string, returning a new highlighted [`Text`].
    fn apply(&self, text: &str) -> Text {
        let mut t = Text::new(text, Style::null());
        self.highlight(&mut t);
        t
    }

    /// Clone a [`Text`], highlight the clone, and return it.
    fn apply_text(&self, text: &Text) -> Text {
        let mut t = text.clone();
        self.highlight(&mut t);
        t
    }
}

// ---------------------------------------------------------------------------
// NullHighlighter
// ---------------------------------------------------------------------------

/// A highlighter that does nothing — used to disable highlighting entirely.
pub struct NullHighlighter;

impl Highlighter for NullHighlighter {
    fn highlight(&self, _text: &mut Text) {}
}

// ---------------------------------------------------------------------------
// RegexHighlighter
// ---------------------------------------------------------------------------

/// Applies highlighting from a list of compiled regular expressions.
///
/// Each regex should use **named capture groups**; matching group names are
/// concatenated with `base_style` to form a key into `DEFAULT_STYLES`.
pub struct RegexHighlighter {
    /// Compiled regex patterns.
    pub highlights: Vec<Regex>,
    /// Style prefix prepended to capture-group names (e.g. `"repr."`).
    pub base_style: String,
}

impl Highlighter for RegexHighlighter {
    fn highlight(&self, text: &mut Text) {
        for re in &self.highlights {
            highlight_with_groups(text, re, &self.base_style);
        }
    }
}

// ---------------------------------------------------------------------------
// ReprHighlighter
// ---------------------------------------------------------------------------

/// Pre-compiled regex patterns for [`ReprHighlighter`].
static REPR_HIGHLIGHTS: Lazy<Vec<Regex>> = Lazy::new(|| {
    let patterns: Vec<&str> = vec![
        // Tags: <tag_name contents>
        r"(?P<tag_start><)(?P<tag_name>[-\w.:|]*)(?P<tag_contents>[\w\W]*)(?P<tag_end>>)",
        // Attribute name=value
        r#"(?P<attrib_name>[\w_]{1,50})=(?P<attrib_value>"?[\w_]+"?)?"#,
        // Braces
        r"(?P<brace>[\[\]{}\(\)])",
        // Combined pattern for everything else
        &REPR_COMBINED,
    ];
    patterns
        .iter()
        .map(|p| Regex::new(p).expect("invalid repr highlight regex"))
        .collect()
});

/// The combined pattern for the repr highlighter (IP addresses, UUIDs, calls,
/// booleans, numbers, paths, strings, URLs).
///
/// Note: Rust's `regex` crate does not support look-behind assertions.
/// Patterns have been adapted to use word boundaries or other anchoring
/// techniques instead.
static REPR_COMBINED: Lazy<String> = Lazy::new(|| {
    combine_regex(&[
        // IPv4
        r"(?P<ipv4>[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3})",
        // IPv6
        r"(?P<ipv6>([A-Fa-f0-9]{1,4}::?){1,7}[A-Fa-f0-9]{1,4})",
        // EUI-64
        r"(?P<eui64>(?:[0-9A-Fa-f]{1,2}-){7}[0-9A-Fa-f]{1,2}|(?:[0-9A-Fa-f]{1,2}:){7}[0-9A-Fa-f]{1,2}|(?:[0-9A-Fa-f]{4}\.){3}[0-9A-Fa-f]{4})",
        // EUI-48
        r"(?P<eui48>(?:[0-9A-Fa-f]{1,2}-){5}[0-9A-Fa-f]{1,2}|(?:[0-9A-Fa-f]{1,2}:){5}[0-9A-Fa-f]{1,2}|(?:[0-9A-Fa-f]{4}\.){2}[0-9A-Fa-f]{4})",
        // UUID
        r"(?P<uuid>[a-fA-F0-9]{8}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{12})",
        // Function call
        r"(?P<call>[\w.]*?)\(",
        // Booleans and None — supports both Python (True/False) and Rust (true/false)
        r"\b(?P<bool_true>true|True)\b|\b(?P<bool_false>false|False)\b|\b(?P<none>None)\b",
        // Ellipsis
        r"(?P<ellipsis>\.\.\.)",
        // Complex number (use \b instead of look-behind)
        r"(?P<number_complex>\b-?[0-9]+\.?[0-9]*(?:e[-+]?\d+?)?(?:[-+](?:[0-9]+\.?[0-9]*(?:e[-+]?\d+)?))?j)",
        // Number (int, float, hex) — replaced look-behind with \b
        r"(?P<number>\b-?[0-9]+\.?[0-9]*(e[-+]?\d+?)?\b|0x[0-9a-fA-F]*)",
        // Path and filename
        r"(?P<path>\B(/[-\w._+]+)*\/)(?P<filename>[-\w._+]*)?",
        // Strings (single/double/triple quoted, optional b prefix)
        // Simplified to avoid look-behind assertions not supported by Rust regex.
        r#"(?P<str>b?'''[^']*'''|b?'[^']*'|b?"""[^"]*"""|b?"[^"]*")"#,
        // URLs
        r"(?P<url>(file|https|http|ws|wss)://[-0-9a-zA-Z$_+!`(),.?/;:&=%#~@]*)",
    ])
});

/// Highlights the text typically produced by `repr` / debug output.
///
/// Pre-configured with patterns for tags, attributes, braces, IP addresses,
/// UUIDs, function calls, booleans, numbers, paths, strings, and URLs.
pub struct ReprHighlighter;

impl ReprHighlighter {
    /// Create a new `ReprHighlighter`.
    pub fn new() -> Self {
        ReprHighlighter
    }
}

impl Default for ReprHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl Highlighter for ReprHighlighter {
    fn highlight(&self, text: &mut Text) {
        for re in REPR_HIGHLIGHTS.iter() {
            highlight_with_groups(text, re, "repr.");
        }
    }
}

// ---------------------------------------------------------------------------
// JSONHighlighter
// ---------------------------------------------------------------------------

/// Regex pattern for JSON string values.
/// Simplified to avoid look-behind assertions not supported by Rust regex crate.
const JSON_STR_PATTERN: &str = r#"(?P<str>"[^"\\]*(?:\\.[^"\\]*)*")"#;

/// Pre-compiled regex patterns for [`JSONHighlighter`].
static JSON_HIGHLIGHTS: Lazy<Vec<Regex>> = Lazy::new(|| {
    let combined = combine_regex(&[
        r"(?P<brace>[\{\[\(\)\]\}])",
        r"\b(?P<bool_true>true)\b|\b(?P<bool_false>false)\b|\b(?P<null>null)\b",
        r"(?P<number>\b-?[0-9]+\.?[0-9]*(e[-+]?\d+?)?\b|0x[0-9a-fA-F]*)",
        JSON_STR_PATTERN,
    ]);
    vec![Regex::new(&combined).expect("invalid json highlight regex")]
});

/// Pre-compiled regex for detecting JSON string spans (used for key detection).
/// Uses a non-capturing version since we only need match positions.
static JSON_STR_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#""[^"\\]*(?:\\.[^"\\]*)*""#).expect("invalid json str regex"));

/// Characters considered whitespace in JSON.
const JSON_WHITESPACE: &[char] = &[' ', '\n', '\r', '\t'];

/// Highlights JSON text.
///
/// After applying the base regex highlights, this highlighter additionally
/// scans for JSON keys (strings followed by `:`) and applies the `"json.key"`
/// style to them.
pub struct JSONHighlighter;

impl JSONHighlighter {
    /// Create a new `JSONHighlighter`.
    pub fn new() -> Self {
        JSONHighlighter
    }
}

impl Default for JSONHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl Highlighter for JSONHighlighter {
    fn highlight(&self, text: &mut Text) {
        // Base regex highlighting using default styles
        for re in JSON_HIGHLIGHTS.iter() {
            highlight_with_groups(text, re, "json.");
        }

        // Additional pass: detect JSON keys (strings followed by ':')
        let plain = text.plain().to_string();
        let plain_chars: Vec<char> = plain.chars().collect();
        let plain_len = plain_chars.len();

        if let Some(key_style) = DEFAULT_STYLES.get("json.key") {
            for mat in JSON_STR_RE.find_iter(&plain) {
                let byte_start = mat.start();
                let byte_end = mat.end();
                let char_start = plain[..byte_start].chars().count();
                let char_end = plain[..byte_end].chars().count();

                // Walk forward from end of match, skipping whitespace
                let mut cursor = char_end;
                while cursor < plain_len {
                    let ch = plain_chars[cursor];
                    cursor += 1;
                    if ch == ':' {
                        // This string is a JSON key
                        text.spans_mut()
                            .push(Span::new(char_start, char_end, key_style.clone()));
                        break;
                    } else if JSON_WHITESPACE.contains(&ch) {
                        continue;
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// ISO8601Highlighter
// ---------------------------------------------------------------------------

/// Pre-compiled regex patterns for [`ISO8601Highlighter`].
///
/// Note: The Rust `regex` crate does not support conditional backreferences
/// `(?(name)...)`. The Python pattern for "calendar date with time" used
/// `(?(hyphen)-)` to conditionally require hyphens/colons. We replace it with
/// two separate patterns: one with delimiters and one without.
static ISO8601_HIGHLIGHTS: Lazy<Vec<Regex>> = Lazy::new(|| {
    let patterns: Vec<&str> = vec![
        // Calendar month (e.g. 2008-08). The hyphen is required.
        r"^(?P<date>(?P<year>[0-9]{4})-(?P<month>1[0-2]|0[1-9]))$",
        // Calendar date without hyphens (e.g. 20080830)
        r"^(?P<date>(?P<year>[0-9]{4})(?P<month>1[0-2]|0[1-9])(?P<day>3[01]|0[1-9]|[12][0-9]))$",
        // Ordinal date (e.g. 2008-243). The hyphen is optional.
        r"^(?P<date>(?P<year>[0-9]{4})-?(?P<day>36[0-6]|3[0-5][0-9]|[12][0-9]{2}|0[1-9][0-9]|00[1-9]))$",
        // Week of the year (e.g. 2008-W35). The hyphen is optional.
        r"^(?P<date>(?P<year>[0-9]{4})-?W(?P<week>5[0-3]|[1-4][0-9]|0[1-9]))$",
        // Week date (e.g. 2008-W35-6). The hyphens are optional.
        r"^(?P<date>(?P<year>[0-9]{4})-?W(?P<week>5[0-3]|[1-4][0-9]|0[1-9])-?(?P<day>[1-7]))$",
        // Hours and minutes (e.g. 17:21). The colon is optional.
        r"^(?P<time>(?P<hour>2[0-3]|[01][0-9]):?(?P<minute>[0-5][0-9]))$",
        // Hours, minutes, and seconds without colons (e.g. 172159)
        r"^(?P<time>(?P<hour>2[0-3]|[01][0-9])(?P<minute>[0-5][0-9])(?P<second>[0-5][0-9]))$",
        // Time zone designator (e.g. Z, +07, +07:00)
        r"^(?P<timezone>Z|[+-](?:2[0-3]|[01][0-9])(?::?(?:[0-5][0-9]))?)$",
        // Hours, minutes, and seconds with timezone (e.g. 17:21:59+07:00)
        r"^(?P<time>(?P<hour>2[0-3]|[01][0-9])(?P<minute>[0-5][0-9])(?P<second>[0-5][0-9]))(?P<timezone>Z|[+-](?:2[0-3]|[01][0-9])(?::?(?:[0-5][0-9]))?)$",
        // Calendar date with time — WITH hyphens/colons (e.g. 2008-08-30 17:21:59)
        r"^(?P<date>(?P<year>[0-9]{4})-(?P<month>1[0-2]|0[1-9])-(?P<day>3[01]|0[1-9]|[12][0-9])) (?P<time>(?P<hour>2[0-3]|[01][0-9]):(?P<minute>[0-5][0-9]):(?P<second>[0-5][0-9]))$",
        // Calendar date with time — WITHOUT hyphens/colons (e.g. 20080830 172159)
        r"^(?P<date>(?P<year>[0-9]{4})(?P<month>1[0-2]|0[1-9])(?P<day>3[01]|0[1-9]|[12][0-9])) (?P<time>(?P<hour>2[0-3]|[01][0-9])(?P<minute>[0-5][0-9])(?P<second>[0-5][0-9]))$",
        // XML Schema date (e.g. 2008-08-30 or 2008-08-30+07:00)
        r"^(?P<date>(?P<year>-?(?:[1-9][0-9]*)?[0-9]{4})-(?P<month>1[0-2]|0[1-9])-(?P<day>3[01]|0[1-9]|[12][0-9]))(?P<timezone>Z|[+-](?:2[0-3]|[01][0-9]):[0-5][0-9])?$",
        // XML Schema time (e.g. 01:45:36 or 01:45:36.123+07:00)
        r"^(?P<time>(?P<hour>2[0-3]|[01][0-9]):(?P<minute>[0-5][0-9]):(?P<second>[0-5][0-9])(?P<frac>\.[0-9]+)?)(?P<timezone>Z|[+-](?:2[0-3]|[01][0-9]):[0-5][0-9])?$",
        // XML Schema dateTime (e.g. 2008-08-30T01:45:36.123Z)
        r"^(?P<date>(?P<year>-?(?:[1-9][0-9]*)?[0-9]{4})-(?P<month>1[0-2]|0[1-9])-(?P<day>3[01]|0[1-9]|[12][0-9]))T(?P<time>(?P<hour>2[0-3]|[01][0-9]):(?P<minute>[0-5][0-9]):(?P<second>[0-5][0-9])(?P<ms>\.[0-9]+)?)(?P<timezone>Z|[+-](?:2[0-3]|[01][0-9]):[0-5][0-9])?$",
    ];
    patterns
        .iter()
        .map(|p| Regex::new(p).expect("invalid iso8601 highlight regex"))
        .collect()
});

/// Highlights ISO 8601 date/time strings.
///
/// Pre-configured with patterns for calendar dates, ordinal dates, week dates,
/// times (with and without timezone), and combined datetime formats.
pub struct ISO8601Highlighter;

impl ISO8601Highlighter {
    /// Create a new `ISO8601Highlighter`.
    pub fn new() -> Self {
        ISO8601Highlighter
    }
}

impl Default for ISO8601Highlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl Highlighter for ISO8601Highlighter {
    fn highlight(&self, text: &mut Text) {
        for re in ISO8601_HIGHLIGHTS.iter() {
            highlight_with_groups(text, re, "iso8601.");
        }
    }
}

// ---------------------------------------------------------------------------
// URLHighlighter
// ---------------------------------------------------------------------------

/// Highlights URLs in text with blue underline styling.
///
/// Matches `http://` and `https://` URLs and applies a bold blue underline
/// style to them.
pub struct URLHighlighter {
    /// The style applied to matched URLs.
    pub style: Style,
}

impl URLHighlighter {
    /// Create a new `URLHighlighter` with the default blue underline style.
    pub fn new() -> Self {
        Self {
            style: Style::parse("bold blue underline").unwrap(),
        }
    }
}

impl Default for URLHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl Highlighter for URLHighlighter {
    fn highlight(&self, text: &mut Text) {
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"https?://[^\s)>\]]+").unwrap());
        text.highlight_regex(&RE, self.style.clone());
    }
}

// ---------------------------------------------------------------------------
// ISODateHighlighter
// ---------------------------------------------------------------------------

/// Highlights ISO 8601 date/time strings with green styling.
///
/// Matches dates like `2024-01-15` and full datetimes like
/// `2024-01-15T10:30:00Z` or `2024-01-15T10:30:00+05:30`.
pub struct ISODateHighlighter {
    /// The style applied to matched date/time strings.
    pub style: Style,
}

impl ISODateHighlighter {
    /// Create a new `ISODateHighlighter` with the default bold green style.
    pub fn new() -> Self {
        Self {
            style: Style::parse("bold green").unwrap(),
        }
    }
}

impl Default for ISODateHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl Highlighter for ISODateHighlighter {
    fn highlight(&self, text: &mut Text) {
        static RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"\d{4}-\d{2}-\d{2}(?:T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})?)?")
                .unwrap()
        });
        text.highlight_regex(&RE, self.style.clone());
    }
}

// ---------------------------------------------------------------------------
// UUIDHighlighter
// ---------------------------------------------------------------------------

/// Highlights UUID strings with yellow styling.
///
/// Matches standard UUID format: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`
/// where `x` is a hexadecimal digit.
pub struct UUIDHighlighter {
    /// The style applied to matched UUIDs.
    pub style: Style,
}

impl UUIDHighlighter {
    /// Create a new `UUIDHighlighter` with the default bold yellow style.
    pub fn new() -> Self {
        Self {
            style: Style::parse("bold yellow").unwrap(),
        }
    }
}

impl Default for UUIDHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl Highlighter for UUIDHighlighter {
    fn highlight(&self, text: &mut Text) {
        static RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(
                r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}",
            )
            .unwrap()
        });
        text.highlight_regex(&RE, self.style.clone());
    }
}

// ---------------------------------------------------------------------------
// JSONPathHighlighter
// ---------------------------------------------------------------------------

/// Highlights JSON-like dot-notation paths with magenta styling.
///
/// Matches paths like `.data.users[0].name` or `$.config.timeout`.
pub struct JSONPathHighlighter {
    /// The style applied to matched JSON paths.
    pub style: Style,
}

impl JSONPathHighlighter {
    /// Create a new `JSONPathHighlighter` with the default bold magenta style.
    pub fn new() -> Self {
        Self {
            style: Style::parse("bold magenta").unwrap(),
        }
    }
}

impl Default for JSONPathHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl Highlighter for JSONPathHighlighter {
    fn highlight(&self, text: &mut Text) {
        // JSON paths like .foo.bar[0].baz or $.data.users
        static RE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"\$?(?:\.[a-zA-Z_]\w*(?:\[\d+\])?)+").unwrap());
        text.highlight_regex(&RE, self.style.clone());
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: extract the substring of `plain` that a span covers.
    fn span_text<'a>(plain: &'a str, span: &Span) -> &'a str {
        let start_byte = plain
            .char_indices()
            .nth(span.start)
            .map(|(i, _)| i)
            .unwrap_or(plain.len());
        let end_byte = plain
            .char_indices()
            .nth(span.end)
            .map(|(i, _)| i)
            .unwrap_or(plain.len());
        &plain[start_byte..end_byte]
    }

    // -- NullHighlighter ----------------------------------------------------

    #[test]
    fn test_null_highlighter_does_nothing() {
        let hl = NullHighlighter;
        let mut text = Text::new("Hello, World!", Style::null());
        hl.highlight(&mut text);
        assert!(text.spans().is_empty());
        assert_eq!(text.plain(), "Hello, World!");
    }

    // -- RegexHighlighter (basic) -------------------------------------------

    #[test]
    fn test_regex_highlighter_basic() {
        let re = Regex::new(r"(?P<brace>[\[\]\{\}\(\)])").unwrap();
        let hl = RegexHighlighter {
            highlights: vec![re],
            base_style: "repr.".to_string(),
        };
        let mut text = Text::new("[hello]", Style::null());
        hl.highlight(&mut text);
        // "repr.brace" exists in DEFAULT_STYLES, so spans should be created
        assert!(!text.spans().is_empty());
        assert_eq!(text.plain(), "[hello]");
    }

    // -- apply() and apply_text() ------------------------------------------

    #[test]
    fn test_apply_creates_new_text() {
        let hl = NullHighlighter;
        let result = hl.apply("Hello");
        assert_eq!(result.plain(), "Hello");
        assert!(result.spans().is_empty());
    }

    #[test]
    fn test_apply_text_clones_and_highlights() {
        let hl = NullHighlighter;
        let original = Text::new("Hello", Style::null());
        let result = hl.apply_text(&original);
        assert_eq!(result.plain(), "Hello");
        assert!(result.spans().is_empty());
        // Original is not consumed
        assert_eq!(original.plain(), "Hello");
    }

    // -- ReprHighlighter ----------------------------------------------------

    #[test]
    fn test_repr_highlighter_new() {
        let hl = ReprHighlighter::new();
        let text = hl.apply("hello");
        assert_eq!(text.plain(), "hello");
    }

    #[test]
    fn test_repr_highlighter_default() {
        let hl = ReprHighlighter::default();
        let text = hl.apply("42");
        assert_eq!(text.plain(), "42");
    }

    #[test]
    fn test_repr_numbers() {
        let hl = ReprHighlighter::new();
        let text = hl.apply("value=42 pi=3.14 hex=0xFF");
        assert_eq!(text.plain(), "value=42 pi=3.14 hex=0xFF");
        // Should have spans for numbers and attribs
        assert!(!text.spans().is_empty());
    }

    #[test]
    fn test_repr_booleans() {
        let hl = ReprHighlighter::new();
        let text = hl.apply("flag=true other=false");
        assert_eq!(text.plain(), "flag=true other=false");
        assert!(!text.spans().is_empty());

        // Check that we have bool_true and bool_false spans
        let plain = text.plain();
        let has_true = text.spans().iter().any(|s| span_text(plain, s) == "true");
        let has_false = text.spans().iter().any(|s| span_text(plain, s) == "false");
        assert!(has_true, "expected a span for 'true'");
        assert!(has_false, "expected a span for 'false'");
    }

    #[test]
    fn test_repr_strings() {
        let hl = ReprHighlighter::new();
        let text = hl.apply(r#"name="hello""#);
        assert_eq!(text.plain(), r#"name="hello""#);
        assert!(!text.spans().is_empty());
    }

    #[test]
    fn test_repr_urls() {
        let hl = ReprHighlighter::new();
        let text = hl.apply("visit https://example.com/path?q=1");
        assert_eq!(text.plain(), "visit https://example.com/path?q=1");
        let plain = text.plain();
        let has_url_span = text
            .spans()
            .iter()
            .any(|s| span_text(plain, s).contains("https://"));
        assert!(has_url_span);
    }

    #[test]
    fn test_repr_paths() {
        let hl = ReprHighlighter::new();
        let text = hl.apply("file at /foo/bar/baz.py here");
        assert_eq!(text.plain(), "file at /foo/bar/baz.py here");
        assert!(!text.spans().is_empty());
    }

    #[test]
    fn test_repr_uuid() {
        let hl = ReprHighlighter::new();
        let text = hl.apply("id=a3db0d5e-66f4-4a29-bfb6-e738b1d3d640");
        assert_eq!(text.plain(), "id=a3db0d5e-66f4-4a29-bfb6-e738b1d3d640");
        assert!(!text.spans().is_empty());

        let plain = text.plain();
        let has_uuid = text
            .spans()
            .iter()
            .any(|s| span_text(plain, s) == "a3db0d5e-66f4-4a29-bfb6-e738b1d3d640");
        assert!(has_uuid, "expected a span for the UUID");
    }

    #[test]
    fn test_repr_ipv4() {
        let hl = ReprHighlighter::new();
        let text = hl.apply("host=192.168.1.1");
        assert_eq!(text.plain(), "host=192.168.1.1");
        assert!(!text.spans().is_empty());

        let plain = text.plain();
        let has_ipv4 = text
            .spans()
            .iter()
            .any(|s| span_text(plain, s) == "192.168.1.1");
        assert!(has_ipv4, "expected a span for the IPv4 address");
    }

    #[test]
    fn test_repr_ipv6() {
        let hl = ReprHighlighter::new();
        let text = hl.apply("addr=2001:0db8:85a3:0000:0000:8a2e:0370:7334");
        assert_eq!(text.plain(), "addr=2001:0db8:85a3:0000:0000:8a2e:0370:7334");
        assert!(!text.spans().is_empty());
    }

    #[test]
    fn test_repr_braces() {
        let hl = ReprHighlighter::new();
        let text = hl.apply("[1, {2: (3)}]");
        assert_eq!(text.plain(), "[1, {2: (3)}]");
        assert!(!text.spans().is_empty());

        let plain = text.plain();
        let brace_spans: Vec<_> = text
            .spans()
            .iter()
            .filter(|s| {
                let t = span_text(plain, s);
                t == "[" || t == "]" || t == "{" || t == "}" || t == "(" || t == ")"
            })
            .collect();
        assert!(
            brace_spans.len() >= 6,
            "expected at least 6 brace spans, got {}",
            brace_spans.len()
        );
    }

    #[test]
    fn test_repr_tags() {
        let hl = ReprHighlighter::new();
        let text = hl.apply("<MyTag contents>");
        assert_eq!(text.plain(), "<MyTag contents>");
        assert!(!text.spans().is_empty());
    }

    // -- JSONHighlighter ----------------------------------------------------

    #[test]
    fn test_json_highlighter_new() {
        let hl = JSONHighlighter::new();
        let text = hl.apply("{}");
        assert_eq!(text.plain(), "{}");
    }

    #[test]
    fn test_json_highlighter_default() {
        let hl = JSONHighlighter::default();
        let text = hl.apply("[]");
        assert_eq!(text.plain(), "[]");
    }

    #[test]
    fn test_json_braces() {
        let hl = JSONHighlighter::new();
        let text = hl.apply("{ }");
        assert!(!text.spans().is_empty());
    }

    #[test]
    fn test_json_strings() {
        let hl = JSONHighlighter::new();
        let text = hl.apply(r#""hello""#);
        assert!(!text.spans().is_empty());
    }

    #[test]
    fn test_json_numbers() {
        let hl = JSONHighlighter::new();
        let text = hl.apply("42 3.14 -1 0xFF");
        assert!(!text.spans().is_empty());
    }

    #[test]
    fn test_json_booleans_and_null() {
        let hl = JSONHighlighter::new();
        let text = hl.apply("true false null");
        assert!(!text.spans().is_empty());

        let plain = text.plain();
        let has_true = text.spans().iter().any(|s| span_text(plain, s) == "true");
        let has_false = text.spans().iter().any(|s| span_text(plain, s) == "false");
        let has_null = text.spans().iter().any(|s| span_text(plain, s) == "null");
        assert!(has_true, "expected a span for 'true'");
        assert!(has_false, "expected a span for 'false'");
        assert!(has_null, "expected a span for 'null'");
    }

    #[test]
    fn test_json_key_detection() {
        let hl = JSONHighlighter::new();
        let text = hl.apply(r#"{"name": "value"}"#);
        assert!(!text.spans().is_empty());

        let plain = text.plain();
        let has_key_span = text
            .spans()
            .iter()
            .any(|s| span_text(plain, s) == r#""name""#);
        assert!(
            has_key_span,
            "expected a span covering the JSON key \"name\""
        );
    }

    #[test]
    fn test_json_key_with_whitespace_before_colon() {
        let hl = JSONHighlighter::new();
        let text = hl.apply(r#"{"key"  : "val"}"#);
        let plain = text.plain();
        let has_key_span = text
            .spans()
            .iter()
            .any(|s| span_text(plain, s) == r#""key""#);
        assert!(
            has_key_span,
            "expected a span covering the JSON key \"key\""
        );
    }

    #[test]
    fn test_json_value_string_not_key() {
        let hl = JSONHighlighter::new();
        // "value" is not followed by ':', so should NOT get a key span
        let text = hl.apply(r#"{"key": "value"}"#);
        let plain = text.plain();

        // Spans covering "value"
        let value_spans: Vec<_> = text
            .spans()
            .iter()
            .filter(|s| span_text(plain, s) == r#""value""#)
            .collect();

        // "value" should have at least 1 span (the str highlight)
        assert!(
            !value_spans.is_empty(),
            "expected at least a str span for \"value\""
        );
    }

    // -- ISO8601Highlighter -------------------------------------------------

    #[test]
    fn test_iso8601_highlighter_new() {
        let hl = ISO8601Highlighter::new();
        let text = hl.apply("2024-01-15");
        assert_eq!(text.plain(), "2024-01-15");
    }

    #[test]
    fn test_iso8601_highlighter_default() {
        let hl = ISO8601Highlighter::default();
        let text = hl.apply("12:30");
        assert_eq!(text.plain(), "12:30");
    }

    #[test]
    fn test_iso8601_calendar_date() {
        let hl = ISO8601Highlighter::new();
        let text = hl.apply("2024-01-15");
        // Should match the XML Schema date pattern and highlight year, month, day
        assert!(!text.spans().is_empty());
    }

    #[test]
    fn test_iso8601_time() {
        let hl = ISO8601Highlighter::new();
        let text = hl.apply("17:21:59");
        assert!(!text.spans().is_empty());
    }

    #[test]
    fn test_iso8601_datetime() {
        let hl = ISO8601Highlighter::new();
        let text = hl.apply("2024-01-15T17:21:59Z");
        assert!(!text.spans().is_empty());
    }

    #[test]
    fn test_iso8601_calendar_month() {
        let hl = ISO8601Highlighter::new();
        let text = hl.apply("2024-01");
        assert!(!text.spans().is_empty());
    }

    #[test]
    fn test_iso8601_time_with_timezone() {
        let hl = ISO8601Highlighter::new();
        let text = hl.apply("17:21:59+07:00");
        assert!(!text.spans().is_empty());
    }

    #[test]
    fn test_iso8601_datetime_with_ms() {
        let hl = ISO8601Highlighter::new();
        let text = hl.apply("2024-01-15T17:21:59.123Z");
        assert!(!text.spans().is_empty());
    }

    // -- combine_regex ------------------------------------------------------

    #[test]
    fn test_combine_regex() {
        let result = combine_regex(&["foo", "bar", "baz"]);
        assert_eq!(result, "foo|bar|baz");
    }

    #[test]
    fn test_combine_regex_empty() {
        let result = combine_regex(&[]);
        assert_eq!(result, "");
    }

    #[test]
    fn test_combine_regex_single() {
        let result = combine_regex(&["only"]);
        assert_eq!(result, "only");
    }

    // -- Highlighter trait default methods ----------------------------------

    #[test]
    fn test_apply_with_repr_highlighter() {
        let hl = ReprHighlighter::new();
        let text = hl.apply("count=42");
        assert_eq!(text.plain(), "count=42");
        assert!(!text.spans().is_empty());
    }

    #[test]
    fn test_apply_text_with_repr_highlighter() {
        let hl = ReprHighlighter::new();
        let original = Text::new("count=42", Style::null());
        let result = hl.apply_text(&original);
        assert_eq!(result.plain(), "count=42");
        assert!(!result.spans().is_empty());
        // Original should be unchanged
        assert!(original.spans().is_empty());
    }

    #[test]
    fn test_apply_with_json_highlighter() {
        let hl = JSONHighlighter::new();
        let text = hl.apply(r#"{"a": 1}"#);
        assert_eq!(text.plain(), r#"{"a": 1}"#);
        assert!(!text.spans().is_empty());
    }

    #[test]
    fn test_apply_text_preserves_original() {
        let hl = JSONHighlighter::new();
        let original = Text::new(r#"{"x": true}"#, Style::null());
        let highlighted = hl.apply_text(&original);
        // Original unmodified
        assert!(original.spans().is_empty());
        // Highlighted version has spans
        assert!(!highlighted.spans().is_empty());
    }

    // -- URLHighlighter -----------------------------------------------------

    #[test]
    fn test_url_highlighter_http() {
        let hl = URLHighlighter::new();
        let text = hl.apply("visit http://example.com/page today");
        assert_eq!(text.plain(), "visit http://example.com/page today");
        let plain = text.plain();
        let has_url = text
            .spans()
            .iter()
            .any(|s| span_text(plain, s) == "http://example.com/page");
        assert!(has_url, "expected a span for the http URL");
    }

    #[test]
    fn test_url_highlighter_https() {
        let hl = URLHighlighter::new();
        let text = hl.apply("see https://secure.example.com/path?q=1#frag for details");
        assert_eq!(
            text.plain(),
            "see https://secure.example.com/path?q=1#frag for details"
        );
        let plain = text.plain();
        let has_url = text
            .spans()
            .iter()
            .any(|s| span_text(plain, s).starts_with("https://"));
        assert!(has_url, "expected a span for the https URL");
    }

    #[test]
    fn test_url_highlighter_no_match_plain_text() {
        let hl = URLHighlighter::new();
        let text = hl.apply("just a plain string with no links");
        assert!(
            text.spans().is_empty(),
            "expected no spans for plain text without URLs"
        );
    }

    #[test]
    fn test_url_highlighter_default() {
        let hl = URLHighlighter::default();
        let text = hl.apply("http://test.com");
        assert!(!text.spans().is_empty());
    }

    // -- ISODateHighlighter -------------------------------------------------

    #[test]
    fn test_iso_date_highlighter_date_only() {
        let hl = ISODateHighlighter::new();
        let text = hl.apply("created on 2024-01-15 at noon");
        assert_eq!(text.plain(), "created on 2024-01-15 at noon");
        let plain = text.plain();
        let has_date = text
            .spans()
            .iter()
            .any(|s| span_text(plain, s) == "2024-01-15");
        assert!(has_date, "expected a span for the date");
    }

    #[test]
    fn test_iso_date_highlighter_full_datetime_with_tz() {
        let hl = ISODateHighlighter::new();
        let text = hl.apply("timestamp: 2024-01-15T10:30:00+05:30 end");
        assert_eq!(text.plain(), "timestamp: 2024-01-15T10:30:00+05:30 end");
        let plain = text.plain();
        let has_datetime = text
            .spans()
            .iter()
            .any(|s| span_text(plain, s) == "2024-01-15T10:30:00+05:30");
        assert!(has_datetime, "expected a span for the full datetime");
    }

    #[test]
    fn test_iso_date_highlighter_datetime_utc() {
        let hl = ISODateHighlighter::new();
        let text = hl.apply("at 2024-06-30T23:59:59Z done");
        let plain = text.plain();
        let has_datetime = text
            .spans()
            .iter()
            .any(|s| span_text(plain, s) == "2024-06-30T23:59:59Z");
        assert!(has_datetime, "expected a span for datetime with Z timezone");
    }

    #[test]
    fn test_iso_date_highlighter_default() {
        let hl = ISODateHighlighter::default();
        let text = hl.apply("2024-01-01");
        assert!(!text.spans().is_empty());
    }

    // -- UUIDHighlighter ----------------------------------------------------

    #[test]
    fn test_uuid_highlighter_valid_uuid() {
        let hl = UUIDHighlighter::new();
        let text = hl.apply("id: 550e8400-e29b-41d4-a716-446655440000 done");
        assert_eq!(
            text.plain(),
            "id: 550e8400-e29b-41d4-a716-446655440000 done"
        );
        let plain = text.plain();
        let has_uuid = text
            .spans()
            .iter()
            .any(|s| span_text(plain, s) == "550e8400-e29b-41d4-a716-446655440000");
        assert!(has_uuid, "expected a span for the UUID");
    }

    #[test]
    fn test_uuid_highlighter_ignores_partial_hex() {
        let hl = UUIDHighlighter::new();
        let text = hl.apply("not a uuid: abcdef12-3456");
        assert!(
            text.spans().is_empty(),
            "expected no spans for partial hex string"
        );
    }

    #[test]
    fn test_uuid_highlighter_uppercase() {
        let hl = UUIDHighlighter::new();
        let text = hl.apply("ID=550E8400-E29B-41D4-A716-446655440000");
        let plain = text.plain();
        let has_uuid = text
            .spans()
            .iter()
            .any(|s| span_text(plain, s) == "550E8400-E29B-41D4-A716-446655440000");
        assert!(has_uuid, "expected a span for the uppercase UUID");
    }

    #[test]
    fn test_uuid_highlighter_default() {
        let hl = UUIDHighlighter::default();
        let text = hl.apply("a1b2c3d4-e5f6-7890-abcd-ef1234567890");
        assert!(!text.spans().is_empty());
    }

    // -- JSONPathHighlighter ------------------------------------------------

    #[test]
    fn test_json_path_highlighter_dot_notation() {
        let hl = JSONPathHighlighter::new();
        let text = hl.apply("access .data.users[0].name for the result");
        assert_eq!(text.plain(), "access .data.users[0].name for the result");
        let plain = text.plain();
        let has_path = text
            .spans()
            .iter()
            .any(|s| span_text(plain, s) == ".data.users[0].name");
        assert!(has_path, "expected a span for the JSON path");
    }

    #[test]
    fn test_json_path_highlighter_dollar_prefix() {
        let hl = JSONPathHighlighter::new();
        let text = hl.apply("query $.config.timeout value");
        let plain = text.plain();
        let has_path = text
            .spans()
            .iter()
            .any(|s| span_text(plain, s) == "$.config.timeout");
        assert!(has_path, "expected a span for the $-prefixed JSON path");
    }

    #[test]
    fn test_json_path_highlighter_no_match() {
        let hl = JSONPathHighlighter::new();
        let text = hl.apply("no paths here at all");
        assert!(
            text.spans().is_empty(),
            "expected no spans for text without JSON paths"
        );
    }

    #[test]
    fn test_json_path_highlighter_default() {
        let hl = JSONPathHighlighter::default();
        let text = hl.apply(".foo.bar");
        assert!(!text.spans().is_empty());
    }
}
