//! Columns module -- displays renderables in neat auto-fitted columns.
//!
//! Port of Python's `rich/columns.py`. Uses `Table::grid()` internally
//! to lay out items in a grid of columns that fits within the available
//! console width.

use std::collections::HashMap;

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::table::{ColumnOptions, Table};
use crate::text::{JustifyMethod, Text};

// ---------------------------------------------------------------------------
// Columns
// ---------------------------------------------------------------------------

/// Display renderables in neat columns.
///
/// Items are laid out in a grid that auto-fits the available console width.
/// Internally a `Table::grid()` is used for rendering.
#[derive(Debug, Clone)]
pub struct Columns {
    /// The renderable items (stored as strings, converted to Text on demand).
    pub renderables: Vec<String>,
    /// Fixed column width, or `None` for auto-detect.
    pub width: Option<usize>,
    /// Padding around cells `(top, right, bottom, left)`.
    pub padding: (usize, usize, usize, usize),
    /// Expand to fill the full available width.
    pub expand: bool,
    /// Arrange into equal-sized columns.
    pub equal: bool,
    /// Fill columns top-to-bottom then left-to-right (instead of left-to-right).
    pub column_first: bool,
    /// Reverse column order within each row.
    pub right_to_left: bool,
    /// Optional alignment for cell content.
    pub align: Option<JustifyMethod>,
    /// Optional title displayed above the columns.
    pub title: Option<String>,
}

impl Columns {
    /// Create a new `Columns` with sensible defaults.
    pub fn new() -> Self {
        Columns {
            renderables: Vec::new(),
            width: None,
            padding: (0, 1, 0, 1),
            expand: false,
            equal: false,
            column_first: false,
            right_to_left: false,
            align: None,
            title: None,
        }
    }

    /// Add a renderable item (as a string).
    pub fn add_renderable(&mut self, text: &str) {
        self.renderables.push(text.to_string());
    }

    /// Set the fixed column width.
    #[must_use]
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the padding around cells.
    #[must_use]
    pub fn with_padding(mut self, padding: (usize, usize, usize, usize)) -> Self {
        self.padding = padding;
        self
    }

    /// Set whether to expand to fill the full width.
    #[must_use]
    pub fn with_expand(mut self, expand: bool) -> Self {
        self.expand = expand;
        self
    }

    /// Set whether to use equal-sized columns.
    #[must_use]
    pub fn with_equal(mut self, equal: bool) -> Self {
        self.equal = equal;
        self
    }

    /// Set whether to fill columns top-to-bottom first.
    #[must_use]
    pub fn with_column_first(mut self, column_first: bool) -> Self {
        self.column_first = column_first;
        self
    }

    /// Set whether to reverse column order within rows.
    #[must_use]
    pub fn with_right_to_left(mut self, right_to_left: bool) -> Self {
        self.right_to_left = right_to_left;
        self
    }

    /// Set the alignment for cell content.
    #[must_use]
    pub fn with_align(mut self, align: JustifyMethod) -> Self {
        self.align = Some(align);
        self
    }

    /// Set the title displayed above the columns.
    #[must_use]
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Iterate renderables in the order determined by `column_first`.
    ///
    /// Yields `(renderable_width, Option<&str>)` tuples. When `column_first`
    /// is true, items fill columns top-to-bottom then left-to-right.
    /// Incomplete final rows are padded with `(0, None)`.
    fn iter_renderables<'a>(
        &self,
        column_count: usize,
        renderable_widths: &'a [usize],
        renderables: &'a [Text],
    ) -> Vec<(usize, Option<&'a Text>)> {
        let item_count = renderables.len();
        let mut result: Vec<(usize, Option<&'a Text>)> = Vec::new();

        if self.column_first {
            // Distribute items into columns top-to-bottom
            let mut column_lengths: Vec<usize> = vec![item_count / column_count; column_count];
            for length in column_lengths.iter_mut().take(item_count % column_count) {
                *length += 1;
            }

            let row_count = item_count.div_ceil(column_count);
            let mut cells = vec![vec![-1i64; column_count]; row_count];
            let mut row: usize = 0;
            let mut col: usize = 0;
            for index in 0..item_count {
                cells[row][col] = index as i64;
                column_lengths[col] -= 1;
                if column_lengths[col] > 0 {
                    row += 1;
                } else {
                    col += 1;
                    row = 0;
                }
            }

            for row_cells in &cells {
                for &index in row_cells {
                    if index == -1 {
                        break;
                    }
                    let idx = index as usize;
                    result.push((renderable_widths[idx], Some(&renderables[idx])));
                }
            }
        } else {
            for (i, renderable) in renderables.iter().enumerate() {
                result.push((renderable_widths[i], Some(renderable)));
            }
        }

        // Pad incomplete final row with empty entries
        if item_count % column_count != 0 {
            for _ in 0..(column_count - (item_count % column_count)) {
                result.push((0, None));
            }
        }

        result
    }
}

impl Default for Columns {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderable for Columns {
    fn rich_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        // Convert all string renderables to Text
        let renderables: Vec<Text> = self
            .renderables
            .iter()
            .map(|s| console.render_str(s, None, None, None))
            .collect();

        if renderables.is_empty() {
            return Vec::new();
        }

        let (_top, right, _bottom, left) = self.padding;
        let width_padding = right.max(left);
        let max_width = options.max_width;

        // Measure each renderable's maximum width
        let mut renderable_widths: Vec<usize> =
            renderables.iter().map(|r| r.measure().maximum).collect();

        // If equal, set all widths to the max
        if self.equal {
            let max_w = renderable_widths.iter().copied().max().unwrap_or(0);
            renderable_widths = vec![max_w; renderable_widths.len()];
        }

        let mut column_count = renderables.len();

        if let Some(fixed_w) = self.width {
            // Fixed width mode: calculate column count from width
            column_count = max_width / (fixed_w + width_padding);
            if column_count == 0 {
                column_count = 1;
            }
        } else {
            // Auto-fit: reduce column count until total width fits
            while column_count > 1 {
                let mut widths: HashMap<usize, usize> = HashMap::new();
                let mut column_no: usize = 0;
                let items = self.iter_renderables(column_count, &renderable_widths, &renderables);
                let mut fits = true;

                for (renderable_width, _) in &items {
                    let entry = widths.entry(column_no).or_insert(0);
                    *entry = (*entry).max(*renderable_width);
                    let total_width: usize =
                        widths.values().sum::<usize>() + width_padding * (widths.len() - 1);
                    if total_width > max_width {
                        column_count = widths.len() - 1;
                        fits = false;
                        break;
                    }
                    column_no = (column_no + 1) % column_count;
                }

                if fits {
                    break;
                }
            }
        }

        // Ensure at least 1 column
        if column_count == 0 {
            column_count = 1;
        }

        // Get the renderables in the correct order
        let items = self.iter_renderables(column_count, &renderable_widths, &renderables);
        let mut final_renderables: Vec<Option<Text>> =
            items.into_iter().map(|(_, r)| r.cloned()).collect();

        // If equal, constrain each renderable to the equal width by truncating
        if self.equal {
            let equal_width = renderable_widths.first().copied().unwrap_or(0);
            for text in final_renderables.iter_mut().flatten() {
                if text.cell_len() > equal_width {
                    text.truncate(equal_width, None, false);
                }
            }
        }

        // Apply alignment wrapping
        if let Some(align_method) = self.align {
            final_renderables = final_renderables
                .into_iter()
                .map(|r| {
                    r.map(|text| {
                        let mut aligned = text;
                        aligned.justify = Some(align_method);
                        aligned
                    })
                })
                .collect();
        }

        // Build the table grid
        let mut table = Table::grid(&[]);
        table.padding = self.padding;
        table.collapse_padding = true;
        table.pad_edge = false;
        table.set_expand(self.expand);
        table.title = self.title.clone();

        // Add columns
        if let Some(fixed_w) = self.width {
            for _ in 0..column_count {
                table.add_column(
                    "",
                    "",
                    ColumnOptions {
                        width: Some(fixed_w),
                        ..Default::default()
                    },
                );
            }
        } else {
            for _ in 0..column_count {
                table.add_column("", "", Default::default());
            }
        }

        // Build rows
        for start in (0..final_renderables.len()).step_by(column_count) {
            let end = (start + column_count).min(final_renderables.len());
            let mut row_strings: Vec<String> = Vec::new();

            for item in &final_renderables[start..end] {
                match item {
                    Some(text) => row_strings.push(text.plain().to_string()),
                    None => row_strings.push(String::new()),
                }
            }

            // Handle right_to_left by reversing the row
            if self.right_to_left {
                row_strings.reverse();
            }

            let row: Vec<&str> = row_strings.iter().map(|s| s.as_str()).collect();
            table.add_row(&row);
        }

        // Render the table
        table.rich_console(console, options)
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl std::fmt::Display for Columns {
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

    fn make_console(width: usize) -> Console {
        Console::builder()
            .width(width)
            .force_terminal(true)
            .no_color(true)
            .markup(false)
            .build()
    }

    fn segments_to_text(segments: &[Segment]) -> String {
        segments.iter().map(|s| s.text.as_str()).collect()
    }

    fn render_columns(columns: &Columns, width: usize) -> String {
        let console = make_console(width);
        let opts = console.options();
        let segments = columns.rich_console(&console, &opts);
        segments_to_text(&segments)
    }

    // -- Default construction -----------------------------------------------

    #[test]
    fn test_default_construction() {
        let cols = Columns::new();
        assert!(cols.renderables.is_empty());
        assert_eq!(cols.width, None);
        assert_eq!(cols.padding, (0, 1, 0, 1));
        assert!(!cols.expand);
        assert!(!cols.equal);
        assert!(!cols.column_first);
        assert!(!cols.right_to_left);
        assert!(cols.align.is_none());
        assert!(cols.title.is_none());
    }

    #[test]
    fn test_default_trait() {
        let cols: Columns = Default::default();
        assert!(cols.renderables.is_empty());
    }

    // -- add_renderable -----------------------------------------------------

    #[test]
    fn test_add_renderable() {
        let mut cols = Columns::new();
        cols.add_renderable("hello");
        cols.add_renderable("world");
        assert_eq!(cols.renderables.len(), 2);
        assert_eq!(cols.renderables[0], "hello");
        assert_eq!(cols.renderables[1], "world");
    }

    // -- Builder methods ----------------------------------------------------

    #[test]
    fn test_with_width() {
        let cols = Columns::new().with_width(10);
        assert_eq!(cols.width, Some(10));
    }

    #[test]
    fn test_with_padding() {
        let cols = Columns::new().with_padding((1, 2, 3, 4));
        assert_eq!(cols.padding, (1, 2, 3, 4));
    }

    #[test]
    fn test_with_expand() {
        let cols = Columns::new().with_expand(true);
        assert!(cols.expand);
    }

    #[test]
    fn test_with_equal() {
        let cols = Columns::new().with_equal(true);
        assert!(cols.equal);
    }

    #[test]
    fn test_with_column_first() {
        let cols = Columns::new().with_column_first(true);
        assert!(cols.column_first);
    }

    #[test]
    fn test_with_right_to_left() {
        let cols = Columns::new().with_right_to_left(true);
        assert!(cols.right_to_left);
    }

    #[test]
    fn test_with_align() {
        let cols = Columns::new().with_align(JustifyMethod::Center);
        assert_eq!(cols.align, Some(JustifyMethod::Center));
    }

    #[test]
    fn test_with_title() {
        let cols = Columns::new().with_title("My Title");
        assert_eq!(cols.title, Some("My Title".to_string()));
    }

    // -- Empty renderables --------------------------------------------------

    #[test]
    fn test_empty_renderables() {
        let cols = Columns::new();
        let output = render_columns(&cols, 80);
        assert!(output.is_empty());
    }

    // -- Single item --------------------------------------------------------

    #[test]
    fn test_single_item() {
        let mut cols = Columns::new();
        cols.add_renderable("hello");
        let output = render_columns(&cols, 80);
        assert!(output.contains("hello"), "output was: {:?}", output);
    }

    // -- Column count auto-fitting ------------------------------------------

    #[test]
    fn test_auto_fit_all_in_one_row() {
        // Three short items should fit in one row in 80 cols
        let mut cols = Columns::new();
        cols.add_renderable("aaa");
        cols.add_renderable("bbb");
        cols.add_renderable("ccc");
        let output = render_columns(&cols, 80);
        // All three should appear in the output
        assert!(output.contains("aaa"));
        assert!(output.contains("bbb"));
        assert!(output.contains("ccc"));
        // Should all be on the same line (only one non-empty line)
        let lines: Vec<&str> = output.lines().filter(|l| !l.trim().is_empty()).collect();
        assert_eq!(lines.len(), 1, "Expected one line, got: {:?}", lines);
    }

    #[test]
    fn test_auto_fit_forces_wrapping() {
        // Items too wide for a single row at width 20
        let mut cols = Columns::new();
        cols.add_renderable("aaaaaaaaaa"); // 10 chars
        cols.add_renderable("bbbbbbbbbb"); // 10 chars
        cols.add_renderable("cccccccccc"); // 10 chars
                                           // With width=20 and padding of 1 between columns, only 2 can fit
                                           // 10 + 1 + 10 = 21 > 20, so only 1 per row... or 2 barely
        let output = render_columns(&cols, 20);
        let lines: Vec<&str> = output.lines().filter(|l| !l.trim().is_empty()).collect();
        assert!(
            lines.len() >= 2,
            "Expected multiple lines, got: {:?}",
            lines
        );
    }

    #[test]
    fn test_auto_fit_two_columns() {
        // Two items that should fit in two columns
        let mut cols = Columns::new();
        cols.add_renderable("abc");
        cols.add_renderable("def");
        let output = render_columns(&cols, 20);
        assert!(output.contains("abc"));
        assert!(output.contains("def"));
        let lines: Vec<&str> = output.lines().filter(|l| !l.trim().is_empty()).collect();
        assert_eq!(lines.len(), 1, "Expected one line, got: {:?}", lines);
    }

    // -- Fixed width mode ---------------------------------------------------

    #[test]
    fn test_fixed_width() {
        let mut cols = Columns::new().with_width(10);
        cols.add_renderable("a");
        cols.add_renderable("b");
        cols.add_renderable("c");
        cols.add_renderable("d");
        let output = render_columns(&cols, 80);
        assert!(output.contains("a"));
        assert!(output.contains("b"));
        assert!(output.contains("c"));
        assert!(output.contains("d"));
    }

    #[test]
    fn test_fixed_width_column_count() {
        // With width=10 and console width=25, padding=(0,1,0,1), width_padding=1
        // column_count = 25 / (10 + 1) = 2
        let mut cols = Columns::new().with_width(10);
        cols.add_renderable("a");
        cols.add_renderable("b");
        cols.add_renderable("c");
        let output = render_columns(&cols, 25);
        let lines: Vec<&str> = output.lines().filter(|l| !l.trim().is_empty()).collect();
        // 3 items in 2 columns = 2 rows
        assert_eq!(lines.len(), 2, "Expected 2 lines, got: {:?}", lines);
    }

    // -- column_first ordering ----------------------------------------------

    #[test]
    fn test_column_first_ordering() {
        // Use fixed width to force exactly 2 columns, then column_first fills
        // top-to-bottom: 4 items in 2 columns:
        //   Col 0: items 0, 1
        //   Col 1: items 2, 3
        // Row 0: item 0, item 2
        // Row 1: item 1, item 3
        let mut cols = Columns::new()
            .with_column_first(true)
            .with_width(8)
            .with_padding((0, 1, 0, 1));
        cols.add_renderable("A");
        cols.add_renderable("B");
        cols.add_renderable("C");
        cols.add_renderable("D");
        // width=8, padding=1, so each col = 8+1=9, console=20 => 2 columns
        let output = render_columns(&cols, 20);
        let lines: Vec<&str> = output.lines().filter(|l| !l.trim().is_empty()).collect();
        // Should have 2 rows
        assert_eq!(lines.len(), 2, "Expected 2 lines, got: {:?}", lines);
        // First row should contain A and C
        assert!(
            lines[0].contains('A'),
            "First row should contain A: {:?}",
            lines[0]
        );
        assert!(
            lines[0].contains('C'),
            "First row should contain C: {:?}",
            lines[0]
        );
        // Second row should contain B and D
        assert!(
            lines[1].contains('B'),
            "Second row should contain B: {:?}",
            lines[1]
        );
        assert!(
            lines[1].contains('D'),
            "Second row should contain D: {:?}",
            lines[1]
        );
    }

    #[test]
    fn test_column_first_uneven() {
        // 5 items in 3 columns with column_first:
        // Col 0: items 0, 1 (2 items)
        // Col 1: items 2, 3 (2 items)
        // Col 2: item 4 (1 item)
        // Row 0: item 0, item 2, item 4
        // Row 1: item 1, item 3
        let mut cols = Columns::new()
            .with_column_first(true)
            .with_padding((0, 1, 0, 1));
        cols.add_renderable("A");
        cols.add_renderable("B");
        cols.add_renderable("C");
        cols.add_renderable("D");
        cols.add_renderable("E");
        let output = render_columns(&cols, 40);
        assert!(output.contains('A'));
        assert!(output.contains('E'));
    }

    // -- right_to_left ordering ---------------------------------------------

    #[test]
    fn test_right_to_left() {
        let mut cols = Columns::new()
            .with_right_to_left(true)
            .with_padding((0, 1, 0, 1));
        cols.add_renderable("AAA");
        cols.add_renderable("BBB");
        let output = render_columns(&cols, 40);
        // Both items should be present
        assert!(output.contains("AAA"));
        assert!(output.contains("BBB"));
        // BBB should appear before AAA in the output (reversed order)
        let aaa_pos = output.find("AAA").unwrap();
        let bbb_pos = output.find("BBB").unwrap();
        assert!(
            bbb_pos < aaa_pos,
            "BBB should appear before AAA in right-to-left mode: {:?}",
            output
        );
    }

    // -- equal sizing -------------------------------------------------------

    #[test]
    fn test_equal_sizing() {
        let mut cols = Columns::new().with_equal(true).with_padding((0, 1, 0, 1));
        cols.add_renderable("a"); // width 1
        cols.add_renderable("longer"); // width 6
        cols.add_renderable("bb"); // width 2
        let output = render_columns(&cols, 40);
        assert!(output.contains("a"));
        assert!(output.contains("longer"));
        assert!(output.contains("bb"));
    }

    // -- Alignment wrapping -------------------------------------------------

    #[test]
    fn test_alignment_center() {
        let mut cols = Columns::new()
            .with_align(JustifyMethod::Center)
            .with_padding((0, 0, 0, 0));
        cols.add_renderable("a");
        cols.add_renderable("b");
        let output = render_columns(&cols, 40);
        assert!(output.contains("a"));
        assert!(output.contains("b"));
    }

    #[test]
    fn test_alignment_right() {
        let mut cols = Columns::new()
            .with_align(JustifyMethod::Right)
            .with_padding((0, 0, 0, 0));
        cols.add_renderable("a");
        cols.add_renderable("b");
        let output = render_columns(&cols, 40);
        assert!(output.contains("a"));
        assert!(output.contains("b"));
    }

    // -- Title support ------------------------------------------------------

    #[test]
    fn test_title() {
        let mut cols = Columns::new().with_title("My Files");
        cols.add_renderable("file1.txt");
        cols.add_renderable("file2.txt");
        let output = render_columns(&cols, 40);
        assert!(
            output.contains("My Files"),
            "Expected title in output: {:?}",
            output
        );
        assert!(output.contains("file1.txt"));
        assert!(output.contains("file2.txt"));
    }

    // -- Rendering integration tests ----------------------------------------

    #[test]
    fn test_render_with_console() {
        let console = make_console(40);
        let mut cols = Columns::new();
        cols.add_renderable("hello");
        cols.add_renderable("world");
        let opts = console.options();
        let segments = cols.rich_console(&console, &opts);
        assert!(!segments.is_empty());
        let text = segments_to_text(&segments);
        assert!(text.contains("hello"));
        assert!(text.contains("world"));
    }

    #[test]
    fn test_render_many_items() {
        let mut cols = Columns::new();
        for i in 0..20 {
            cols.add_renderable(&format!("item{i}"));
        }
        let output = render_columns(&cols, 80);
        for i in 0..20 {
            assert!(
                output.contains(&format!("item{i}")),
                "Missing item{i} in output"
            );
        }
    }

    #[test]
    fn test_render_narrow_console() {
        // Very narrow console should still work (one column)
        let mut cols = Columns::new();
        cols.add_renderable("hello");
        cols.add_renderable("world");
        let output = render_columns(&cols, 8);
        assert!(output.contains("hello"));
        assert!(output.contains("world"));
        let lines: Vec<&str> = output.lines().filter(|l| !l.trim().is_empty()).collect();
        assert_eq!(
            lines.len(),
            2,
            "Expected 2 lines in narrow mode: {:?}",
            lines
        );
    }

    #[test]
    fn test_expand_mode() {
        let mut cols = Columns::new().with_expand(true);
        cols.add_renderable("a");
        cols.add_renderable("b");
        let output = render_columns(&cols, 40);
        assert!(output.contains("a"));
        assert!(output.contains("b"));
    }

    #[test]
    fn test_no_padding() {
        let mut cols = Columns::new().with_padding((0, 0, 0, 0));
        cols.add_renderable("aaa");
        cols.add_renderable("bbb");
        let output = render_columns(&cols, 40);
        // Items should be right next to each other with no space
        assert!(output.contains("aaa"));
        assert!(output.contains("bbb"));
    }

    #[test]
    fn test_column_first_and_right_to_left_combined() {
        let mut cols = Columns::new()
            .with_column_first(true)
            .with_right_to_left(true)
            .with_padding((0, 1, 0, 1));
        cols.add_renderable("A");
        cols.add_renderable("B");
        cols.add_renderable("C");
        cols.add_renderable("D");
        let output = render_columns(&cols, 20);
        assert!(output.contains('A'));
        assert!(output.contains('B'));
        assert!(output.contains('C'));
        assert!(output.contains('D'));
    }

    #[test]
    fn test_single_item_column_first() {
        let mut cols = Columns::new().with_column_first(true);
        cols.add_renderable("only");
        let output = render_columns(&cols, 40);
        assert!(output.contains("only"));
    }

    #[test]
    fn test_equal_with_fixed_width() {
        let mut cols = Columns::new().with_equal(true).with_width(15);
        cols.add_renderable("short");
        cols.add_renderable("medium text");
        cols.add_renderable("a very long item");
        let output = render_columns(&cols, 80);
        assert!(output.contains("short"));
        assert!(output.contains("medium text"));
    }

    #[test]
    fn test_builder_chaining() {
        let cols = Columns::new()
            .with_width(10)
            .with_expand(true)
            .with_equal(true)
            .with_column_first(true)
            .with_right_to_left(true)
            .with_align(JustifyMethod::Center)
            .with_title("Test")
            .with_padding((1, 2, 3, 4));
        assert_eq!(cols.width, Some(10));
        assert!(cols.expand);
        assert!(cols.equal);
        assert!(cols.column_first);
        assert!(cols.right_to_left);
        assert_eq!(cols.align, Some(JustifyMethod::Center));
        assert_eq!(cols.title, Some("Test".to_string()));
        assert_eq!(cols.padding, (1, 2, 3, 4));
    }

    #[test]
    fn test_display_trait() {
        let mut cols = Columns::new();
        cols.add_renderable("one");
        cols.add_renderable("two");
        cols.add_renderable("three");
        let s = format!("{}", cols);
        assert!(!s.is_empty());
        assert!(s.contains("one"));
    }
}
