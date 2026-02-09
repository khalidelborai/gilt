//! Colored diff rendering widget.
//!
//! Provides a [`Diff`] widget that computes and renders line-level diffs
//! between two texts, supporting both unified and side-by-side display styles.
//!
//! The diff algorithm uses a simple LCS (Longest Common Subsequence) approach
//! with O(n*m) complexity, suitable for typical text sizes.
//!
//! # Example
//!
//! ```rust
//! use gilt::diff::{Diff, DiffStyle};
//! use gilt::console::Console;
//!
//! let old = "fn main() {\n    println!(\"hello\");\n}\n";
//! let new = "fn main() {\n    println!(\"world\");\n    return;\n}\n";
//!
//! // Unified diff
//! let diff = Diff::new(old, new)
//!     .with_labels("a/main.rs", "b/main.rs");
//!
//! // Side-by-side diff
//! let diff = Diff::side_by_side(old, new);
//! ```

use crate::cells::cell_len;
use crate::console::{Console, ConsoleOptions, Renderable};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::Style;

// ---------------------------------------------------------------------------
// DiffOp
// ---------------------------------------------------------------------------

/// A single operation in a line-level diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffOp {
    /// A line that is identical in both texts.
    Equal(String),
    /// A line that was inserted (present only in the new text).
    Insert(String),
    /// A line that was deleted (present only in the old text).
    Delete(String),
}

// ---------------------------------------------------------------------------
// LCS-based diff algorithm
// ---------------------------------------------------------------------------

/// Compute the LCS table for two slices of lines.
///
/// Returns a 2D table where `table[i][j]` is the length of the longest
/// common subsequence of `old[..i]` and `new[..j]`.
fn lcs_table(old: &[&str], new: &[&str]) -> Vec<Vec<usize>> {
    let n = old.len();
    let m = new.len();
    let mut table = vec![vec![0usize; m + 1]; n + 1];

    for i in 1..=n {
        for j in 1..=m {
            if old[i - 1] == new[j - 1] {
                table[i][j] = table[i - 1][j - 1] + 1;
            } else {
                table[i][j] = table[i - 1][j].max(table[i][j - 1]);
            }
        }
    }

    table
}

/// Compute a line-level diff between two texts using LCS backtracking.
///
/// Returns a sequence of [`DiffOp`] values describing the transformation
/// from `old_lines` to `new_lines`.
pub fn compute_diff(old_lines: &[&str], new_lines: &[&str]) -> Vec<DiffOp> {
    let table = lcs_table(old_lines, new_lines);
    let mut ops = Vec::new();

    let mut i = old_lines.len();
    let mut j = new_lines.len();

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && old_lines[i - 1] == new_lines[j - 1] {
            ops.push(DiffOp::Equal(old_lines[i - 1].to_string()));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || table[i][j - 1] >= table[i - 1][j]) {
            ops.push(DiffOp::Insert(new_lines[j - 1].to_string()));
            j -= 1;
        } else {
            ops.push(DiffOp::Delete(old_lines[i - 1].to_string()));
            i -= 1;
        }
    }

    ops.reverse();
    ops
}

// ---------------------------------------------------------------------------
// DiffStyle
// ---------------------------------------------------------------------------

/// Display style for the diff output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffStyle {
    /// Git-style unified diff with `+`/`-` markers.
    Unified,
    /// Two-column side-by-side comparison.
    SideBySide,
}

// ---------------------------------------------------------------------------
// Hunk
// ---------------------------------------------------------------------------

/// A contiguous region of changes with surrounding context lines.
#[derive(Debug, Clone)]
struct Hunk {
    /// Starting line number in the old text (1-based).
    old_start: usize,
    /// Number of old lines in the hunk.
    old_count: usize,
    /// Starting line number in the new text (1-based).
    new_start: usize,
    /// Number of new lines in the hunk.
    new_count: usize,
    /// The diff ops included in this hunk.
    ops: Vec<DiffOp>,
}

/// Group diff ops into hunks with the given number of context lines.
fn build_hunks(ops: &[DiffOp], context_lines: usize) -> Vec<Hunk> {
    if ops.is_empty() {
        return Vec::new();
    }

    // Find indices of non-equal ops (changes)
    let change_indices: Vec<usize> = ops
        .iter()
        .enumerate()
        .filter(|(_, op)| !matches!(op, DiffOp::Equal(_)))
        .map(|(i, _)| i)
        .collect();

    if change_indices.is_empty() {
        return Vec::new();
    }

    // Build groups of change indices that are close enough to share context
    let mut groups: Vec<Vec<usize>> = Vec::new();
    let mut current_group: Vec<usize> = vec![change_indices[0]];

    for &idx in &change_indices[1..] {
        let prev = *current_group.last().expect("group is non-empty after push");
        // If the gap between changes is small enough, merge into same hunk
        if idx - prev <= context_lines * 2 + 1 {
            current_group.push(idx);
        } else {
            groups.push(current_group);
            current_group = vec![idx];
        }
    }
    groups.push(current_group);

    // Convert groups into hunks
    let mut hunks = Vec::new();
    for group in &groups {
        let first_change = group[0];
        let last_change = *group.last().expect("group is non-empty after push");

        let start = first_change.saturating_sub(context_lines);
        let end = (last_change + context_lines + 1).min(ops.len());

        let hunk_ops: Vec<DiffOp> = ops[start..end].to_vec();

        // Compute old/new line numbers
        let mut old_line = 1usize;
        let mut new_line = 1usize;
        for op in &ops[..start] {
            match op {
                DiffOp::Equal(_) => {
                    old_line += 1;
                    new_line += 1;
                }
                DiffOp::Delete(_) => old_line += 1,
                DiffOp::Insert(_) => new_line += 1,
            }
        }

        let old_start = old_line;
        let new_start = new_line;

        let mut old_count = 0;
        let mut new_count = 0;
        for op in &hunk_ops {
            match op {
                DiffOp::Equal(_) => {
                    old_count += 1;
                    new_count += 1;
                }
                DiffOp::Delete(_) => old_count += 1,
                DiffOp::Insert(_) => new_count += 1,
            }
        }

        hunks.push(Hunk {
            old_start,
            old_count,
            new_start,
            new_count,
            ops: hunk_ops,
        });
    }

    hunks
}

// ---------------------------------------------------------------------------
// Diff
// ---------------------------------------------------------------------------

/// A widget that computes and renders a colored diff between two texts.
///
/// Supports both unified (git-style) and side-by-side display.
///
/// # Example
///
/// ```rust
/// use gilt::diff::{Diff, DiffStyle};
///
/// let diff = Diff::new("old text\n", "new text\n")
///     .with_labels("a/file.rs", "b/file.rs")
///     .with_context(3);
/// ```
#[derive(Debug, Clone)]
pub struct Diff {
    /// The original (old) text.
    old_text: String,
    /// The modified (new) text.
    new_text: String,
    /// Label for the old text (e.g., "a/file.rs").
    old_label: String,
    /// Label for the new text (e.g., "b/file.rs").
    new_label: String,
    /// Display style (unified or side-by-side).
    style: DiffStyle,
    /// Number of unchanged context lines around each change.
    context_lines: usize,
}

impl Diff {
    /// Create a new `Diff` with default settings (unified, 3 context lines).
    pub fn new(old_text: &str, new_text: &str) -> Self {
        Diff {
            old_text: old_text.to_string(),
            new_text: new_text.to_string(),
            old_label: "old".to_string(),
            new_label: "new".to_string(),
            style: DiffStyle::Unified,
            context_lines: 3,
        }
    }

    /// Set the labels for old and new texts.
    #[must_use]
    pub fn with_labels(mut self, old: &str, new: &str) -> Self {
        self.old_label = old.to_string();
        self.new_label = new.to_string();
        self
    }

    /// Set the display style.
    #[must_use]
    pub fn with_style(mut self, style: DiffStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the number of context lines around changes.
    #[must_use]
    pub fn with_context(mut self, lines: usize) -> Self {
        self.context_lines = lines;
        self
    }

    /// Create a side-by-side diff with default settings.
    pub fn side_by_side(old_text: &str, new_text: &str) -> Self {
        Diff::new(old_text, new_text).with_style(DiffStyle::SideBySide)
    }

    /// Create a unified diff with default settings.
    pub fn unified(old_text: &str, new_text: &str) -> Self {
        Diff::new(old_text, new_text).with_style(DiffStyle::Unified)
    }

    /// Split text into lines, preserving trailing empty lines for diffing.
    fn split_lines(text: &str) -> Vec<&str> {
        if text.is_empty() {
            return Vec::new();
        }
        let lines: Vec<&str> = text.lines().collect();
        // If the text ends with a newline, lines() does not produce a trailing
        // empty string. We keep the output as-is so that the diff result
        // reflects actual content lines.
        lines
    }

    /// Compute the diff operations.
    pub fn ops(&self) -> Vec<DiffOp> {
        let old_lines = Self::split_lines(&self.old_text);
        let new_lines = Self::split_lines(&self.new_text);
        compute_diff(&old_lines, &new_lines)
    }

    // -- Unified rendering --------------------------------------------------

    /// Render the diff in unified format, returning segments.
    fn render_unified(&self, max_width: usize) -> Vec<Segment> {
        let ops = self.ops();
        let hunks = build_hunks(&ops, self.context_lines);

        let delete_style = Style::parse("red").unwrap_or_else(|_| Style::null());
        let insert_style = Style::parse("green").unwrap_or_else(|_| Style::null());
        let header_del_style = Style::parse("bold red").unwrap_or_else(|_| Style::null());
        let header_ins_style = Style::parse("bold green").unwrap_or_else(|_| Style::null());
        let hunk_style = Style::parse("cyan").unwrap_or_else(|_| Style::null());
        let context_style = Style::parse("dim").unwrap_or_else(|_| Style::null());

        let mut segments = Vec::new();

        // If texts are identical, nothing to render
        if hunks.is_empty() {
            return segments;
        }

        // File headers
        let old_header = format!("--- {}", self.old_label);
        let new_header = format!("+++ {}", self.new_label);

        segments.push(Segment::styled(
            &truncate_to_width(&old_header, max_width),
            header_del_style.clone(),
        ));
        segments.push(Segment::line());
        segments.push(Segment::styled(
            &truncate_to_width(&new_header, max_width),
            header_ins_style.clone(),
        ));
        segments.push(Segment::line());

        for hunk in &hunks {
            // Hunk header
            let hunk_header = format!(
                "@@ -{},{} +{},{} @@",
                hunk.old_start, hunk.old_count, hunk.new_start, hunk.new_count
            );
            segments.push(Segment::styled(
                &truncate_to_width(&hunk_header, max_width),
                hunk_style.clone(),
            ));
            segments.push(Segment::line());

            // Hunk body
            for op in &hunk.ops {
                match op {
                    DiffOp::Equal(line) => {
                        let display = format!(" {}", line);
                        segments.push(Segment::styled(
                            &truncate_to_width(&display, max_width),
                            context_style.clone(),
                        ));
                        segments.push(Segment::line());
                    }
                    DiffOp::Delete(line) => {
                        let display = format!("-{}", line);
                        segments.push(Segment::styled(
                            &truncate_to_width(&display, max_width),
                            delete_style.clone(),
                        ));
                        segments.push(Segment::line());
                    }
                    DiffOp::Insert(line) => {
                        let display = format!("+{}", line);
                        segments.push(Segment::styled(
                            &truncate_to_width(&display, max_width),
                            insert_style.clone(),
                        ));
                        segments.push(Segment::line());
                    }
                }
            }
        }

        segments
    }

    // -- Side-by-side rendering ---------------------------------------------

    /// Render the diff in side-by-side format, returning segments.
    fn render_side_by_side(&self, max_width: usize) -> Vec<Segment> {
        let ops = self.ops();

        let delete_style = Style::parse("red").unwrap_or_else(|_| Style::null());
        let insert_style = Style::parse("green").unwrap_or_else(|_| Style::null());
        let context_style = Style::parse("dim").unwrap_or_else(|_| Style::null());
        let border_style = Style::parse("dim").unwrap_or_else(|_| Style::null());
        let header_style = Style::parse("bold").unwrap_or_else(|_| Style::null());

        let mut segments = Vec::new();

        // Calculate column widths:
        // Layout: "| " + left_num + " | " + left_text + " | " + right_num + " | " + right_text + " |"
        // Minimal: 2 borders + separators overhead
        // We use a simpler layout with line numbers.

        // Count old/new lines for number width
        let old_lines = Self::split_lines(&self.old_text);
        let new_lines = Self::split_lines(&self.new_text);
        let old_num_width = if old_lines.is_empty() {
            1
        } else {
            old_lines.len().to_string().len()
        };
        let new_num_width = if new_lines.is_empty() {
            1
        } else {
            new_lines.len().to_string().len()
        };

        // Layout: "{old_num} | {old_text} | {new_num} | {new_text}"
        // Overhead: " | " * 3 = 9 chars, plus num widths
        let overhead = old_num_width + 3 + 3 + new_num_width + 3;
        let available = max_width.saturating_sub(overhead);
        let half_width = available / 2;
        let left_width = half_width;
        let right_width = available.saturating_sub(left_width);

        // Headers
        let left_header = pad_or_truncate(&self.old_label, left_width);
        let right_header = pad_or_truncate(&self.new_label, right_width);
        let left_num_pad = " ".repeat(old_num_width);
        let right_num_pad = " ".repeat(new_num_width);

        segments.push(Segment::styled(
            &format!("{} | ", left_num_pad),
            border_style.clone(),
        ));
        segments.push(Segment::styled(&left_header, header_style.clone()));
        segments.push(Segment::styled(" | ", border_style.clone()));
        segments.push(Segment::styled(
            &format!("{} | ", right_num_pad),
            border_style.clone(),
        ));
        segments.push(Segment::styled(&right_header, header_style.clone()));
        segments.push(Segment::line());

        // Separator
        let sep_total = max_width;
        let sep_line: String = "\u{2500}".repeat(sep_total);
        segments.push(Segment::styled(
            &truncate_to_width(&sep_line, max_width),
            border_style.clone(),
        ));
        segments.push(Segment::line());

        // Render rows
        let mut old_idx = 0usize;
        let mut new_idx = 0usize;

        for op in &ops {
            match op {
                DiffOp::Equal(line) => {
                    old_idx += 1;
                    new_idx += 1;
                    let left_num = format!("{:>width$}", old_idx, width = old_num_width);
                    let right_num = format!("{:>width$}", new_idx, width = new_num_width);
                    let left_text = pad_or_truncate(line, left_width);
                    let right_text = pad_or_truncate(line, right_width);

                    segments.push(Segment::styled(
                        &format!("{} | ", left_num),
                        border_style.clone(),
                    ));
                    segments.push(Segment::styled(&left_text, context_style.clone()));
                    segments.push(Segment::styled(" | ", border_style.clone()));
                    segments.push(Segment::styled(
                        &format!("{} | ", right_num),
                        border_style.clone(),
                    ));
                    segments.push(Segment::styled(&right_text, context_style.clone()));
                    segments.push(Segment::line());
                }
                DiffOp::Delete(line) => {
                    old_idx += 1;
                    let left_num = format!("{:>width$}", old_idx, width = old_num_width);
                    let right_num = " ".repeat(new_num_width);
                    let left_text = pad_or_truncate(line, left_width);
                    let right_text = " ".repeat(right_width);

                    segments.push(Segment::styled(
                        &format!("{} | ", left_num),
                        border_style.clone(),
                    ));
                    segments.push(Segment::styled(&left_text, delete_style.clone()));
                    segments.push(Segment::styled(" | ", border_style.clone()));
                    segments.push(Segment::styled(
                        &format!("{} | ", right_num),
                        border_style.clone(),
                    ));
                    segments.push(Segment::styled(&right_text, context_style.clone()));
                    segments.push(Segment::line());
                }
                DiffOp::Insert(line) => {
                    new_idx += 1;
                    let left_num = " ".repeat(old_num_width);
                    let right_num = format!("{:>width$}", new_idx, width = new_num_width);
                    let left_text = " ".repeat(left_width);
                    let right_text = pad_or_truncate(line, right_width);

                    segments.push(Segment::styled(
                        &format!("{} | ", left_num),
                        border_style.clone(),
                    ));
                    segments.push(Segment::styled(&left_text, context_style.clone()));
                    segments.push(Segment::styled(" | ", border_style.clone()));
                    segments.push(Segment::styled(
                        &format!("{} | ", right_num),
                        border_style.clone(),
                    ));
                    segments.push(Segment::styled(&right_text, insert_style.clone()));
                    segments.push(Segment::line());
                }
            }
        }

        segments
    }

    /// Compute the maximum line width across both texts.
    fn max_line_width(&self) -> usize {
        let old_max = self.old_text.lines().map(cell_len).max().unwrap_or(0);
        let new_max = self.new_text.lines().map(cell_len).max().unwrap_or(0);
        old_max.max(new_max)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Truncate a string to fit within a given cell width.
fn truncate_to_width(s: &str, max_width: usize) -> String {
    let len = cell_len(s);
    if len <= max_width {
        s.to_string()
    } else {
        // Simple byte-level truncation with respect to char boundaries
        let mut width = 0;
        let mut end = 0;
        for (i, ch) in s.char_indices() {
            let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
            if width + cw > max_width {
                break;
            }
            width += cw;
            end = i + ch.len_utf8();
        }
        s[..end].to_string()
    }
}

/// Pad or truncate a string to exactly `width` cells.
fn pad_or_truncate(s: &str, width: usize) -> String {
    let len = cell_len(s);
    if len == width {
        s.to_string()
    } else if len < width {
        format!("{}{}", s, " ".repeat(width - len))
    } else {
        truncate_to_width(s, width)
    }
}

// ---------------------------------------------------------------------------
// Renderable
// ---------------------------------------------------------------------------

impl Renderable for Diff {
    fn rich_console(&self, _console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let max_width = options.max_width;
        match self.style {
            DiffStyle::Unified => self.render_unified(max_width),
            DiffStyle::SideBySide => self.render_side_by_side(max_width),
        }
    }
}

// ---------------------------------------------------------------------------
// Measure
// ---------------------------------------------------------------------------

impl Diff {
    /// Measure the minimum and maximum widths needed to render this diff.
    pub fn measure(&self, _console: &Console, _options: &ConsoleOptions) -> Measurement {
        let content_width = self.max_line_width();
        match self.style {
            DiffStyle::Unified => {
                // "+line" or "-line" or " line" adds 1 char prefix
                let min = 20; // reasonable minimum for unified diff
                let max = (content_width + 4).max(min); // prefix + some margin
                Measurement::new(min, max)
            }
            DiffStyle::SideBySide => {
                // Two columns + line numbers + separators
                let min = 40;
                let max = ((content_width * 2) + 20).max(min);
                Measurement::new(min, max)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl std::fmt::Display for Diff {
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

    fn make_console() -> Console {
        Console::builder()
            .width(80)
            .force_terminal(true)
            .no_color(true)
            .build()
    }

    // -- LCS / compute_diff tests -------------------------------------------

    #[test]
    fn test_identical_texts_no_changes() {
        let old = "line1\nline2\nline3";
        let new = "line1\nline2\nline3";
        let ops = compute_diff(
            &old.lines().collect::<Vec<_>>(),
            &new.lines().collect::<Vec<_>>(),
        );
        assert!(ops.iter().all(|op| matches!(op, DiffOp::Equal(_))));
        assert_eq!(ops.len(), 3);
    }

    #[test]
    fn test_completely_different_texts() {
        let old = vec!["a", "b"];
        let new = vec!["c", "d"];
        let ops = compute_diff(&old, &new);
        let deletes: Vec<_> = ops
            .iter()
            .filter(|op| matches!(op, DiffOp::Delete(_)))
            .collect();
        let inserts: Vec<_> = ops
            .iter()
            .filter(|op| matches!(op, DiffOp::Insert(_)))
            .collect();
        assert_eq!(deletes.len(), 2);
        assert_eq!(inserts.len(), 2);
    }

    #[test]
    fn test_single_line_added() {
        let old = vec!["a", "c"];
        let new = vec!["a", "b", "c"];
        let ops = compute_diff(&old, &new);
        assert_eq!(
            ops,
            vec![
                DiffOp::Equal("a".to_string()),
                DiffOp::Insert("b".to_string()),
                DiffOp::Equal("c".to_string()),
            ]
        );
    }

    #[test]
    fn test_single_line_removed() {
        let old = vec!["a", "b", "c"];
        let new = vec!["a", "c"];
        let ops = compute_diff(&old, &new);
        assert_eq!(
            ops,
            vec![
                DiffOp::Equal("a".to_string()),
                DiffOp::Delete("b".to_string()),
                DiffOp::Equal("c".to_string()),
            ]
        );
    }

    #[test]
    fn test_single_line_changed() {
        let old = vec!["a", "b", "c"];
        let new = vec!["a", "x", "c"];
        let ops = compute_diff(&old, &new);
        assert_eq!(
            ops,
            vec![
                DiffOp::Equal("a".to_string()),
                DiffOp::Delete("b".to_string()),
                DiffOp::Insert("x".to_string()),
                DiffOp::Equal("c".to_string()),
            ]
        );
    }

    #[test]
    fn test_multiple_changes_with_context() {
        let old = vec!["a", "b", "c", "d", "e"];
        let new = vec!["a", "x", "c", "d", "y"];
        let ops = compute_diff(&old, &new);
        let expected = vec![
            DiffOp::Equal("a".to_string()),
            DiffOp::Delete("b".to_string()),
            DiffOp::Insert("x".to_string()),
            DiffOp::Equal("c".to_string()),
            DiffOp::Equal("d".to_string()),
            DiffOp::Delete("e".to_string()),
            DiffOp::Insert("y".to_string()),
        ];
        assert_eq!(ops, expected);
    }

    #[test]
    fn test_empty_old_text_all_inserts() {
        let old: Vec<&str> = vec![];
        let new = vec!["a", "b"];
        let ops = compute_diff(&old, &new);
        assert_eq!(
            ops,
            vec![
                DiffOp::Insert("a".to_string()),
                DiffOp::Insert("b".to_string()),
            ]
        );
    }

    #[test]
    fn test_empty_new_text_all_deletes() {
        let old = vec!["a", "b"];
        let new: Vec<&str> = vec![];
        let ops = compute_diff(&old, &new);
        assert_eq!(
            ops,
            vec![
                DiffOp::Delete("a".to_string()),
                DiffOp::Delete("b".to_string()),
            ]
        );
    }

    #[test]
    fn test_both_empty() {
        let old: Vec<&str> = vec![];
        let new: Vec<&str> = vec![];
        let ops = compute_diff(&old, &new);
        assert!(ops.is_empty());
    }

    #[test]
    fn test_diffop_equality() {
        assert_eq!(
            DiffOp::Equal("test".to_string()),
            DiffOp::Equal("test".to_string())
        );
        assert_ne!(
            DiffOp::Equal("test".to_string()),
            DiffOp::Insert("test".to_string())
        );
        assert_ne!(
            DiffOp::Delete("a".to_string()),
            DiffOp::Delete("b".to_string())
        );
    }

    // -- Diff widget tests --------------------------------------------------

    #[test]
    fn test_unified_output_format() {
        let old = "hello\nworld\n";
        let new = "hello\nearth\n";
        let diff = Diff::new(old, new).with_labels("a/test.txt", "b/test.txt");

        let output = format!("{}", diff);

        assert!(output.contains("--- a/test.txt"));
        assert!(output.contains("+++ b/test.txt"));
        assert!(output.contains("@@"));
        assert!(output.contains("-world"));
        assert!(output.contains("+earth"));
    }

    #[test]
    fn test_side_by_side_output_format() {
        let old = "hello\nworld\n";
        let new = "hello\nearth\n";
        let diff = Diff::side_by_side(old, new).with_labels("old.txt", "new.txt");

        let output = format!("{}", diff);

        assert!(output.contains("old.txt"));
        assert!(output.contains("new.txt"));
        // Should contain separator lines
        assert!(output.contains("\u{2500}"));
    }

    #[test]
    fn test_context_lines_parameter() {
        // Build a large enough text so context matters
        let mut old_lines = Vec::new();
        let mut new_lines = Vec::new();
        for i in 0..20 {
            old_lines.push(format!("line {}", i));
            if i == 10 {
                new_lines.push("changed line 10".to_string());
            } else {
                new_lines.push(format!("line {}", i));
            }
        }
        let old = old_lines.join("\n");
        let new = new_lines.join("\n");

        let diff_1 = Diff::new(&old, &new).with_context(1);
        let diff_5 = Diff::new(&old, &new).with_context(5);

        let out_1 = format!("{}", diff_1);
        let out_5 = format!("{}", diff_5);

        // More context should produce more output lines
        assert!(out_5.len() > out_1.len());
    }

    #[test]
    fn test_labels() {
        let diff = Diff::new("a\n", "b\n").with_labels("src/old.rs", "src/new.rs");

        let output = format!("{}", diff);
        assert!(output.contains("src/old.rs"));
        assert!(output.contains("src/new.rs"));
    }

    #[test]
    fn test_large_text_100_lines() {
        let old: Vec<String> = (0..100).map(|i| format!("line {}", i)).collect();
        let mut new = old.clone();
        new[50] = "CHANGED LINE 50".to_string();
        new.insert(75, "INSERTED LINE".to_string());

        let old_text = old.join("\n");
        let new_text = new.join("\n");

        let diff = Diff::new(&old_text, &new_text);
        let ops = diff.ops();

        // Should have changes
        let has_changes = ops.iter().any(|op| !matches!(op, DiffOp::Equal(_)));
        assert!(has_changes);

        // Should render without panicking
        let output = format!("{}", diff);
        assert!(!output.is_empty());
    }

    #[test]
    fn test_trailing_newline_handling() {
        let old = "a\nb\n";
        let new = "a\nb";
        let diff = Diff::new(old, new);
        let ops = diff.ops();

        // Both have lines "a" and "b" — lines() strips trailing newlines
        // so the content is the same
        assert!(ops.iter().all(|op| matches!(op, DiffOp::Equal(_))));
    }

    #[test]
    fn test_whitespace_only_changes() {
        let old = "hello\n  world\n";
        let new = "hello\n    world\n";
        let diff = Diff::new(old, new);
        let ops = diff.ops();

        // "  world" vs "    world" are different strings
        let has_changes = ops.iter().any(|op| !matches!(op, DiffOp::Equal(_)));
        assert!(has_changes);
    }

    #[test]
    fn test_renderable_unified() {
        let diff = Diff::new("old line\n", "new line\n");
        let console = make_console();
        let options = console.options();
        let segments = diff.rich_console(&console, &options);

        // Should produce segments
        assert!(!segments.is_empty());

        // Should have styled segments (headers, diffs)
        let styled_count = segments.iter().filter(|s| s.style.is_some()).count();
        assert!(styled_count > 0);
    }

    #[test]
    fn test_renderable_side_by_side() {
        let diff = Diff::side_by_side("old\n", "new\n");
        let console = make_console();
        let options = console.options();
        let segments = diff.rich_console(&console, &options);

        assert!(!segments.is_empty());
    }

    #[test]
    fn test_measure_unified() {
        let diff = Diff::new("short\n", "also short\n");
        let console = make_console();
        let options = console.options();
        let m = diff.measure(&console, &options);

        assert!(m.minimum > 0);
        assert!(m.maximum >= m.minimum);
    }

    #[test]
    fn test_measure_side_by_side() {
        let diff = Diff::side_by_side("short\n", "also short\n");
        let console = make_console();
        let options = console.options();
        let m = diff.measure(&console, &options);

        assert!(m.minimum > 0);
        assert!(m.maximum >= m.minimum);
    }

    #[test]
    fn test_display_trait() {
        let diff = Diff::new("hello\n", "world\n");
        let output = format!("{}", diff);
        // Should contain the diff markers
        assert!(output.contains("-hello"));
        assert!(output.contains("+world"));
    }

    #[test]
    fn test_lcs_correctness() {
        let old = vec!["a", "b", "c", "d"];
        let new = vec!["a", "c", "d", "e"];
        let table = lcs_table(&old, &new);

        // LCS of [a,b,c,d] and [a,c,d,e] is [a,c,d] with length 3
        assert_eq!(table[4][4], 3);
    }

    #[test]
    fn test_hunks_identical_produces_no_hunks() {
        let ops = vec![
            DiffOp::Equal("a".to_string()),
            DiffOp::Equal("b".to_string()),
        ];
        let hunks = build_hunks(&ops, 3);
        assert!(hunks.is_empty());
    }

    #[test]
    fn test_hunks_single_change() {
        let ops = vec![
            DiffOp::Equal("a".to_string()),
            DiffOp::Delete("b".to_string()),
            DiffOp::Insert("x".to_string()),
            DiffOp::Equal("c".to_string()),
        ];
        let hunks = build_hunks(&ops, 3);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].old_start, 1);
        assert_eq!(hunks[0].new_start, 1);
    }

    #[test]
    fn test_unified_builder() {
        let diff = Diff::unified("a\n", "b\n");
        assert_eq!(diff.style, DiffStyle::Unified);
    }

    #[test]
    fn test_side_by_side_builder() {
        let diff = Diff::side_by_side("a\n", "b\n");
        assert_eq!(diff.style, DiffStyle::SideBySide);
    }

    #[test]
    fn test_with_style_builder() {
        let diff = Diff::new("a\n", "b\n").with_style(DiffStyle::SideBySide);
        assert_eq!(diff.style, DiffStyle::SideBySide);
    }

    #[test]
    fn test_pad_or_truncate_helper() {
        assert_eq!(pad_or_truncate("hi", 5), "hi   ");
        assert_eq!(pad_or_truncate("hello", 5), "hello");
        assert_eq!(pad_or_truncate("hello world", 5), "hello");
    }

    #[test]
    fn test_truncate_to_width_helper() {
        assert_eq!(truncate_to_width("hello", 10), "hello");
        assert_eq!(truncate_to_width("hello world", 5), "hello");
        assert_eq!(truncate_to_width("", 5), "");
    }

    #[test]
    fn test_identical_texts_unified_empty() {
        let diff = Diff::new("same\ntext\n", "same\ntext\n");
        let console = make_console();
        let options = console.options();
        let segments = diff.rich_console(&console, &options);
        // No changes, no output
        assert!(segments.is_empty());
    }
}
