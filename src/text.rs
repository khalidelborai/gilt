//! Rich text module - the core text manipulation type.
//!
//! This module provides the `Text` type which represents styled terminal text,
//! along with supporting types `Span`, `Lines`, and related enums.
//! Port of Python's rich/text.py.

use std::cmp::{min, Ordering};
use std::fmt;
use std::ops::{Add, Index, IndexMut};

use regex::Regex;

use crate::ansi::AnsiDecoder;
use crate::cells::{cell_len, set_cell_size};
use crate::errors::MarkupError;
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::Style;
use crate::wrap::divide_line;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JustifyMethod {
    Default,
    Left,
    Center,
    Right,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OverflowMethod {
    Fold,
    Crop,
    Ellipsis,
    Ignore,
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Strip control codes from text (Bell, Backspace, VT, FF, CR).
pub fn strip_control_codes(text: &str) -> String {
    text.chars()
        .filter(|c| !matches!(*c as u32, 7 | 8 | 11 | 12 | 13))
        .collect()
}

/// Convert a char index to a byte index within a string.
fn char_to_byte_index(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

/// Get a substring by char indices `[start..end)`.
fn char_slice(s: &str, start: usize, end: usize) -> &str {
    let byte_start = char_to_byte_index(s, start);
    let byte_end = char_to_byte_index(s, end);
    &s[byte_start..byte_end]
}

/// Compute GCD of two numbers (iterative).
fn gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

// ---------------------------------------------------------------------------
// Span
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub style: Style,
}

impl Span {
    pub fn new(start: usize, end: usize, style: Style) -> Self {
        Span { start, end, style }
    }

    pub fn is_empty(&self) -> bool {
        self.end <= self.start
    }

    /// Split span at `offset` (char index).
    /// If offset is outside the span, returns (self, None).
    /// Otherwise returns (left, Some(right)).
    pub fn split(&self, offset: usize) -> (Span, Option<Span>) {
        if offset < self.start || offset >= self.end {
            return (self.clone(), None);
        }
        let left = Span::new(self.start, offset, self.style.clone());
        let right = Span::new(offset, self.end, self.style.clone());
        (left, Some(right))
    }

    /// Shift span by `offset` positions.
    pub fn move_span(&self, offset: usize) -> Span {
        Span::new(
            self.start.saturating_add(offset),
            self.end.saturating_add(offset),
            self.style.clone(),
        )
    }

    /// Crop the end to `min(offset, self.end)`.
    pub fn right_crop(&self, offset: usize) -> Span {
        Span::new(self.start, min(offset, self.end), self.style.clone())
    }

    /// Extend end by `cells`.
    pub fn extend(&self, cells: usize) -> Span {
        Span::new(self.start, self.end + cells, self.style.clone())
    }
}

impl PartialOrd for Span {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Span {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.start, self.end).cmp(&(other.start, other.end))
    }
}

impl std::hash::Hash for Span {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.start.hash(state);
        self.end.hash(state);
        self.style.hash(state);
    }
}

// ---------------------------------------------------------------------------
// TextPart - for assemble()
// ---------------------------------------------------------------------------

pub enum TextPart {
    Raw(String),
    Styled(String, Style),
    Rich(Text),
}

// ---------------------------------------------------------------------------
// TextOrStr - for generic append
// ---------------------------------------------------------------------------

pub enum TextOrStr<'a> {
    Str(&'a str, Option<Style>),
    Text(&'a Text),
}

// ---------------------------------------------------------------------------
// Lines
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct Lines {
    pub lines: Vec<Text>,
}

impl Lines {
    pub fn new(lines: Vec<Text>) -> Self {
        Lines { lines }
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn push(&mut self, text: Text) {
        self.lines.push(text);
    }

    pub fn extend(&mut self, other: impl IntoIterator<Item = Text>) {
        self.lines.extend(other);
    }

    pub fn pop(&mut self) -> Option<Text> {
        self.lines.pop()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Text> {
        self.lines.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Text> {
        self.lines.iter_mut()
    }

    pub fn justify(
        &mut self,
        width: usize,
        justify: JustifyMethod,
        overflow: OverflowMethod,
    ) {
        match justify {
            JustifyMethod::Default | JustifyMethod::Left => {
                for line in &mut self.lines {
                    line.truncate(width, Some(overflow), true);
                }
            }
            JustifyMethod::Center => {
                for line in &mut self.lines {
                    line.rstrip();
                    line.truncate(width, Some(overflow), false);
                    let line_width = line.cell_len();
                    if line_width < width {
                        let left_pad = (width - line_width) / 2;
                        let right_pad = width - line_width - left_pad;
                        line.pad_left(left_pad, ' ');
                        line.pad_right(right_pad, ' ');
                    }
                }
            }
            JustifyMethod::Right => {
                for line in &mut self.lines {
                    line.rstrip();
                    line.truncate(width, Some(overflow), false);
                    let line_width = line.cell_len();
                    if line_width < width {
                        line.pad_left(width - line_width, ' ');
                    }
                }
            }
            JustifyMethod::Full => {
                let line_count = self.lines.len();
                for (i, line) in self.lines.iter_mut().enumerate() {
                    if i == line_count - 1 {
                        // Last line: left justify
                        line.truncate(width, Some(overflow), true);
                        continue;
                    }
                    let plain = line.plain().to_string();
                    let words: Vec<&str> = plain.split(' ').collect();
                    if words.len() <= 1 {
                        line.truncate(width, Some(overflow), true);
                        continue;
                    }
                    let text_width = line.cell_len();
                    if text_width >= width {
                        line.truncate(width, Some(overflow), false);
                        continue;
                    }
                    let extra_spaces = width - text_width;
                    let gaps = words.len() - 1;
                    let per_gap = extra_spaces / gaps;
                    let mut remainder = extra_spaces % gaps;

                    // Build new text with distributed spaces
                    // We need to find positions of spaces in the original text and expand them
                    let plain_chars: Vec<char> = plain.chars().collect();
                    let mut new_text = String::new();
                    let mut space_adjustments: Vec<(usize, usize)> = Vec::new(); // (char_pos, extra_spaces)

                    // Find space positions and compute extra spaces for each
                    let mut space_positions = Vec::new();
                    for (ci, ch) in plain_chars.iter().enumerate() {
                        if *ch == ' ' {
                            space_positions.push(ci);
                        }
                    }

                    // Distribute right-to-left
                    let mut extras_per_space = vec![per_gap; space_positions.len()];
                    for j in (0..space_positions.len()).rev() {
                        if remainder == 0 {
                            break;
                        }
                        extras_per_space[j] += 1;
                        remainder -= 1;
                    }

                    // Build new string and track shift amounts
                    let mut space_idx = 0;
                    for (ci, ch) in plain_chars.iter().enumerate() {
                        new_text.push(*ch);
                        if *ch == ' ' && space_idx < extras_per_space.len() {
                            let extra = extras_per_space[space_idx];
                            for _ in 0..extra {
                                new_text.push(' ');
                            }
                            space_adjustments.push((ci, extra));

                            space_idx += 1;
                        }
                    }

                    // Adjust spans
                    let mut new_spans = Vec::new();
                    for span in line.spans() {
                        let mut new_start = span.start;
                        let mut new_end = span.end;
                        let mut shift_start = 0usize;
                        let mut shift_end = 0usize;
                        for (sp_pos, extra) in &space_adjustments {
                            // For start: shift by cumulative extras of spaces before start
                            if *sp_pos < span.start {
                                shift_start += extra;
                            }
                            // For end: shift by cumulative extras of spaces before end
                            if *sp_pos < span.end {
                                shift_end += extra;
                            }
                        }
                        new_start += shift_start;
                        new_end += shift_end;
                        new_spans.push(Span::new(new_start, new_end, span.style.clone()));
                    }

                    line.set_plain(&new_text);
                    *line.spans_mut() = new_spans;
                }
            }
        }
    }
}

impl Index<usize> for Lines {
    type Output = Text;

    fn index(&self, index: usize) -> &Self::Output {
        &self.lines[index]
    }
}

impl IndexMut<usize> for Lines {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.lines[index]
    }
}

// ---------------------------------------------------------------------------
// Text
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Text {
    text: String,
    spans: Vec<Span>,
    style: Style,
    pub justify: Option<JustifyMethod>,
    pub overflow: Option<OverflowMethod>,
    pub no_wrap: Option<bool>,
    pub end: String,
    pub tab_size: Option<usize>,
}

impl Text {
    // -- Constructors -------------------------------------------------------

    pub fn new(text: &str, style: Style) -> Self {
        Text {
            text: strip_control_codes(text),
            spans: Vec::new(),
            style,
            justify: None,
            overflow: None,
            no_wrap: None,
            end: "\n".to_string(),
            tab_size: None,
        }
    }

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

    pub fn plain(&self) -> &str {
        &self.text
    }

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
        self.text = new_text;
    }

    pub fn spans(&self) -> &[Span] {
        &self.spans
    }

    pub fn spans_mut(&mut self) -> &mut Vec<Span> {
        &mut self.spans
    }

    pub fn len(&self) -> usize {
        self.text.chars().count()
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

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
        let max_text_width = text
            .lines()
            .map(cell_len)
            .max()
            .unwrap_or(0);
        let min_text_width = text
            .split_whitespace()
            .map(cell_len)
            .max()
            .unwrap_or(0);
        Measurement::new(min_text_width, max_text_width)
    }

    // -- Display & comparison -----------------------------------------------

    pub fn contains_str(&self, s: &str) -> bool {
        self.text.contains(s)
    }

    pub fn contains_text(&self, t: &Text) -> bool {
        self.text.contains(t.plain())
    }

    // -- Core manipulation --------------------------------------------------

    pub fn copy(&self) -> Text {
        self.clone()
    }

    pub fn blank_copy(&self, plain: &str) -> Text {
        Text {
            text: strip_control_codes(plain),
            spans: Vec::new(),
            style: self.style.clone(),
            justify: self.justify,
            overflow: self.overflow,
            no_wrap: self.no_wrap,
            end: self.end.clone(),
            tab_size: self.tab_size,
        }
    }

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

    pub fn append_text(&mut self, text: &Text) -> &mut Self {
        let offset = self.len();
        self.text.push_str(&text.text);
        for span in &text.spans {
            self.spans.push(span.move_span(offset));
        }
        self
    }

    pub fn append(&mut self, text: TextOrStr) -> &mut Self {
        match text {
            TextOrStr::Str(s, style) => self.append_str(s, style),
            TextOrStr::Text(t) => self.append_text(t),
        }
    }

    pub fn append_tokens(&mut self, tokens: &[(String, Option<Style>)]) -> &mut Self {
        for (token_text, style) in tokens {
            self.append_str(token_text, style.clone());
        }
        self
    }

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

    pub fn copy_styles(&mut self, other: &Text) {
        self.spans.extend(other.spans.iter().cloned());
    }

    // -- Splitting and dividing ---------------------------------------------

    pub fn split(
        &self,
        separator: &str,
        include_separator: bool,
        allow_blank: bool,
    ) -> Lines {
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

    pub fn truncate(
        &mut self,
        max_width: usize,
        overflow: Option<OverflowMethod>,
        pad: bool,
    ) {
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
                let new_text = set_cell_size(&self.text, max_width.saturating_sub(1));
                // Count chars of new_text for span adjustment
                self.set_plain(&new_text);
                self.append_str("\u{2026}", None); // ellipsis
            }
            OverflowMethod::Crop | OverflowMethod::Fold => {
                let new_text = set_cell_size(&self.text, max_width);
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

    pub fn pad(&mut self, count: usize, character: char) {
        self.pad_left(count, character);
        self.pad_right(count, character);
    }

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

    pub fn pad_right(&mut self, count: usize, character: char) {
        if count == 0 {
            return;
        }
        let padding: String = std::iter::repeat_n(character, count).collect();
        self.text.push_str(&padding);
    }

    pub fn rstrip(&mut self) {
        let trimmed = self.text.trim_end().to_string();
        if trimmed.len() != self.text.len() {
            self.set_plain(&trimmed);
        }
    }

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

    pub fn set_length(&mut self, new_length: usize) {
        let current_length = self.len();
        if new_length < current_length {
            let new_text = char_slice(&self.text, 0, new_length).to_string();
            self.set_plain(&new_text);
        } else if new_length > current_length {
            self.pad_right(new_length - current_length, ' ');
        }
    }

    pub fn remove_suffix(&mut self, suffix: &str) {
        if self.text.ends_with(suffix) {
            let suffix_chars = suffix.chars().count();
            let new_len = self.len() - suffix_chars;
            let new_text = char_slice(&self.text, 0, new_len).to_string();
            self.set_plain(&new_text);
        }
    }

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

    pub fn highlight_regex_with_groups(
        &mut self,
        pattern: &Regex,
        style_prefix: &str,
    ) -> usize {
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

    pub fn highlight_words(
        &mut self,
        words: &[&str],
        style: Style,
        case_sensitive: bool,
    ) -> usize {
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

    pub fn fit(&self, width: usize) -> Lines {
        let lines = self.split("\n", true, true);
        let mut result = Lines::default();
        for mut line in lines.lines {
            let new_text = set_cell_size(line.plain(), width);
            line.set_plain(&new_text);
            // Pad if needed
            if line.cell_len() < width {
                line.pad_right(width - line.cell_len(), ' ');
            }
            result.push(line);
        }
        result
    }

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
            events.push((span.end, true, i + 1));     // leaving
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn bold() -> Style {
        Style::parse("bold").unwrap()
    }

    fn italic() -> Style {
        Style::parse("italic").unwrap()
    }

    fn underline() -> Style {
        Style::parse("underline").unwrap()
    }

    fn red() -> Style {
        Style::parse("red").unwrap()
    }

    // -- Span tests ---------------------------------------------------------

    #[test]
    fn test_span() {
        let span = Span::new(0, 10, bold());
        assert!(!span.is_empty());
        let empty_span = Span::new(5, 5, bold());
        assert!(empty_span.is_empty());
        let empty_span2 = Span::new(10, 5, bold());
        assert!(empty_span2.is_empty());
    }

    #[test]
    fn test_span_split() {
        let span = Span::new(5, 10, bold());

        // Split in middle
        let (left, right) = span.split(7);
        assert_eq!(left, Span::new(5, 7, bold()));
        assert_eq!(right.unwrap(), Span::new(7, 10, bold()));

        // Split before start
        let (left, right) = span.split(3);
        assert_eq!(left, span);
        assert!(right.is_none());

        // Split at or after end
        let (left, right) = span.split(10);
        assert_eq!(left, span);
        assert!(right.is_none());

        // Split at start
        let (left, right) = span.split(5);
        assert_eq!(left, Span::new(5, 5, bold()));
        assert_eq!(right.unwrap(), Span::new(5, 10, bold()));
    }

    #[test]
    fn test_span_move() {
        let span = Span::new(5, 10, bold());
        let moved = span.move_span(3);
        assert_eq!(moved, Span::new(8, 13, bold()));
    }

    #[test]
    fn test_span_right_crop() {
        let span = Span::new(5, 10, bold());
        let cropped = span.right_crop(8);
        assert_eq!(cropped, Span::new(5, 8, bold()));
        let cropped2 = span.right_crop(15);
        assert_eq!(cropped2, Span::new(5, 10, bold()));
    }

    // -- Text constructor tests ---------------------------------------------

    #[test]
    fn test_len() {
        let text = Text::new("Hello", Style::null());
        assert_eq!(text.len(), 5);
        assert!(!text.is_empty());

        let empty = Text::new("", Style::null());
        assert_eq!(empty.len(), 0);
        assert!(empty.is_empty());
    }

    #[test]
    fn test_cell_len() {
        let text = Text::new("Hello", Style::null());
        assert_eq!(text.cell_len(), 5);

        // CJK
        let text = Text::new("わさび", Style::null());
        assert_eq!(text.cell_len(), 6);
    }

    #[test]
    fn test_bool() {
        let text = Text::new("Hello", Style::null());
        assert!(!text.is_empty());

        let empty = Text::empty();
        assert!(empty.is_empty());
    }

    #[test]
    fn test_str() {
        let text = Text::new("Hello, World!", Style::null());
        assert_eq!(format!("{}", text), "Hello, World!");
    }

    #[test]
    fn test_repr() {
        let text = Text::new("Hello", bold());
        assert_eq!(text.plain(), "Hello");
    }

    #[test]
    fn test_add() {
        let t1 = Text::new("Hello", bold());
        let t2 = Text::new(" World", italic());
        let combined = t1 + t2;
        assert_eq!(combined.plain(), "Hello World");
    }

    #[test]
    fn test_add_str() {
        let t1 = Text::new("Hello", Style::null());
        let combined = t1 + " World";
        assert_eq!(combined.plain(), "Hello World");
    }

    #[test]
    fn test_eq() {
        let t1 = Text::new("Hello", bold());
        let t2 = Text::new("Hello", bold());
        // Both have no spans, same text
        assert_eq!(t1, t2);

        let mut t3 = Text::new("Hello", Style::null());
        t3.stylize(bold(), 0, Some(5));
        let mut t4 = Text::new("Hello", Style::null());
        t4.stylize(bold(), 0, Some(5));
        assert_eq!(t3, t4);
    }

    #[test]
    fn test_contain() {
        let text = Text::new("Hello, World!", Style::null());
        assert!(text.contains_str("World"));
        assert!(!text.contains_str("Universe"));
    }

    // -- Plain property tests -----------------------------------------------

    #[test]
    fn test_plain_property() {
        let text = Text::new("Hello, World!", Style::null());
        assert_eq!(text.plain(), "Hello, World!");
    }

    #[test]
    fn test_plain_property_setter() {
        let mut text = Text::new("Hello, World!", Style::null());
        text.stylize(bold(), 0, Some(13));
        text.set_plain("Goodbye!");
        assert_eq!(text.plain(), "Goodbye!");
        // Span should be trimmed to new length
        assert_eq!(text.spans().len(), 1);
        assert_eq!(text.spans()[0].end, 8);
    }

    // -- Copy test ----------------------------------------------------------

    #[test]
    fn test_copy() {
        let mut original = Text::new("Hello", bold());
        original.stylize(italic(), 0, Some(3));
        let copy = original.copy();
        assert_eq!(copy.plain(), "Hello");
        assert_eq!(copy.spans().len(), 1);
    }

    // -- Strip tests --------------------------------------------------------

    #[test]
    fn test_rstrip() {
        let mut text = Text::new("Hello   ", Style::null());
        text.rstrip();
        assert_eq!(text.plain(), "Hello");
    }

    #[test]
    fn test_rstrip_end() {
        let mut text = Text::new("Hello   World   ", Style::null());
        text.rstrip_end(12);
        // Only strip whitespace beyond char position 12
        assert_eq!(text.plain(), "Hello   World");
    }

    // -- Stylize tests ------------------------------------------------------

    #[test]
    fn test_stylize() {
        let mut text = Text::new("Hello, World!", Style::null());
        text.stylize(bold(), 0, Some(5));
        assert_eq!(text.spans().len(), 1);
        assert_eq!(text.spans()[0], Span::new(0, 5, bold()));
    }

    #[test]
    fn test_stylize_before() {
        let mut text = Text::new("Hello, World!", Style::null());
        text.stylize(bold(), 0, Some(5));
        text.stylize_before(italic(), 0, Some(5));
        assert_eq!(text.spans().len(), 2);
        // italic should be first
        assert_eq!(text.spans()[0].style, italic());
        assert_eq!(text.spans()[1].style, bold());
    }

    // -- Highlight tests ----------------------------------------------------

    #[test]
    fn test_highlight_regex() {
        let mut text = Text::new("Hello, World!", Style::null());
        let re = Regex::new(r"World").unwrap();
        let count = text.highlight_regex(&re, bold());
        assert_eq!(count, 1);
        assert_eq!(text.spans().len(), 1);
        assert_eq!(text.spans()[0], Span::new(7, 12, bold()));
    }

    #[test]
    fn test_highlight_words() {
        let mut text = Text::new("The quick brown fox", Style::null());
        let count = text.highlight_words(&["quick", "fox"], bold(), false);
        assert_eq!(count, 2);
        assert_eq!(text.spans().len(), 2);
    }

    // -- Set length test ----------------------------------------------------

    #[test]
    fn test_set_length() {
        let mut text = Text::new("Hello", Style::null());
        text.set_length(10);
        assert_eq!(text.len(), 10);
        assert_eq!(text.plain(), "Hello     ");

        let mut text = Text::new("Hello, World!", Style::null());
        text.set_length(5);
        assert_eq!(text.plain(), "Hello");
    }

    // -- Join test ----------------------------------------------------------

    #[test]
    fn test_join() {
        let separator = Text::new(", ", Style::null());
        let texts = vec![
            Text::new("Hello", Style::null()),
            Text::new("World", Style::null()),
        ];
        let joined = separator.join(&texts);
        assert_eq!(joined.plain(), "Hello, World");
    }

    // -- Trim spans test ----------------------------------------------------

    #[test]
    fn test_trim_spans() {
        let mut text = Text::new("Hello", Style::null());
        text.spans.push(Span::new(0, 20, bold())); // Exceeds text length
        text.trim_spans();
        assert_eq!(text.spans()[0].end, 5);
    }

    // -- Pad tests ----------------------------------------------------------

    #[test]
    fn test_pad_left() {
        let mut text = Text::new("Hello", Style::null());
        text.stylize(bold(), 0, Some(5));
        text.pad_left(3, ' ');
        assert_eq!(text.plain(), "   Hello");
        // Span should be shifted
        assert_eq!(text.spans()[0], Span::new(3, 8, bold()));
    }

    #[test]
    fn test_pad_right() {
        let mut text = Text::new("Hello", Style::null());
        text.pad_right(3, ' ');
        assert_eq!(text.plain(), "Hello   ");
    }

    #[test]
    fn test_pad() {
        let mut text = Text::new("Hello", Style::null());
        text.pad(2, '-');
        assert_eq!(text.plain(), "--Hello--");
    }

    // -- Append tests -------------------------------------------------------

    #[test]
    fn test_append() {
        let mut text = Text::new("Hello", Style::null());
        text.append_str(", World!", None);
        assert_eq!(text.plain(), "Hello, World!");
    }

    #[test]
    fn test_append_text() {
        let mut text = Text::new("Hello", Style::null());
        let mut other = Text::new(" World", Style::null());
        other.stylize(bold(), 0, Some(6));
        text.append_text(&other);
        assert_eq!(text.plain(), "Hello World");
        assert_eq!(text.spans().len(), 1);
        assert_eq!(text.spans()[0], Span::new(5, 11, bold()));
    }

    // -- Split test ---------------------------------------------------------

    #[test]
    fn test_split() {
        let text = Text::new("Hello\nWorld\nFoo", Style::null());
        let lines = text.split("\n", false, false);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].plain(), "Hello");
        assert_eq!(lines[1].plain(), "World");
        assert_eq!(lines[2].plain(), "Foo");
    }

    // -- Divide test --------------------------------------------------------

    #[test]
    fn test_divide() {
        let mut text = Text::new("Hello World", Style::null());
        text.stylize(bold(), 0, Some(5));
        text.stylize(italic(), 6, Some(11));

        let divided = text.divide(&[6]);
        assert_eq!(divided.len(), 2);
        assert_eq!(divided[0].plain(), "Hello ");
        assert_eq!(divided[1].plain(), "World");

        // Check spans
        assert_eq!(divided[0].spans().len(), 1);
        assert_eq!(divided[0].spans()[0], Span::new(0, 5, bold()));
        assert_eq!(divided[1].spans().len(), 1);
        assert_eq!(divided[1].spans()[0], Span::new(0, 5, italic()));
    }

    #[test]
    fn test_divide_multi_span() {
        let mut text = Text::new("ABCDEFGHIJ", Style::null());
        // Span covers characters 2..8
        text.stylize(bold(), 2, Some(8));
        // Divide at 3 and 7
        let divided = text.divide(&[3, 7]);
        assert_eq!(divided.len(), 3);
        assert_eq!(divided[0].plain(), "ABC");
        assert_eq!(divided[1].plain(), "DEFG");
        assert_eq!(divided[2].plain(), "HIJ");

        // First line: span covers 2..3 (local)
        assert_eq!(divided[0].spans().len(), 1);
        assert_eq!(divided[0].spans()[0], Span::new(2, 3, bold()));

        // Second line: span covers 0..4 (local) -> full line
        assert_eq!(divided[1].spans().len(), 1);
        assert_eq!(divided[1].spans()[0], Span::new(0, 4, bold()));

        // Third line: span covers 0..1 (local)
        assert_eq!(divided[2].spans().len(), 1);
        assert_eq!(divided[2].spans()[0], Span::new(0, 1, bold()));
    }

    // -- Right crop test ----------------------------------------------------

    #[test]
    fn test_right_crop() {
        let mut text = Text::new("Hello, World!", Style::null());
        text.right_crop(7);
        assert_eq!(text.plain(), "Hello,");
    }

    // -- Truncate tests -----------------------------------------------------

    #[test]
    fn test_truncate_ellipsis() {
        let mut text = Text::new("Hello, World!", Style::null());
        text.truncate(10, Some(OverflowMethod::Ellipsis), false);
        assert_eq!(text.cell_len(), 10);
        assert!(text.plain().ends_with('\u{2026}'));
    }

    #[test]
    fn test_truncate_ellipsis_pad() {
        let mut text = Text::new("Hello", Style::null());
        text.truncate(10, Some(OverflowMethod::Ellipsis), true);
        // "Hello" is only 5 chars, should be padded to 10
        assert_eq!(text.cell_len(), 10);
        assert_eq!(text.plain(), "Hello     ");
    }

    // -- Fit test -----------------------------------------------------------

    #[test]
    fn test_fit() {
        let text = Text::new("Hello\nWorld", Style::null());
        let lines = text.fit(10);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].cell_len(), 10);
        assert_eq!(lines[1].cell_len(), 10);
    }

    // -- Tabs test ----------------------------------------------------------

    #[test]
    fn test_tabs_to_spaces() {
        let mut text = Text::new("Hello\tWorld", Style::null());
        text.expand_tabs(Some(4));
        assert_eq!(text.plain(), "Hello    World");
    }

    // -- Strip control codes test -------------------------------------------

    #[test]
    fn test_strip_control_codes() {
        let result = strip_control_codes("Hello\x07World\x08!\x0B\x0C\x0D");
        assert_eq!(result, "HelloWorld!");
    }

    // -- Align tests --------------------------------------------------------

    #[test]
    fn test_align_left() {
        let mut text = Text::new("Hello", Style::null());
        text.align(JustifyMethod::Left, 10, ' ');
        assert_eq!(text.plain(), "Hello     ");
    }

    #[test]
    fn test_align_right() {
        let mut text = Text::new("Hello", Style::null());
        text.align(JustifyMethod::Right, 10, ' ');
        assert_eq!(text.plain(), "     Hello");
    }

    #[test]
    fn test_align_center() {
        let mut text = Text::new("Hello", Style::null());
        text.align(JustifyMethod::Center, 11, ' ');
        assert_eq!(text.plain(), "   Hello   ");
    }

    // -- Detect indentation test --------------------------------------------

    #[test]
    fn test_detect_indentation() {
        let text = Text::new("    foo\n        bar\n    baz", Style::null());
        assert_eq!(text.detect_indentation(), 4);

        let text = Text::new("  foo\n    bar\n  baz", Style::null());
        assert_eq!(text.detect_indentation(), 2);
    }

    // -- Indent guides test -------------------------------------------------

    #[test]
    fn test_indentation_guides() {
        let text = Text::new("    foo\n        bar\n    baz", Style::null());
        let result = text.with_indent_guides(Some(4), '|', Style::null());
        let lines: Vec<&str> = result.plain().lines().collect();
        assert!(lines[0].starts_with('|'));
        assert!(lines[1].starts_with('|'));
    }

    // -- Slice test ---------------------------------------------------------

    #[test]
    fn test_slice() {
        let mut text = Text::new("Hello, World!", Style::null());
        text.stylize(bold(), 7, Some(12));
        let sliced = text.slice(7, 12);
        assert_eq!(sliced.plain(), "World");
        assert_eq!(sliced.spans().len(), 1);
        assert_eq!(sliced.spans()[0], Span::new(0, 5, bold()));
    }

    // -- Extend style test --------------------------------------------------

    #[test]
    fn test_extend_style() {
        let mut text = Text::new("Hello", Style::null());
        text.stylize(bold(), 0, Some(5));
        text.extend_style(3);
        assert_eq!(text.plain(), "Hello   ");
        assert_eq!(text.spans()[0].end, 8); // Extended by 3
    }

    // -- Append tokens test -------------------------------------------------

    #[test]
    fn test_append_tokens() {
        let mut text = Text::new("", Style::null());
        text.append_tokens(&[
            ("Hello".to_string(), Some(bold())),
            (" World".to_string(), None),
        ]);
        assert_eq!(text.plain(), "Hello World");
        assert_eq!(text.spans().len(), 1);
        assert_eq!(text.spans()[0], Span::new(0, 5, bold()));
    }

    // -- Assemble test ------------------------------------------------------

    #[test]
    fn test_assemble() {
        let text = Text::assemble(
            &[
                TextPart::Raw("Hello ".to_string()),
                TextPart::Styled("World".to_string(), bold()),
            ],
            Style::null(),
        );
        assert_eq!(text.plain(), "Hello World");
        assert_eq!(text.spans().len(), 1);
        assert_eq!(text.spans()[0], Span::new(6, 11, bold()));
    }

    // -- Styled test --------------------------------------------------------

    #[test]
    fn test_styled() {
        let text = Text::styled("Hello", bold());
        assert_eq!(text.plain(), "Hello");
        assert_eq!(text.spans().len(), 1);
        assert_eq!(text.spans()[0], Span::new(0, 5, bold()));
        // Base style should be null
        assert!(text.style.is_null());
    }

    // -- Render test --------------------------------------------------------

    #[test]
    fn test_render() {
        let mut text = Text::new("Hello World", Style::null());
        text.stylize(bold(), 0, Some(5));
        text.end = String::new(); // no end segment

        let segments = text.render();
        assert!(segments.len() >= 2);
        assert_eq!(segments[0].text, "Hello");
        assert!(segments[0].style.is_some());
        assert_eq!(segments[1].text, " World");
    }

    #[test]
    fn test_render_no_spans() {
        let mut text = Text::new("Hello", Style::null());
        text.end = String::new();
        let segments = text.render();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "Hello");
    }

    // -- Wrap tests ---------------------------------------------------------

    #[test]
    fn test_wrap_3() {
        let text = Text::new("foo bar baz", Style::null());
        let lines = text.wrap(7, None, None, 8, false);
        assert!(lines.len() >= 2);
        assert_eq!(lines[0].plain().trim(), "foo bar");
        assert_eq!(lines[1].plain().trim(), "baz");
    }

    #[test]
    fn test_wrap_4() {
        let text = Text::new("foo bar baz egg", Style::null());
        let lines = text.wrap(7, None, None, 8, false);
        assert!(lines.len() >= 2);
    }

    #[test]
    fn test_wrap_long() {
        let text = Text::new("abcdefghijklmnop", Style::null());
        let lines = text.wrap(4, None, None, 8, false);
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0].plain(), "abcd");
        assert_eq!(lines[1].plain(), "efgh");
        assert_eq!(lines[2].plain(), "ijkl");
        assert_eq!(lines[3].plain(), "mnop");
    }

    #[test]
    fn test_wrap_long_words() {
        let text = Text::new("longword short", Style::null());
        let lines = text.wrap(4, None, None, 8, false);
        assert!(lines.len() >= 3);
        assert_eq!(lines[0].plain(), "long");
        assert_eq!(lines[1].plain(), "word");
    }

    #[test]
    fn test_wrap_cjk() {
        // Each CJK char is 2 cells wide
        let text = Text::new("ああああ", Style::null());
        let lines = text.wrap(4, None, None, 8, false);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].plain(), "ああ");
        assert_eq!(lines[1].plain(), "ああ");
    }

    #[test]
    fn test_wrap_cjk_width_mid_character() {
        // Width 3 with CJK chars (each 2 wide) - can only fit 1 per line
        let text = Text::new("ああああ", Style::null());
        let lines = text.wrap(3, None, None, 8, false);
        assert_eq!(lines.len(), 4);
        for line in lines.iter() {
            assert_eq!(line.plain().chars().count(), 1);
        }
    }

    #[test]
    fn test_wrap_long_words_2() {
        let text = Text::new("abcdefghij klmnop", Style::null());
        let lines = text.wrap(4, None, None, 8, false);
        assert!(lines.len() >= 4);
    }

    #[test]
    fn test_wrap_long_words_followed_by_other_words() {
        let text = Text::new("abcdefgh foo bar", Style::null());
        let lines = text.wrap(4, None, None, 8, false);
        assert!(lines.len() >= 3);
    }

    #[test]
    fn test_wrap_leading_and_trailing_whitespace() {
        let text = Text::new("  Hello  ", Style::null());
        let lines = text.wrap(20, None, None, 8, false);
        assert_eq!(lines.len(), 1);
        // Trailing whitespace stripped by rstrip_end
    }

    #[test]
    fn test_append_loop_regression() {
        // Ensure appending in a loop doesn't corrupt spans
        let mut text = Text::new("", Style::null());
        for i in 0..10 {
            text.append_str(&format!("item{} ", i), Some(bold()));
        }
        assert_eq!(text.spans().len(), 10);
        // Verify spans don't overlap incorrectly
        for i in 0..text.spans().len() - 1 {
            assert!(text.spans()[i].end <= text.spans()[i + 1].start);
        }
    }

    #[test]
    fn test_remove_suffix() {
        let mut text = Text::new("Hello, World!", Style::null());
        text.remove_suffix("!");
        assert_eq!(text.plain(), "Hello, World");

        // No suffix match
        let mut text2 = Text::new("Hello", Style::null());
        text2.remove_suffix("xyz");
        assert_eq!(text2.plain(), "Hello");
    }

    // -- from_markup tests --------------------------------------------------

    #[test]
    fn test_from_markup_basic() {
        let text = Text::from_markup("[bold]Hello[/bold] world").unwrap();
        assert_eq!(text.plain(), "Hello world");
        assert_eq!(text.spans().len(), 1);
        assert_eq!(text.spans()[0], Span::new(0, 5, bold()));
    }

    #[test]
    fn test_from_markup_empty() {
        let text = Text::from_markup("").unwrap();
        assert_eq!(text.plain(), "");
        assert!(text.spans().is_empty());
    }

    #[test]
    fn test_from_markup_no_tags() {
        let text = Text::from_markup("plain text").unwrap();
        assert_eq!(text.plain(), "plain text");
        assert!(text.spans().is_empty());
    }

    #[test]
    fn test_from_markup_nested_styles() {
        let text =
            Text::from_markup("[bold][italic]nested[/italic][/bold]").unwrap();
        assert_eq!(text.plain(), "nested");
        assert_eq!(text.spans().len(), 2);
    }

    #[test]
    fn test_from_markup_error() {
        let result = Text::from_markup("[bold]hello[/italic]");
        assert!(result.is_err());
    }

    // -- from_ansi tests ----------------------------------------------------

    #[test]
    fn test_from_ansi_basic() {
        let text = Text::from_ansi("\x1b[1mBold\x1b[0m");
        assert_eq!(text.plain(), "Bold");
        assert_eq!(text.spans().len(), 1);
        assert_eq!(text.spans()[0].style.bold(), Some(true));
    }

    #[test]
    fn test_from_ansi_empty() {
        let text = Text::from_ansi("");
        assert_eq!(text.plain(), "");
        assert!(text.spans().is_empty());
    }

    #[test]
    fn test_from_ansi_plain_text() {
        let text = Text::from_ansi("no ansi here");
        assert_eq!(text.plain(), "no ansi here");
        assert!(text.spans().is_empty());
    }

    #[test]
    fn test_from_ansi_color_codes() {
        let text = Text::from_ansi("\x1b[31mRed\x1b[0m Normal");
        assert_eq!(text.plain(), "Red Normal");
        assert_eq!(text.spans().len(), 1);
        let color = text.spans()[0].style.color().unwrap();
        assert_eq!(color.number, Some(1));
    }

    #[test]
    fn test_from_ansi_multiple_styles() {
        let text = Text::from_ansi("\x1b[1mBold\x1b[0m \x1b[3mItalic\x1b[0m");
        assert_eq!(text.plain(), "Bold Italic");
        assert_eq!(text.spans().len(), 2);
        assert_eq!(text.spans()[0].style.bold(), Some(true));
        assert_eq!(text.spans()[1].style.italic(), Some(true));
    }

    // -- Introspection tests ------------------------------------------------

    #[test]
    fn test_get_style_at_offset_no_spans() {
        let style = Style::parse("bold").unwrap();
        let text = Text::new("hello", style.clone());
        let result = text.get_style_at_offset(2);
        assert_eq!(result.bold(), Some(true));
    }

    #[test]
    fn test_get_style_at_offset_single_span() {
        let mut text = Text::new("hello", Style::null());
        text.stylize(Style::parse("bold").unwrap(), 1, Some(4));
        // offset 2 is inside the span [1..4)
        let result = text.get_style_at_offset(2);
        assert_eq!(result.bold(), Some(true));
        // offset 0 is outside
        let result = text.get_style_at_offset(0);
        assert!(result.is_null());
    }

    #[test]
    fn test_get_style_at_offset_overlapping_spans() {
        let mut text = Text::new("hello world", Style::null());
        text.stylize(Style::parse("bold").unwrap(), 0, Some(8));
        text.stylize(Style::parse("italic").unwrap(), 3, Some(11));
        // offset 5 overlaps both spans
        let result = text.get_style_at_offset(5);
        assert_eq!(result.bold(), Some(true));
        assert_eq!(result.italic(), Some(true));
        // offset 1 only overlaps bold
        let result = text.get_style_at_offset(1);
        assert_eq!(result.bold(), Some(true));
        assert_eq!(result.italic(), None);
    }

    #[test]
    fn test_get_style_at_offset_out_of_range() {
        let mut text = Text::new("hi", Style::parse("bold").unwrap());
        text.stylize(Style::parse("italic").unwrap(), 0, Some(2));
        // offset 99 is beyond text length; only root style returned
        let result = text.get_style_at_offset(99);
        assert_eq!(result.bold(), Some(true));
        assert_eq!(result.italic(), None);
    }

    #[test]
    fn test_flatten_spans_no_overlaps() {
        let mut text = Text::new("hello world", Style::null());
        text.stylize(Style::parse("bold").unwrap(), 0, Some(5));
        text.stylize(Style::parse("italic").unwrap(), 6, Some(11));
        let flat = text.flatten_spans();
        assert_eq!(flat.len(), 2);
        assert_eq!(flat[0].start, 0);
        assert_eq!(flat[0].end, 5);
        assert_eq!(flat[0].style.bold(), Some(true));
        assert_eq!(flat[1].start, 6);
        assert_eq!(flat[1].end, 11);
        assert_eq!(flat[1].style.italic(), Some(true));
    }

    #[test]
    fn test_flatten_spans_overlapping() {
        let mut text = Text::new("hello world", Style::null());
        text.stylize(Style::parse("bold").unwrap(), 0, Some(8));
        text.stylize(Style::parse("italic").unwrap(), 3, Some(11));
        let flat = text.flatten_spans();
        // Expected regions: [0..3) bold, [3..8) bold+italic, [8..11) italic
        assert_eq!(flat.len(), 3);
        assert_eq!(flat[0].start, 0);
        assert_eq!(flat[0].end, 3);
        assert_eq!(flat[0].style.bold(), Some(true));
        assert_eq!(flat[0].style.italic(), None);
        assert_eq!(flat[1].start, 3);
        assert_eq!(flat[1].end, 8);
        assert_eq!(flat[1].style.bold(), Some(true));
        assert_eq!(flat[1].style.italic(), Some(true));
        assert_eq!(flat[2].start, 8);
        assert_eq!(flat[2].end, 11);
        assert_eq!(flat[2].style.italic(), Some(true));
        assert_eq!(flat[2].style.bold(), None);
    }

    #[test]
    fn test_flatten_spans_empty() {
        let text = Text::new("hello", Style::null());
        let flat = text.flatten_spans();
        assert!(flat.is_empty());
    }

    #[test]
    fn test_get_text_at_basic() {
        let text = Text::new("hello world", Style::null());
        assert_eq!(text.get_text_at(0, 5), Some("hello"));
        assert_eq!(text.get_text_at(6, 5), Some("world"));
        assert_eq!(text.get_text_at(0, 11), Some("hello world"));
    }

    #[test]
    fn test_get_text_at_unicode() {
        let text = Text::new("cafe\u{0301}s rock", Style::null());
        // "cafe\u{0301}" is 5 chars (c, a, f, e, combining-accent)
        // get_text_at works on char offsets
        assert_eq!(text.get_text_at(0, 5), Some("cafe\u{0301}"));
    }

    #[test]
    fn test_get_text_at_out_of_bounds() {
        let text = Text::new("hi", Style::null());
        assert_eq!(text.get_text_at(10, 5), None);
    }

    #[test]
    fn test_from_str_for_text() {
        let text = Text::from("hello");
        assert_eq!(text.plain(), "hello");
    }

    #[test]
    fn test_from_string_for_text() {
        let text = Text::from(String::from("hello"));
        assert_eq!(text.plain(), "hello");
    }

    #[test]
    fn test_into_text() {
        let text: Text = "hello".into();
        assert_eq!(text.plain(), "hello");
    }

    #[test]
    fn test_from_cow_str_for_text() {
        use std::borrow::Cow;
        let text = Text::from(Cow::Borrowed("hello"));
        assert_eq!(text.plain(), "hello");
        let text = Text::from(Cow::Owned(String::from("world")));
        assert_eq!(text.plain(), "world");
    }

}
