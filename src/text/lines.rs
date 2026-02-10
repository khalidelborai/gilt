//! Lines type for collections of Text lines.

use std::ops::{Index, IndexMut};


use super::{JustifyMethod, OverflowMethod, Span, Text};

/// A collection of [`Text`] lines, typically produced by wrapping or splitting.
#[derive(Clone, Debug, Default)]
pub struct Lines {
    /// The individual text lines.
    pub lines: Vec<Text>,
}

impl Lines {
    /// Create a new `Lines` collection from a vector of [`Text`] objects.
    pub fn new(lines: Vec<Text>) -> Self {
        Lines { lines }
    }

    /// Return the number of lines.
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Return `true` if there are no lines.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Append a [`Text`] line to the end.
    pub fn push(&mut self, text: Text) {
        self.lines.push(text);
    }

    /// Extend the collection with lines from an iterator.
    pub fn extend(&mut self, other: impl IntoIterator<Item = Text>) {
        self.lines.extend(other);
    }

    /// Remove and return the last line, or `None` if empty.
    pub fn pop(&mut self) -> Option<Text> {
        self.lines.pop()
    }

    /// Return an iterator over the lines.
    pub fn iter(&self) -> impl Iterator<Item = &Text> {
        self.lines.iter()
    }

    /// Return a mutable iterator over the lines.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Text> {
        self.lines.iter_mut()
    }

    /// Justify every line according to the given method, truncating or padding to `width`.
    ///
    /// `Full` justification distributes extra space between words on all lines
    /// except the last, which is left-justified.
    pub fn justify(&mut self, width: usize, justify: JustifyMethod, overflow: OverflowMethod) {
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
