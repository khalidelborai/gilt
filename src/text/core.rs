//! Rich text module - the core text manipulation type.
//!
//! This module provides the `Text` type which represents styled terminal text,
//! along with supporting types `Span`, `Lines`, and related enums.
//! Port of Python's rich/text.py.

use std::cmp::min;
use std::fmt;
use std::ops::Add;

use regex::Regex;

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
    Rich(Text),
}

/// Either a string slice or a [`Text`] reference, for use with [`Text::append`].
pub enum TextOrStr<'a> {
    /// A borrowed string with an optional style.
    Str(&'a str, Option<Style>),
    /// A borrowed [`Text`] object.
    Text(&'a Text),
}

/// Rich text with styles, spans, and formatting metadata.
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
/// text.stylize(Style::parse("bold").unwrap(), 0, Some(5));
/// assert_eq!(text.plain(), "Hello, World!");
/// assert_eq!(text.spans()[0], Span::new(0, 5, Style::parse("bold").unwrap()));
/// # }
/// ```
#[derive(Clone, Debug)]
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
    /// let text = Text::new("Hello", Style::parse("bold").unwrap());
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
        }
    }

    /// Create an empty `Text` with a null style.
    pub fn empty() -> Self {
        Text::new("", Style::null())
    }

    /// Create Text with style applied as a span (not as base style).
    pub fn styled(text: &str, style: Style) -> Self {
        let mut t = Text::new(text, Style::null());
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
    ///         TextPart::Styled("World".into(), Style::parse("bold").unwrap()),
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
                TextPart::Rich(t) => {
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
    /// Delegates to [`AnsiDecoder::decode_line`].
    pub fn from_ansi(text: &str) -> Text {
        AnsiDecoder::new().decode_line(text)
    }

    // -- Properties ---------------------------------------------------------

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
        self.text.chars().count()
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
    /// This is the Rust equivalent of Python's `Text.__rich_measure__`.
    pub fn measure(&self) -> Measurement {
        let text = self.plain();
        if text.is_empty() {
            return Measurement::new(0, 0);
        }
        let max_text_width = text.lines().map(cell_len).max().unwrap_or(0);
        let min_text_width = text.split_whitespace().map(cell_len).max().unwrap_or(0);
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
    /// text.append_str(", World!", Some(Style::parse("italic").unwrap()));
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
        if let Some(s) = style {
            if !s.is_null() {
                self.spans.push(Span::new(offset, offset + new_len, s));
            }
        }
        self
    }

    /// Append another [`Text`] object, preserving its spans with adjusted offsets.
    pub fn append_text(&mut self, text: &Text) -> &mut Self {
        let offset = self.len();
        self.text.push_str(&text.text);
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
    /// text.stylize(Style::parse("bold red").unwrap(), 0, Some(5));
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
        let re = Regex::new(&regex::escape(separator)).unwrap();
        let plain = &self.text;

        if include_separator {
            let mut offsets = Vec::new();
            // Collect char-index positions of separator ends
            for mat in re.find_iter(plain) {
                let byte_end = mat.end();
                let char_end = plain[..byte_end].chars().count();
                offsets.push(char_end);
            }
            let lines = self.divide(&offsets);
            if !allow_blank {
                // Check if text ends with separator — if so, remove trailing empty
                if plain.ends_with(separator) {
                    let mut lines = lines;
                    if let Some(last) = lines.lines.last() {
                        if last.is_empty() {
                            lines.pop();
                        }
                    }
                    return lines;
                }
            }
            lines
        } else {
            // Split at separator boundaries but exclude separator text
            let mut offsets = Vec::new();
            for mat in re.find_iter(plain) {
                let byte_start = mat.start();
                let byte_end = mat.end();
                let char_start = plain[..byte_start].chars().count();
                let char_end = plain[..byte_end].chars().count();
                offsets.push(char_start);
                offsets.push(char_end);
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
            return;
        }
        let new_length = length - amount;
        let new_text = char_slice(&self.text, 0, new_length).to_string();
        self.text = new_text;
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
        // Shift all spans right by count
        for span in &mut self.spans {
            span.start += count;
            span.end += count;
        }
        self.text = format!("{}{}", padding, self.text);
    }

    /// Append `count` copies of `character` to the right side of the text.
    pub fn pad_right(&mut self, count: usize, character: char) {
        if count == 0 {
            return;
        }
        let padding: String = std::iter::repeat_n(character, count).collect();
        self.text.push_str(&padding);
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

    /// Pad the text to `width` terminal cells using the given alignment and fill character.
    ///
    /// If the text already meets or exceeds `width`, no padding is added.
    pub fn align(&mut self, align: JustifyMethod, width: usize, character: char) {
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
    /// let count = text.highlight_regex(&re, Style::parse("bold red").unwrap());
    /// assert_eq!(count, 1);
    /// # }
    /// ```
    pub fn highlight_regex(&mut self, pattern: &Regex, style: Style) -> usize {
        let plain = self.text.clone();
        let mut count = 0;
        for mat in pattern.find_iter(&plain) {
            let byte_start = mat.start();
            let byte_end = mat.end();
            let char_start = plain[..byte_start].chars().count();
            let char_end = plain[..byte_end].chars().count();
            self.stylize(style.clone(), char_start, Some(char_end));
            count += 1;
        }
        count
    }

    /// Highlight named capture groups from `pattern`, using `style_prefix` concatenated
    /// with each group name as the style string. Returns the total number of styled groups.
    pub fn highlight_regex_with_groups(&mut self, pattern: &Regex, style_prefix: &str) -> usize {
        let plain = self.text.clone();
        let mut count = 0;
        for captures in pattern.captures_iter(&plain) {
            for name in pattern.capture_names().flatten() {
                if let Some(mat) = captures.name(name) {
                    let style_str = format!("{}{}", style_prefix, name);
                    if let Ok(style) = Style::parse(&style_str) {
                        let byte_start = mat.start();
                        let byte_end = mat.end();
                        let char_start = plain[..byte_start].chars().count();
                        let char_end = plain[..byte_end].chars().count();
                        self.stylize(style, char_start, Some(char_end));
                        count += 1;
                    }
                }
            }
        }
        count
    }

    /// Apply `style` to every occurrence of each word (matched at word boundaries).
    ///
    /// Returns the total number of matches across all words.
    pub fn highlight_words(&mut self, words: &[&str], style: Style, case_sensitive: bool) -> usize {
        let mut count = 0;
        for word in words {
            let escaped = regex::escape(word);
            let pattern_str = if case_sensitive {
                format!(r"\b{}\b", escaped)
            } else {
                format!(r"(?i)\b{}\b", escaped)
            };
            if let Ok(re) = Regex::new(&pattern_str) {
                count += self.highlight_regex(&re, style.clone());
            }
        }
        count
    }

    // -- Tab expansion ------------------------------------------------------

    /// Replace tab characters with spaces, adjusting span positions accordingly.
    ///
    /// Uses the given `tab_size`, falling back to [`Text::tab_size`], then to 8.
    pub fn expand_tabs(&mut self, tab_size: Option<usize>) {
        let tab_size = tab_size.unwrap_or(self.tab_size.unwrap_or(8));
        if !self.text.contains('\t') {
            return;
        }

        let old_text = self.text.clone();
        let spaces: String = std::iter::repeat_n(' ', tab_size).collect();
        let new_text = old_text.replace('\t', &spaces);

        // Adjust spans: for each tab, chars shift by (tab_size - 1)
        let old_chars: Vec<char> = old_text.chars().collect();
        let mut char_offset_map: Vec<usize> = Vec::with_capacity(old_chars.len() + 1);
        let mut new_pos = 0usize;
        for &c in &old_chars {
            char_offset_map.push(new_pos);
            if c == '\t' {
                new_pos += tab_size;
            } else {
                new_pos += 1;
            }
        }
        char_offset_map.push(new_pos); // end sentinel

        let mut new_spans = Vec::new();
        for span in &self.spans {
            let new_start = if span.start < char_offset_map.len() {
                char_offset_map[span.start]
            } else {
                new_pos
            };
            let new_end = if span.end < char_offset_map.len() {
                char_offset_map[span.end]
            } else {
                new_pos
            };
            if new_start < new_end {
                new_spans.push(Span::new(new_start, new_end, span.style.clone()));
            }
        }

        self.text = new_text;
        self.spans = new_spans;
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
            let indent = line.len() - line.trim_start().len();
            if indent == 0 {
                continue;
            }
            // Only consider even indentation levels
            if indent % 2 != 0 {
                // Include odd indentation too, but prefer even
                indent_gcd = gcd(indent_gcd, indent);
            } else {
                indent_gcd = gcd(indent_gcd, indent);
            }
        }
        if indent_gcd == 0 {
            // Fallback: check if any lines are indented
            for line in self.text.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                let indent = line.len() - line.trim_start().len();
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

        // Style map: index 0 = base style, index 1..n = span styles
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
                    let combined = Style::combine(
                        &active_spans
                            .iter()
                            .map(|&id| style_map[id].clone())
                            .collect::<Vec<_>>(),
                    );
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

        // Emit remaining text
        if last_offset < text_len {
            let slice = char_slice(&self.text, last_offset, text_len);
            if !slice.is_empty() {
                let combined = Style::combine(
                    &active_spans
                        .iter()
                        .map(|&id| style_map[id].clone())
                        .collect::<Vec<_>>(),
                );
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
                // 3. Wrap the line
                let offsets = divide_line(line.plain(), width, true);
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
