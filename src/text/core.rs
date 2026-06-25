//! Text module - the core text manipulation type.
//!
//! This module provides the `Text` type which represents styled terminal text,
//! along with supporting types `Span`, `Lines`, and related enums.

use std::cmp::min;
use std::collections::HashMap;
use std::fmt;
use std::ops::Add;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use regex::Regex;

use crate::default_styles::DEFAULT_STYLES;
use crate::error::MarkupError;
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::Style;
use crate::utils::ansi::AnsiDecoder;
use crate::utils::cells::{cell_len, set_cell_size};
use crate::wrap::divide_line;

use super::{JustifyMethod, Lines, OverflowMethod, Span};
use crate::text::helpers::{char_slice, gcd, strip_control_codes};

/// A building block for [`Text::assemble`], representing one segment of text.
pub enum TextPart {
    /// Plain unstyled text.
    Raw(String),
    /// Text with an explicit style.
    Styled(String, Style),
    /// An existing [`Text`] object to embed.
    Inner(Text),
}

/// Either a string slice or a [`Text`] reference, for use with [`Text::append`].
pub enum TextOrStr<'a> {
    /// A borrowed string with an optional style.
    Str(&'a str, Option<Style>),
    /// A borrowed [`Text`] object.
    Text(&'a Text),
}

/// Text with styles, spans, and formatting metadata.
///
/// `Text` is the central type for styled terminal output. It stores a plain-text
/// string alongside a list of [`Span`]s that apply styles to character ranges,
/// and optional formatting hints such as justification, overflow, and tab size.
///
/// # Examples
///
/// ```
/// # fn main() {
/// use gilt::prelude::*;
/// use gilt::text::Span;
///
/// let mut text = Text::new("Hello, World!", Style::null());
/// text.stylize(Style::parse("bold"), 0, Some(5));
/// assert_eq!(text.plain(), "Hello, World!");
/// assert_eq!(text.spans()[0], Span::new(0, 5, Style::parse("bold")));
/// # }
/// ```
#[derive(Debug)]
pub struct Text {
    text: String,
    /// The style spans applied to ranges of text.
    pub spans: Vec<Span>,
    style: Style,
    /// Optional justification method for this text.
    pub justify: Option<JustifyMethod>,
    /// Optional overflow strategy when text exceeds the available width.
    pub overflow: Option<OverflowMethod>,
    /// When `Some(true)`, wrapping is suppressed for this text.
    pub no_wrap: Option<bool>,
    /// String appended after the text when rendering (default `"\n"`).
    pub end: String,
    /// Tab stop width override; `None` uses the default of 8.
    pub tab_size: Option<usize>,
    // Memoized `text.chars().count()`. `usize::MAX` sentinel = uninitialized
    // (impossible real value: 18 EB on 64-bit). `AtomicUsize` (8 B) keeps
    // `Text: Sync` and avoids the size penalty of `OnceLock<usize>`.
    char_len_cache: AtomicUsize,
}

impl Clone for Text {
    fn clone(&self) -> Self {
        Text {
            text: self.text.clone(),
            spans: self.spans.clone(),
            style: self.style.clone(),
            justify: self.justify,
            overflow: self.overflow,
            no_wrap: self.no_wrap,
            end: self.end.clone(),
            tab_size: self.tab_size,
            char_len_cache: AtomicUsize::new(self.char_len_cache.load(Ordering::Relaxed)),
        }
    }
}

impl Text {
    // -- Constructors -------------------------------------------------------

    /// Create a new `Text` with the given plain string and base style.
    ///
    /// Control codes (Bell, Backspace, VT, FF, CR) are stripped automatically.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() {
    /// use gilt::prelude::*;
    ///
    /// let text = Text::new("Hello", Style::parse("bold"));
    /// assert_eq!(text.plain(), "Hello");
    /// # }
    /// ```
    pub fn new(text: &str, style: Style) -> Self {
        Text {
            text: strip_control_codes(text).into_owned(),
            spans: Vec::new(),
            style,
            justify: None,
            overflow: None,
            no_wrap: None,
            end: "\n".to_string(),
            tab_size: None,
            char_len_cache: AtomicUsize::new(usize::MAX),
        }
    }

    /// Create an empty `Text` with a null style.
    pub fn empty() -> Self {
        Text::new("", Style::null())
    }

    /// Create `Text` with a style applied as a span. The style is parsed
    /// from a string via [`Style::parse`] (lossy: bad input → null span).
    ///
    /// This is the recommended ergonomic constructor — no `?`, no wrapping
    /// `Style::parse(...)` at the callsite. Content is treated as **literal**
    /// (markup tags inside the content are not parsed; for that, use
    /// [`Text::from_markup`](Self::from_markup)).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::text::Text;
    ///
    /// let warn = Text::styled("watch out", "bold yellow");
    /// let url  = Text::styled("https://example.com", "link blue underline");
    /// ```
    ///
    /// For an existing `Style` value, use [`styled_with`](Self::styled_with).
    pub fn styled(text: impl Into<String>, style: &str) -> Self {
        Self::styled_with(text, Style::parse(style))
    }

    /// Create `Text` with a pre-built [`Style`] applied as a span.
    /// Use this when you already have a `Style` value; for the common
    /// case of a string-form style, prefer [`Text::styled`].
    pub fn styled_with(text: impl Into<String>, style: Style) -> Self {
        let owned = text.into();
        let mut t = Text::new(&owned, Style::null());
        let len = t.len();
        if len > 0 && !style.is_null() {
            t.spans.push(Span::new(0, len, style));
        }
        t
    }

    /// Assemble a `Text` from a slice of [`TextPart`] segments with a shared base style.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() {
    /// use gilt::prelude::*;
    /// use gilt::text::TextPart;
    ///
    /// let text = Text::assemble(
    ///     &[
    ///         TextPart::Raw("Hello ".into()),
    ///         TextPart::Styled("World".into(), Style::parse("bold")),
    ///     ],
    ///     Style::null(),
    /// );
    /// assert_eq!(text.plain(), "Hello World");
    /// # }
    /// ```
    pub fn assemble(parts: &[TextPart], style: Style) -> Self {
        let mut result = Text::new("", style);
        for part in parts {
            match part {
                TextPart::Raw(s) => {
                    result.append_str(s, None);
                }
                TextPart::Styled(s, st) => {
                    result.append_str(s, Some(st.clone()));
                }
                TextPart::Inner(t) => {
                    result.append_text(t);
                }
            }
        }
        result
    }

    /// Create a `Text` from a console markup string like `"[bold red]Hello[/bold red] world"`.
    ///
    /// Delegates to [`crate::markup::render`].
    ///
    /// # Errors
    ///
    /// Returns [`MarkupError`] if the markup contains mismatched closing tags.
    pub fn from_markup(markup: &str) -> Result<Text, MarkupError> {
        crate::markup::render(markup, Style::null())
    }

    /// Create a `Text` from a string containing ANSI escape codes.
    ///
    /// Preserves trailing newlines in the input, matching the behavior of
    /// rich v15.0.0 (`Text.from_ansi("Hello\n").plain == "Hello\n"`).
    pub fn from_ansi(text: &str) -> Text {
        Self::from_ansi_decoded(&mut AnsiDecoder::new(), text)
    }

    /// Private helper: decode `text` via `decoder`, joining all lines into
    /// a single `Text`.  Added near `from_ansi` to avoid touching unrelated
    /// functions; `decode_line` is left unchanged for backward compatibility.
    fn from_ansi_decoded(decoder: &mut AnsiDecoder, text: &str) -> Text {
        let lines = decoder.decode(text);
        let mut result = Text::new("", Style::null());
        for line in lines {
            result.append_text(&line);
        }
        result
    }

    // -- Properties ---------------------------------------------------------

    /// Return the base style of this text.
    pub fn style(&self) -> &Style {
        &self.style
    }

    /// Return the plain (unstyled) text content.
    pub fn plain(&self) -> &str {
        &self.text
    }

    /// Replace the plain text, trimming any spans that exceed the new length.
    pub fn set_plain(&mut self, new_text: &str) {
        let new_text = strip_control_codes(new_text);
        let new_len = new_text.chars().count();
        // Trim spans that exceed new length
        self.spans.retain_mut(|span| {
            if span.start >= new_len {
                return false;
            }
            if span.end > new_len {
                span.end = new_len;
            }
            !span.is_empty()
        });
        self.text = new_text.into_owned();
        self.set_char_len(new_len);
    }

    /// Return the style spans applied to this text.
    pub fn spans(&self) -> &[Span] {
        &self.spans
    }

    /// Return a mutable reference to the style spans.
    pub fn spans_mut(&mut self) -> &mut Vec<Span> {
        &mut self.spans
    }

    /// Return the length of the text in Unicode characters.
    pub fn len(&self) -> usize {
        let cached = self.char_len_cache.load(Ordering::Relaxed);
        if cached != usize::MAX {
            return cached;
        }
        let computed = self.text.chars().count();
        self.char_len_cache.store(computed, Ordering::Relaxed);
        computed
    }

    // Drop the cached `chars().count()` so the next `len()` recomputes.
    // Call from every site that mutates `self.text`.
    fn invalidate_char_len(&mut self) {
        self.char_len_cache.store(usize::MAX, Ordering::Relaxed);
    }

    // Re-prime the cache to a known value after a mutation. Cheaper than
    // invalidate-then-recompute when the new length is already in hand.
    fn set_char_len(&self, value: usize) {
        self.char_len_cache.store(value, Ordering::Relaxed);
    }

    /// Return `true` if the text is empty.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Return the display width of the text in terminal cells.
    ///
    /// Wide characters (e.g. CJK) count as two cells.
    pub fn cell_len(&self) -> usize {
        cell_len(&self.text)
    }

    /// Measure the text, returning minimum (longest word) and maximum (longest line) widths.
    ///
    /// For whitespace-only text (no non-whitespace words), the minimum equals
    /// the maximum (matching Python rich's behaviour where the whole string
    /// must fit on one line).
    ///
    /// This is the Rust equivalent of Python's `Text.__gilt_measure__`.
    pub fn measure(&self) -> Measurement {
        let text = self.plain();
        if text.is_empty() {
            return Measurement::new(0, 0);
        }
        let max_text_width = text.lines().map(cell_len).max().unwrap_or(0);
        let min_text_width = text
            .split_whitespace()
            .map(cell_len)
            .max()
            .unwrap_or(max_text_width);
        Measurement::new(min_text_width, max_text_width)
    }

    // -- Display & comparison -----------------------------------------------

    /// Return `true` if the plain text contains the given substring.
    pub fn contains_str(&self, s: &str) -> bool {
        self.text.contains(s)
    }

    /// Return `true` if the plain text contains the plain text of `t`.
    pub fn contains_text(&self, t: &Text) -> bool {
        self.text.contains(t.plain())
    }

    // -- Core manipulation --------------------------------------------------

    /// Return a deep clone of this text (identical to `clone()`).
    pub fn copy(&self) -> Text {
        self.clone()
    }

    /// Create a copy that shares formatting metadata (style, justify, overflow, etc.)
    /// but has different plain text and no spans.
    pub fn blank_copy(&self, plain: &str) -> Text {
        Text {
            text: strip_control_codes(plain).into_owned(),
            spans: Vec::new(),
            style: self.style.clone(),
            justify: self.justify,
            overflow: self.overflow,
            no_wrap: self.no_wrap,
            end: self.end.clone(),
            tab_size: self.tab_size,
            char_len_cache: AtomicUsize::new(usize::MAX),
        }
    }

    /// Append a string to the text, optionally applying a style to the appended portion.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() {
    /// use gilt::prelude::*;
    ///
    /// let mut text = Text::new("Hello", Style::null());
    /// text.append_str(", World!", Some(Style::parse("italic")));
    /// assert_eq!(text.plain(), "Hello, World!");
    /// # }
    /// ```
    pub fn append_str(&mut self, text: &str, style: Option<Style>) -> &mut Self {
        let text = strip_control_codes(text);
        if text.is_empty() {
            return self;
        }
        let offset = self.len();
        let new_len = text.chars().count();
        self.text.push_str(&text);
        // Invalidate then re-prime: we already know the new length.
        self.invalidate_char_len();
        self.set_char_len(offset + new_len);
        if let Some(s) = style {
            if !s.is_null() {
                self.spans.push(Span::new(offset, offset + new_len, s));
            }
        }
        self
    }

    /// Append another [`Text`] object, preserving its spans with adjusted offsets.
    ///
    /// When the appended text carries a non-null base `style`, that style is
    /// promoted to a span covering the entire appended range, matching Python
    /// rich's behaviour where `Text.style` acts as a base style layer.
    pub fn append_text(&mut self, text: &Text) -> &mut Self {
        let offset = self.len();
        let other_len = text.len();
        self.text.push_str(&text.text);
        self.invalidate_char_len();
        self.set_char_len(offset + other_len);
        // Promote the appended text's base style as a span over the appended range.
        if other_len > 0 && !text.style.is_null() {
            self.spans
                .push(Span::new(offset, offset + other_len, text.style.clone()));
        }
        for span in &text.spans {
            self.spans.push(span.move_span(offset));
        }
        self
    }

    /// Append either a string or a [`Text`] via [`TextOrStr`].
    pub fn append(&mut self, text: TextOrStr) -> &mut Self {
        match text {
            TextOrStr::Str(s, style) => self.append_str(s, style),
            TextOrStr::Text(t) => self.append_text(t),
        }
    }

    /// Append multiple `(text, optional_style)` pairs in order.
    pub fn append_tokens(&mut self, tokens: &[(String, Option<Style>)]) -> &mut Self {
        for (token_text, style) in tokens {
            self.append_str(token_text, style.clone());
        }
        self
    }

    /// Apply a style to the character range `[start, end)`.
    ///
    /// If `end` is `None`, the style extends to the end of the text.
    /// The span is appended after any existing spans.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() {
    /// use gilt::prelude::*;
    ///
    /// let mut text = Text::new("Hello, World!", Style::null());
    /// text.stylize(Style::parse("bold red"), 0, Some(5));
    /// assert_eq!(text.spans().len(), 1);
    /// # }
    /// ```
    pub fn stylize(&mut self, style: Style, start: usize, end: Option<usize>) {
        let length = self.len();
        if length == 0 {
            return;
        }
        let end = end.unwrap_or(length);
        let start = min(start, length);
        let end = min(end, length);
        if start >= end {
            return;
        }
        self.spans.push(Span::new(start, end, style));
    }

    /// Apply a style to the character range `[start, end)`, inserting it before
    /// all existing spans so it has lowest priority.
    pub fn stylize_before(&mut self, style: Style, start: usize, end: Option<usize>) {
        let length = self.len();
        if length == 0 {
            return;
        }
        let end = end.unwrap_or(length);
        let start = min(start, length);
        let end = min(end, length);
        if start >= end {
            return;
        }
        self.spans.insert(0, Span::new(start, end, style));
    }

    /// Copy all spans from another [`Text`] into this one (without adjusting offsets).
    pub fn copy_styles(&mut self, other: &Text) {
        self.spans.extend(other.spans.iter().cloned());
    }

    // -- Splitting and dividing ---------------------------------------------

    /// Split the text on a literal separator string, returning [`Lines`].
    ///
    /// When `include_separator` is `true`, the separator remains attached to the
    /// end of each resulting line.  When `allow_blank` is `false`, empty lines
    /// are removed from the result.
    pub fn split(&self, separator: &str, include_separator: bool, allow_blank: bool) -> Lines {
        let plain = &self.text;
        let sep_byte_len = separator.len();

        if include_separator {
            let mut offsets = Vec::new();
            for (byte_start, _) in plain.match_indices(separator) {
                let byte_end = byte_start + sep_byte_len;
                offsets.push(plain[..byte_end].chars().count());
            }
            let lines = self.divide(&offsets);
            if !allow_blank && plain.ends_with(separator) {
                let mut lines = lines;
                if let Some(last) = lines.lines.last() {
                    if last.is_empty() {
                        lines.pop();
                    }
                }
                return lines;
            }
            lines
        } else {
            let mut offsets = Vec::new();
            for (byte_start, _) in plain.match_indices(separator) {
                let byte_end = byte_start + sep_byte_len;
                offsets.push(plain[..byte_start].chars().count());
                offsets.push(plain[..byte_end].chars().count());
            }

            let divided = self.divide(&offsets);
            let sep_len = separator.chars().count();

            let mut result = Lines::default();
            for line in divided.lines {
                // Skip lines that are exactly the separator
                if line.len() == sep_len && line.plain() == separator {
                    continue;
                }
                if !allow_blank && line.is_empty() {
                    continue;
                }
                result.push(line);
            }

            if !allow_blank {
                // If the original text ends with separator, there might be a trailing empty
                if let Some(last) = result.lines.last() {
                    if last.is_empty() {
                        result.pop();
                    }
                }
            }

            result
        }
    }

    /// Divide the text at the given character offsets, distributing spans across the
    /// resulting lines with locally adjusted positions.
    ///
    /// Each offset produces a split point; the text is divided into `offsets.len() + 1`
    /// lines (after deduplication). Spans that cross a boundary are clipped to each
    /// line's local range.
    pub fn divide(&self, offsets: &[usize]) -> Lines {
        let text_length = self.len();
        if offsets.is_empty() {
            return Lines::new(vec![self.copy()]);
        }

        // Build line ranges: [0, offsets[0]], [offsets[0], offsets[1]], ..., [offsets[n-1], text_length]
        let mut boundaries = Vec::with_capacity(offsets.len() + 2);
        boundaries.push(0usize);
        for &o in offsets {
            let o = min(o, text_length);
            boundaries.push(o);
        }
        boundaries.push(text_length);
        // Deduplicate consecutive equal boundaries
        boundaries.dedup();

        let line_count = boundaries.len() - 1;
        let mut lines: Vec<Text> = Vec::with_capacity(line_count);

        for i in 0..line_count {
            let start = boundaries[i];
            let end = boundaries[i + 1];
            let slice_text = char_slice(&self.text, start, end);
            let line = self.blank_copy(slice_text);
            lines.push(line);
        }

        // Now assign spans to lines using binary search
        // boundaries[i] = start of line i, boundaries[i+1] = end of line i
        for span in &self.spans {
            if span.is_empty() {
                continue;
            }

            // Find first line that this span could overlap with
            // Line i has range [boundaries[i], boundaries[i+1])
            // Span range is [span.start, span.end)
            // We need to find line indices where span overlaps
            // We can binary search for the first line where boundaries[i+1] > span.start

            // Use partition_point on boundaries to find the first boundary > span.start
            // That boundary index - 1 is the line index where the span starts
            let first_boundary_after_start = boundaries.partition_point(|&b| b <= span.start);
            // Line index = first_boundary_after_start - 1
            let start_line = if first_boundary_after_start > 0 {
                first_boundary_after_start - 1
            } else {
                0
            };

            let first_boundary_at_or_after_end = boundaries.partition_point(|&b| b < span.end);
            let end_line = if first_boundary_at_or_after_end > 0 {
                min(first_boundary_at_or_after_end - 1, line_count - 1)
            } else {
                0
            };

            for line_idx in start_line..=end_line {
                if line_idx >= line_count {
                    break;
                }
                let line_start = boundaries[line_idx];
                let line_end = boundaries[line_idx + 1];

                // Compute the overlap
                let overlap_start = span.start.max(line_start);
                let overlap_end = span.end.min(line_end);

                if overlap_start < overlap_end {
                    lines[line_idx].spans.push(Span::new(
                        overlap_start - line_start,
                        overlap_end - line_start,
                        span.style.clone(),
                    ));
                }
            }
        }

        Lines::new(lines)
    }

    // -- Indexing ------------------------------------------------------------

    /// Return a single-character [`Text`] at the given character index, preserving
    /// any overlapping styles. Returns an empty text if `index` is out of bounds.
    pub fn get_char(&self, index: usize) -> Text {
        let length = self.len();
        if index >= length {
            return self.blank_copy("");
        }
        let ch = char_slice(&self.text, index, index + 1);
        let mut result = self.blank_copy(ch);
        for span in &self.spans {
            if span.start <= index && span.end > index {
                result.spans.push(Span::new(0, 1, span.style.clone()));
            }
        }
        result
    }

    /// Extract a sub-range `[start, end)` as a new [`Text`] with locally adjusted spans.
    pub fn slice(&self, start: usize, end: usize) -> Text {
        let length = self.len();
        let start = min(start, length);
        let end = min(end, length);
        if start >= end {
            return self.blank_copy("");
        }
        // Use divide to get the slice
        let divided = self.divide(&[start, end]);
        if divided.len() >= 2 {
            divided.lines[1].clone()
        } else if divided.len() == 1 {
            divided.lines[0].clone()
        } else {
            self.blank_copy("")
        }
    }

    // -- Cropping and padding -----------------------------------------------

    /// Remove `amount` characters from the right side of the text, adjusting spans.
    pub fn right_crop(&mut self, amount: usize) {
        let length = self.len();
        if amount >= length {
            self.text.clear();
            self.spans.clear();
            self.invalidate_char_len();
            self.set_char_len(0);
            return;
        }
        let new_length = length - amount;
        let new_text = char_slice(&self.text, 0, new_length).to_string();
        self.text = new_text;
        self.invalidate_char_len();
        self.set_char_len(new_length);
        self.spans.retain_mut(|span| {
            if span.start >= new_length {
                return false;
            }
            if span.end > new_length {
                span.end = new_length;
            }
            !span.is_empty()
        });
    }

    /// Truncate (or pad) the text to fit within `max_width` terminal cells.
    ///
    /// The `overflow` strategy controls how excess text is handled (see
    /// [`OverflowMethod`]). When `pad` is `true` and the text is shorter than
    /// `max_width`, spaces are appended to fill the remaining width.
    pub fn truncate(&mut self, max_width: usize, overflow: Option<OverflowMethod>, pad: bool) {
        let current_width = self.cell_len();
        let overflow = overflow.unwrap_or(OverflowMethod::Fold);

        if current_width <= max_width {
            if pad && current_width < max_width {
                self.pad_right(max_width - current_width, ' ');
            }
            return;
        }

        match overflow {
            OverflowMethod::Ellipsis => {
                if max_width == 0 {
                    self.set_plain("");
                    return;
                }
                let new_text = set_cell_size(&self.text, max_width.saturating_sub(1)).into_owned();
                // Count chars of new_text for span adjustment
                self.set_plain(&new_text);
                self.append_str("\u{2026}", None); // ellipsis
            }
            OverflowMethod::Crop | OverflowMethod::Fold => {
                let new_text = set_cell_size(&self.text, max_width).into_owned();
                self.set_plain(&new_text);
            }
            OverflowMethod::Ignore => {
                // Do nothing
            }
        }

        if pad {
            let current_width = self.cell_len();
            if current_width < max_width {
                self.pad_right(max_width - current_width, ' ');
            }
        }
    }

    /// Pad both sides of the text with `count` copies of `character`.
    pub fn pad(&mut self, count: usize, character: char) {
        self.pad_left(count, character);
        self.pad_right(count, character);
    }

    /// Prepend `count` copies of `character`, shifting all span offsets right.
    pub fn pad_left(&mut self, count: usize, character: char) {
        if count == 0 {
            return;
        }
        let padding: String = std::iter::repeat_n(character, count).collect();
        let old_len = self.len();
        // Shift all spans right by count
        for span in &mut self.spans {
            span.start += count;
            span.end += count;
        }
        self.text = format!("{}{}", padding, self.text);
        self.invalidate_char_len();
        self.set_char_len(old_len + count);
    }

    /// Append `count` copies of `character` to the right side of the text.
    pub fn pad_right(&mut self, count: usize, character: char) {
        if count == 0 {
            return;
        }
        let padding: String = std::iter::repeat_n(character, count).collect();
        let old_len = self.len();
        self.text.push_str(&padding);
        self.invalidate_char_len();
        self.set_char_len(old_len + count);
    }

    /// Remove trailing whitespace from the text, adjusting spans.
    pub fn rstrip(&mut self) {
        let trimmed = self.text.trim_end().to_string();
        if trimmed.len() != self.text.len() {
            self.set_plain(&trimmed);
        }
    }

    /// Strip trailing whitespace that occurs beyond character position `size`.
    pub fn rstrip_end(&mut self, size: usize) {
        let length = self.len();
        if length <= size {
            return;
        }
        // Only strip trailing whitespace beyond `size` chars
        let text_after_size = char_slice(&self.text, size, length);
        let trimmed_after = text_after_size.trim_end();
        if trimmed_after.len() == text_after_size.len() {
            return; // nothing to strip
        }
        let new_end_len = size + trimmed_after.chars().count();
        let new_text = char_slice(&self.text, 0, new_end_len).to_string();
        self.set_plain(&new_text);
    }

    /// Set the text to exactly `new_length` characters by truncating or padding with spaces.
    pub fn set_length(&mut self, new_length: usize) {
        let current_length = self.len();
        if new_length < current_length {
            let new_text = char_slice(&self.text, 0, new_length).to_string();
            self.set_plain(&new_text);
        } else if new_length > current_length {
            self.pad_right(new_length - current_length, ' ');
        }
    }

    /// Remove `suffix` from the end of the text if present.
    pub fn remove_suffix(&mut self, suffix: &str) {
        if self.text.ends_with(suffix) {
            let suffix_chars = suffix.chars().count();
            let new_len = self.len() - suffix_chars;
            let new_text = char_slice(&self.text, 0, new_len).to_string();
            self.set_plain(&new_text);
        }
    }

    /// Pad or truncate the text to exactly `width` terminal cells using the
    /// given alignment and fill character.
    ///
    /// Content wider than `width` is truncated first (fold/crop), then padding
    /// is applied.  This matches Python rich's `Text.align` behaviour.
    pub fn align(&mut self, align: JustifyMethod, width: usize, character: char) {
        // Truncate first so over-wide content does not escape the target width.
        let text_width = self.cell_len();
        if text_width > width {
            self.truncate(width, Some(OverflowMethod::Fold), false);
        }

        let text_width = self.cell_len();
        if text_width >= width {
            return;
        }
        let excess = width - text_width;
        match align {
            JustifyMethod::Left | JustifyMethod::Default => {
                self.pad_right(excess, character);
            }
            JustifyMethod::Center => {
                let left = excess / 2;
                let right = excess - left;
                self.pad_left(left, character);
                self.pad_right(right, character);
            }
            JustifyMethod::Right => {
                self.pad_left(excess, character);
            }
            JustifyMethod::Full => {
                self.pad_right(excess, character);
            }
        }
    }

    // -- Highlighting -------------------------------------------------------

    /// Apply `style` to every match of the compiled regex `pattern`.
    ///
    /// Returns the number of matches found.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() {
    /// use gilt::prelude::*;
    /// use regex::Regex;
    ///
    /// let mut text = Text::new("error: not found", Style::null());
    /// let re = Regex::new(r"error").unwrap();
    /// let count = text.highlight_regex(&re, Style::parse("bold red"));
    /// assert_eq!(count, 1);
    /// # }
    /// ```
    pub fn highlight_regex(&mut self, pattern: &Regex, style: Style) -> usize {
        // build_byte_to_char_index inlined as a private helper —
        // regex match positions are always at codepoint boundaries so
        // entries at mid-codepoint byte indices are unused.
        fn build_byte_to_char_index(text: &str) -> Vec<usize> {
            let mut map = vec![0usize; text.len() + 1];
            let mut char_idx = 0usize;
            for (byte_idx, _) in text.char_indices() {
                map[byte_idx] = char_idx;
                char_idx += 1;
            }
            map[text.len()] = char_idx;
            map
        }
        // T12: collect (byte_start, byte_end) into a Vec from an immutable
        // borrow of self.text, then apply stylize after the iterator drops.
        // Was previously cloning the entire text and re-walking the prefix
        // chars on every match (O(M·N) char-counting).
        let matches: Vec<(usize, usize)> = pattern
            .find_iter(&self.text)
            .map(|m| (m.start(), m.end()))
            .collect();
        if matches.is_empty() {
            return 0;
        }
        let b2c = build_byte_to_char_index(&self.text);
        let count = matches.len();
        for (bs, be) in matches {
            let char_start = b2c[bs];
            let char_end = b2c[be];
            self.stylize(style.clone(), char_start, Some(char_end));
        }
        count
    }

    /// Apply a per-match computed style to every match of `pattern`.
    ///
    /// `style_fn` is invoked once per match with the matched substring and
    /// must return the [`Style`] to apply to that match. This mirrors
    /// Python rich's `Text.highlight_regex` callable overload.
    ///
    /// Returns the number of matches that produced a span (matches with an
    /// empty matched range are still counted in iteration but stylize() is a
    /// no-op on zero-width ranges, so they contribute no visible span).
    pub fn highlight_regex_callable<F>(
        &mut self,
        pattern: &Regex,
        style_fn: F,
    ) -> usize
    where
        F: Fn(&str) -> Style,
    {
        fn build_byte_to_char_index(text: &str) -> Vec<usize> {
            let mut map = vec![0usize; text.len() + 1];
            let mut char_idx = 0usize;
            for (byte_idx, _) in text.char_indices() {
                map[byte_idx] = char_idx;
                char_idx += 1;
            }
            map[text.len()] = char_idx;
            map
        }
        // Collect (byte_start, byte_end, matched_str) up front; the iterator
        // borrows self.text immutably, so we drop it before mutating self.
        let matches: Vec<(usize, usize, String)> = pattern
            .find_iter(&self.text)
            .map(|m| (m.start(), m.end(), m.as_str().to_string()))
            .collect();
        if matches.is_empty() {
            return 0;
        }
        let b2c = build_byte_to_char_index(&self.text);
        let count = matches.len();
        for (bs, be, matched) in matches {
            let char_start = b2c[bs];
            let char_end = b2c[be];
            let style = style_fn(&matched);
            self.stylize(style, char_start, Some(char_end));
        }
        count
    }

    /// Highlight named capture groups from `pattern`, using `style_prefix` concatenated
    /// with each group name as the style string. Returns the total number of styled groups.
    pub fn highlight_regex_with_groups(&mut self, pattern: &Regex, style_prefix: &str) -> usize {
        fn build_byte_to_char_index(text: &str) -> Vec<usize> {
            let mut map = vec![0usize; text.len() + 1];
            let mut char_idx = 0usize;
            for (byte_idx, _) in text.char_indices() {
                map[byte_idx] = char_idx;
                char_idx += 1;
            }
            map[text.len()] = char_idx;
            map
        }
        // T12: same pattern as highlight_regex — collect (style, byte_start,
        // byte_end) up front, then apply after dropping the iterator borrow.
        let mut pending: Vec<(Style, usize, usize)> = Vec::new();
        for captures in pattern.captures_iter(&self.text) {
            for name in pattern.capture_names().flatten() {
                if let Some(mat) = captures.name(name) {
                    let style_str = format!("{}{}", style_prefix, name);
                    let style = DEFAULT_STYLES
                        .get(&style_str as &str)
                        .cloned()
                        .or_else(|| Style::parse_strict(&style_str).ok());
                    if let Some(style) = style {
                        pending.push((style, mat.start(), mat.end()));
                    }
                }
            }
        }
        if pending.is_empty() {
            return 0;
        }
        let b2c = build_byte_to_char_index(&self.text);
        let count = pending.len();
        for (style, bs, be) in pending {
            self.stylize(style, b2c[bs], Some(b2c[be]));
        }
        count
    }

    /// Apply `style` to every occurrence of each word.
    ///
    /// Matches are plain substring matches (no word-boundary anchors), which
    /// is consistent with Python rich's `highlight_words` behaviour — it also
    /// matches inside larger words.
    ///
    /// Returns the total number of matches across all words.
    pub fn highlight_words(&mut self, words: &[&str], style: Style, case_sensitive: bool) -> usize {
        if words.is_empty() {
            return 0;
        }
        // Build a plain alternation: (?:word1|word2|...)
        let escaped: Vec<String> = words.iter().map(|w| regex::escape(w)).collect();
        let alternation = escaped.join("|");
        let pattern_str = if case_sensitive {
            format!(r"(?:{})", alternation)
        } else {
            format!(r"(?i)(?:{})", alternation)
        };
        if let Ok(re) = Regex::new(&pattern_str) {
            self.highlight_regex(&re, style)
        } else {
            0
        }
    }

    // -- Tab expansion ------------------------------------------------------

    /// Replace tab characters with spaces, advancing to the next tab stop.
    ///
    /// Each `\t` emits `tab_size - (cell_offset % tab_size)` spaces, where
    /// `cell_offset` is the current column position (in terminal cells, so CJK
    /// before a tab counts as 2).  This matches Python rich's behaviour.
    ///
    /// Uses the given `tab_size`, falling back to [`Text::tab_size`], then to 8.
    pub fn expand_tabs(&mut self, tab_size: Option<usize>) {
        let tab_size = tab_size.unwrap_or(self.tab_size.unwrap_or(8));
        if tab_size == 0 || !self.text.contains('\t') {
            return;
        }

        // Build (new_text, char_offset_map) in a single streaming pass.
        // char_offset_map[old_char_idx] = new_char_idx (after expansion).
        let old_len = self.text.chars().count();
        let mut new_text = String::with_capacity(self.text.len() + 64);
        let mut char_offset_map: Vec<usize> = Vec::with_capacity(old_len + 1);
        let mut new_pos = 0usize; // char position in new_text
        let mut cell_offset = 0usize; // current column (in terminal cells)

        for c in self.text.chars() {
            char_offset_map.push(new_pos);
            if c == '\t' {
                // Advance to the next tab stop.
                let spaces = tab_size - (cell_offset % tab_size);
                new_text.extend(std::iter::repeat_n(' ', spaces));
                new_pos += spaces;
                cell_offset += spaces;
            } else {
                new_text.push(c);
                let w = cell_len(c.encode_utf8(&mut [0u8; 4]));
                new_pos += 1;
                cell_offset += w;
                // Reset cell_offset at newlines (a new visual line starts at column 0).
                if c == '\n' {
                    cell_offset = 0;
                }
            }
        }
        char_offset_map.push(new_pos); // end sentinel

        let mut new_spans = Vec::new();
        for span in &self.spans {
            let new_start = char_offset_map.get(span.start).copied().unwrap_or(new_pos);
            let new_end = char_offset_map.get(span.end).copied().unwrap_or(new_pos);
            if new_start < new_end {
                new_spans.push(Span::new(new_start, new_end, span.style.clone()));
            }
        }

        self.text = new_text;
        self.spans = new_spans;
        self.invalidate_char_len();
    }

    /// Append `spaces` whitespace characters and extend any spans that reach
    /// the current end of text to cover the new characters.
    pub fn extend_style(&mut self, spaces: usize) {
        if spaces == 0 {
            return;
        }
        let old_len = self.len();
        // Extend spans that reach the end of text
        for span in &mut self.spans {
            if span.end >= old_len {
                span.end += spaces;
            }
        }
        let padding: String = std::iter::repeat_n(' ', spaces).collect();
        self.text.push_str(&padding);
        self.invalidate_char_len();
        self.set_char_len(old_len + spaces);
    }

    // -- Advanced -----------------------------------------------------------

    /// Join a slice of [`Text`] objects using `self` as the separator.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() {
    /// use gilt::prelude::*;
    ///
    /// let sep = Text::new(", ", Style::null());
    /// let items = vec![Text::new("a", Style::null()), Text::new("b", Style::null())];
    /// assert_eq!(sep.join(&items).plain(), "a, b");
    /// # }
    /// ```
    pub fn join(&self, texts: &[Text]) -> Text {
        if texts.is_empty() {
            return Text::empty();
        }
        let mut result = texts[0].copy();
        for t in &texts[1..] {
            result.append_text(self);
            result.append_text(t);
        }
        result
    }

    /// Split the text on newlines and force each line to exactly `width` cells
    /// by truncating or padding with spaces.
    pub fn fit(&self, width: usize) -> Lines {
        let lines = self.split("\n", true, true);
        let mut result = Lines::default();
        for mut line in lines.lines {
            let new_text = set_cell_size(line.plain(), width).into_owned();
            line.set_plain(&new_text);
            // Pad if needed
            if line.cell_len() < width {
                line.pad_right(width - line.cell_len(), ' ');
            }
            result.push(line);
        }
        result
    }

    /// Detect the indentation step size by computing the GCD of all leading
    /// whitespace widths. Returns 1 if no indentation is found.
    pub fn detect_indentation(&self) -> usize {
        let mut indent_gcd = 0usize;
        for line in self.text.lines() {
            if line.trim().is_empty() {
                continue;
            }
            // Count code points (not bytes) so multibyte whitespace like
            // U+3000 IDEOGRAPHIC SPACE is counted as one indent step.
            let indent = line.chars().count() - line.trim_start().chars().count();
            if indent == 0 {
                continue;
            }
            indent_gcd = gcd(indent_gcd, indent);
        }
        if indent_gcd == 0 {
            // Fallback: return the first non-zero indentation found.
            for line in self.text.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                let indent = line.chars().count() - line.trim_start().chars().count();
                if indent > 0 {
                    return indent;
                }
            }
            return 1;
        }
        indent_gcd
    }

    /// Return a copy of this text with indent guide characters inserted at every
    /// `indent_size` leading-space boundary.
    ///
    /// If `indent_size` is `None`, it is auto-detected via [`detect_indentation`](Text::detect_indentation).
    /// The `character` (e.g. `'|'` or `'\u{2502}'`) is styled with `guide_style`.
    pub fn with_indent_guides(
        &self,
        indent_size: Option<usize>,
        character: char,
        guide_style: Style,
    ) -> Text {
        let indent_size = indent_size.unwrap_or_else(|| self.detect_indentation());
        let lines = self.text.lines().collect::<Vec<&str>>();
        let mut new_text = String::new();
        let mut new_spans: Vec<Span> = Vec::new();
        let mut char_pos = 0usize;

        for (line_idx, line) in lines.iter().enumerate() {
            if line_idx > 0 {
                new_text.push('\n');
                char_pos += 1;
            }

            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();

            if indent == 0 || trimmed.is_empty() {
                new_text.push_str(line);
                char_pos += line.chars().count();
                continue;
            }

            // Replace leading spaces with guide characters at indent boundaries
            let mut i = 0;
            while i < indent {
                if i % indent_size == 0 && i + indent_size <= indent {
                    new_text.push(character);
                    let guide_pos = char_pos;
                    new_spans.push(Span::new(guide_pos, guide_pos + 1, guide_style.clone()));
                    char_pos += 1;
                    i += 1;
                } else {
                    new_text.push(' ');
                    char_pos += 1;
                    i += 1;
                }
            }
            new_text.push_str(trimmed);
            char_pos += trimmed.chars().count();
        }

        // Copy original spans, adjusting positions for guide character insertions
        // Since we only replace spaces with guide chars at the same positions,
        // the original spans should map similarly. But to be safe, we start fresh
        // with the guide spans and add original spans.
        let mut final_text = Text::new(&new_text, self.style.clone());
        final_text.spans = new_spans;
        // Re-add original spans (they are based on original char positions which may differ)
        // For simplicity, copy the original spans - they still reference the same char offsets
        for span in &self.spans {
            final_text.spans.push(span.clone());
        }
        final_text.justify = self.justify;
        final_text.overflow = self.overflow;
        final_text.no_wrap = self.no_wrap;
        final_text.end = self.end.clone();
        final_text.tab_size = self.tab_size;
        final_text
    }

    /// Remove spans that extend beyond the text length and clamp those that partially exceed it.
    pub fn trim_spans(&mut self) {
        let length = self.len();
        self.spans.retain_mut(|span| {
            if span.start >= length {
                return false;
            }
            if span.end > length {
                span.end = length;
            }
            !span.is_empty()
        });
    }

    // -- Rendering ----------------------------------------------------------

    /// Render the text into a list of [`Segment`]s, resolving any named (theme) spans
    /// through `console`'s active theme stack at render time.
    ///
    /// This is the themed analogue of [`render`](Self::render).  It mirrors the same
    /// sweep-line algorithm, but for each span that carries a `style_name` (created
    /// via [`Span::named`]) the style is resolved at call time via
    /// [`Console::get_style`], falling back to [`Style::null`] for unknown names.
    /// Ordinary spans (no `style_name`) are used as-is, byte-identical to `render()`.
    ///
    /// **Performance:** when no span carries a theme name (the common case —
    /// table cells, panel content, every Live frame) this delegates directly to
    /// [`render()`](Self::render), adding only a single linear scan of the spans
    /// vec and zero heap allocations.  The upfront `Vec<Style>` collect is only
    /// paid when at least one named span is present.
    ///
    /// **Borrow-checker note (named-span branch):** resolved styles are collected
    /// into an owned `Vec<Style>` *before* building the `style_map` slice, so that
    /// the resolved `Style` values outlive the entire sweep-line loop.  Do not
    /// refactor this into a lazy iterator — the borrow-checker will reject it.
    pub fn render_themed(&self, console: &crate::console::Console) -> Vec<Segment> {
        if self.spans.is_empty() {
            let style = if self.style.is_null() {
                None
            } else {
                Some(self.style.clone())
            };
            let mut segments = vec![Segment::new(&self.text, style.clone(), None)];
            if !self.end.is_empty() {
                segments.push(Segment::new(&self.end, style.clone(), None));
            }
            return segments;
        }

        // Fast path: no span carries a theme name → delegate to render() with
        // zero extra allocations or Style::clone() calls.  This covers the vast
        // majority of render_themed() call sites (widget layer, Live frames,
        // every call via gilt_console on ordinary Text).
        if !self.spans.iter().any(|s| s.style_name().is_some()) {
            return self.render();
        }

        // Named-span branch: resolve each span's style upfront into an owned
        // Vec<Style>.  Named spans are looked up through the console's theme
        // stack; ordinary spans are cloned as-is.  This vec must outlive the
        // style_map slice and the entire sweep-line loop below.
        let resolved_styles: Vec<Style> = self
            .spans
            .iter()
            .map(|span| {
                if let Some(name) = span.style_name() {
                    console.get_style(name).unwrap_or_else(|_| Style::null())
                } else {
                    span.style.clone()
                }
            })
            .collect();

        // Sweep-line algorithm (mirrors render() exactly, using resolved_styles).
        let mut events: Vec<(usize, bool, usize)> = Vec::new();
        for (i, span) in self.spans.iter().enumerate() {
            events.push((span.start, false, i + 1)); // entering
            events.push((span.end, true, i + 1)); // leaving
        }
        events.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

        let text_len = self.len();
        let mut segments = Vec::new();
        let mut active_spans: Vec<usize> = vec![0]; // 0 = base style always active
        let mut last_offset = 0;

        // Style map: index 0 = base style, index 1..n = resolved span styles.
        // All references point into either &self.style or &resolved_styles[i],
        // both of which outlive this function.
        let style_map: Vec<&Style> = {
            let mut v: Vec<&Style> = vec![&self.style];
            for rs in &resolved_styles {
                v.push(rs);
            }
            v
        };

        for &(offset, is_leaving, style_id) in &events {
            let offset = min(offset, text_len);
            if offset > last_offset {
                let slice = char_slice(&self.text, last_offset, offset);
                if !slice.is_empty() {
                    let combined =
                        Style::combine_refs(active_spans.iter().map(|&id| style_map[id]));
                    let style = if combined.is_null() {
                        None
                    } else {
                        Some(combined)
                    };
                    segments.push(Segment::new(slice, style, None));
                }
            }
            last_offset = offset;

            if is_leaving {
                if let Some(pos) = active_spans.iter().position(|&x| x == style_id) {
                    active_spans.remove(pos);
                }
            } else {
                active_spans.push(style_id);
            }
        }

        if last_offset < text_len {
            let slice = char_slice(&self.text, last_offset, text_len);
            if !slice.is_empty() {
                let combined = Style::combine_refs(active_spans.iter().map(|&id| style_map[id]));
                let style = if combined.is_null() {
                    None
                } else {
                    Some(combined)
                };
                segments.push(Segment::new(slice, style, None));
            }
        }

        if !self.end.is_empty() {
            segments.push(Segment::new(&self.end, None, None));
        }

        segments
    }

    /// Render the text into a list of [`Segment`]s, each carrying a combined style.
    ///
    /// Uses a sweep-line algorithm to merge overlapping spans into non-overlapping
    /// styled segments. An end segment (containing [`Text::end`]) is appended if
    /// the end string is non-empty.
    pub fn render(&self) -> Vec<Segment> {
        if self.spans.is_empty() {
            let style = if self.style.is_null() {
                None
            } else {
                Some(self.style.clone())
            };
            let mut segments = vec![Segment::new(&self.text, style.clone(), None)];
            if !self.end.is_empty() {
                segments.push(Segment::new(&self.end, style.clone(), None));
            }
            return segments;
        }

        // Fast path: no span carries a theme name → keep the zero-clone hot path
        // unchanged.  This covers the vast majority of render() call sites.
        if !self.spans.iter().any(|s| s.style_name().is_some()) {
            // Sweep-line algorithm (zero-clone path)
            // Build events: (offset, is_leaving, span_index)
            // span_index 0 is self.style (always active), 1..n are spans
            let mut events: Vec<(usize, bool, usize)> = Vec::new();
            for (i, span) in self.spans.iter().enumerate() {
                events.push((span.start, false, i + 1)); // entering
                events.push((span.end, true, i + 1)); // leaving
            }
            events.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

            let text_len = self.len();
            let mut segments = Vec::new();
            let mut active_spans: Vec<usize> = vec![0]; // 0 = base style always active
            let mut last_offset = 0;

            // Style map: index 0 = base style, index 1..n = span styles
            // T2/T3: zero-clone — references straight into span storage.
            let style_map: Vec<&Style> = {
                let mut v: Vec<&Style> = vec![&self.style];
                for span in &self.spans {
                    v.push(&span.style);
                }
                v
            };

            for &(offset, is_leaving, style_id) in &events {
                let offset = min(offset, text_len);
                if offset > last_offset {
                    // Emit segment for [last_offset, offset)
                    let slice = char_slice(&self.text, last_offset, offset);
                    if !slice.is_empty() {
                        // T2/T3: avoid the per-event Vec<Style> alloc and the
                        // per-element style.clone() inside Style::combine — pass
                        // references straight from style_map via combine_refs.
                        let combined =
                            Style::combine_refs(active_spans.iter().map(|&id| style_map[id]));
                        let style = if combined.is_null() {
                            None
                        } else {
                            Some(combined)
                        };
                        segments.push(Segment::new(slice, style, None));
                    }
                }
                last_offset = offset;

                if is_leaving {
                    if let Some(pos) = active_spans.iter().position(|&x| x == style_id) {
                        active_spans.remove(pos);
                    }
                } else {
                    active_spans.push(style_id);
                }
            }

            // Emit remaining text (T2/T3: same combine_refs optimisation)
            if last_offset < text_len {
                let slice = char_slice(&self.text, last_offset, text_len);
                if !slice.is_empty() {
                    let combined =
                        Style::combine_refs(active_spans.iter().map(|&id| style_map[id]));
                    let style = if combined.is_null() {
                        None
                    } else {
                        Some(combined)
                    };
                    segments.push(Segment::new(slice, style, None));
                }
            }

            // Append end segment
            if !self.end.is_empty() {
                segments.push(Segment::new(&self.end, None, None));
            }

            return segments;
        }

        // Named-span branch: resolve each span's style upfront into an owned
        // Vec<Style>.  Named spans are looked up via DEFAULT_STYLES; ordinary
        // spans are cloned as-is.  This vec must outlive style_map and the
        // entire sweep-line loop below.
        let resolved_styles: Vec<Style> = self
            .spans
            .iter()
            .map(|span| {
                if let Some(name) = span.style_name() {
                    DEFAULT_STYLES
                        .get(name)
                        .cloned()
                        .unwrap_or_else(Style::null)
                } else {
                    span.style.clone()
                }
            })
            .collect();

        // Sweep-line algorithm
        // Build events: (offset, is_leaving, span_index)
        // span_index 0 is self.style (always active), 1..n are spans
        let mut events: Vec<(usize, bool, usize)> = Vec::new();
        for (i, span) in self.spans.iter().enumerate() {
            events.push((span.start, false, i + 1)); // entering
            events.push((span.end, true, i + 1)); // leaving
        }
        events.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

        let text_len = self.len();
        let mut segments = Vec::new();
        let mut active_spans: Vec<usize> = vec![0]; // 0 = base style always active
        let mut last_offset = 0;

        // Style map: index 0 = base style, index 1..n = resolved span styles.
        // All references point into either &self.style or &resolved_styles[i],
        // both of which outlive this function.
        let style_map: Vec<&Style> = {
            let mut v: Vec<&Style> = vec![&self.style];
            for rs in &resolved_styles {
                v.push(rs);
            }
            v
        };

        for &(offset, is_leaving, style_id) in &events {
            let offset = min(offset, text_len);
            if offset > last_offset {
                // Emit segment for [last_offset, offset)
                let slice = char_slice(&self.text, last_offset, offset);
                if !slice.is_empty() {
                    // T2/T3: avoid the per-event Vec<Style> alloc and the
                    // per-element style.clone() inside Style::combine — pass
                    // references straight from style_map via combine_refs.
                    let combined =
                        Style::combine_refs(active_spans.iter().map(|&id| style_map[id]));
                    let style = if combined.is_null() {
                        None
                    } else {
                        Some(combined)
                    };
                    segments.push(Segment::new(slice, style, None));
                }
            }
            last_offset = offset;

            if is_leaving {
                if let Some(pos) = active_spans.iter().position(|&x| x == style_id) {
                    active_spans.remove(pos);
                }
            } else {
                active_spans.push(style_id);
            }
        }

        // Emit remaining text (T2/T3: same combine_refs optimisation)
        if last_offset < text_len {
            let slice = char_slice(&self.text, last_offset, text_len);
            if !slice.is_empty() {
                let combined = Style::combine_refs(active_spans.iter().map(|&id| style_map[id]));
                let style = if combined.is_null() {
                    None
                } else {
                    Some(combined)
                };
                segments.push(Segment::new(slice, style, None));
            }
        }

        // Append end segment
        if !self.end.is_empty() {
            segments.push(Segment::new(&self.end, None, None));
        }

        segments
    }

    // -- Wrapping -----------------------------------------------------------

    /// Word-wrap the text to fit within `width` terminal cells, returning [`Lines`].
    ///
    /// The text is first split on newlines, tabs are expanded, and each line is
    /// wrapped using [`crate::wrap::divide_line`]. Optional justification and
    /// overflow truncation are applied afterwards.
    ///
    /// When `no_wrap` is `true`, lines are not wrapped but may still be truncated
    /// according to the `overflow` strategy.
    pub fn wrap(
        &self,
        width: usize,
        justify: Option<JustifyMethod>,
        overflow: Option<OverflowMethod>,
        tab_size: usize,
        no_wrap: bool,
    ) -> Lines {
        let overflow = overflow.unwrap_or(OverflowMethod::Fold);

        // 1. Split on newlines (include_separator=false, matching Python's default)
        let new_lines = self.split("\n", false, true);
        let mut all_lines = Lines::default();

        for mut line in new_lines.lines {
            // 2. Expand tabs
            line.expand_tabs(Some(tab_size));

            if no_wrap {
                if overflow != OverflowMethod::Ignore {
                    // Still keep as single line
                }
                all_lines.push(line);
            } else {
                // 3. Wrap the line; fold only when overflow == Fold.
                let offsets = divide_line(line.plain(), width, overflow == OverflowMethod::Fold);
                if offsets.is_empty() {
                    all_lines.push(line);
                } else {
                    let divided = line.divide(&offsets);
                    for mut dl in divided.lines {
                        dl.rstrip_end(width);
                        all_lines.push(dl);
                    }
                }
            }
        }

        // 4. Justify
        if let Some(j) = justify {
            all_lines.justify(width, j, overflow);
        }

        // 5. Truncate each line
        for line in all_lines.iter_mut() {
            if line.cell_len() > width {
                line.truncate(width, Some(overflow), false);
            }
        }

        all_lines
    }

    // -- Introspection ------------------------------------------------------

    /// Get the resolved style at the given character offset.
    ///
    /// Combines the root style with all spans that overlap the offset.
    pub fn get_style_at_offset(&self, offset: usize) -> Style {
        let mut style = self.style.clone();
        for span in &self.spans {
            if offset >= span.start && offset < span.end {
                style = style + span.style.clone();
            }
        }
        style
    }

    /// Get the resolved style at the given character offset, with theme resolution.
    ///
    /// Like `get_style_at_offset`, but uses `console` to resolve the text's
    /// base style through the active theme stack before combining span styles.
    /// This means named theme styles (e.g. `"highlight"`) in the text's base
    /// style are resolved correctly.
    pub fn get_style_at_offset_themed(
        &self,
        console: &crate::console::Console,
        offset: usize,
    ) -> Style {
        // Resolve the text's base style through the console's theme stack.
        let base_str = self.style.to_string();
        let base = if base_str == "none" {
            Style::null()
        } else {
            console
                .get_style(&base_str)
                .unwrap_or_else(|_| self.style.clone())
        };
        let mut style = base;
        for span in &self.spans {
            if offset >= span.start && offset < span.end {
                // Resolve each span's style through the console's theme stack.
                let span_str = span.style.to_string();
                let resolved = if span_str == "none" {
                    span.style.clone()
                } else {
                    console
                        .get_style(&span_str)
                        .unwrap_or_else(|_| span.style.clone())
                };
                style = style + resolved;
            }
        }
        style
    }

    // -- Metadata -----------------------------------------------------------

    /// Attach arbitrary key/value metadata to the character range `[start, end)`.
    ///
    /// Inserts a [`Span`] with [`Style::null()`] and the given `meta` map over
    /// `[start.clamp(0, len), end.clamp(0, len))`.  If the clamped range is empty
    /// the call is a no-op.
    ///
    /// This mirrors Python rich's `Text.apply_meta`.
    pub fn apply_meta(&mut self, meta: HashMap<String, String>, start: usize, end: usize) {
        let length = self.len();
        let start = min(start, length);
        let end = min(end, length);
        if start >= end {
            return;
        }
        self.spans.push(Span::with_meta(
            start,
            end,
            Style::null(),
            Some(Arc::new(meta)),
        ));
    }

    /// Return a new `Text` whose base style has the given hyperlink URL set.
    ///
    /// The URL is stamped onto `self.style` via [`Style::with_link`] so that
    /// the entire text renders as an OSC-8 hyperlink.
    ///
    /// # Note
    ///
    /// Python rich's `Text.on` attaches arbitrary event metadata/handlers.
    /// In gilt (a terminal renderer) the meaningful effect of `on` is the
    /// OSC-8 hyperlink; other event semantics are out of scope for the
    /// terminal layer.
    pub fn on(mut self, url: &str) -> Self {
        self.style = self.style + Style::with_link(url);
        self
    }

    /// Return the metadata map of the last (topmost) [`Span`] covering `offset`,
    /// if any span with metadata covers that offset.
    ///
    /// "Topmost" means the span that appears last in the span list — consistent
    /// with gilt's rendering model where later spans override earlier ones.
    ///
    /// Returns `None` if no meta span covers `offset`.
    pub fn get_meta_at(&self, offset: usize) -> Option<&HashMap<String, String>> {
        // Walk in reverse to find the last (topmost) meta span covering offset.
        self.spans
            .iter()
            .rev()
            .filter(|s| s.start <= offset && offset < s.end && s.meta.is_some())
            .find_map(|s| s.meta.as_deref())
    }

    /// Convert this `Text` back into a markup string.
    ///
    /// Produces a string that, when re-parsed via [`Text::from_markup`], yields
    /// an equivalent `Text` (same plain text and equivalent style spans).
    ///
    /// Plain-text characters that would be interpreted as markup tags are
    /// escaped automatically.  Null-style spans (e.g. those produced from
    /// unresolved theme names) are omitted — they carry no information that can
    /// be round-tripped without a console present.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() {
    /// use gilt::text::Text;
    /// let text = Text::from_markup("[bold]Hello[/bold] world").unwrap();
    /// let m = text.markup();
    /// let rt = Text::from_markup(&m).unwrap();
    /// assert_eq!(rt.plain(), text.plain());
    /// # }
    /// ```
    pub fn markup(&self) -> String {
        use crate::markup::escape;
        use std::collections::BTreeMap;

        // Collect open/close tags keyed by char offset.
        // At each offset: opens come AFTER emitting text up to that point;
        // closes come BEFORE emitting text (i.e. they appear after the last
        // char of their range, which is *before* any new opens at the same offset).
        // Ordering at a boundary: closes first, then opens.
        let mut opens: BTreeMap<usize, Vec<String>> = BTreeMap::new();
        let mut closes: BTreeMap<usize, Vec<String>> = BTreeMap::new();

        for span in &self.spans {
            // Item 3: meta spans (null style + `meta` map) must be preserved in the
            // round-trip.  Build their open-tag string as `@key` (bare flag) or
            // `@key=value` (non-"true" value) and close-tag as `/@key`.
            if let Some(meta) = &span.meta {
                for (key, val) in meta.as_ref() {
                    // key already includes the leading `@` (e.g. "@click").
                    // The emit loop prepends "[/" automatically, so the closes
                    // map stores the bare tag name (without the `/` prefix).
                    let open_tag = if val == "true" {
                        key.clone()
                    } else {
                        format!("{}={}", key, val)
                    };
                    // close_tag stored WITHOUT `/` — the emitter adds `[/…]`
                    opens.entry(span.start).or_default().push(open_tag);
                    closes.entry(span.end).or_default().push(key.clone());
                }
                continue; // meta span handled — don't also emit a style tag
            }

            let style_str = span.style.to_string();
            // Skip null/unresolved styles (theme-name spans) — nothing to round-trip
            // without a console present; only meta spans (handled above) are exempt.
            if style_str == "none" {
                continue;
            }
            opens.entry(span.start).or_default().push(style_str.clone());
            closes.entry(span.end).or_default().push(style_str);
        }

        // Gather all unique boundary offsets.
        let mut boundaries: Vec<usize> = opens.keys().chain(closes.keys()).copied().collect();
        boundaries.sort_unstable();
        boundaries.dedup();

        // Collect chars once for O(n) slicing.
        let chars: Vec<char> = self.text.chars().collect();
        let text_len = chars.len();

        let mut result = String::new();
        let mut prev = 0usize;

        for &boundary in &boundaries {
            // 1. Emit escaped plain text from prev to boundary.
            if prev < boundary {
                let end = boundary.min(text_len);
                if prev < end {
                    let segment: String = chars[prev..end].iter().collect();
                    result.push_str(&escape(&segment));
                }
                prev = boundary;
            }

            // 2. Emit close tags (they end *at* this boundary, so appear here).
            if let Some(tags) = closes.get(&boundary) {
                for tag in tags {
                    result.push_str("[/");
                    result.push_str(tag);
                    result.push(']');
                }
            }

            // 3. Emit open tags (they start *at* this boundary).
            if let Some(tags) = opens.get(&boundary) {
                for tag in tags {
                    result.push('[');
                    result.push_str(tag);
                    result.push(']');
                }
            }
        }

        // 4. Emit any remaining plain text after the last boundary.
        if prev < text_len {
            let segment: String = chars[prev..].iter().collect();
            result.push_str(&escape(&segment));
        }

        result
    }

    /// Flatten overlapping spans into non-overlapping spans.
    ///
    /// Each resulting span covers a contiguous range with a single resolved
    /// style computed by combining all overlapping source spans.  Ranges
    /// whose resolved style is null are omitted.
    pub fn flatten_spans(&self) -> Vec<Span> {
        // Collect every unique boundary from existing spans.
        let mut boundaries: Vec<usize> = Vec::new();
        for span in &self.spans {
            boundaries.push(span.start);
            boundaries.push(span.end);
        }
        boundaries.sort_unstable();
        boundaries.dedup();

        let mut result: Vec<Span> = Vec::new();
        for pair in boundaries.windows(2) {
            let (start, end) = (pair[0], pair[1]);
            if start >= end {
                continue;
            }
            // Resolve style for this range by combining all overlapping spans.
            let mut style = Style::null();
            for span in &self.spans {
                if span.start <= start && span.end >= end {
                    style = style + span.style.clone();
                }
            }
            if !style.is_null() {
                result.push(Span::new(start, end, style));
            }
        }
        result
    }

    /// Get a substring starting at `offset` with the given character `length`.
    ///
    /// Returns `None` if the range is out of bounds.
    pub fn get_text_at(&self, offset: usize, length: usize) -> Option<&str> {
        let mut chars = self.text.char_indices();
        let start_byte = match chars.nth(offset) {
            Some((idx, _)) => idx,
            None => return None,
        };
        // Advance `length - 1` more characters (nth(0) would be the next char).
        let end_byte = if length == 0 {
            start_byte
        } else {
            match chars.nth(length - 1) {
                Some((idx, _)) => idx,
                None => self.text.len(),
            }
        };
        Some(&self.text[start_byte..end_byte])
    }
}

// -- Display ----------------------------------------------------------------

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

// -- PartialEq --------------------------------------------------------------

impl PartialEq for Text {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text && self.spans == other.spans
    }
}

impl Eq for Text {}

// -- Add --------------------------------------------------------------------

impl Add<Text> for Text {
    type Output = Text;

    fn add(self, rhs: Text) -> Text {
        let mut result = self.copy();
        result.append_text(&rhs);
        result
    }
}

impl Add<&str> for Text {
    type Output = Text;

    fn add(self, rhs: &str) -> Text {
        let mut result = self.copy();
        result.append_str(rhs, None);
        result
    }
}

impl From<&str> for Text {
    fn from(s: &str) -> Self {
        Text::new(s, Style::null())
    }
}

impl From<String> for Text {
    fn from(s: String) -> Self {
        Text::new(&s, Style::null())
    }
}

impl From<&String> for Text {
    fn from(s: &String) -> Self {
        Text::new(s, Style::null())
    }
}

impl From<std::borrow::Cow<'_, str>> for Text {
    fn from(s: std::borrow::Cow<'_, str>) -> Self {
        Text::new(&s, Style::null())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Style;
    use crate::text::Span;

    // -- markup() round-trip tests ------------------------------------------

    /// Simple bold round-trip: `[bold]Hi[/bold]` → markup → re-parse → equivalent.
    #[test]
    fn markup_roundtrip_simple_text() {
        let original = Text::from_markup("[bold]Hi[/bold]").unwrap();
        let m = original.markup();
        let roundtripped = Text::from_markup(&m).unwrap();
        assert_eq!(roundtripped.plain(), original.plain());
        assert_eq!(roundtripped.spans().len(), original.spans().len());
        for (a, b) in roundtripped.spans().iter().zip(original.spans().iter()) {
            assert_eq!(a.start, b.start);
            assert_eq!(a.end, b.end);
            assert_eq!(a.style, b.style);
        }
    }

    /// Plain text with no spans → `markup()` returns just the escaped plain text.
    #[test]
    fn markup_roundtrip_no_spans() {
        let text = Text::new("hello world", Style::null());
        let m = text.markup();
        assert_eq!(m, "hello world");
        let rt = Text::from_markup(&m).unwrap();
        assert_eq!(rt.plain(), "hello world");
        assert_eq!(rt.spans().len(), 0);
    }

    /// Literal `[tag]` in plain text is escaped so it is not interpreted as markup.
    #[test]
    fn markup_escapes_literal_brackets() {
        let text = Text::new("foo[bar]", Style::null());
        let m = text.markup();
        // The `[bar]` sequence looks like a tag so escape() should prefix it with `\`.
        assert!(
            m.contains(r"\[bar]"),
            "expected escaped bracket, got: {m:?}"
        );
        // Re-parsing should recover the original plain text.
        let rt = Text::from_markup(&m).unwrap();
        assert_eq!(rt.plain(), "foo[bar]");
    }

    /// Empty text returns an empty markup string.
    #[test]
    fn markup_roundtrip_empty_text() {
        let text = Text::new("", Style::null());
        assert_eq!(text.markup(), "");
    }

    /// Overlapping spans round-trip: the re-parsed Text has spans covering the
    /// same ranges with the same styles (order may differ after sort).
    #[test]
    fn markup_roundtrip_overlapping_spans() {
        // Build a Text with two overlapping spans manually:
        //   green: [0, 2)   → "XY"
        //   bold:  [1, 3)   → "YZ"
        // plain = "XYZ"
        let mut text = Text::new("XYZ", Style::null());
        text.spans_mut()
            .push(Span::new(0, 2, Style::parse("green")));
        text.spans_mut().push(Span::new(1, 3, Style::parse("bold")));

        let m = text.markup();
        let rt = Text::from_markup(&m).unwrap();

        assert_eq!(rt.plain(), "XYZ");
        assert_eq!(rt.spans().len(), 2);

        // Verify both original spans are present (order-independent comparison).
        let has_green = rt
            .spans()
            .iter()
            .any(|s| s.start == 0 && s.end == 2 && s.style == Style::parse("green"));
        let has_bold = rt
            .spans()
            .iter()
            .any(|s| s.start == 1 && s.end == 3 && s.style == Style::parse("bold"));
        assert!(has_green, "missing green span in round-trip");
        assert!(has_bold, "missing bold span in round-trip");
    }

    // -- get_style_at_offset_themed tests -----------------------------------

    // -- apply_meta / get_meta_at tests ------------------------------------

    #[test]
    fn apply_meta_inserts_meta_span() {
        let mut text = Text::new("hello world", Style::null());
        let mut m = std::collections::HashMap::new();
        m.insert("action".to_string(), "click".to_string());
        text.apply_meta(m, 0, 5);
        assert_eq!(text.spans().len(), 1);
        let span = &text.spans()[0];
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 5);
        assert!(span.style.is_null());
        let meta = span.meta.as_ref().expect("must have meta");
        assert_eq!(meta.get("action").map(|v| v.as_str()), Some("click"));
    }

    #[test]
    fn apply_meta_clamps_to_text_length() {
        let mut text = Text::new("hi", Style::null()); // len = 2
        let mut m = std::collections::HashMap::new();
        m.insert("k".to_string(), "v".to_string());
        // start=0, end=100 → clamped to 0..2
        text.apply_meta(m, 0, 100);
        assert_eq!(text.spans().len(), 1);
        assert_eq!(text.spans()[0].end, 2);
    }

    #[test]
    fn apply_meta_empty_range_is_noop() {
        let mut text = Text::new("abc", Style::null());
        let m = std::collections::HashMap::new();
        text.apply_meta(m, 2, 2);
        assert_eq!(text.spans().len(), 0);
    }

    #[test]
    fn get_meta_at_returns_last_covering_span() {
        let mut text = Text::new("abcde", Style::null());
        let mut m1 = std::collections::HashMap::new();
        m1.insert("layer".to_string(), "first".to_string());
        let mut m2 = std::collections::HashMap::new();
        m2.insert("layer".to_string(), "second".to_string());
        text.apply_meta(m1, 0, 5);
        text.apply_meta(m2, 0, 5);
        // get_meta_at should return the last (topmost) span's meta.
        let meta = text.get_meta_at(2).expect("must have meta at offset 2");
        assert_eq!(meta.get("layer").map(|v| v.as_str()), Some("second"));
    }

    #[test]
    fn get_meta_at_returns_none_for_uncovered_offset() {
        let mut text = Text::new("hello world", Style::null());
        let mut m = std::collections::HashMap::new();
        m.insert("k".to_string(), "v".to_string());
        text.apply_meta(m, 0, 5); // covers "hello"
                                  // offset 6 ("w") is not covered
        assert!(text.get_meta_at(6).is_none());
    }

    #[test]
    fn on_sets_link_on_base_style() {
        let text = Text::new("click me", Style::null()).on("https://example.com");
        assert_eq!(text.style().link(), Some("https://example.com"));
    }

    #[test]
    fn on_preserves_existing_style() {
        let text = Text::new("click me", Style::parse("bold")).on("https://example.com");
        // bold should still be present
        assert_eq!(text.style().bold(), Some(true));
        assert_eq!(text.style().link(), Some("https://example.com"));
    }

    // -- render_themed tests ------------------------------------------------

    /// Task 2.3: Named span "warning" → bold red is resolved through the
    /// console's theme at render time.
    #[test]
    fn theme_name_span_resolved_at_render_time() {
        use crate::color::theme::Theme;
        use crate::console::Console;
        use std::collections::HashMap;

        let mut styles = HashMap::new();
        styles.insert("warning".to_string(), Style::parse("bold red"));
        let theme = Theme::new(Some(styles), true);
        let console = Console::builder().theme(theme).no_color(false).build();

        // render_str with markup=true produces a named span for "warning"
        let text = console.render_str("[warning]x[/warning]", None, None, None);
        assert_eq!(
            text.spans()[0].style_name(),
            Some("warning"),
            "span should carry the theme name"
        );

        let segments = text.render_themed(&console);
        let x_seg = segments
            .iter()
            .find(|s| s.text.contains('x'))
            .expect("must find the 'x' segment");
        let st = x_seg.style().expect("'x' segment must have a style");
        assert_eq!(st.bold(), Some(true), "expected bold from theme");
        assert!(
            st.color().is_some_and(|c| c.name().contains("red")),
            "expected red foreground from theme, got: {:?}",
            st.color().map(|c| c.name().into_owned())
        );
    }

    /// Task 2.3: Unknown theme name falls back to null style — no panic.
    #[test]
    fn render_themed_fallback_to_null_for_unknown_name() {
        use crate::console::Console;

        let console = Console::builder().build();
        let mut text = Text::new("x", Style::null());
        text.spans_mut().push(Span::named(0, 1, "does.not.exist"));
        let segs = text.render_themed(&console);
        assert!(
            segs.iter().any(|s| s.text.contains('x')),
            "must produce a segment containing 'x' without panic"
        );
    }

    /// Task 2.3: A Text with no named spans takes the fast path and produces
    /// segments byte-identical to render() — zero extra allocs/clones.
    ///
    /// This exercises the common case: table cells, panel content, every Live
    /// frame.  The fast path delegates directly to render() when no span carries
    /// a style_name, skipping the upfront Vec<Style> collect entirely.
    #[test]
    fn render_themed_no_named_spans_matches_render() {
        use crate::console::Console;

        let console = Console::builder().build();

        // Single ordinary span — the most common case.
        let mut text = Text::new("hello world", Style::null());
        text.spans_mut().push(Span::new(0, 5, Style::parse("bold")));
        assert_eq!(
            text.render(),
            text.render_themed(&console),
            "single ordinary span: render_themed must equal render()"
        );

        // Multiple overlapping ordinary spans — previously each would have been
        // cloned into the upfront Vec<Style>; fast path avoids all of that.
        let mut text2 = Text::new("hello world", Style::null());
        text2
            .spans_mut()
            .push(Span::new(0, 5, Style::parse("bold")));
        text2.spans_mut().push(Span::new(2, 8, Style::parse("red")));
        text2
            .spans_mut()
            .push(Span::new(6, 11, Style::parse("italic")));
        assert_eq!(
            text2.render(),
            text2.render_themed(&console),
            "multiple overlapping ordinary spans: render_themed must equal render()"
        );
    }

    /// A Console with a theme entry "highlight" → bold red; a Text whose span
    /// carries the resolved highlight style should return bold red from
    /// `get_style_at_offset_themed`.
    #[test]
    fn get_style_at_offset_themed_resolves_named_style() {
        use crate::color::theme::Theme;
        use crate::console::Console;
        use std::collections::HashMap;

        // Build a theme that maps "highlight" → bold red.
        let highlight_style = Style::parse("bold red");
        let mut styles = HashMap::new();
        styles.insert("highlight".to_string(), highlight_style.clone());
        let theme = Theme::new(Some(styles), false);

        let console = Console::builder().theme(theme).build();

        // Build a Text with a span that carries the "highlight" style.
        // We look the style up from the console so the span stores the resolved Style.
        let resolved = console.get_style("highlight").unwrap();
        let mut text = Text::new("hello", Style::null());
        text.spans_mut().push(Span::new(0, 5, resolved));

        let style_at_0 = text.get_style_at_offset_themed(&console, 0);
        assert_eq!(style_at_0.bold(), Some(true), "expected bold");
        assert_eq!(
            style_at_0.color().map(|c| c.name().into_owned()),
            Some("red".to_string()),
            "expected red foreground"
        );
    }

    // --- highlight_regex_with_groups DEFAULT_STYLES resolution (#7) ---

    #[test]
    fn highlight_regex_with_groups_resolves_default_style_names() {
        let re = regex::Regex::new(r"(?P<number>\d+)").unwrap();
        let mut text = Text::new("count=42", Style::null());
        let count = text.highlight_regex_with_groups(&re, "repr.");
        assert_eq!(
            count, 1,
            "repr.number exists in DEFAULT_STYLES → count must be 1"
        );
        // span covering "42" should carry repr.number = bold not-italic cyan
        let plain = text.plain().to_string();
        let s = text
            .spans()
            .iter()
            .find(|sp| {
                let b = |n: usize| {
                    plain
                        .char_indices()
                        .nth(n)
                        .map(|(i, _)| i)
                        .unwrap_or(plain.len())
                };
                &plain[b(sp.start)..b(sp.end)] == "42"
            })
            .expect("expected a span covering '42'");
        assert_eq!(s.style.bold(), Some(true), "repr.number should be bold");
        assert_eq!(
            s.style.italic(),
            Some(false),
            "repr.number should be not italic"
        );
        assert!(
            s.style.color().is_some_and(|c| c.name().contains("cyan")),
            "repr.number should have cyan foreground"
        );
    }

    #[test]
    fn highlight_regex_with_groups_unknown_name_produces_no_span() {
        let re = regex::Regex::new(r"(?P<frobnicator>\d+)").unwrap();
        let mut text = Text::new("x=99", Style::null());
        // "repr.frobnicator" is in neither DEFAULT_STYLES nor parseable as a style literal
        let count = text.highlight_regex_with_groups(&re, "repr.");
        assert_eq!(count, 0, "unknown group name should produce no span");
    }

    // --- Task 2.6: standalone render() resolves named spans via DEFAULT_STYLES ---

    /// A Text whose span carries a theme name should have that name resolved
    /// via DEFAULT_STYLES when rendered without a Console.
    #[test]
    fn standalone_render_resolves_named_span_against_default_styles() {
        let mut text = Text::new("42", Style::null());
        text.spans_mut().push(Span::named(0, 2, "repr.number"));
        let segs = text.render();
        let seg = segs.iter().find(|s| s.text.contains("42")).unwrap();
        let st = seg.style().unwrap();
        assert_eq!(st.bold(), Some(true));
        assert!(st.color().is_some_and(|c| c.name().contains("cyan")));
    }

    /// An unknown theme name in a standalone render() should produce a null
    /// style segment without panic (graceful degradation).
    #[test]
    fn standalone_render_unknown_named_span_renders_null_without_panic() {
        let mut text = Text::new("x", Style::null());
        text.spans_mut().push(Span::named(0, 1, "does.not.exist"));
        let segs = text.render();
        assert!(
            segs.iter().any(|s| s.text.contains('x')),
            "must produce a segment containing 'x' without panic"
        );
        // The unknown name resolves to null style, so the segment has no style.
        let seg = segs.iter().find(|s| s.text.contains('x')).unwrap();
        assert!(
            seg.style().is_none(),
            "unknown named span should resolve to null (no style)"
        );
    }

    /// Non-named spans must still use the zero-clone fast path after Task 2.6.
    /// render_themed() on those spans must equal render() (the fast-path gate).
    #[test]
    fn standalone_render_non_named_spans_unchanged_zero_clone() {
        // Ordinary spans: render() must equal render_themed() (fast path delegate)
        use crate::console::Console;
        let console = Console::builder().build();

        let mut text = Text::new("hello world", Style::null());
        text.spans_mut().push(Span::new(0, 5, Style::parse("bold")));
        text.spans_mut()
            .push(Span::new(6, 11, Style::parse("italic")));

        // No named spans → fast path in render() → identical to render_themed()
        assert_eq!(
            text.render(),
            text.render_themed(&console),
            "non-named spans: render() must equal render_themed() (fast-path gate)"
        );
    }

    // -- Phase 7 / Task 7.14: markup() meta-span round-trip (Item 3) ----------

    /// A bare `[@flag]` meta span must survive a `markup()` → `from_markup()` round-trip.
    #[test]
    fn markup_roundtrip_meta_bare_flag() {
        // Build a Text directly (not via from_markup) to avoid any parse dependency.
        use std::collections::HashMap;
        use std::sync::Arc;

        let mut text = Text::new("hello", Style::null());
        let mut m = HashMap::new();
        m.insert("@click".to_string(), "true".to_string());
        text.spans_mut()
            .push(Span::with_meta(0, 5, Style::null(), Some(Arc::new(m))));

        let markup_str = text.markup();
        // Must contain `[@click]` (bare flag form) and NOT `[@click=true]`
        // (either form is valid, but bare flag is canonical for "true")
        assert!(
            markup_str.contains("[@click]"),
            "markup() must emit bare-flag form for value=true; got: {markup_str:?}"
        );

        // Round-trip: re-parse and verify the meta span is preserved.
        let rt = crate::markup::render(&markup_str, Style::null()).unwrap();
        assert_eq!(rt.plain(), "hello");
        let meta_spans: Vec<_> = rt.spans().iter().filter(|s| s.meta.is_some()).collect();
        assert_eq!(meta_spans.len(), 1, "meta span must survive round-trip");
        let meta = meta_spans[0].meta.as_ref().unwrap();
        assert_eq!(meta.get("@click").map(|v| v.as_str()), Some("true"));
    }

    /// A `[@key=val]` meta span with a non-"true" value must survive the round-trip.
    #[test]
    fn markup_roundtrip_meta_key_value() {
        use std::collections::HashMap;
        use std::sync::Arc;

        let mut text = Text::new("world", Style::null());
        let mut m = HashMap::new();
        m.insert("@action".to_string(), "submit".to_string());
        text.spans_mut()
            .push(Span::with_meta(0, 5, Style::null(), Some(Arc::new(m))));

        let markup_str = text.markup();
        assert!(
            markup_str.contains("[@action=submit]"),
            "markup() must emit `[@key=val]` for non-true value; got: {markup_str:?}"
        );

        let rt = crate::markup::render(&markup_str, Style::null()).unwrap();
        assert_eq!(rt.plain(), "world");
        let meta_spans: Vec<_> = rt.spans().iter().filter(|s| s.meta.is_some()).collect();
        assert_eq!(meta_spans.len(), 1, "meta span must survive round-trip");
        let meta = meta_spans[0].meta.as_ref().unwrap();
        assert_eq!(meta.get("@action").map(|v| v.as_str()), Some("submit"));
    }

    /// Markup round-trip: meta span from `from_markup` survives `markup()` + re-parse.
    #[test]
    fn markup_roundtrip_meta_from_parse() {
        let original = crate::markup::render("[@click]btn[/]", Style::null()).unwrap();
        let m = original.markup();
        let rt = crate::markup::render(&m, Style::null()).unwrap();
        assert_eq!(rt.plain(), "btn");
        let meta_spans: Vec<_> = rt.spans().iter().filter(|s| s.meta.is_some()).collect();
        assert_eq!(
            meta_spans.len(),
            1,
            "meta span must survive markup→re-parse round-trip"
        );
        let meta = meta_spans[0].meta.as_ref().unwrap();
        assert_eq!(meta.get("@click").map(|v| v.as_str()), Some("true"));
    }

    /// Named (theme) spans with null style must still be skipped by markup()
    /// (intentional — cannot be round-tripped without a console). Only meta spans
    /// are exempt.
    #[test]
    fn markup_roundtrip_named_theme_span_still_dropped() {
        let mut text = Text::new("hello", Style::null());
        text.spans_mut().push(Span::named(0, 5, "warning"));

        let markup_str = text.markup();
        // Named spans are intentionally not round-tripped (no console = no theme).
        // The markup should just be the escaped plain text.
        assert_eq!(
            markup_str, "hello",
            "named spans must be dropped from markup()"
        );
    }
    // --- Task 1 (P2): highlight_regex callable-style overload ---

    /// A callable that maps each matched substring to a Style should produce
    /// distinct spans per match — odd-length matches get bold, even get italic.
    /// This proves the closure receives the matched text and runs per-match.
    #[test]
    fn highlight_regex_callable_per_match_styles() {
        let bold_s = Style::parse("bold");
        let italic_s = Style::parse("italic");
        let mut text = Text::new("1 22 333 4444", Style::null());
        let re = Regex::new(r"\d+").unwrap();
        let count = text.highlight_regex_callable(&re, |m| {
            if m.chars().count() % 2 == 1 {
                bold_s.clone()
            } else {
                italic_s.clone()
            }
        });
        assert_eq!(count, 4);
        // matches: "1" (odd→bold), "22" (even→italic), "333" (odd→bold), "4444" (even→italic)
        let spans = text.spans();
        assert_eq!(spans.len(), 4);
        assert_eq!(spans[0].start, 0);
        assert_eq!(spans[0].end, 1);
        assert_eq!(spans[0].style, bold_s);
        assert_eq!(spans[1].start, 2);
        assert_eq!(spans[1].end, 4);
        assert_eq!(spans[1].style, italic_s);
        assert_eq!(spans[2].start, 5);
        assert_eq!(spans[2].end, 8);
        assert_eq!(spans[2].style, bold_s);
        assert_eq!(spans[3].start, 9);
        assert_eq!(spans[3].end, 13);
        assert_eq!(spans[3].style, italic_s);
    }
}
