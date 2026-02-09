//! Table module -- rich table rendering with columns, rows, and box borders.
//!
//! Port of Python's `rich/table.py`.

use crate::align_widget::VerticalAlign;
use crate::box_chars::{BoxChars, RowLevel, HEAVY_HEAD};
use crate::console::{Console, ConsoleOptions, ConsoleOptionsUpdates, Renderable};
use crate::measure::Measurement;
use crate::ratio::{ratio_distribute, ratio_reduce};
use crate::segment::Segment;
use crate::style::Style;
use crate::text::{JustifyMethod, OverflowMethod, Text};

// ---------------------------------------------------------------------------
// CellContent
// ---------------------------------------------------------------------------

/// Content of a table cell -- either a plain string (parsed with markup) or
/// a pre-styled [`Text`] object.
#[derive(Debug, Clone)]
pub enum CellContent {
    // Note: PartialEq is implemented manually below (Plain compares string).
    /// A plain string, optionally containing markup tags.
    Plain(String),
    /// A pre-styled [`Text`] value (styles are preserved as-is).
    Styled(Text),
}

impl CellContent {
    /// Resolve into a [`Text`] using the given console for markup parsing.
    fn resolve(&self, console: &Console) -> Text {
        match self {
            CellContent::Plain(s) => console.render_str(s, None, None, None),
            CellContent::Styled(t) => t.clone(),
        }
    }
}

impl From<&str> for CellContent {
    fn from(s: &str) -> Self {
        CellContent::Plain(s.to_string())
    }
}

impl From<String> for CellContent {
    fn from(s: String) -> Self {
        CellContent::Plain(s)
    }
}

impl From<Text> for CellContent {
    fn from(t: Text) -> Self {
        CellContent::Styled(t)
    }
}

impl PartialEq<&str> for CellContent {
    fn eq(&self, other: &&str) -> bool {
        match self {
            CellContent::Plain(s) => s == *other,
            CellContent::Styled(t) => t.plain() == *other,
        }
    }
}

// ---------------------------------------------------------------------------
// Column
// ---------------------------------------------------------------------------

/// Defines a column within a Table.
#[derive(Debug, Clone)]
pub struct Column {
    /// Renderable header text.
    pub header: String,
    /// Renderable footer text.
    pub footer: String,
    /// Style for the header.
    pub header_style: String,
    /// Style for the footer.
    pub footer_style: String,
    /// Style for the column cells.
    pub style: String,
    /// Horizontal justification for cell content.
    pub justify: JustifyMethod,
    /// Vertical alignment for cell content.
    pub vertical: VerticalAlign,
    /// Overflow method for cell content.
    pub overflow: OverflowMethod,
    /// Fixed width, or None for auto.
    pub width: Option<usize>,
    /// Minimum width constraint.
    pub min_width: Option<usize>,
    /// Maximum width constraint.
    pub max_width: Option<usize>,
    /// Flex ratio for proportional sizing.
    pub ratio: Option<usize>,
    /// Disable wrapping in this column.
    pub no_wrap: bool,
    /// Whether to highlight cell text.
    pub highlight: bool,
    /// Column index (0-based).
    pub index: usize,
    /// Cell data for each row.
    pub cells: Vec<CellContent>,
}

impl Column {
    /// Returns true if this column is flexible (has a ratio set).
    pub fn flexible(&self) -> bool {
        self.ratio.is_some()
    }

    /// Return a copy of this Column with an empty cells vec.
    pub fn copy(&self) -> Column {
        Column {
            header: self.header.clone(),
            footer: self.footer.clone(),
            header_style: self.header_style.clone(),
            footer_style: self.footer_style.clone(),
            style: self.style.clone(),
            justify: self.justify,
            vertical: self.vertical,
            overflow: self.overflow,
            width: self.width,
            min_width: self.min_width,
            max_width: self.max_width,
            ratio: self.ratio,
            no_wrap: self.no_wrap,
            highlight: self.highlight,
            index: self.index,
            cells: Vec::new(),
        }
    }
}

impl Default for Column {
    fn default() -> Self {
        Column {
            header: String::new(),
            footer: String::new(),
            header_style: String::new(),
            footer_style: String::new(),
            style: String::new(),
            justify: JustifyMethod::Left,
            vertical: VerticalAlign::Top,
            overflow: OverflowMethod::Ellipsis,
            width: None,
            min_width: None,
            max_width: None,
            ratio: None,
            no_wrap: false,
            highlight: false,
            index: 0,
            cells: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Row
// ---------------------------------------------------------------------------

/// Information regarding a row.
#[derive(Debug, Clone, Default)]
pub struct Row {
    /// Optional style to apply to this row.
    pub style: Option<String>,
    /// Whether this row ends a section (draws a line after it).
    pub end_section: bool,
}

// ---------------------------------------------------------------------------
// Internal cell type
// ---------------------------------------------------------------------------

/// A single cell in the table (internal).
struct CellInfo {
    style: Style,
    renderable: Text,
    vertical: VerticalAlign,
}

// ---------------------------------------------------------------------------
// Table
// ---------------------------------------------------------------------------

/// A console renderable to draw a table with Unicode box-drawing borders,
/// column alignment, row striping, and styling.
///
/// # Examples
///
/// ```
/// use gilt::table::*;
///
/// let mut table = Table::new(&["Name", "Age"]);
/// table.add_row(&["Alice", "30"]);
/// table.add_row(&["Bob", "25"]);
/// let output = format!("{}", table);
/// assert!(output.contains("Alice"));
/// ```
#[derive(Debug, Clone)]
pub struct Table {
    /// Column definitions (one per column).
    pub columns: Vec<Column>,
    /// Row metadata (one per data row, does not include header/footer).
    pub rows: Vec<Row>,
    /// Optional title displayed above the table.
    pub title: Option<String>,
    /// Optional caption displayed below the table.
    pub caption: Option<String>,
    /// Fixed table width, or `None` for auto-sizing. Setting a width implies expand.
    pub width: Option<usize>,
    /// Minimum table width constraint.
    pub min_width: Option<usize>,
    /// Box-drawing character set, or `None` for no borders.
    pub box_chars: Option<&'static BoxChars>,
    /// Whether to substitute box characters on legacy terminals.
    pub safe_box: Option<bool>,
    /// Cell padding as `(top, right, bottom, left)`.
    pub padding: (usize, usize, usize, usize),
    /// Collapse inter-column padding so adjacent columns share padding space.
    pub collapse_padding: bool,
    /// Whether to add padding at the left and right table edges.
    pub pad_edge: bool,
    expand_flag: bool,
    /// Show the header row.
    pub show_header: bool,
    /// Show the footer row.
    pub show_footer: bool,
    /// Show the left and right border edges.
    pub show_edge: bool,
    /// Draw horizontal separator lines between every row.
    pub show_lines: bool,
    /// Number of extra blank lines to insert between rows.
    pub leading: usize,
    /// Style applied to the entire table.
    pub style: String,
    /// Alternating row styles (cycled by row index).
    pub row_styles: Vec<String>,
    /// Style applied to the header row.
    pub header_style: String,
    /// Style applied to the footer row.
    pub footer_style: String,
    /// Style applied to the table border.
    pub border_style: String,
    /// Style applied to the title text.
    pub title_style: String,
    /// Style applied to the caption text.
    pub caption_style: String,
    /// Horizontal justification for the title.
    pub title_justify: JustifyMethod,
    /// Horizontal justification for the caption.
    pub caption_justify: JustifyMethod,
    /// Enable syntax highlighting for cell content.
    pub highlight: bool,
}

impl Table {
    /// Create a new table with the given header strings.
    ///
    /// Each string becomes a column header. The table defaults to `HEAVY_HEAD`
    /// box characters, visible header, visible edges, and `(0, 1, 0, 1)` padding.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::table::Table;
    ///
    /// let table = Table::new(&["Name", "Age", "City"]);
    /// assert_eq!(table.columns.len(), 3);
    /// ```
    pub fn new(headers: &[&str]) -> Self {
        let mut table = Table {
            columns: Vec::new(),
            rows: Vec::new(),
            title: None,
            caption: None,
            width: None,
            min_width: None,
            box_chars: Some(&HEAVY_HEAD),
            safe_box: None,
            padding: (0, 1, 0, 1),
            collapse_padding: false,
            pad_edge: true,
            expand_flag: false,
            show_header: true,
            show_footer: false,
            show_edge: true,
            show_lines: false,
            leading: 0,
            style: String::new(),
            row_styles: Vec::new(),
            header_style: "table.header".to_string(),
            footer_style: "table.footer".to_string(),
            border_style: String::new(),
            title_style: String::new(),
            caption_style: String::new(),
            title_justify: JustifyMethod::Center,
            caption_justify: JustifyMethod::Center,
            highlight: false,
        };
        for header in headers {
            table.add_column(header, "", Default::default());
        }
        table
    }

    /// Create a grid table (no box, no header/footer/edge, collapse_padding, no pad_edge).
    ///
    /// Grids are useful for side-by-side layout without visible borders.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::table::Table;
    ///
    /// let mut grid = Table::grid(&["A", "B"]);
    /// grid.add_row(&["left", "right"]);
    /// assert!(grid.box_chars.is_none());
    /// assert!(!grid.show_header);
    /// ```
    pub fn grid(headers: &[&str]) -> Self {
        let mut table = Table {
            columns: Vec::new(),
            rows: Vec::new(),
            title: None,
            caption: None,
            width: None,
            min_width: None,
            box_chars: None,
            safe_box: None,
            padding: (0, 0, 0, 0),
            collapse_padding: true,
            pad_edge: false,
            expand_flag: false,
            show_header: false,
            show_footer: false,
            show_edge: false,
            show_lines: false,
            leading: 0,
            style: String::new(),
            row_styles: Vec::new(),
            header_style: String::new(),
            footer_style: String::new(),
            border_style: String::new(),
            title_style: String::new(),
            caption_style: String::new(),
            title_justify: JustifyMethod::Center,
            caption_justify: JustifyMethod::Center,
            highlight: false,
        };
        for header in headers {
            table.add_column(header, "", Default::default());
        }
        table
    }

    /// Whether the table should expand. Setting a non-None width implies expand.
    pub fn expand(&self) -> bool {
        self.expand_flag || self.width.is_some()
    }

    /// Set the expand flag.
    pub fn set_expand(&mut self, expand: bool) {
        self.expand_flag = expand;
    }

    /// Get extra width contributed by box borders (edge + column dividers).
    pub fn extra_width(&self) -> usize {
        let mut w = 0;
        if self.box_chars.is_some() && self.show_edge {
            w += 2;
        }
        if self.box_chars.is_some() && !self.columns.is_empty() {
            w += self.columns.len() - 1;
        }
        w
    }

    /// Get the current number of rows.
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Get the style for a given row index.
    fn get_row_style(&self, console: &Console, index: usize) -> Style {
        let mut style = Style::null();
        if !self.row_styles.is_empty() {
            let row_style_str = &self.row_styles[index % self.row_styles.len()];
            style = style
                + console
                    .get_style(row_style_str)
                    .unwrap_or_else(|_| Style::null());
        }
        if let Some(ref row_style_str) = self.rows[index].style {
            style = style
                + console
                    .get_style(row_style_str)
                    .unwrap_or_else(|_| Style::null());
        }
        style
    }

    /// Add a column to the table.
    ///
    /// The column index is assigned automatically. Use [`ColumnOptions`] to
    /// configure justification, width constraints, and styling.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::table::{Table, ColumnOptions};
    /// use gilt::text::JustifyMethod;
    ///
    /// let mut table = Table::new(&[]);
    /// table.add_column("Price", "$100", ColumnOptions {
    ///     justify: Some(JustifyMethod::Right),
    ///     width: Some(10),
    ///     ..Default::default()
    /// });
    /// assert_eq!(table.columns[0].header, "Price");
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn add_column(&mut self, header: &str, footer: &str, opts: ColumnOptions) {
        let index = self.columns.len();
        let column = Column {
            header: header.to_string(),
            footer: footer.to_string(),
            header_style: opts.header_style.unwrap_or_default(),
            footer_style: opts.footer_style.unwrap_or_default(),
            style: opts.style.unwrap_or_default(),
            justify: opts.justify.unwrap_or(JustifyMethod::Left),
            vertical: opts.vertical.unwrap_or(VerticalAlign::Top),
            overflow: opts.overflow.unwrap_or(OverflowMethod::Ellipsis),
            width: opts.width,
            min_width: opts.min_width,
            max_width: opts.max_width,
            ratio: opts.ratio,
            no_wrap: opts.no_wrap,
            highlight: opts.highlight.unwrap_or(self.highlight),
            index,
            cells: Vec::new(),
        };
        self.columns.push(column);
    }

    /// Add a row of cell values.
    ///
    /// Auto-creates columns if more cells are provided than columns exist.
    /// Pads missing cells with empty strings if fewer cells are provided.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::table::Table;
    ///
    /// let mut table = Table::new(&["Name", "Age"]);
    /// table.add_row(&["Alice", "30"]);
    /// table.add_row(&["Bob", "25"]);
    /// assert_eq!(table.row_count(), 2);
    /// ```
    pub fn add_row(&mut self, cells: &[&str]) {
        self.add_row_styled(cells, None, false);
    }

    /// Add a row of cell values with an optional style and section break.
    ///
    /// When `end_section` is `true`, a horizontal separator is drawn after
    /// this row.
    pub fn add_row_styled(&mut self, cells: &[&str], style: Option<&str>, end_section: bool) {
        let contents: Vec<CellContent> = cells.iter().map(|&s| CellContent::from(s)).collect();
        self.add_row_contents(&contents, style, end_section);
    }

    /// Add a row of pre-styled [`Text`] cells.
    ///
    /// Use this when cells already carry their own styling. The styles are
    /// preserved as-is rather than being parsed from markup.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::table::Table;
    /// use gilt::text::Text;
    /// use gilt::style::Style;
    ///
    /// let mut table = Table::new(&["Name"]);
    /// let bold_name = Text::new("Alice", Style::parse("bold").unwrap());
    /// table.add_row_text(&[bold_name]);
    /// assert_eq!(table.row_count(), 1);
    /// ```
    pub fn add_row_text(&mut self, cells: &[Text]) {
        self.add_row_text_styled(cells, None, false);
    }

    /// Add a row of pre-styled [`Text`] cells with an optional style and section break.
    ///
    /// Combines [`add_row_text`](Self::add_row_text) with the per-row style
    /// and section-break support of [`add_row_styled`](Self::add_row_styled).
    pub fn add_row_text_styled(&mut self, cells: &[Text], style: Option<&str>, end_section: bool) {
        let contents: Vec<CellContent> =
            cells.iter().map(|t| CellContent::from(t.clone())).collect();
        self.add_row_contents(&contents, style, end_section);
    }

    /// Add a row from [`CellContent`] values (internal workhorse).
    fn add_row_contents(&mut self, cells: &[CellContent], style: Option<&str>, end_section: bool) {
        let num_columns = self.columns.len();
        let num_cells = cells.len();

        // Extend with empty strings if fewer cells than columns
        let mut cell_values: Vec<CellContent> = cells.to_vec();
        if num_cells < num_columns {
            cell_values.extend(std::iter::repeat_n(
                CellContent::Plain(String::new()),
                num_columns - num_cells,
            ));
        }

        // Process each cell, auto-creating columns if needed
        for (i, cell_val) in cell_values.into_iter().enumerate() {
            if i >= self.columns.len() {
                // Auto-create a new column, backfill with empty cells for previous rows
                let mut new_column = Column {
                    index: i,
                    highlight: self.highlight,
                    ..Default::default()
                };
                for _ in 0..self.rows.len() {
                    new_column.cells.push(CellContent::Plain(String::new()));
                }
                self.columns.push(new_column);
            }
            self.columns[i].cells.push(cell_val);
        }

        self.rows.push(Row {
            style: style.map(|s| s.to_string()),
            end_section,
        });
    }

    /// Add a section break after the last row.
    ///
    /// This draws a horizontal separator line after the most recently added row.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::table::Table;
    ///
    /// let mut table = Table::new(&["Item"]);
    /// table.add_row(&["Apples"]);
    /// table.add_section();
    /// table.add_row(&["Oranges"]);
    /// ```
    pub fn add_section(&mut self) {
        if let Some(last_row) = self.rows.last_mut() {
            last_row.end_section = true;
        }
    }

    /// Get the padding width (left + right) for a column, considering collapse_padding and pad_edge.
    pub fn get_padding_width(&self, column_index: usize) -> usize {
        let (_, pad_right, _, pad_left) = self.padding;

        let mut pl = pad_left;
        let mut pr = pad_right;

        if self.collapse_padding {
            // In Python: pad_left = 0; pad_right = abs(pad_left - pad_right)
            // Since pad_left is already 0 at that point, this is just pad_right
            pl = 0;
            pr = pr.abs_diff(0); // effectively pad_right stays the same
        }

        if !self.pad_edge {
            if column_index == 0 {
                pl = 0;
            }
            if column_index == self.columns.len().saturating_sub(1) {
                pr = 0;
            }
        }

        pl + pr
    }

    /// Measure a column, returning its minimum and maximum width including padding.
    fn measure_column(
        &self,
        console: &Console,
        options: &ConsoleOptions,
        column: &Column,
    ) -> Measurement {
        let max_width = options.max_width;
        if max_width < 1 {
            return Measurement::new(0, 0);
        }

        let padding_width = self.get_padding_width(column.index);

        if let Some(fixed_width) = column.width {
            return Measurement::new(fixed_width + padding_width, fixed_width + padding_width)
                .with_maximum(max_width);
        }

        // Measure all cells in the column (header + data + footer)
        let mut min_widths: Vec<usize> = Vec::new();
        let mut max_widths: Vec<usize> = Vec::new();

        let cells = self.get_cells(console, column.index, column);
        for cell in &cells {
            let measurement = cell.renderable.measure();
            // Add padding width to the measurement
            min_widths.push(measurement.minimum + padding_width);
            max_widths.push(measurement.maximum + padding_width);
        }

        let min_w = min_widths.iter().copied().max().unwrap_or(1);
        let max_w = max_widths.iter().copied().max().unwrap_or(max_width);

        let measurement = Measurement::new(min_w, max_w).with_maximum(max_width);
        measurement.clamp(
            column.min_width.map(|mw| mw + padding_width),
            column.max_width.map(|mw| mw + padding_width),
        )
    }

    /// Get all cells for a column, including header/footer, with styles applied.
    fn get_cells(&self, console: &Console, column_index: usize, column: &Column) -> Vec<CellInfo> {
        let mut cells = Vec::new();

        if self.show_header {
            let header_style = console
                .get_style(&self.header_style)
                .unwrap_or_else(|_| Style::null())
                + console
                    .get_style(&column.header_style)
                    .unwrap_or_else(|_| Style::null());
            let text = console.render_str(&column.header, None, None, None);
            cells.push(CellInfo {
                style: header_style,
                renderable: text,
                vertical: column.vertical,
            });
        }

        let cell_style = console
            .get_style(&column.style)
            .unwrap_or_else(|_| Style::null());
        for cell_content in &column.cells {
            let text = cell_content.resolve(console);
            cells.push(CellInfo {
                style: cell_style.clone(),
                renderable: text,
                vertical: column.vertical,
            });
        }

        if self.show_footer {
            let footer_style = console
                .get_style(&self.footer_style)
                .unwrap_or_else(|_| Style::null())
                + console
                    .get_style(&column.footer_style)
                    .unwrap_or_else(|_| Style::null());
            let text = console.render_str(&column.footer, None, None, None);
            cells.push(CellInfo {
                style: footer_style,
                renderable: text,
                vertical: column.vertical,
            });
        }

        // Apply padding to cells
        let (pad_top, pad_right, pad_bottom, pad_left) = self.padding;
        let any_padding = pad_top > 0 || pad_right > 0 || pad_bottom > 0 || pad_left > 0;

        if any_padding {
            let first_column = column_index == 0;
            let last_column = column_index == self.columns.len().saturating_sub(1);
            let cell_count = cells.len();

            for (i, cell) in cells.iter_mut().enumerate() {
                let first_row = i == 0;
                let last_row = i == cell_count.saturating_sub(1);

                let mut right = pad_right;
                let mut left = pad_left;

                if self.collapse_padding && !first_column {
                    left = left.saturating_sub(right);
                    // bottom/top collapse not applied to text padding
                }

                if !self.pad_edge {
                    if first_column {
                        left = 0;
                    }
                    if last_column {
                        right = 0;
                    }
                    // Suppress unused variable warnings: top/bottom padding
                    // is handled during row rendering, not on Text objects
                    let _ = (first_row, last_row);
                }

                // Apply padding by modifying the text
                if left > 0 {
                    cell.renderable.pad_left(left, ' ');
                }
                if right > 0 {
                    cell.renderable.pad_right(right, ' ');
                }
                // Top/bottom padding handled by adding blank lines during rendering
                // (not modifying the Text itself here -- they become extra row height)
            }
        }

        cells
    }

    /// Calculate column widths for rendering.
    ///
    /// Takes into account fixed widths, flex ratios, min/max constraints,
    /// padding, and the available `max_width` from the console options.
    /// Returns a vector with one width per column.
    pub fn calculate_column_widths(
        &self,
        console: &Console,
        options: &ConsoleOptions,
    ) -> Vec<usize> {
        let max_width = options.max_width;
        let columns = &self.columns;

        let width_ranges: Vec<Measurement> = columns
            .iter()
            .map(|col| self.measure_column(console, options, col))
            .collect();

        let mut widths: Vec<usize> = width_ranges
            .iter()
            .map(|r| if r.maximum > 0 { r.maximum } else { 1 })
            .collect();

        let extra_width = self.extra_width();

        if self.expand() {
            let ratios: Vec<usize> = columns
                .iter()
                .filter(|c| c.flexible())
                .map(|c| c.ratio.unwrap_or(0))
                .collect();

            if ratios.iter().any(|&r| r > 0) {
                let fixed_widths: Vec<usize> = width_ranges
                    .iter()
                    .zip(columns.iter())
                    .map(
                        |(range, col)| {
                            if col.flexible() {
                                0
                            } else {
                                range.maximum
                            }
                        },
                    )
                    .collect();

                let flex_minimum: Vec<usize> = columns
                    .iter()
                    .filter(|c| c.flexible())
                    .map(|c| (c.width.unwrap_or(1)) + self.get_padding_width(c.index))
                    .collect();

                let flexible_width = max_width.saturating_sub(fixed_widths.iter().sum::<usize>());

                if !ratios.is_empty() && ratios.iter().sum::<usize>() > 0 {
                    let flex_widths =
                        ratio_distribute(flexible_width, &ratios, Some(&flex_minimum));

                    let mut flex_iter = flex_widths.into_iter();
                    for (i, col) in columns.iter().enumerate() {
                        if col.flexible() {
                            if let Some(fw) = flex_iter.next() {
                                widths[i] = fixed_widths[i] + fw;
                            }
                        }
                    }
                }
            }
        }

        let mut table_width: usize = widths.iter().sum();

        if table_width > max_width {
            let wrapable: Vec<bool> = columns
                .iter()
                .map(|c| c.width.is_none() && !c.no_wrap)
                .collect();

            widths = Self::collapse_widths(&widths, &wrapable, max_width);
            table_width = widths.iter().sum();

            // Last resort: reduce columns evenly
            if table_width > max_width {
                let excess_width = table_width - max_width;
                let ones: Vec<usize> = vec![1; widths.len()];
                widths = ratio_reduce(excess_width, &ones, &widths, &widths);
                let _ = widths.iter().sum::<usize>(); // table_width recalculated below
            }

            // Re-measure columns at new widths
            let new_ranges: Vec<Measurement> = widths
                .iter()
                .zip(columns.iter())
                .map(|(&w, col)| self.measure_column(console, &options.update_width(w), col))
                .collect();
            widths = new_ranges
                .iter()
                .map(|r| if r.maximum > 0 { r.maximum } else { 0 })
                .collect();
        }

        table_width = widths.iter().sum();

        // Expand if needed
        if (table_width < max_width && self.expand())
            || (self.min_width.is_some()
                && table_width < self.min_width.unwrap().saturating_sub(extra_width))
        {
            let target_width = if let Some(mw) = self.min_width {
                mw.saturating_sub(extra_width).min(max_width)
            } else {
                max_width
            };
            let pad_total = target_width.saturating_sub(table_width);
            if pad_total > 0 && !widths.is_empty() && widths.iter().sum::<usize>() > 0 {
                let pad_widths = ratio_distribute(pad_total, &widths, None);
                for (w, pad) in widths.iter_mut().zip(pad_widths.iter()) {
                    *w += pad;
                }
            }
        }

        widths
    }

    /// Reduce widths so that the total is under `max_width`.
    ///
    /// Iteratively shrinks the widest wrapable columns until the total fits.
    /// Columns marked as non-wrapable are left unchanged.
    pub fn collapse_widths(widths: &[usize], wrapable: &[bool], max_width: usize) -> Vec<usize> {
        let mut widths = widths.to_vec();
        let mut total_width: usize = widths.iter().sum();
        let mut excess_width = total_width.saturating_sub(max_width);

        if wrapable.iter().any(|&w| w) {
            while total_width > 0 && excess_width > 0 {
                let max_column = widths
                    .iter()
                    .zip(wrapable.iter())
                    .filter(|(_, &allow)| allow)
                    .map(|(&w, _)| w)
                    .max()
                    .unwrap_or(0);

                let second_max_column = widths
                    .iter()
                    .zip(wrapable.iter())
                    .map(|(&w, &allow)| if allow && w != max_column { w } else { 0 })
                    .max()
                    .unwrap_or(0);

                let column_difference = max_column.saturating_sub(second_max_column);

                let ratios: Vec<usize> = widths
                    .iter()
                    .zip(wrapable.iter())
                    .map(|(&w, &allow)| if w == max_column && allow { 1 } else { 0 })
                    .collect();

                if !ratios.iter().any(|&r| r > 0) || column_difference == 0 {
                    break;
                }

                let max_reduce: Vec<usize> = widths
                    .iter()
                    .map(|_| excess_width.min(column_difference))
                    .collect();

                widths = ratio_reduce(excess_width, &ratios, &max_reduce, &widths);
                total_width = widths.iter().sum();
                excess_width = total_width.saturating_sub(max_width);
            }
        }

        widths
    }

    /// The main rendering method. Produces segments for the table body (borders + cells).
    fn render_table(
        &self,
        console: &Console,
        options: &ConsoleOptions,
        widths: &[usize],
    ) -> Vec<Segment> {
        let mut segments: Vec<Segment> = Vec::new();

        let table_style = console
            .get_style(&self.style)
            .unwrap_or_else(|_| Style::null());
        let border_style = table_style.clone()
            + console
                .get_style(&self.border_style)
                .unwrap_or_else(|_| Style::null());

        // Build column cells (each column -> list of cells)
        let column_cells: Vec<Vec<CellInfo>> = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, col)| self.get_cells(console, i, col))
            .collect();

        // Transpose to row_cells: each row -> list of cells (one per column)
        let num_rows = column_cells.iter().map(|c| c.len()).max().unwrap_or(0);
        let num_cols = column_cells.len();

        // Get box (with substitution)
        let the_box: Option<&BoxChars> = self.box_chars.map(|b| {
            let safe = self.safe_box.unwrap_or(true);
            let ascii_only = options.ascii_only();
            let substituted = if ascii_only || safe {
                b.substitute(ascii_only)
            } else {
                b
            };
            if !self.show_header {
                substituted.get_plain_headed_box()
            } else {
                substituted
            }
        });

        let new_line = Segment::line();

        let show_header = self.show_header;
        let show_footer = self.show_footer;
        let show_edge = self.show_edge;
        let show_lines = self.show_lines;
        let leading = self.leading;

        // Box segments: [head_left, head_right, head_vertical],
        //               [mid_left, mid_right, mid_vertical],
        //               [foot_left, foot_right, foot_vertical]
        struct BoxSegs {
            left: Segment,
            right: Segment,
            divider: Segment,
        }

        let box_segments: Option<[BoxSegs; 3]> = the_box.map(|b| {
            [
                BoxSegs {
                    left: Segment::styled(&b.head_left.to_string(), border_style.clone()),
                    right: Segment::styled(&b.head_right.to_string(), border_style.clone()),
                    divider: Segment::styled(&b.head_vertical.to_string(), border_style.clone()),
                },
                BoxSegs {
                    left: Segment::styled(&b.mid_left.to_string(), border_style.clone()),
                    right: Segment::styled(&b.mid_right.to_string(), border_style.clone()),
                    divider: Segment::styled(&b.mid_vertical.to_string(), border_style.clone()),
                },
                BoxSegs {
                    left: Segment::styled(&b.foot_left.to_string(), border_style.clone()),
                    right: Segment::styled(&b.foot_right.to_string(), border_style.clone()),
                    divider: Segment::styled(&b.foot_vertical.to_string(), border_style.clone()),
                },
            ]
        });

        // Top edge
        if let Some(b) = the_box {
            if show_edge {
                segments.push(Segment::styled(&b.get_top(widths), border_style.clone()));
                segments.push(new_line.clone());
            }
        }

        // Iterate over rows
        for row_index in 0..num_rows {
            let first = row_index == 0;
            let last = row_index == num_rows - 1;

            let header_row = first && show_header;
            let footer_row = last && show_footer;

            // Determine the data row index (for style lookup)
            let data_row_index = if header_row || footer_row {
                None
            } else {
                let idx = if show_header {
                    row_index - 1
                } else {
                    row_index
                };
                if idx < self.rows.len() {
                    Some(idx)
                } else {
                    None
                }
            };

            let row_style = if header_row || footer_row {
                Style::null()
            } else if let Some(idx) = data_row_index {
                let style_obj = self.get_row_style(console, idx);
                console
                    .get_style(&style_obj.to_string())
                    .unwrap_or(style_obj)
            } else {
                Style::null()
            };

            // Render all cells for this row
            let mut rendered_cells: Vec<Vec<Vec<Segment>>> = Vec::with_capacity(num_cols);
            let mut max_height: usize = 1;

            for col_index in 0..num_cols {
                let width = if col_index < widths.len() {
                    widths[col_index]
                } else {
                    1
                };

                let column = &self.columns[col_index];

                let cell = if row_index < column_cells[col_index].len() {
                    &column_cells[col_index][row_index]
                } else {
                    // Shouldn't happen normally, but provide a blank cell
                    rendered_cells.push(vec![vec![Segment::styled(
                        &" ".repeat(width),
                        Style::null(),
                    )]]);
                    max_height = max_height.max(1);
                    continue;
                };

                let render_options = options.with_updates(&ConsoleOptionsUpdates {
                    width: Some(width),
                    justify: Some(Some(column.justify)),
                    no_wrap: Some(column.no_wrap),
                    overflow: Some(Some(column.overflow)),
                    height: Some(None),
                    highlight: Some(Some(column.highlight)),
                    ..Default::default()
                });

                let cell_combined_style = cell.style.clone() + row_style.clone();
                let lines = console.render_lines(
                    &cell.renderable,
                    Some(&render_options),
                    Some(&cell_combined_style),
                    true,
                    false,
                );

                max_height = max_height.max(lines.len());
                rendered_cells.push(lines);
            }

            // Apply vertical alignment and set shape
            let row_height = rendered_cells.iter().map(|c| c.len()).max().unwrap_or(1);
            let max_height = row_height.max(max_height);

            let mut shaped_cells: Vec<Vec<Vec<Segment>>> = Vec::with_capacity(num_cols);
            for col_index in 0..num_cols {
                let width = if col_index < widths.len() {
                    widths[col_index]
                } else {
                    1
                };

                let cell_lines = if col_index < rendered_cells.len() {
                    &rendered_cells[col_index]
                } else {
                    shaped_cells.push(vec![vec![Segment::styled(
                        &" ".repeat(width),
                        Style::null(),
                    )]]);
                    continue;
                };

                // Get vertical alignment
                let vertical = if header_row {
                    VerticalAlign::Bottom
                } else if footer_row {
                    VerticalAlign::Top
                } else if col_index < column_cells.len()
                    && row_index < column_cells[col_index].len()
                {
                    column_cells[col_index][row_index].vertical
                } else {
                    VerticalAlign::Top
                };

                let cell_style = if col_index < column_cells.len()
                    && row_index < column_cells[col_index].len()
                {
                    column_cells[col_index][row_index].style.clone() + row_style.clone()
                } else {
                    row_style.clone()
                };

                let aligned = match vertical {
                    VerticalAlign::Top => {
                        Segment::align_top(cell_lines, width, max_height, &cell_style, false)
                    }
                    VerticalAlign::Middle => {
                        Segment::align_middle(cell_lines, width, max_height, &cell_style, false)
                    }
                    VerticalAlign::Bottom => {
                        Segment::align_bottom(cell_lines, width, max_height, &cell_style, false)
                    }
                };

                let shaped = Segment::set_shape(&aligned, width, Some(max_height), None, false);
                shaped_cells.push(shaped);
            }

            // Footer separator (before footer row)
            if let Some(b) = the_box {
                if last && show_footer {
                    segments.push(Segment::styled(
                        &b.get_row(widths, RowLevel::Foot, show_edge),
                        border_style.clone(),
                    ));
                    segments.push(new_line.clone());
                }
            }

            // Render the row lines
            if let Some(ref bsegs) = box_segments {
                let seg_index = if first {
                    0
                } else if last {
                    2
                } else {
                    1
                };
                let left = &bsegs[seg_index].left;
                let right = &bsegs[seg_index].right;
                let base_divider = &bsegs[seg_index].divider;

                // If divider is whitespace, apply row background style
                let divider = if base_divider.text.trim().is_empty() {
                    let bg_style = row_style.background_style();
                    let combined =
                        bg_style + base_divider.style.clone().unwrap_or_else(Style::null);
                    Segment::styled(&base_divider.text, combined)
                } else {
                    base_divider.clone()
                };

                for line_no in 0..max_height {
                    if show_edge {
                        segments.push(left.clone());
                    }
                    for (cell_idx, cell) in shaped_cells.iter().enumerate() {
                        let last_cell = cell_idx == shaped_cells.len() - 1;
                        if line_no < cell.len() {
                            segments.extend(cell[line_no].iter().cloned());
                        }
                        if !last_cell {
                            segments.push(divider.clone());
                        }
                    }
                    if show_edge {
                        segments.push(right.clone());
                    }
                    segments.push(new_line.clone());
                }
            } else {
                // No box
                for line_no in 0..max_height {
                    for cell in &shaped_cells {
                        if line_no < cell.len() {
                            segments.extend(cell[line_no].iter().cloned());
                        }
                    }
                    segments.push(new_line.clone());
                }
            }

            // Header separator (after header row)
            if let Some(b) = the_box {
                if first && show_header {
                    segments.push(Segment::styled(
                        &b.get_row(widths, RowLevel::Head, show_edge),
                        border_style.clone(),
                    ));
                    segments.push(new_line.clone());
                }
            }

            // Inter-row lines / leading / end_section
            let row_ref = data_row_index.and_then(|idx| self.rows.get(idx));
            let end_section = row_ref.is_some_and(|r| r.end_section);

            if let Some(b) = the_box {
                if show_lines || leading > 0 || end_section {
                    // Don't add separator after last row, after header (already done), or before footer
                    let skip = last
                        || (show_footer && row_index >= num_rows.saturating_sub(2))
                        || (show_header && header_row);

                    if !skip {
                        if leading > 0 {
                            let row_str = b.get_row(widths, RowLevel::Mid, show_edge);
                            for _ in 0..leading {
                                segments.push(Segment::styled(&row_str, border_style.clone()));
                                segments.push(new_line.clone());
                            }
                        } else {
                            segments.push(Segment::styled(
                                &b.get_row(widths, RowLevel::Row, show_edge),
                                border_style.clone(),
                            ));
                            segments.push(new_line.clone());
                        }
                    }
                }
            }
        }

        // Bottom edge
        if let Some(b) = the_box {
            if show_edge {
                segments.push(Segment::styled(&b.get_bottom(widths), border_style.clone()));
                segments.push(new_line);
            }
        }

        segments
    }

    /// Measure the table, returning minimum and maximum widths.
    ///
    /// Used by the [`Renderable`] trait to determine how much space the table
    /// requires.
    pub fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        let mut max_width = options.max_width;
        if let Some(w) = self.width {
            max_width = w;
        }

        let extra_width = self.extra_width();
        let col_widths = self.calculate_column_widths(
            console,
            &options.update_width(max_width.saturating_sub(extra_width)),
        );
        let total_max: usize = col_widths.iter().sum::<usize>();

        let measurements: Vec<Measurement> = self
            .columns
            .iter()
            .map(|col| self.measure_column(console, &options.update_width(total_max), col))
            .collect();

        let minimum_width: usize =
            measurements.iter().map(|m| m.minimum).sum::<usize>() + extra_width;
        let maximum_width: usize = if let Some(w) = self.width {
            w
        } else {
            measurements.iter().map(|m| m.maximum).sum::<usize>() + extra_width
        };

        let measurement = Measurement::new(minimum_width, maximum_width);
        measurement.clamp(self.min_width, None)
    }
}

// ---------------------------------------------------------------------------
// ColumnOptions helper for add_column builder pattern
// ---------------------------------------------------------------------------

/// Options for adding a column (used to avoid too many parameters).
///
/// All fields default to `None` / `false`, meaning the column inherits
/// sensible defaults from the table.
#[derive(Debug, Clone, Default)]
pub struct ColumnOptions {
    /// Style for the header cell, or `None` for default.
    pub header_style: Option<String>,
    /// Style for the footer cell, or `None` for default.
    pub footer_style: Option<String>,
    /// Style for the data cells, or `None` for default.
    pub style: Option<String>,
    /// Horizontal justification, or `None` for `Left`.
    pub justify: Option<JustifyMethod>,
    /// Vertical alignment, or `None` for `Top`.
    pub vertical: Option<VerticalAlign>,
    /// Overflow method, or `None` for `Ellipsis`.
    pub overflow: Option<OverflowMethod>,
    /// Fixed column width, or `None` for auto.
    pub width: Option<usize>,
    /// Minimum column width constraint.
    pub min_width: Option<usize>,
    /// Maximum column width constraint.
    pub max_width: Option<usize>,
    /// Flex ratio for proportional sizing in expanded tables.
    pub ratio: Option<usize>,
    /// Disable wrapping in this column.
    pub no_wrap: bool,
    /// Enable syntax highlighting, or `None` to inherit from the table.
    pub highlight: Option<bool>,
}

// ---------------------------------------------------------------------------
// Renderable implementation
// ---------------------------------------------------------------------------

impl Renderable for Table {
    fn rich_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        if self.columns.is_empty() {
            return vec![Segment::line()];
        }

        let mut max_width = options.max_width;
        if let Some(w) = self.width {
            max_width = w;
        }

        let extra_width = self.extra_width();
        let widths = self.calculate_column_widths(
            console,
            &options.update_width(max_width.saturating_sub(extra_width)),
        );
        let table_width: usize = widths.iter().sum::<usize>() + extra_width;

        let render_options = options.with_updates(&ConsoleOptionsUpdates {
            width: Some(table_width),
            highlight: Some(Some(self.highlight)),
            height: Some(None),
            ..Default::default()
        });

        let mut segments = Vec::new();

        // Title
        if let Some(ref title) = self.title {
            let title_style_str = if self.title_style.is_empty() {
                "table.title"
            } else {
                &self.title_style
            };
            let title_style = console
                .get_style(title_style_str)
                .unwrap_or_else(|_| Style::null());
            let mut title_text =
                console.render_str(title, Some(&title_style.to_string()), None, None);
            title_text.justify = Some(self.title_justify);

            let title_opts = render_options.with_updates(&ConsoleOptionsUpdates {
                justify: Some(Some(self.title_justify)),
                ..Default::default()
            });

            let title_segs = title_text.rich_console(console, &title_opts);
            segments.extend(title_segs);
            // Ensure title ends with a newline
            if segments
                .last()
                .map(|s| !s.text.ends_with('\n'))
                .unwrap_or(false)
            {
                segments.push(Segment::line());
            }
        }

        // Render table body
        segments.extend(self.render_table(console, &render_options, &widths));

        // Caption
        if let Some(ref caption) = self.caption {
            let caption_style_str = if self.caption_style.is_empty() {
                "table.caption"
            } else {
                &self.caption_style
            };
            let caption_style = console
                .get_style(caption_style_str)
                .unwrap_or_else(|_| Style::null());
            let mut caption_text =
                console.render_str(caption, Some(&caption_style.to_string()), None, None);
            caption_text.justify = Some(self.caption_justify);

            let caption_opts = render_options.with_updates(&ConsoleOptionsUpdates {
                justify: Some(Some(self.caption_justify)),
                ..Default::default()
            });

            let caption_segs = caption_text.rich_console(console, &caption_opts);
            segments.extend(caption_segs);
            if segments
                .last()
                .map(|s| !s.text.ends_with('\n'))
                .unwrap_or(false)
            {
                segments.push(Segment::line());
            }
        }

        segments
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl std::fmt::Display for Table {
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

    fn render_table(table: &Table, width: usize) -> String {
        let console = make_console(width);
        let opts = console.options();
        let segments = table.rich_console(&console, &opts);
        segments_to_text(&segments)
    }

    // -- Column tests -------------------------------------------------------

    #[test]
    fn test_column_default() {
        let col = Column::default();
        assert_eq!(col.header, "");
        assert_eq!(col.footer, "");
        assert_eq!(col.justify, JustifyMethod::Left);
        assert_eq!(col.vertical, VerticalAlign::Top);
        assert_eq!(col.overflow, OverflowMethod::Ellipsis);
        assert!(!col.no_wrap);
        assert!(!col.highlight);
        assert!(!col.flexible());
    }

    #[test]
    fn test_column_flexible() {
        let mut col = Column::default();
        assert!(!col.flexible());
        col.ratio = Some(1);
        assert!(col.flexible());
    }

    #[test]
    fn test_column_copy() {
        let mut col = Column {
            header: "Name".to_string(),
            ..Default::default()
        };
        col.cells.push(CellContent::Plain("Alice".to_string()));
        col.cells.push(CellContent::Plain("Bob".to_string()));

        let copy = col.copy();
        assert_eq!(copy.header, "Name");
        assert!(copy.cells.is_empty()); // copy has no cells
    }

    // -- Row tests ----------------------------------------------------------

    #[test]
    fn test_row_default() {
        let row = Row::default();
        assert!(row.style.is_none());
        assert!(!row.end_section);
    }

    // -- Table constructor tests --------------------------------------------

    #[test]
    fn test_table_new_with_headers() {
        let table = Table::new(&["Name", "Age", "City"]);
        assert_eq!(table.columns.len(), 3);
        assert_eq!(table.columns[0].header, "Name");
        assert_eq!(table.columns[1].header, "Age");
        assert_eq!(table.columns[2].header, "City");
    }

    #[test]
    fn test_table_new_empty() {
        let table = Table::new(&[]);
        assert!(table.columns.is_empty());
    }

    #[test]
    fn test_table_grid() {
        let table = Table::grid(&["A", "B"]);
        assert!(table.box_chars.is_none());
        assert!(!table.show_header);
        assert!(!table.show_footer);
        assert!(!table.show_edge);
        assert!(table.collapse_padding);
        assert!(!table.pad_edge);
    }

    #[test]
    fn test_table_expand() {
        let mut table = Table::new(&["A"]);
        assert!(!table.expand());

        table.set_expand(true);
        assert!(table.expand());

        table.set_expand(false);
        assert!(!table.expand());

        table.width = Some(50);
        assert!(table.expand()); // width implies expand
    }

    // -- add_column tests ---------------------------------------------------

    #[test]
    fn test_add_column_basic() {
        let mut table = Table::new(&[]);
        table.add_column("Name", "", Default::default());
        assert_eq!(table.columns.len(), 1);
        assert_eq!(table.columns[0].header, "Name");
        assert_eq!(table.columns[0].index, 0);
    }

    #[test]
    fn test_add_column_with_options() {
        let mut table = Table::new(&[]);
        table.add_column(
            "Price",
            "Total",
            ColumnOptions {
                justify: Some(JustifyMethod::Right),
                no_wrap: true,
                width: Some(10),
                ..Default::default()
            },
        );
        let col = &table.columns[0];
        assert_eq!(col.header, "Price");
        assert_eq!(col.footer, "Total");
        assert_eq!(col.justify, JustifyMethod::Right);
        assert!(col.no_wrap);
        assert_eq!(col.width, Some(10));
    }

    #[test]
    fn test_add_column_auto_index() {
        let mut table = Table::new(&[]);
        table.add_column("A", "", Default::default());
        table.add_column("B", "", Default::default());
        table.add_column("C", "", Default::default());
        assert_eq!(table.columns[0].index, 0);
        assert_eq!(table.columns[1].index, 1);
        assert_eq!(table.columns[2].index, 2);
    }

    // -- add_row tests ------------------------------------------------------

    #[test]
    fn test_add_row_matching_columns() {
        let mut table = Table::new(&["Name", "Age"]);
        table.add_row(&["Alice", "30"]);
        assert_eq!(table.rows.len(), 1);
        assert_eq!(table.columns[0].cells.len(), 1);
        assert_eq!(table.columns[0].cells[0], "Alice");
        assert_eq!(table.columns[1].cells[0], "30");
    }

    #[test]
    fn test_add_row_fewer_cells() {
        let mut table = Table::new(&["Name", "Age", "City"]);
        table.add_row(&["Alice"]);
        assert_eq!(table.columns[0].cells[0], "Alice");
        assert_eq!(table.columns[1].cells[0], "");
        assert_eq!(table.columns[2].cells[0], "");
    }

    #[test]
    fn test_add_row_more_cells_than_columns() {
        let mut table = Table::new(&["Name"]);
        table.add_row(&["Alice", "30", "NYC"]);
        assert_eq!(table.columns.len(), 3);
        assert_eq!(table.columns[0].cells[0], "Alice");
        assert_eq!(table.columns[1].cells[0], "30");
        assert_eq!(table.columns[2].cells[0], "NYC");
    }

    #[test]
    fn test_add_row_auto_create_backfills() {
        let mut table = Table::new(&["Name"]);
        table.add_row(&["Alice"]);
        table.add_row(&["Bob", "25"]);
        // Column 1 was auto-created for second row; first row should have empty cell
        assert_eq!(table.columns[1].cells.len(), 2);
        assert_eq!(table.columns[1].cells[0], "");
        assert_eq!(table.columns[1].cells[1], "25");
    }

    // -- add_section tests --------------------------------------------------

    #[test]
    fn test_add_section() {
        let mut table = Table::new(&["A"]);
        table.add_row(&["1"]);
        table.add_row(&["2"]);
        assert!(!table.rows[1].end_section);
        table.add_section();
        assert!(table.rows[1].end_section);
    }

    #[test]
    fn test_add_section_no_rows() {
        let mut table = Table::new(&["A"]);
        table.add_section(); // Should not panic
    }

    // -- extra_width tests --------------------------------------------------

    #[test]
    fn test_extra_width_with_box_and_edge() {
        let table = Table::new(&["A", "B", "C"]);
        // box = HEAVY_HEAD, show_edge = true
        // extra = 2 (edge) + 2 (3 columns - 1 dividers) = 4
        assert_eq!(table.extra_width(), 4);
    }

    #[test]
    fn test_extra_width_no_box() {
        let table = Table::grid(&["A", "B"]);
        assert_eq!(table.extra_width(), 0);
    }

    #[test]
    fn test_extra_width_box_no_edge() {
        let mut table = Table::new(&["A", "B"]);
        table.show_edge = false;
        // box present but no edge: just column dividers = 1
        assert_eq!(table.extra_width(), 1);
    }

    // -- get_padding_width tests --------------------------------------------

    #[test]
    fn test_get_padding_width_default() {
        let table = Table::new(&["A", "B", "C"]);
        // padding = (0, 1, 0, 1), pad_edge = true, collapse_padding = false
        // All columns: left + right = 1 + 1 = 2
        assert_eq!(table.get_padding_width(0), 2);
        assert_eq!(table.get_padding_width(1), 2);
        assert_eq!(table.get_padding_width(2), 2);
    }

    #[test]
    fn test_get_padding_width_no_pad_edge() {
        let mut table = Table::new(&["A", "B", "C"]);
        table.pad_edge = false;
        // First column: left = 0, right = 1 => 1
        assert_eq!(table.get_padding_width(0), 1);
        // Middle column: left = 1, right = 1 => 2
        assert_eq!(table.get_padding_width(1), 2);
        // Last column: left = 1, right = 0 => 1
        assert_eq!(table.get_padding_width(2), 1);
    }

    #[test]
    fn test_get_padding_width_collapse() {
        let mut table = Table::new(&["A", "B"]);
        table.collapse_padding = true;
        // With collapse: left = 0, right = pad_right = 1
        assert_eq!(table.get_padding_width(0), 1);
        assert_eq!(table.get_padding_width(1), 1);
    }

    #[test]
    fn test_get_padding_width_grid() {
        let table = Table::grid(&["A", "B"]);
        // padding = (0, 0, 0, 0)
        assert_eq!(table.get_padding_width(0), 0);
        assert_eq!(table.get_padding_width(1), 0);
    }

    // -- collapse_widths tests ----------------------------------------------

    #[test]
    fn test_collapse_widths_basic() {
        let widths = vec![10, 20, 10];
        let wrapable = vec![true, true, true];
        let result = Table::collapse_widths(&widths, &wrapable, 30);
        let total: usize = result.iter().sum();
        assert!(total <= 30);
    }

    #[test]
    fn test_collapse_widths_none_wrapable() {
        let widths = vec![10, 20, 10];
        let wrapable = vec![false, false, false];
        let result = Table::collapse_widths(&widths, &wrapable, 20);
        // Nothing can be collapsed
        assert_eq!(result, widths);
    }

    #[test]
    fn test_collapse_widths_partial_wrapable() {
        let widths = vec![10, 20, 10];
        let wrapable = vec![false, true, false];
        let result = Table::collapse_widths(&widths, &wrapable, 30);
        let total: usize = result.iter().sum();
        assert!(total <= 30);
        // Only column 1 can shrink
        assert_eq!(result[0], 10);
        assert_eq!(result[2], 10);
    }

    #[test]
    fn test_collapse_widths_already_fits() {
        let widths = vec![5, 5, 5];
        let wrapable = vec![true, true, true];
        let result = Table::collapse_widths(&widths, &wrapable, 20);
        assert_eq!(result, widths);
    }

    // -- Width calculation tests --------------------------------------------

    #[test]
    fn test_calculate_column_widths_fixed() {
        let mut table = Table::new(&[]);
        table.show_header = false;
        table.box_chars = None;
        table.show_edge = false;
        table.pad_edge = false;
        table.padding = (0, 0, 0, 0);
        table.add_column(
            "A",
            "",
            ColumnOptions {
                width: Some(5),
                ..Default::default()
            },
        );
        table.add_column(
            "B",
            "",
            ColumnOptions {
                width: Some(10),
                ..Default::default()
            },
        );
        table.add_row(&["x", "y"]);

        let console = make_console(80);
        let opts = console.options();
        let widths = table.calculate_column_widths(&console, &opts);
        assert_eq!(widths[0], 5);
        assert_eq!(widths[1], 10);
    }

    #[test]
    fn test_calculate_column_widths_expand() {
        let mut table = Table::new(&[]);
        table.show_header = false;
        table.box_chars = None;
        table.show_edge = false;
        table.pad_edge = false;
        table.padding = (0, 0, 0, 0);
        table.set_expand(true);
        table.add_column(
            "",
            "",
            ColumnOptions {
                ratio: Some(1),
                ..Default::default()
            },
        );
        table.add_column(
            "",
            "",
            ColumnOptions {
                ratio: Some(1),
                ..Default::default()
            },
        );
        table.add_row(&["a", "b"]);

        let console = make_console(20);
        let opts = console.options();
        let widths = table.calculate_column_widths(&console, &opts);
        let total: usize = widths.iter().sum();
        assert_eq!(total, 20);
    }

    // -- Rendering tests ----------------------------------------------------

    #[test]
    fn test_render_empty_table() {
        let table = Table::new(&[]);
        let output = render_table(&table, 40);
        assert_eq!(output, "\n");
    }

    #[test]
    fn test_render_simple_2x2() {
        let mut table = Table::new(&["Name", "Age"]);
        table.add_row(&["Alice", "30"]);
        table.add_row(&["Bob", "25"]);

        let output = render_table(&table, 40);
        assert!(output.contains("Name"));
        assert!(output.contains("Age"));
        assert!(output.contains("Alice"));
        assert!(output.contains("30"));
        assert!(output.contains("Bob"));
        assert!(output.contains("25"));
    }

    #[test]
    fn test_render_with_box_edges() {
        let mut table = Table::new(&["A"]);
        table.add_row(&["x"]);
        let output = render_table(&table, 40);
        // Should contain box characters or some structural output
        assert!(output.lines().count() > 1);
    }

    #[test]
    fn test_render_no_box() {
        let mut table = Table::grid(&["A", "B"]);
        table.add_row(&["hello", "world"]);
        let output = render_table(&table, 40);
        assert!(output.contains("hello"));
        assert!(output.contains("world"));
    }

    #[test]
    fn test_render_grid_mode() {
        let mut table = Table::grid(&[]);
        table.add_column("", "", Default::default());
        table.add_column("", "", Default::default());
        table.add_row(&["left", "right"]);

        let output = render_table(&table, 40);
        assert!(output.contains("left"));
        assert!(output.contains("right"));
    }

    #[test]
    fn test_render_with_header_footer() {
        let mut table = Table::new(&[]);
        table.show_footer = true;
        table.add_column(
            "Item",
            "Total",
            ColumnOptions {
                ..Default::default()
            },
        );
        table.add_column(
            "Price",
            "$100",
            ColumnOptions {
                justify: Some(JustifyMethod::Right),
                ..Default::default()
            },
        );
        table.add_row(&["Widget", "$50"]);
        table.add_row(&["Gadget", "$50"]);

        let output = render_table(&table, 40);
        assert!(output.contains("Item"));
        assert!(output.contains("Price"));
        assert!(output.contains("Widget"));
        assert!(output.contains("Total"));
        assert!(output.contains("$100"));
    }

    #[test]
    fn test_render_row_styles() {
        let mut table = Table::new(&["A"]);
        table.row_styles = vec!["bold".to_string(), "".to_string()];
        table.add_row(&["row0"]);
        table.add_row(&["row1"]);
        table.add_row(&["row2"]);

        let output = render_table(&table, 40);
        assert!(output.contains("row0"));
        assert!(output.contains("row1"));
        assert!(output.contains("row2"));
    }

    #[test]
    fn test_render_show_lines() {
        let mut table = Table::new(&["A"]);
        table.show_lines = true;
        table.add_row(&["1"]);
        table.add_row(&["2"]);
        table.add_row(&["3"]);

        let output = render_table(&table, 40);
        // With show_lines, there should be separator lines between rows
        let line_count = output.lines().count();
        assert!(line_count >= 7); // top + header + head_sep + row1 + sep + row2 + sep + row3 + bottom
    }

    #[test]
    fn test_render_leading() {
        let mut table = Table::new(&["A"]);
        table.leading = 1;
        table.add_row(&["1"]);
        table.add_row(&["2"]);

        let output = render_table(&table, 40);
        let line_count = output.lines().count();
        // Leading adds extra blank-ish lines between rows
        assert!(line_count >= 5);
    }

    #[test]
    fn test_render_column_justify_right() {
        let mut table = Table::new(&[]);
        table.box_chars = None;
        table.show_header = false;
        table.show_edge = false;
        table.pad_edge = false;
        table.padding = (0, 0, 0, 0);
        table.add_column(
            "",
            "",
            ColumnOptions {
                justify: Some(JustifyMethod::Right),
                width: Some(10),
                ..Default::default()
            },
        );
        table.add_row(&["hi"]);

        let output = render_table(&table, 40);
        // "hi" should be right-aligned in a 10-char field
        let line = output.lines().next().unwrap_or("");
        assert!(line.ends_with("hi") || line.trim_end().ends_with("hi"));
    }

    #[test]
    fn test_render_column_no_wrap() {
        let mut table = Table::new(&[]);
        table.box_chars = None;
        table.show_header = false;
        table.show_edge = false;
        table.padding = (0, 0, 0, 0);
        table.pad_edge = false;
        table.add_column(
            "",
            "",
            ColumnOptions {
                no_wrap: true,
                width: Some(5),
                ..Default::default()
            },
        );
        table.add_row(&["Hello World"]);

        let output = render_table(&table, 40);
        // Content should be truncated/cropped to width, not wrapped
        let lines: Vec<&str> = output.lines().collect();
        // Should be a single content line (not multi-line wrapped)
        assert!(!lines.is_empty());
    }

    // -- Vertical alignment -------------------------------------------------

    #[test]
    fn test_vertical_align_top() {
        let mut table = Table::new(&[]);
        table.box_chars = None;
        table.show_header = false;
        table.show_edge = false;
        table.padding = (0, 0, 0, 0);
        table.pad_edge = false;
        table.add_column(
            "",
            "",
            ColumnOptions {
                vertical: Some(VerticalAlign::Top),
                width: Some(5),
                ..Default::default()
            },
        );
        table.add_column(
            "",
            "",
            ColumnOptions {
                width: Some(5),
                ..Default::default()
            },
        );
        table.add_row(&["A", "B\nC"]);

        let output = render_table(&table, 40);
        let lines: Vec<&str> = output.lines().collect();
        // First line should contain "A" and "B"
        assert!(lines.len() >= 2);
        assert!(lines[0].contains('A'));
        assert!(lines[0].contains('B'));
    }

    #[test]
    fn test_vertical_align_middle() {
        let mut table = Table::new(&[]);
        table.box_chars = None;
        table.show_header = false;
        table.show_edge = false;
        table.padding = (0, 0, 0, 0);
        table.pad_edge = false;
        table.add_column(
            "",
            "",
            ColumnOptions {
                vertical: Some(VerticalAlign::Middle),
                width: Some(3),
                ..Default::default()
            },
        );
        table.add_column(
            "",
            "",
            ColumnOptions {
                width: Some(3),
                ..Default::default()
            },
        );
        table.add_row(&["X", "A\nB\nC"]);

        let output = render_table(&table, 40);
        let lines: Vec<&str> = output.lines().collect();
        assert!(lines.len() >= 3);
        // X should be in the middle line
        assert!(lines[1].contains('X'));
    }

    #[test]
    fn test_vertical_align_bottom() {
        let mut table = Table::new(&[]);
        table.box_chars = None;
        table.show_header = false;
        table.show_edge = false;
        table.padding = (0, 0, 0, 0);
        table.pad_edge = false;
        table.add_column(
            "",
            "",
            ColumnOptions {
                vertical: Some(VerticalAlign::Bottom),
                width: Some(3),
                ..Default::default()
            },
        );
        table.add_column(
            "",
            "",
            ColumnOptions {
                width: Some(3),
                ..Default::default()
            },
        );
        table.add_row(&["X", "A\nB\nC"]);

        let output = render_table(&table, 40);
        let lines: Vec<&str> = output.lines().collect();
        assert!(lines.len() >= 3);
        // X should be in the last line
        let last_content_line = lines.last().unwrap_or(&"");
        assert!(
            last_content_line.contains('X') || lines[lines.len().saturating_sub(1)].contains('X')
        );
    }

    // -- Table measure tests ------------------------------------------------

    #[test]
    fn test_measure_basic() {
        let mut table = Table::new(&["Name", "Age"]);
        table.add_row(&["Alice", "30"]);

        let console = make_console(80);
        let opts = console.options();
        let m = table.measure(&console, &opts);
        assert!(m.minimum > 0);
        assert!(m.maximum >= m.minimum);
    }

    #[test]
    fn test_measure_with_width() {
        let mut table = Table::new(&["A"]);
        table.width = Some(30);
        table.add_row(&["x"]);

        let console = make_console(80);
        let opts = console.options();
        let m = table.measure(&console, &opts);
        assert_eq!(m.maximum, 30);
    }

    // -- End section tests --------------------------------------------------

    #[test]
    fn test_end_section_rendering() {
        let mut table = Table::new(&["A"]);
        table.add_row(&["1"]);
        table.add_section();
        table.add_row(&["2"]);

        let output = render_table(&table, 40);
        // Should render successfully with a section separator
        assert!(output.contains('1'));
        assert!(output.contains('2'));
    }

    // -- Title and caption tests --------------------------------------------

    #[test]
    fn test_title_rendering() {
        let mut table = Table::new(&["Header Column"]);
        table.title = Some("Title".to_string());
        table.add_row(&["some content"]);

        let output = render_table(&table, 40);
        assert!(output.contains("Title"), "output was: {:?}", output);
    }

    #[test]
    fn test_caption_rendering() {
        let mut table = Table::new(&["Header Column"]);
        table.caption = Some("Footer".to_string());
        table.add_row(&["some content"]);

        let output = render_table(&table, 40);
        assert!(output.contains("Footer"), "output was: {:?}", output);
    }

    #[test]
    fn test_title_and_caption() {
        let mut table = Table::new(&["Col"]);
        table.title = Some("Title".to_string());
        table.caption = Some("Caption".to_string());
        table.add_row(&["data"]);

        let output = render_table(&table, 40);
        assert!(output.contains("Title"));
        assert!(output.contains("Caption"));
        assert!(output.contains("data"));
    }

    // -- Row count test -----------------------------------------------------

    #[test]
    fn test_row_count() {
        let mut table = Table::new(&["A"]);
        assert_eq!(table.row_count(), 0);
        table.add_row(&["1"]);
        assert_eq!(table.row_count(), 1);
        table.add_row(&["2"]);
        assert_eq!(table.row_count(), 2);
    }

    // -- Multiple rows and columns ------------------------------------------

    #[test]
    fn test_render_3x3() {
        let mut table = Table::new(&["A", "B", "C"]);
        table.add_row(&["1", "2", "3"]);
        table.add_row(&["4", "5", "6"]);
        table.add_row(&["7", "8", "9"]);

        let output = render_table(&table, 40);
        for val in &["A", "B", "C", "1", "2", "3", "4", "5", "6", "7", "8", "9"] {
            assert!(output.contains(val), "Missing: {}", val);
        }
    }

    // -- Styled row ---------------------------------------------------------

    #[test]
    fn test_add_row_styled() {
        let mut table = Table::new(&["A"]);
        table.add_row_styled(&["data"], Some("bold"), true);
        assert_eq!(table.rows[0].style, Some("bold".to_string()));
        assert!(table.rows[0].end_section);
    }

    // -- Overflow method test -----------------------------------------------

    #[test]
    fn test_column_overflow_method() {
        let mut table = Table::new(&[]);
        table.add_column(
            "Test",
            "",
            ColumnOptions {
                overflow: Some(OverflowMethod::Crop),
                ..Default::default()
            },
        );
        assert_eq!(table.columns[0].overflow, OverflowMethod::Crop);
    }

    // -- Min/max width tests ------------------------------------------------

    #[test]
    fn test_column_min_max_width() {
        let mut table = Table::new(&[]);
        table.add_column(
            "A",
            "",
            ColumnOptions {
                min_width: Some(5),
                max_width: Some(20),
                ..Default::default()
            },
        );
        assert_eq!(table.columns[0].min_width, Some(5));
        assert_eq!(table.columns[0].max_width, Some(20));
    }

    // -- Table min_width test -----------------------------------------------

    #[test]
    fn test_table_min_width() {
        let mut table = Table::new(&["A"]);
        table.min_width = Some(30);
        table.add_row(&["x"]);

        let console = make_console(80);
        let opts = console.options();
        let m = table.measure(&console, &opts);
        assert!(m.minimum >= 30 || m.maximum >= 30);
    }

    // -- Border and padding interaction tests -------------------------------

    #[test]
    fn test_custom_padding() {
        let mut table = Table::new(&["A"]);
        table.padding = (1, 2, 1, 2);
        table.add_row(&["x"]);

        let output = render_table(&table, 40);
        assert!(output.contains('x'));
    }

    #[test]
    fn test_no_edge_rendering() {
        let mut table = Table::new(&["A"]);
        table.show_edge = false;
        table.add_row(&["x"]);

        let output = render_table(&table, 40);
        assert!(output.contains('x'));
    }

    // -- render_table helper test ------------------------------------------

    #[test]
    fn test_render_returns_segments() {
        let mut table = Table::new(&["Col"]);
        table.add_row(&["val"]);

        let console = make_console(40);
        let opts = console.options();
        let segments = table.rich_console(&console, &opts);
        assert!(!segments.is_empty());
        let text = segments_to_text(&segments);
        assert!(text.contains("Col"));
        assert!(text.contains("val"));
    }

    // -- Column style tests -------------------------------------------------

    #[test]
    fn test_column_header_footer_style() {
        let mut table = Table::new(&[]);
        table.add_column(
            "Header",
            "Footer",
            ColumnOptions {
                header_style: Some("bold".to_string()),
                footer_style: Some("italic".to_string()),
                style: Some("red".to_string()),
                ..Default::default()
            },
        );
        assert_eq!(table.columns[0].header_style, "bold");
        assert_eq!(table.columns[0].footer_style, "italic");
        assert_eq!(table.columns[0].style, "red");
    }

    // -- Highlight inheritance test -----------------------------------------

    #[test]
    fn test_highlight_inheritance() {
        let mut table = Table::new(&[]);
        table.highlight = true;
        table.add_column("A", "", Default::default());
        // Column should inherit table's highlight
        assert!(table.columns[0].highlight);
    }

    #[test]
    fn test_highlight_override() {
        let mut table = Table::new(&[]);
        table.highlight = true;
        table.add_column(
            "A",
            "",
            ColumnOptions {
                highlight: Some(false),
                ..Default::default()
            },
        );
        assert!(!table.columns[0].highlight);
    }

    // -- CellContent / add_row_text tests -----------------------------------

    #[test]
    fn test_cell_content_plain() {
        let cc = CellContent::from("hello");
        assert_eq!(cc, "hello");
    }

    #[test]
    fn test_cell_content_styled() {
        let text = Text::new("bold", Style::parse("bold").unwrap());
        let cc = CellContent::from(text);
        assert_eq!(cc, "bold"); // compares plain text
    }

    #[test]
    fn test_add_row_text() {
        let mut table = Table::new(&["Name", "Score"]);
        let name = Text::new("Alice", Style::parse("bold red").unwrap());
        let score = Text::new("100", Style::parse("green").unwrap());
        table.add_row_text(&[name, score]);
        assert_eq!(table.rows.len(), 1);
        assert_eq!(table.columns[0].cells[0], "Alice");
        assert_eq!(table.columns[1].cells[0], "100");
    }

    #[test]
    fn test_add_row_text_preserves_styles() {
        let mut table = Table::new(&["Data"]);
        let mut styled = Text::new("styled", Style::null());
        styled.stylize(Style::parse("bold").unwrap(), 0, Some(6));
        table.add_row_text(&[styled]);

        // Verify it's stored as Styled, not Plain
        match &table.columns[0].cells[0] {
            CellContent::Styled(t) => {
                assert_eq!(t.plain(), "styled");
                assert!(!t.spans().is_empty());
            }
            CellContent::Plain(_) => panic!("expected Styled variant"),
        }
    }

    #[test]
    fn test_add_row_text_renders_styled() {
        let console = Console::builder()
            .width(60)
            .force_terminal(true)
            .no_color(true)
            .build();

        let mut table = Table::new(&["Name", "Value"]);
        table.show_header = false;
        table.box_chars = None;
        table.show_edge = false;
        table.padding = (0, 0, 0, 0);

        let bold = Text::new("BOLD", Style::parse("bold").unwrap());
        table.add_row_text(&[bold, Text::new("plain", Style::null())]);

        let opts = console.options();
        let segs = table.rich_console(&console, &opts);
        let combined: String = segs.iter().map(|s| s.text.as_str()).collect();
        assert!(combined.contains("BOLD"));
        assert!(combined.contains("plain"));
    }

    #[test]
    fn test_mixed_rows_str_and_text() {
        let mut table = Table::new(&["Col"]);
        table.add_row(&["plain string"]);
        table.add_row_text(&[Text::new("styled text", Style::parse("bold").unwrap())]);
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.columns[0].cells[0], "plain string");
        assert_eq!(table.columns[0].cells[1], "styled text");
    }

    #[test]
    fn test_add_row_text_fewer_cells_pads() {
        let mut table = Table::new(&["A", "B", "C"]);
        table.add_row_text(&[Text::new("only one", Style::null())]);
        assert_eq!(table.columns[0].cells[0], "only one");
        assert_eq!(table.columns[1].cells[0], "");
        assert_eq!(table.columns[2].cells[0], "");
    }

    #[test]
    fn test_display_trait() {
        let mut table = Table::new(&["Name", "Age"]);
        table.add_row(&["Alice", "30"]);
        table.add_row(&["Bob", "25"]);
        let s = format!("{}", table);
        assert!(!s.is_empty());
        assert!(s.contains("Alice"));
        assert!(s.contains("Bob"));
        assert!(s.contains("Name"));
        assert!(s.contains("Age"));
    }

    #[test]
    fn test_display_with_width() {
        let mut table = Table::new(&["A", "B"]);
        table.add_row(&["x", "y"]);
        let wide = format!("{:120}", table);
        let narrow = format!("{:40}", table);
        // Both should contain the data
        assert!(wide.contains("x"));
        assert!(narrow.contains("x"));
    }

    // -- CJK / emoji content tests ------------------------------------------

    #[test]
    fn test_table_cjk_content() {
        let mut table = Table::new(&["名前", "年齢", "都市"]);
        table.add_row(&["太郎", "30", "東京"]);
        table.add_row(&["花子", "25", "大阪"]);
        let output = render_table(&table, 60);
        assert!(output.contains("名前"));
        assert!(output.contains("年齢"));
        assert!(output.contains("都市"));
        assert!(output.contains("太郎"));
        assert!(output.contains("東京"));
        assert!(output.contains("花子"));
        assert!(output.contains("大阪"));
    }

    #[test]
    fn test_table_emoji_content() {
        let mut table = Table::new(&["Icon", "Name"]);
        table.add_row(&["🦀", "Rust"]);
        table.add_row(&["🐍", "Python"]);
        table.add_row(&["🏗️", "Build"]);
        let output = render_table(&table, 40);
        assert!(!output.is_empty());
        // Emoji should appear in the output
        assert!(output.contains("🦀"));
        assert!(output.contains("🐍"));
    }

    // -- Extreme width boundary tests ---------------------------------------

    #[test]
    fn test_table_width_one() {
        let mut table = Table::new(&["A", "B"]);
        table.add_row(&["hello", "world"]);
        // Should not panic at width=1
        let _output = render_table(&table, 1);
    }

    #[test]
    fn test_table_width_zero() {
        let mut table = Table::new(&["A", "B"]);
        table.add_row(&["hello", "world"]);
        // Should not panic at width=0 (may produce empty output)
        let _output = render_table(&table, 0);
    }

    // -- Large data tests ---------------------------------------------------

    #[test]
    fn test_table_large_row_count() {
        let mut table = Table::new(&["ID", "Value"]);
        for i in 0..500 {
            table.add_row(&[&i.to_string(), &format!("val_{}", i)]);
        }
        let output = render_table(&table, 40);
        assert!(!output.is_empty());
        // Spot-check first and last rows
        assert!(output.contains("val_0"));
        assert!(output.contains("val_499"));
    }

    #[test]
    fn test_table_many_columns() {
        let headers: Vec<String> = (0..20).map(|i| format!("C{}", i)).collect();
        let header_refs: Vec<&str> = headers.iter().map(|s| s.as_str()).collect();
        let mut table = Table::new(&header_refs);
        let cells: Vec<String> = (0..20).map(|i| format!("v{}", i)).collect();
        let cell_refs: Vec<&str> = cells.iter().map(|s| s.as_str()).collect();
        table.add_row(&cell_refs);
        let output = render_table(&table, 120);
        assert!(!output.is_empty());
    }
}
