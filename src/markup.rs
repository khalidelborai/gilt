//! Rich markup parser — parses `[bold red]text[/]` syntax into styled `Text`.
//!
//! Port of Python's rich/markup.py.

use std::fmt;

use regex::Regex;
use std::sync::LazyLock;

use crate::errors::MarkupError;
use crate::style::Style;
use crate::text::{Span, Text};

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

/// Regex for `escape()`: find potential tag sequences, capturing preceding
/// backslashes.  Group 1 = backslashes, Group 2 = the tag (with brackets).
static RE_ESCAPE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\\*)(\[[a-z#/@][^\[]*?\])").unwrap());

/// Regex for `parse_markup()`: captures the whole match, backslashes, and the
/// inner tag text (without brackets).
/// Group 1 = full match (backslashes + bracketed tag)
/// Group 2 = backslashes before the `[`
/// Group 3 = tag content inside brackets
static RE_MARKUP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\\*)(\[[a-z#/@][^\[]*?\])").unwrap());

// ---------------------------------------------------------------------------
// escape
// ---------------------------------------------------------------------------

/// Escape `markup` so that it will not be interpreted as Rich markup.
///
/// Potential `[tag]` sequences are escaped by prepending `\` before the
/// opening bracket.  Existing backslashes before a tag are doubled.
///
/// ```
/// # use gilt::markup::escape;
/// assert_eq!(escape("foo[bar]"), r"foo\[bar]");
/// ```
pub fn escape(markup: &str) -> String {
    let result = RE_ESCAPE.replace_all(markup, |caps: &regex::Captures| {
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

/// Render Rich markup into a styled `Text` object.
///
/// # Errors
///
/// Returns `MarkupError` if a closing tag does not match any open tag.
pub fn render(markup: &str, style: Style) -> Result<Text, MarkupError> {
    // Fast path: no markup at all.
    if !markup.contains('[') {
        return Ok(Text::new(markup, style));
    }

    let mut text = Text::new("", style);
    let mut style_stack: Vec<(usize, Tag)> = Vec::new();

    let elements = parse_markup(markup);

    for (position, plain_text, tag) in &elements {
        if let Some(plain) = plain_text {
            // Replace escaped opening brackets with literal `[`.
            let unescaped = plain.replace("\\[", "[");
            text.append_str(&unescaped, None);
        } else if let Some(tag) = tag {
            if tag.name.starts_with('/') {
                // Closing tag.
                let style_name = &tag.name[1..]; // strip leading '/'

                if style_name.is_empty() {
                    // Implicit close `[/]` — pop the most recent tag.
                    if let Some((start, open_tag)) = style_stack.pop() {
                        // Skip `@` event tags (no style to apply).
                        if !open_tag.name.starts_with('@') {
                            let tag_style = resolve_tag_style(&open_tag);
                            let end = text.len();
                            if end > start {
                                text.spans_mut().push(Span::new(start, end, tag_style));
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

                    let found = style_stack
                        .iter()
                        .rposition(|(_, t)| t.name.to_lowercase().trim() == normalized);

                    if let Some(idx) = found {
                        let (start, open_tag) = style_stack.remove(idx);
                        if !open_tag.name.starts_with('@') {
                            let tag_style = resolve_tag_style(&open_tag);
                            let end = text.len();
                            if end > start {
                                text.spans_mut().push(Span::new(start, end, tag_style));
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
                let normalized_name = tag.name.to_lowercase();
                let normalized_name = normalized_name.trim().to_string();
                let open_tag = Tag {
                    name: normalized_name,
                    parameters: tag.parameters.clone(),
                };
                let current_len = text.len();
                style_stack.push((current_len, open_tag));
            }
        }
    }

    // Close any remaining unclosed tags (unclosed tags are valid in Rich).
    for (start, open_tag) in style_stack.into_iter().rev() {
        if !open_tag.name.starts_with('@') {
            let tag_style = resolve_tag_style(&open_tag);
            let end = text.len();
            if end > start {
                text.spans_mut().push(Span::new(start, end, tag_style));
            }
        }
    }

    // Sort spans by start position for deterministic output.
    text.spans_mut().sort_by_key(|s| s.start);

    Ok(text)
}

/// Resolve a tag to a `Style`.
///
/// Uses `Style::parse` on the tag's string representation.  If parsing fails
/// (e.g. it's a theme name like "warning"), falls back to `Style::null()`.
/// Theme resolution will be added when Console is implemented.
fn resolve_tag_style(tag: &Tag) -> Style {
    let tag_str = tag.to_string();
    Style::parse(&tag_str).unwrap_or_else(|_| {
        // Tag is probably a theme/class name (e.g. "warning", "repr.number").
        // Console will resolve these via its Theme; for now use null style.
        Style::null()
    })
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
        assert_eq!(result.spans()[0].style, Style::parse("bold").unwrap());
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
        assert_eq!(result.spans()[0].style, Style::parse("green").unwrap());
        assert_eq!(result.spans()[1].start, 1);
        assert_eq!(result.spans()[1].end, 2);
        assert_eq!(result.spans()[1].style, Style::parse("blue").unwrap());
    }

    #[test]
    fn test_render_overlap() {
        let result = render("[green]X[bold]Y[/green]Z[/bold]", Style::null()).unwrap();
        assert_eq!(result.plain(), "XYZ");
        assert_eq!(result.spans().len(), 2);
        // Sorted by start: green(0,2), bold(1,3)
        assert_eq!(result.spans()[0].start, 0);
        assert_eq!(result.spans()[0].end, 2);
        assert_eq!(result.spans()[0].style, Style::parse("green").unwrap());
        assert_eq!(result.spans()[1].start, 1);
        assert_eq!(result.spans()[1].end, 3);
        assert_eq!(result.spans()[1].style, Style::parse("bold").unwrap());
    }

    #[test]
    fn test_render_implicit_close() {
        let result = render("[bold]X[/]Y", Style::null()).unwrap();
        assert_eq!(result.plain(), "XY");
        assert_eq!(result.spans().len(), 1);
        assert_eq!(result.spans()[0].start, 0);
        assert_eq!(result.spans()[0].end, 1);
        assert_eq!(result.spans()[0].style, Style::parse("bold").unwrap());
    }

    #[test]
    fn test_render_close_ambiguous() {
        let result = render("[green]X[bold]Y[/]Z[/]", Style::null()).unwrap();
        assert_eq!(result.plain(), "XYZ");
        assert_eq!(result.spans().len(), 2);
        // Sorted by start: green(0,3), bold(1,2)
        assert_eq!(result.spans()[0].start, 0);
        assert_eq!(result.spans()[0].end, 3);
        assert_eq!(result.spans()[0].style, Style::parse("green").unwrap());
        assert_eq!(result.spans()[1].start, 1);
        assert_eq!(result.spans()[1].end, 2);
        assert_eq!(result.spans()[1].style, Style::parse("bold").unwrap());
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
        assert_eq!(result.spans()[0].style, Style::parse("link foo").unwrap());
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
        assert_eq!(result.spans()[0].style, Style::parse("bold").unwrap());
    }

    #[test]
    fn test_render_empty_markup() {
        let result = render("", Style::null()).unwrap();
        assert_eq!(result.plain(), "");
        assert_eq!(result.spans().len(), 0);
    }

    #[test]
    fn test_render_with_base_style() {
        let base = Style::parse("italic").unwrap();
        let result = render("[bold]hello[/bold]", base.clone()).unwrap();
        assert_eq!(result.plain(), "hello");
        // The bold span should be present.
        assert_eq!(result.spans().len(), 1);
        assert_eq!(result.spans()[0].style, Style::parse("bold").unwrap());
    }

    #[test]
    fn test_render_at_event_tag() {
        // Event tags starting with @ should be skipped (no style applied).
        let result = render("[@click]hello[/]", Style::null()).unwrap();
        assert_eq!(result.plain(), "hello");
        // @click is an event tag, so no span is created.
        assert_eq!(result.spans().len(), 0);
    }

    #[test]
    fn test_render_nested_same_style() {
        let result = render("[bold][bold]X[/bold][/bold]", Style::null()).unwrap();
        assert_eq!(result.plain(), "X");
        assert_eq!(result.spans().len(), 2);
    }

    #[test]
    fn test_render_theme_name_fallback() {
        // Unknown style name (e.g. "warning") falls back to null style.
        let result = render("[repr.number]42[/repr.number]", Style::null()).unwrap();
        assert_eq!(result.plain(), "42");
        // Should have one span with null style (theme not resolved yet).
        // The span will exist but will be null since Style::parse fails for
        // theme names.
        // Actually, null-style spans are still inserted since we don't know
        // if they'll be resolved later by Console.
        assert_eq!(result.spans().len(), 1);
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
}
