//! gilt markup parser — parses `[bold red]text[/]` syntax into styled `Text`.
//!

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use regex::Regex;
use std::sync::LazyLock;

use crate::error::MarkupError;
use crate::style::Style;
use crate::text::{Span, Text};
use crate::utils::emoji_replace::emoji_replace;

// ---------------------------------------------------------------------------
// Tag
// ---------------------------------------------------------------------------

/// A parsed markup tag like `[bold]` or `[link=https://example.com]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    /// Tag name, e.g. "bold", "/bold", "/".
    pub name: String,
    /// Optional parameters after `=`, e.g. the URL in `[link=url]`.
    pub parameters: Option<String>,
}

impl Tag {
    /// Returns the markup representation, e.g. `"[bold]"` or `"[link=url]"`.
    pub fn markup(&self) -> String {
        match &self.parameters {
            Some(params) => format!("[{}={}]", self.name, params),
            None => format!("[{}]", self.name),
        }
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.parameters {
            Some(params) => write!(f, "{} {}", self.name, params),
            None => write!(f, "{}", self.name),
        }
    }
}

// ---------------------------------------------------------------------------
// Regexes
// ---------------------------------------------------------------------------

/// Regex matching markup tags: `\*` runs of backslashes followed by a
/// bracketed tag.
///
/// Group 1 = backslashes before the `[`
/// Group 2 = the bracketed tag (including `[` and `]`)
///
/// Used by both [`escape`] and [`parse_markup`].
static RE_MARKUP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\\*)(\[[a-z#/@][^\[]*?\])").unwrap());

// ---------------------------------------------------------------------------
// escape
// ---------------------------------------------------------------------------

/// Escape `markup` so that it will not be interpreted as gilt markup.
///
/// Potential `[tag]` sequences are escaped by prepending `\` before the
/// opening bracket.  Existing backslashes before a tag are doubled.
///
/// ```
/// # use gilt::markup::escape;
/// assert_eq!(escape("foo[bar]"), r"foo\[bar]");
/// ```
pub fn escape(markup: &str) -> String {
    let result = RE_MARKUP.replace_all(markup, |caps: &regex::Captures| {
        let bs = &caps[1];
        let tag = &caps[2];
        // Double existing backslashes, then prepend one more before the tag.
        format!("{}{}\\{}", bs, bs, tag)
    });
    // If the result ends with a single backslash (not \\), append another.
    let s = result.into_owned();
    if s.ends_with('\\') && !s.ends_with("\\\\") {
        format!("{}\\", s)
    } else {
        s
    }
}

// ---------------------------------------------------------------------------
// parse_markup
// ---------------------------------------------------------------------------

/// An element produced by `parse_markup`.
///
/// Each tuple is `(position, optional_plain_text, optional_tag)`.
/// Exactly one of `plain_text` / `tag` is `Some`.
pub type MarkupElement = (usize, Option<String>, Option<Tag>);

/// Parse `markup` into a sequence of plain-text / tag elements.
///
/// Backslash-escaped tags are emitted as literal text.
pub fn parse_markup(markup: &str) -> Vec<MarkupElement> {
    let mut elements: Vec<MarkupElement> = Vec::new();
    let mut position: usize = 0;

    for caps in RE_MARKUP.captures_iter(markup) {
        let full_match = caps.get(0).unwrap();
        let match_start = full_match.start();

        // Emit any plain text between the previous match and this one.
        if match_start > position {
            let text = &markup[position..match_start];
            elements.push((position, Some(text.to_string()), None));
        }

        let backslashes = &caps[1];
        let tag_text = &caps[2]; // includes brackets

        let bs_count = backslashes.len();

        if bs_count > 0 {
            // Even number of backslashes → half of them are literal, tag is real.
            // Odd number → half are literal, tag is escaped (literal text).
            let literal_bs: String = "\\".repeat(bs_count / 2);

            if bs_count % 2 == 0 {
                // Emit literal backslashes.
                if !literal_bs.is_empty() {
                    elements.push((match_start, Some(literal_bs), None));
                }
                // Process the tag normally.
                let inner = &tag_text[1..tag_text.len() - 1]; // strip [ ]
                let tag = parse_tag_inner(inner);
                elements.push((match_start + bs_count, None, Some(tag)));
            } else {
                // Tag is escaped — emit as literal text.
                let escaped = format!("{}{}", literal_bs, tag_text);
                elements.push((match_start, Some(escaped), None));
            }
        } else {
            // No backslashes, normal tag.
            let inner = &tag_text[1..tag_text.len() - 1];
            let tag = parse_tag_inner(inner);
            elements.push((match_start, None, Some(tag)));
        }

        position = full_match.end();
    }

    // Remaining text after the last match.
    if position < markup.len() {
        let text = &markup[position..];
        elements.push((position, Some(text.to_string()), None));
    }

    elements
}

/// Split a tag's inner text (between `[` and `]`) into name and optional
/// parameters.  E.g. `"link=url"` → Tag { name: "link", parameters: Some("url") }.
fn parse_tag_inner(inner: &str) -> Tag {
    if let Some(eq_pos) = inner.find('=') {
        Tag {
            name: inner[..eq_pos].to_string(),
            parameters: Some(inner[eq_pos + 1..].to_string()),
        }
    } else {
        Tag {
            name: inner.to_string(),
            parameters: None,
        }
    }
}

// ---------------------------------------------------------------------------
// render
// ---------------------------------------------------------------------------

/// Render gilt markup into a styled `Text` object.
///
/// # Errors
///
/// Returns `MarkupError` if a closing tag does not match any open tag.
pub fn render(markup: &str, style: Style) -> Result<Text, MarkupError> {
    // Fast path: no markup at all — only emoji replacement needed.
    if !markup.contains('[') {
        let replaced = emoji_replace(markup, None);
        return Ok(Text::new(replaced.as_ref(), style));
    }

    let mut text = Text::new("", style);
    let mut style_stack: Vec<(usize, Tag)> = Vec::new();

    let elements = parse_markup(markup);

    for (position, plain_text, tag) in &elements {
        if let Some(plain) = plain_text {
            // parse_markup has already fully handled escape sequences; plain text
            // segments are ready to use as-is (no redundant re-escape needed).
            // Apply emoji shortcode replacement (:name: → Unicode).
            let with_emoji = emoji_replace(plain, None);
            text.append_str(with_emoji.as_ref(), None);
        } else if let Some(tag) = tag {
            if tag.name.starts_with('/') {
                // Closing tag.
                let style_name = &tag.name[1..]; // strip leading '/'

                if style_name.is_empty() {
                    // Implicit close `[/]` — pop the most recent tag.
                    if let Some((start, open_tag)) = style_stack.pop() {
                        if open_tag.name.starts_with('@') {
                            // Meta tag — build a meta span over the enclosed range.
                            let end = text.len();
                            if end > start {
                                let meta_arc = parse_meta_tag(&open_tag);
                                text.spans_mut().push(Span::with_meta(
                                    start,
                                    end,
                                    Style::null(),
                                    Some(meta_arc),
                                ));
                            }
                        } else {
                            let end = text.len();
                            if end > start {
                                match try_resolve_literal_style(&open_tag) {
                                    Some(style) => {
                                        text.spans_mut().push(Span::new(start, end, style));
                                    }
                                    None => {
                                        text.spans_mut().push(Span::named(
                                            start,
                                            end,
                                            &open_tag.name,
                                        ));
                                    }
                                }
                            }
                        }
                    } else {
                        return Err(MarkupError::NothingToClose {
                            position: *position,
                        });
                    }
                } else {
                    // Explicit close `[/bold]` — find matching open tag.
                    let normalized = style_name.to_lowercase();
                    let normalized = normalized.trim();

                    // Opening tag names are already normalized (lowercased + trimmed)
                    // when pushed onto the stack, so compare directly without
                    // allocating a new String per iteration.
                    let found = style_stack
                        .iter()
                        .rposition(|(_, t)| t.name.as_str() == normalized);

                    if let Some(idx) = found {
                        let (start, open_tag) = style_stack.remove(idx);
                        if open_tag.name.starts_with('@') {
                            let end = text.len();
                            if end > start {
                                let meta_arc = parse_meta_tag(&open_tag);
                                text.spans_mut().push(Span::with_meta(
                                    start,
                                    end,
                                    Style::null(),
                                    Some(meta_arc),
                                ));
                            }
                        } else {
                            let end = text.len();
                            if end > start {
                                match try_resolve_literal_style(&open_tag) {
                                    Some(style) => {
                                        text.spans_mut().push(Span::new(start, end, style));
                                    }
                                    None => {
                                        text.spans_mut().push(Span::named(
                                            start,
                                            end,
                                            &open_tag.name,
                                        ));
                                    }
                                }
                            }
                        }
                    } else {
                        return Err(MarkupError::MismatchedTag {
                            tag: tag.name.clone(),
                            position: *position,
                        });
                    }
                }
            } else {
                // Opening tag — push onto the stack.
                // Trim before lowercasing so we make one allocation, not two.
                let normalized_name = tag.name.trim().to_lowercase();
                let open_tag = Tag {
                    name: normalized_name,
                    parameters: tag.parameters.clone(),
                };
                let current_len = text.len();
                style_stack.push((current_len, open_tag));
            }
        }
    }

    // Close any remaining unclosed tags (unclosed tags are valid in gilt).
    for (start, open_tag) in style_stack.into_iter().rev() {
        let end = text.len();
        if end > start {
            if open_tag.name.starts_with('@') {
                let meta_arc = parse_meta_tag(&open_tag);
                text.spans_mut()
                    .push(Span::with_meta(start, end, Style::null(), Some(meta_arc)));
            } else {
                match try_resolve_literal_style(&open_tag) {
                    Some(style) => {
                        text.spans_mut().push(Span::new(start, end, style));
                    }
                    None => {
                        text.spans_mut()
                            .push(Span::named(start, end, &open_tag.name));
                    }
                }
            }
        }
    }

    // Sort spans by start position for deterministic output.
    text.spans_mut().sort_by_key(|s| s.start);

    Ok(text)
}

/// Parse an `@`-prefixed meta tag into a shared `HashMap<String, String>`.
///
/// The tag name (after stripping the leading `@`) is the key.  If the tag has a
/// `parameters` value it becomes the value (surrounding quotes are stripped,
/// whitespace is trimmed); otherwise the value is `"true"`.
///
/// Examples:
/// - `[@key=value]`  → `{"key": "value"}`
/// - `[@flag]`        → `{"flag": "true"}`
/// - `[@label="hello world"]` → `{"label": "hello world"}`
fn parse_meta_tag(tag: &Tag) -> Arc<HashMap<String, String>> {
    // Strip the leading '@' from the tag name.
    let key = tag.name.trim_start_matches('@').trim().to_string();

    let raw_value = match &tag.parameters {
        Some(v) => {
            // Trim whitespace, then strip surrounding quotes (single or double).
            let v = v.trim();
            if (v.starts_with('"') && v.ends_with('"'))
                || (v.starts_with('\'') && v.ends_with('\''))
            {
                v[1..v.len() - 1].to_string()
            } else {
                v.to_string()
            }
        }
        None => "true".to_string(),
    };

    let mut map = HashMap::new();
    map.insert(key, raw_value);
    Arc::new(map)
}

/// Try to resolve a tag as a literal style.
///
/// Returns `Some(style)` when `Style::parse_strict` succeeds (e.g. `"bold"`,
/// `"red"`, `"link=url"`).  Returns `None` when the tag is a theme/class name
/// that could not be parsed as a literal style (e.g. `"warning"`,
/// `"repr.number"`).  In the `None` case the caller should create a named span
/// via [`Span::named`] so the theme token is preserved for later resolution.
///
/// # LIMITATION: parse-first classification
///
/// Tags are classified as "literal style" vs "theme name" solely by whether
/// `Style::parse_strict` succeeds.  This means a theme token that happens to
/// share a name with a valid style keyword (e.g. a future theme that overrides
/// `red` or `bold`) would be silently treated as the literal style — its name
/// would be dropped and it would **not** resolve against the console theme.
///
/// Today this is not a problem: all registered gilt theme names (`warning`,
/// `repr.number`, `repr.bool_true`, etc.) are dotted or non-style-word names
/// that `parse_strict` rejects, so there is no collision with the literal-style
/// namespace.
///
/// Full rich parity — resolving every tag against the active theme first, then
/// falling back to literal-style parsing — is a deferred enhancement tracked
/// under the Task 2.3+ theme-resolution work.
fn try_resolve_literal_style(tag: &Tag) -> Option<Style> {
    let tag_str = tag.to_string();
    Style::parse_strict(&tag_str).ok()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- escape tests -------------------------------------------------------

    #[test]
    fn test_escape_basic_tag() {
        assert_eq!(escape("foo[bar]"), r"foo\[bar]");
    }

    #[test]
    fn test_escape_already_escaped() {
        assert_eq!(escape(r"foo\[bar]"), r"foo\\\[bar]");
    }

    #[test]
    fn test_escape_not_a_tag() {
        // Starts with a digit — not a valid tag, so not escaped.
        assert_eq!(escape("[5]"), "[5]");
    }

    #[test]
    fn test_escape_at_tag() {
        assert_eq!(escape("[@foo]"), r"\[@foo]");
    }

    #[test]
    fn test_escape_backslash_end() {
        assert_eq!(escape(r"C:\"), r"C:\\");
    }

    // -- Tag tests ----------------------------------------------------------

    #[test]
    fn test_tag_display_no_params() {
        let tag = Tag {
            name: "bold".to_string(),
            parameters: None,
        };
        assert_eq!(tag.to_string(), "bold");
    }

    #[test]
    fn test_tag_display_with_params() {
        let tag = Tag {
            name: "link".to_string(),
            parameters: Some("https://example.com".to_string()),
        };
        assert_eq!(tag.to_string(), "link https://example.com");
    }

    #[test]
    fn test_tag_markup_no_params() {
        let tag = Tag {
            name: "bold".to_string(),
            parameters: None,
        };
        assert_eq!(tag.markup(), "[bold]");
    }

    #[test]
    fn test_tag_markup_with_params() {
        let tag = Tag {
            name: "link".to_string(),
            parameters: Some("url".to_string()),
        };
        assert_eq!(tag.markup(), "[link=url]");
    }

    // -- parse_markup tests -------------------------------------------------

    #[test]
    fn test_parse_basic() {
        let elements = parse_markup("[foo]hello[/foo]");
        assert_eq!(elements.len(), 3);

        // First: tag "foo"
        assert_eq!(elements[0].1, None);
        assert_eq!(
            elements[0].2,
            Some(Tag {
                name: "foo".to_string(),
                parameters: None,
            })
        );

        // Second: plain text "hello"
        assert_eq!(elements[1].1, Some("hello".to_string()));
        assert_eq!(elements[1].2, None);

        // Third: tag "/foo"
        assert_eq!(elements[2].1, None);
        assert_eq!(
            elements[2].2,
            Some(Tag {
                name: "/foo".to_string(),
                parameters: None,
            })
        );
    }

    #[test]
    fn test_parse_with_params() {
        let elements = parse_markup("[link=https://example.com]click[/link]");
        let tag = elements[0].2.as_ref().unwrap();
        assert_eq!(tag.name, "link");
        assert_eq!(tag.parameters, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_parse_plain_only() {
        let elements = parse_markup("hello world");
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].1, Some("hello world".to_string()));
    }

    // -- render tests -------------------------------------------------------

    #[test]
    fn test_render_basic() {
        let result = render("[bold]FOO[/bold]", Style::null()).unwrap();
        assert_eq!(result.plain(), "FOO");
        assert_eq!(result.spans().len(), 1);
        assert_eq!(result.spans()[0].start, 0);
        assert_eq!(result.spans()[0].end, 3);
        assert_eq!(result.spans()[0].style, Style::parse("bold"));
    }

    #[test]
    fn test_render_not_tags() {
        // Numbers in brackets are not tags (regex requires [a-z#/@] start).
        let result = render("[[1], [1,2,3,4]]", Style::null()).unwrap();
        assert_eq!(result.plain(), "[[1], [1,2,3,4]]");
    }

    #[test]
    fn test_render_combine() {
        let result = render("[green]X[blue]Y[/blue]Z[/green]", Style::null()).unwrap();
        assert_eq!(result.plain(), "XYZ");
        assert_eq!(result.spans().len(), 2);
        // Spans sorted by start: green(0,3), blue(1,2)
        assert_eq!(result.spans()[0].start, 0);
        assert_eq!(result.spans()[0].end, 3);
        assert_eq!(result.spans()[0].style, Style::parse("green"));
        assert_eq!(result.spans()[1].start, 1);
        assert_eq!(result.spans()[1].end, 2);
        assert_eq!(result.spans()[1].style, Style::parse("blue"));
    }

    #[test]
    fn test_render_overlap() {
        let result = render("[green]X[bold]Y[/green]Z[/bold]", Style::null()).unwrap();
        assert_eq!(result.plain(), "XYZ");
        assert_eq!(result.spans().len(), 2);
        // Sorted by start: green(0,2), bold(1,3)
        assert_eq!(result.spans()[0].start, 0);
        assert_eq!(result.spans()[0].end, 2);
        assert_eq!(result.spans()[0].style, Style::parse("green"));
        assert_eq!(result.spans()[1].start, 1);
        assert_eq!(result.spans()[1].end, 3);
        assert_eq!(result.spans()[1].style, Style::parse("bold"));
    }

    #[test]
    fn test_render_implicit_close() {
        let result = render("[bold]X[/]Y", Style::null()).unwrap();
        assert_eq!(result.plain(), "XY");
        assert_eq!(result.spans().len(), 1);
        assert_eq!(result.spans()[0].start, 0);
        assert_eq!(result.spans()[0].end, 1);
        assert_eq!(result.spans()[0].style, Style::parse("bold"));
    }

    #[test]
    fn test_render_close_ambiguous() {
        let result = render("[green]X[bold]Y[/]Z[/]", Style::null()).unwrap();
        assert_eq!(result.plain(), "XYZ");
        assert_eq!(result.spans().len(), 2);
        // Sorted by start: green(0,3), bold(1,2)
        assert_eq!(result.spans()[0].start, 0);
        assert_eq!(result.spans()[0].end, 3);
        assert_eq!(result.spans()[0].style, Style::parse("green"));
        assert_eq!(result.spans()[1].start, 1);
        assert_eq!(result.spans()[1].end, 2);
        assert_eq!(result.spans()[1].style, Style::parse("bold"));
    }

    #[test]
    fn test_markup_error_nothing_to_close() {
        let result = render("foo[/]", Style::null());
        assert!(result.is_err());
    }

    #[test]
    fn test_markup_error_mismatched_explicit() {
        let result = render("foo[/bar]", Style::null());
        assert!(result.is_err());
    }

    #[test]
    fn test_markup_error_mismatched_tags() {
        let result = render("[foo]hello[/bar]", Style::null());
        assert!(result.is_err());
    }

    #[test]
    fn test_escape_escape_double_backslash() {
        let result = render(r"\\[bold]FOO", Style::null()).unwrap();
        assert_eq!(result.plain(), r"\FOO");
        // The bold tag should still apply to FOO.
        assert_eq!(result.spans().len(), 1);
        assert_eq!(result.spans()[0].start, 1);
        assert_eq!(result.spans()[0].end, 4);
    }

    #[test]
    fn test_escape_escape_single_backslash() {
        let result = render(r"\[bold]FOO", Style::null()).unwrap();
        assert_eq!(result.plain(), "[bold]FOO");
        // No spans — the tag is escaped.
        assert_eq!(result.spans().len(), 0);
    }

    #[test]
    fn test_render_link() {
        let result = render("[link=foo]FOO[/link]", Style::null()).unwrap();
        assert_eq!(result.plain(), "FOO");
        assert_eq!(result.spans().len(), 1);
        assert_eq!(result.spans()[0].style, Style::parse("link foo"));
    }

    #[test]
    fn test_render_no_markup() {
        // Fast path: no brackets at all.
        let result = render("hello world", Style::null()).unwrap();
        assert_eq!(result.plain(), "hello world");
        assert_eq!(result.spans().len(), 0);
    }

    #[test]
    fn test_render_unclosed_tags() {
        // Unclosed tags are valid — they apply to the rest of the text.
        let result = render("[bold]hello", Style::null()).unwrap();
        assert_eq!(result.plain(), "hello");
        assert_eq!(result.spans().len(), 1);
        assert_eq!(result.spans()[0].start, 0);
        assert_eq!(result.spans()[0].end, 5);
        assert_eq!(result.spans()[0].style, Style::parse("bold"));
    }

    #[test]
    fn test_render_empty_markup() {
        let result = render("", Style::null()).unwrap();
        assert_eq!(result.plain(), "");
        assert_eq!(result.spans().len(), 0);
    }

    #[test]
    fn test_render_with_base_style() {
        let base = Style::parse("italic");
        let result = render("[bold]hello[/bold]", base.clone()).unwrap();
        assert_eq!(result.plain(), "hello");
        // The bold span should be present.
        assert_eq!(result.spans().len(), 1);
        assert_eq!(result.spans()[0].style, Style::parse("bold"));
    }

    #[test]
    fn test_render_at_event_tag() {
        // `@`-prefixed tags are now meta tags: they produce a Span with metadata.
        // Updated expectation: one meta span covering "hello", no style.
        let result = render("[@click]hello[/]", Style::null()).unwrap();
        assert_eq!(result.plain(), "hello");
        // One meta span is produced with null style and meta key "click" = "true".
        assert_eq!(result.spans().len(), 1);
        let span = &result.spans()[0];
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 5);
        assert!(span.style.is_null());
        let meta = span.meta.as_ref().expect("meta span must have metadata");
        assert_eq!(meta.get("click").map(|v| v.as_str()), Some("true"));
    }

    #[test]
    fn test_render_nested_same_style() {
        let result = render("[bold][bold]X[/bold][/bold]", Style::null()).unwrap();
        assert_eq!(result.plain(), "X");
        assert_eq!(result.spans().len(), 2);
    }

    #[test]
    fn test_render_theme_name_fallback() {
        // Theme names like "repr.number" cannot be parsed as literal styles, so
        // they are now preserved as named spans (style_name = Some("repr.number"))
        // instead of being silently collapsed to null-style anonymous spans.
        // The theme will be resolved against the active Console theme at render
        // time (Task 2.3); this test verifies the name is preserved (not dropped).
        let result = render("[repr.number]42[/repr.number]", Style::null()).unwrap();
        assert_eq!(result.plain(), "42");
        assert_eq!(result.spans().len(), 1);
        let span = &result.spans()[0];
        assert!(
            span.style.is_null(),
            "theme-name span must have null resolved style"
        );
        assert_eq!(span.style_name(), Some("repr.number"));
    }

    // -- Task 2.2 tests: named spans for theme tags ----------------------------

    #[test]
    fn theme_tag_span_carries_name() {
        let result = render("[warning]hello[/warning]", Style::null()).unwrap();
        assert_eq!(result.plain(), "hello");
        let span = &result.spans()[0];
        assert!(span.style.is_null());
        assert_eq!(span.style_name(), Some("warning"));
    }

    #[test]
    fn literal_style_tag_has_no_style_name() {
        let result = render("[bold]hello[/bold]", Style::null()).unwrap();
        assert_eq!(result.spans()[0].style, Style::parse("bold"));
        assert_eq!(result.spans()[0].style_name(), None);
    }

    #[test]
    fn repr_number_tag_carries_name() {
        let result = render("[repr.number]42[/repr.number]", Style::null()).unwrap();
        assert_eq!(result.spans()[0].style_name(), Some("repr.number"));
    }

    #[test]
    fn test_parse_markup_escaped_tag() {
        let elements = parse_markup(r"\[bold]");
        // Should be emitted as literal text "[bold]"
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].1, Some("[bold]".to_string()));
        assert_eq!(elements[0].2, None);
    }

    #[test]
    fn test_render_link_url() {
        let result = render("[link=https://example.com]click here[/link]", Style::null()).unwrap();
        assert_eq!(result.plain(), "click here");
        assert_eq!(result.spans().len(), 1);
        let span_style = &result.spans()[0].style;
        assert_eq!(span_style.link(), Some("https://example.com"));
    }

    #[test]
    fn test_render_link_with_style() {
        let result = render(
            "[bold][link=https://example.com]click[/link][/bold]",
            Style::null(),
        )
        .unwrap();
        assert_eq!(result.plain(), "click");
        assert_eq!(result.spans().len(), 2);
        // Both spans cover the same text range
        let has_link = result
            .spans()
            .iter()
            .any(|s| s.style.link() == Some("https://example.com"));
        let has_bold = result.spans().iter().any(|s| s.style.bold() == Some(true));
        assert!(has_link);
        assert!(has_bold);
    }

    #[test]
    fn test_render_link_implicit_close() {
        let result = render("[link=https://example.com]click[/]", Style::null()).unwrap();
        assert_eq!(result.plain(), "click");
        assert_eq!(result.spans().len(), 1);
        assert_eq!(result.spans()[0].style.link(), Some("https://example.com"));
    }

    // -- Meta tag tests -------------------------------------------------------

    #[test]
    fn test_render_meta_key_value() {
        // `[@key=val]x[/]` — meta span covers "x", meta = {"key": "val"}.
        let result = render("[@key=val]x[/]", Style::null()).unwrap();
        assert_eq!(result.plain(), "x");
        assert_eq!(result.spans().len(), 1);
        let span = &result.spans()[0];
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 1);
        assert!(span.style.is_null(), "meta span must have null style");
        let meta = span.meta.as_ref().expect("must have meta");
        assert_eq!(meta.get("key").map(|v| v.as_str()), Some("val"));
    }

    #[test]
    fn test_render_meta_bare_flag() {
        // `[@flag]text[/]` — bare flag → value = "true".
        let result = render("[@flag]text[/]", Style::null()).unwrap();
        assert_eq!(result.plain(), "text");
        assert_eq!(result.spans().len(), 1);
        let meta = result.spans()[0].meta.as_ref().expect("must have meta");
        assert_eq!(meta.get("flag").map(|v| v.as_str()), Some("true"));
    }

    #[test]
    fn test_render_meta_quoted_value() {
        // `[@label="hello world"]x[/]` — quoted value is unquoted.
        let result = render(r#"[@label="hello world"]x[/]"#, Style::null()).unwrap();
        assert_eq!(result.plain(), "x");
        let meta = result.spans()[0].meta.as_ref().expect("must have meta");
        assert_eq!(meta.get("label").map(|v| v.as_str()), Some("hello world"));
    }

    #[test]
    fn test_render_meta_explicit_close() {
        // `[@key=val]x[/@key]` — explicit close.
        let result = render("[@mykey=42]text[/@mykey]", Style::null()).unwrap();
        assert_eq!(result.plain(), "text");
        assert_eq!(result.spans().len(), 1);
        let meta = result.spans()[0].meta.as_ref().expect("must have meta");
        assert_eq!(meta.get("mykey").map(|v| v.as_str()), Some("42"));
    }

    #[test]
    fn test_render_meta_visible_text_only() {
        // The plain text visible via `text.plain()` must contain only the content,
        // not the tag syntax.
        let result = render("before[@k=v]inside[/]after", Style::null()).unwrap();
        assert_eq!(result.plain(), "beforeinsideafter");
        // One meta span covering "inside" (chars 6..12).
        let meta_spans: Vec<_> = result.spans().iter().filter(|s| s.meta.is_some()).collect();
        assert_eq!(meta_spans.len(), 1);
        assert_eq!(meta_spans[0].start, 6);
        assert_eq!(meta_spans[0].end, 12);
    }

    // -- Finding #1: escaped-bracket no double-escape corruption --------------

    /// `render("foo \\[bar]")` must yield plain `foo [bar]`, not corrupt it.
    /// Previously the redundant `replace("\\[", "[")` on already-processed plain
    /// text was a no-op for fully-matched sequences but would corrupt literal
    /// backslash-bracket text that did NOT form a complete `[tag]` sequence.
    #[test]
    fn test_render_escaped_bracket_no_corruption() {
        // r"\[bar]" is the 7-char string `\[bar]`.
        // parse_markup sees 1 backslash → escaped tag → emits "[bar]" as plain text.
        // render should produce the literal text "[bar]" without further mutation.
        let result = render(r"\[bar]", Style::null()).unwrap();
        assert_eq!(result.plain(), "[bar]");
        assert_eq!(
            result.spans().len(),
            0,
            "escaped tag must not produce a span"
        );
    }

    /// When the input contains a backslash followed by a tag that IS processed,
    /// the plain text before it must not be corrupted either.
    #[test]
    fn test_render_mixed_escaped_and_real_tag() {
        // "foo \[bar] [bold]baz[/bold]" — first tag is escaped, second is real.
        let result = render(r"foo \[bar] [bold]baz[/bold]", Style::null()).unwrap();
        assert_eq!(result.plain(), "foo [bar] baz");
        assert_eq!(result.spans().len(), 1);
        assert_eq!(result.spans()[0].style, Style::parse("bold"));
        // The bold span covers "baz", which is at byte offset 10..13.
        assert_eq!(result.spans()[0].start, 10);
        assert_eq!(result.spans()[0].end, 13);
    }

    // -- Finding #2: emoji replacement in both render paths -------------------

    /// Fast path (no `[` in input): emoji shortcodes must be expanded.
    #[test]
    fn test_render_emoji_fast_path() {
        let result = render("Hello :heart:!", Style::null()).unwrap();
        // U+2764 is the heart emoji
        assert!(
            result.plain().contains('\u{2764}'),
            "expected heart emoji in fast path, got {:?}",
            result.plain()
        );
        assert!(!result.plain().contains(":heart:"));
    }

    /// Full parse path: emoji shortcodes in plain text segments must be expanded.
    #[test]
    fn test_render_emoji_full_path() {
        let result = render("[bold]:smile:[/bold]", Style::null()).unwrap();
        // :smile: should expand to its Unicode character; it must not remain as-is.
        assert!(
            !result.plain().contains(":smile:"),
            "emoji shortcode must be expanded in full parse path, got {:?}",
            result.plain()
        );
        // A span for bold should still exist.
        assert_eq!(result.spans().len(), 1);
        assert_eq!(result.spans()[0].style, Style::parse("bold"));
    }

    /// Emoji inside mixed markup and plain text.
    #[test]
    fn test_render_emoji_mixed_with_markup() {
        let result = render(":heart: [bold]world[/bold]", Style::null()).unwrap();
        assert!(result.plain().contains('\u{2764}'));
        assert!(result.plain().contains("world"));
        assert!(!result.plain().contains(":heart:"));
    }
}
