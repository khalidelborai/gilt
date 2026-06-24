//! Table module -- rich table rendering with columns, rows, and box borders.
//!

use crate::console::{Console, ConsoleOptions, ConsoleOptionsUpdates, Renderable, RenderableArc};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::Style;
use crate::text::{JustifyMethod, OverflowMethod, Text};
use crate::utils::align_widget::VerticalAlign;
use crate::utils::box_chars::{BoxChars, RowLevel, HEAVY_HEAD};
use crate::utils::ratio::{ratio_distribute, ratio_reduce};
use crate::widgets::table::{CellContent, Column, ColumnOptions, Row};
use std::sync::Arc;

/// A single cell in the table (internal).
pub(crate) struct CellInfo {
    pub(crate) style: Style,
    /// Pre-resolved `Text` used for plain/styled cells (includes any left/right
    /// padding spaces embedded by `get_cells`).  For `CellContent::Renderable`
    /// cells this is `Text::empty()` — rendering is done via `renderable_arc`.
    pub(crate) renderable: Text,
    /// For `CellContent::Renderable` cells: the original `Arc` kept so the
    /// cell widget can be rendered at the resolved column width at render time
    /// (fixing audit gap #19).  `None` for plain/styled cells.
    pub(crate) renderable_arc: Option<RenderableArc>,
    /// Effective left-padding width for `Renderable` cells (columns of spaces to
    /// prepend to every rendered line so the widget's geometry matches the plain-
    /// cell path which embeds padding in the `Text`).  Zero for plain/styled cells.
    pub(crate) eff_left_pad: usize,
    /// Effective right-padding width for `Renderable` cells.  Zero for plain/styled.
    pub(crate) eff_right_pad: usize,
    pub(crate) vertical: VerticalAlign,
}

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
    /// Build a table from headers paired with their justification —
    /// the v1.0 shorthand for the common "header + alignment" case.
    /// Saves the 7-line `add_column(_, _, ColumnOptions { justify: ... })`
    /// dance per column.
    ///
    /// Cells in the table parse markup (`"[bold red]hi[/]"`) by default —
    /// no need to wrap each value in `Text::from_markup`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::table::Table;
    /// use gilt::text::JustifyMethod;
    ///
    /// let mut t = Table::with_columns([
    ///     ("PID", JustifyMethod::Right),
    ///     ("Command", JustifyMethod::Left),
    ///     ("CPU %", JustifyMethod::Right),
    /// ]);
    /// t.add_row(&["1234", "rustc", "[bold red]42.1%[/]"]);
    /// ```
    pub fn with_columns<I, S>(columns: I) -> Self
    where
        I: IntoIterator<Item = (S, JustifyMethod)>,
        S: AsRef<str>,
    {
        let mut table = Self::new(&[]);
        for (header, justify) in columns {
            table.add_column(
                header.as_ref(),
                "",
                ColumnOptions {
                    justify: Some(justify),
                    ..Default::default()
                },
            );
        }
        table
    }

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

    // -- Builder methods ----------------------------------------------------

    /// Set the table title (builder pattern).
    #[must_use]
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Set the table caption (builder pattern).
    #[must_use]
    pub fn with_caption(mut self, caption: &str) -> Self {
        self.caption = Some(caption.to_string());
        self
    }

    /// Set the style for the title text (builder pattern).
    #[must_use]
    pub fn with_title_style(mut self, style: &str) -> Self {
        self.title_style = style.to_string();
        self
    }

    /// Set the style for the caption text (builder pattern).
    #[must_use]
    pub fn with_caption_style(mut self, style: &str) -> Self {
        self.caption_style = style.to_string();
        self
    }

    /// Set the style for the header row (builder pattern).
    #[must_use]
    pub fn with_header_style(mut self, style: &str) -> Self {
        self.header_style = style.to_string();
        self
    }

    /// Set the style for the footer row (builder pattern).
    #[must_use]
    pub fn with_footer_style(mut self, style: &str) -> Self {
        self.footer_style = style.to_string();
        self
    }

    /// Set the style for the table border (builder pattern).
    #[must_use]
    pub fn with_border_style(mut self, style: &str) -> Self {
        self.border_style = style.to_string();
        self
    }

    /// Set the overall table style (builder pattern).
    #[must_use]
    pub fn with_style(mut self, style: &str) -> Self {
        self.style = style.to_string();
        self
    }

    /// Set the box-drawing character set (builder pattern).
    ///
    /// Pass `None` to disable borders entirely.
    #[must_use]
    pub fn with_box_chars(mut self, chars: Option<&'static BoxChars>) -> Self {
        self.box_chars = chars;
        self
    }

    /// Set whether to show horizontal separator lines between rows (builder pattern).
    #[must_use]
    pub fn with_show_lines(mut self, show: bool) -> Self {
        self.show_lines = show;
        self
    }

    /// Set whether to show the header row (builder pattern).
    #[must_use]
    pub fn with_show_header(mut self, show: bool) -> Self {
        self.show_header = show;
        self
    }

    /// Set whether to show the footer row (builder pattern).
    #[must_use]
    pub fn with_show_footer(mut self, show: bool) -> Self {
        self.show_footer = show;
        self
    }

    /// Set whether to show the left and right border edges (builder pattern).
    #[must_use]
    pub fn with_show_edge(mut self, show: bool) -> Self {
        self.show_edge = show;
        self
    }

    /// Set whether the table should expand to fill available width (builder pattern).
    #[must_use]
    pub fn with_expand(mut self, expand: bool) -> Self {
        self.expand_flag = expand;
        self
    }

    /// Set the fixed table width (builder pattern).
    ///
    /// Setting a width implies `expand`.
    #[must_use]
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the minimum table width constraint (builder pattern).
    #[must_use]
    pub fn with_min_width(mut self, min_width: usize) -> Self {
        self.min_width = Some(min_width);
        self
    }

    /// Set alternating row styles (builder pattern).
    ///
    /// Styles are cycled by row index.
    #[must_use]
    pub fn with_row_styles(mut self, styles: Vec<String>) -> Self {
        self.row_styles = styles;
        self
    }

    /// Set cell padding as `(top, right, bottom, left)` (builder pattern).
    #[must_use]
    pub fn with_padding(mut self, padding: (usize, usize, usize, usize)) -> Self {
        self.padding = padding;
        self
    }

    /// Set whether to collapse inter-column padding (builder pattern).
    #[must_use]
    pub fn with_collapse_padding(mut self, collapse: bool) -> Self {
        self.collapse_padding = collapse;
        self
    }

    /// Set whether to add padding at the left and right table edges (builder pattern).
    #[must_use]
    pub fn with_pad_edge(mut self, pad: bool) -> Self {
        self.pad_edge = pad;
        self
    }

    /// Set the number of extra blank lines between rows (builder pattern).
    #[must_use]
    pub fn with_leading(mut self, leading: usize) -> Self {
        self.leading = leading;
        self
    }

    /// Set whether to substitute box characters on legacy terminals (builder pattern).
    #[must_use]
    pub fn with_safe_box(mut self, safe: Option<bool>) -> Self {
        self.safe_box = safe;
        self
    }

    /// Set horizontal justification for the title (builder pattern).
    #[must_use]
    pub fn with_title_justify(mut self, justify: JustifyMethod) -> Self {
        self.title_justify = justify;
        self
    }

    /// Set horizontal justification for the caption (builder pattern).
    #[must_use]
    pub fn with_caption_justify(mut self, justify: JustifyMethod) -> Self {
        self.caption_justify = justify;
        self
    }

    /// Set whether to enable syntax highlighting for cell content (builder pattern).
    #[must_use]
    pub fn with_highlight(mut self, highlight: bool) -> Self {
        self.highlight = highlight;
        self
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
            header_text: None,
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

    /// Add a column with a pre-styled `Text` header.
    pub fn add_column_text(&mut self, header_text: Text, footer: &str, opts: ColumnOptions) {
        let plain = header_text.plain().to_string();
        let index = self.columns.len();
        let column = Column {
            header: plain,
            header_text: Some(header_text),
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
    /// let bold_name = Text::new("Alice", Style::parse("bold"));
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

    /// Add a row where each cell holds an arbitrary [`Renderable`] widget.
    ///
    /// This is the richest cell variant: any widget that implements
    /// [`Renderable`] (e.g. [`Panel`](crate::panel::Panel),
    /// [`Tree`](crate::tree::Tree), a nested [`Table`]) can be placed in a
    /// table cell.  Wrap the widget in an [`Arc`] so it is cheap to clone
    /// (the table stores cells in each column's `cells` vec).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::table::Table;
    /// use gilt::text::Text;
    /// use gilt::style::Style;
    /// use std::sync::Arc;
    ///
    /// let mut table = Table::new(&["Widget"]);
    /// let cell_text = Arc::new(Text::new("hello", Style::null()));
    /// table.add_row_renderable(vec![cell_text]);
    /// assert_eq!(table.row_count(), 1);
    /// ```
    pub fn add_row_renderable(&mut self, cells: Vec<Arc<dyn Renderable + Send + Sync>>) {
        self.add_row_renderable_styled(cells, None, false);
    }

    /// Add a row of [`Renderable`] cells with an optional row style and section break.
    pub fn add_row_renderable_styled(
        &mut self,
        cells: Vec<Arc<dyn Renderable + Send + Sync>>,
        style: Option<&str>,
        end_section: bool,
    ) {
        let contents: Vec<CellContent> = cells.into_iter().map(CellContent::Renderable).collect();
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
            col_spans: Vec::new(),
        });
    }

    /// Add a row where cells can span multiple columns.
    ///
    /// `cells[i]` is the text content of the i-th logical cell; `spans[i]`
    /// is how many physical columns that cell occupies (1 = normal).  If
    /// `spans` is shorter than `cells`, the missing tail entries default to 1.
    ///
    /// A spanned cell occupies its own column width plus the widths of the
    /// next `k-1` columns and the `k-1` inter-column separators between them.
    /// The physical columns consumed by a span emit nothing for that row.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::table::Table;
    ///
    /// // 3-column table; row 1 has one cell that spans all 3 columns.
    /// let mut table = Table::new(&["A", "B", "C"]);
    /// table.add_row(&["a", "b", "c"]);
    /// table.add_row_spanned(&["wide cell"], &[3]);
    /// let output = format!("{table}");
    /// assert!(output.contains("wide cell"));
    /// ```
    pub fn add_row_spanned(&mut self, cells: &[&str], spans: &[usize]) {
        // Convert &str cells to CellContent.
        let contents: Vec<CellContent> = cells.iter().map(|&s| CellContent::from(s)).collect();

        let num_columns = self.columns.len();

        // Expand logical cells into physical columns based on spans.
        // Each logical cell[i] with span[i]=k occupies columns
        // [physical_col, physical_col+k).  Columns consumed by a span after
        // the first in that group store a sentinel (empty placeholder) so the
        // column vec lengths stay consistent.
        let mut physical_cells: Vec<CellContent> = Vec::with_capacity(num_columns);
        let mut col_spans_vec: Vec<usize> = Vec::new();
        let mut phys = 0usize;

        for (i, cell_val) in contents.into_iter().enumerate() {
            if phys >= num_columns {
                break;
            }
            let span = if i < spans.len() { spans[i].max(1) } else { 1 };
            // Cap span at remaining columns.
            let capped_span = span.min(num_columns - phys);

            physical_cells.push(cell_val);
            col_spans_vec.push(capped_span);
            phys += capped_span;

            // Fill spanned-over slots with placeholders so column lengths stay equal.
            // Note: phys was already fully incremented by capped_span above.
            for _ in 1..capped_span {
                physical_cells.push(CellContent::Plain(String::new()));
                col_spans_vec.push(0); // 0 = "consumed by previous span"
            }
        }

        // Pad to num_columns if needed.
        while physical_cells.len() < num_columns {
            physical_cells.push(CellContent::Plain(String::new()));
            col_spans_vec.push(1);
        }

        // Push cells into column vecs, creating columns if needed.
        for (i, cell_val) in physical_cells.into_iter().enumerate() {
            if i >= self.columns.len() {
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
            style: None,
            end_section: false,
            col_spans: col_spans_vec,
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

    /// Get the padding width (left + right) for a column, considering
    /// `collapse_padding` and `pad_edge`.
    ///
    /// `collapse_padding` removes the **left** padding of every column
    /// after the first — so consecutive cells visually share a single
    /// padding gap rather than doubling it. `pad_edge` controls whether
    /// the very-first-column left and last-column right pad against the
    /// table border.
    ///
    /// Both `measure_column` and the cell-padding pass in `get_cells`
    /// must agree on this width or rendered text will be truncated.
    pub fn get_padding_width(&self, column_index: usize) -> usize {
        let (_, pad_right, _, pad_left) = self.padding;

        let mut pl = pad_left;
        let mut pr = pad_right;

        if self.collapse_padding && column_index > 0 {
            // rich: max(0, left - right) so shared gap = right only when left > right
            pl = pl.saturating_sub(pr);
        }

        if !self.pad_edge {
            if column_index == 0 {
                pl = 0;
            }
            if column_index + 1 == self.columns.len() {
                pr = 0;
            }
        }

        pl + pr
    }

    /// Measure a column, returning its minimum and maximum width including padding.
    #[allow(dead_code)]
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
        // Slow path — builds the cells. Internal callers in `gilt_console`
        // should use [`Self::measure_column_with_cells`] to avoid the rebuild.
        let cells = self.get_cells(console, column.index, column);
        self.measure_with_cells(console, options, column, padding_width, &cells)
    }

    /// Measure a column using pre-built `CellInfo`s. Avoids the second
    /// `get_cells` pass that the public [`measure_column`] would do.
    fn measure_column_with_cells(
        &self,
        console: &Console,
        options: &ConsoleOptions,
        column: &Column,
        cells: &[CellInfo],
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
        self.measure_with_cells(console, options, column, padding_width, cells)
    }

    /// Shared measurement body for both [`measure_column`] and
    /// [`measure_column_with_cells`].
    ///
    /// For `CellContent::Renderable` cells (those carrying a `renderable_arc`)
    /// the measurement is obtained via `gilt_measure` so that the widget's own
    /// sizing logic (e.g. a `Panel`'s content-fit mode) is honoured.
    /// For plain/styled cells the no-arg `Text::measure()` is used as before.
    fn measure_with_cells(
        &self,
        console: &Console,
        options: &ConsoleOptions,
        column: &Column,
        padding_width: usize,
        cells: &[CellInfo],
    ) -> Measurement {
        let max_width = options.max_width;
        let mut min_w = 0usize;
        let mut max_w = 0usize;
        for cell in cells {
            let m = if let Some(ref arc) = cell.renderable_arc {
                // Renderable cell: delegate to the widget's own measure.
                arc.gilt_measure(console, options)
            } else {
                cell.renderable.measure()
            };
            if m.minimum + padding_width > min_w {
                min_w = m.minimum + padding_width;
            }
            if m.maximum + padding_width > max_w {
                max_w = m.maximum + padding_width;
            }
        }
        if min_w == 0 {
            min_w = 1;
        }
        if max_w == 0 {
            max_w = max_width;
        }
        let measurement = Measurement::new(min_w, max_w).with_maximum(max_width);
        measurement.clamp(
            column.min_width.map(|mw| mw + padding_width),
            column.max_width.map(|mw| mw + padding_width),
        )
    }

    /// Get all cells for a column, including header/footer, with styles applied.
    pub(crate) fn get_cells(
        &self,
        console: &Console,
        column_index: usize,
        column: &Column,
    ) -> Vec<CellInfo> {
        let mut cells = Vec::new();

        if self.show_header {
            let header_style = console
                .get_style(&self.header_style)
                .unwrap_or_else(|_| Style::null())
                + console
                    .get_style(&column.header_style)
                    .unwrap_or_else(|_| Style::null());
            let text = if let Some(ref ht) = column.header_text {
                ht.clone()
            } else {
                console.render_str(&column.header, None, None, None)
            };
            cells.push(CellInfo {
                style: header_style,
                renderable: text,
                renderable_arc: None,
                eff_left_pad: 0,
                eff_right_pad: 0,
                vertical: column.vertical,
            });
        }

        let cell_style = console
            .get_style(&column.style)
            .unwrap_or_else(|_| Style::null());
        for cell_content in &column.cells {
            // For Renderable cells keep the Arc so the render loop can later
            // render it at the resolved column width (audit gap #19 fix).
            // For Plain/Styled, pre-resolve to Text as before.
            let (text, arc) = match cell_content {
                CellContent::Renderable(r) => (Text::empty(), Some(Arc::clone(r))),
                _ => (cell_content.resolve(console), None),
            };
            cells.push(CellInfo {
                style: cell_style.clone(),
                renderable: text,
                renderable_arc: arc,
                eff_left_pad: 0,
                eff_right_pad: 0,
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
                renderable_arc: None,
                eff_left_pad: 0,
                eff_right_pad: 0,
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
                    // Mirror get_padding_width: max(0, left - right) so shared gap
                    // = right only when left > right (rich parity).
                    left = left.saturating_sub(right);
                }

                if !self.pad_edge {
                    if first_column {
                        left = 0;
                    }
                    if last_column {
                        right = 0;
                    }
                    // Top/bottom padding is handled during row rendering, not
                    // on the cell text itself.
                    let _ = (first_row, last_row);
                }

                if cell.renderable_arc.is_some() {
                    // Renderable cells: record the effective padding so the render
                    // loop can apply it as space columns around the widget output,
                    // matching exactly the geometry of the plain-cell path.
                    cell.eff_left_pad = left;
                    cell.eff_right_pad = right;
                } else {
                    // Plain/styled cells: embed padding as spaces in the Text.
                    if left > 0 {
                        cell.renderable.pad_left(left, ' ');
                    }
                    if right > 0 {
                        cell.renderable.pad_right(right, ' ');
                    }
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
        // Public entry — builds cells internally for callers that don't have
        // them pre-computed. `render_table` uses
        // [`calculate_column_widths_with_cells`] instead.
        let column_cells: Vec<Vec<CellInfo>> = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, col)| self.get_cells(console, i, col))
            .collect();
        self.calculate_column_widths_with_cells(console, options, &column_cells)
    }

    /// Internal: same as [`calculate_column_widths`] but uses pre-built
    /// per-column cells, avoiding a second round of `get_cells` calls.
    pub(crate) fn calculate_column_widths_with_cells(
        &self,
        console: &Console,
        options: &ConsoleOptions,
        column_cells: &[Vec<CellInfo>],
    ) -> Vec<usize> {
        let max_width = options.max_width;
        let columns = &self.columns;

        let width_ranges: Vec<Measurement> = columns
            .iter()
            .enumerate()
            .map(|(i, col)| self.measure_column_with_cells(console, options, col, &column_cells[i]))
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

            // Re-measure columns at new widths using the same cached cells.
            let new_ranges: Vec<Measurement> = widths
                .iter()
                .zip(columns.iter())
                .enumerate()
                .map(|(i, (&w, col))| {
                    self.measure_column_with_cells(
                        console,
                        &options.update_width(w),
                        col,
                        &column_cells[i],
                    )
                })
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
            // Pre-allocate ratios vec once; updated in each loop iteration.
            let mut ratios: Vec<usize> = vec![0usize; widths.len()];
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

                for (ratio, (&w, &allow)) in
                    ratios.iter_mut().zip(widths.iter().zip(wrapable.iter()))
                {
                    *ratio = if w == max_column && allow { 1 } else { 0 };
                }

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

    /// Internal: render the table body (borders + cells) using pre-built
    /// per-column cells. Called by `gilt_console` so the cell construction
    /// (which calls `console.render_str` for every header / data / footer
    /// string) happens exactly once per render — measure and render share it.
    pub(crate) fn render_table_with_cells(
        &self,
        console: &Console,
        options: &ConsoleOptions,
        widths: &[usize],
        column_cells: Vec<Vec<CellInfo>>,
    ) -> Vec<Segment> {
        let mut segments: Vec<Segment> = Vec::new();

        let table_style = console
            .get_style(&self.style)
            .unwrap_or_else(|_| Style::null());
        let border_style = table_style.clone()
            + console
                .get_style(&self.border_style)
                .unwrap_or_else(|_| Style::null());

        // Transpose to row_cells: each row -> list of cells (one per column)
        let num_rows = column_cells.iter().map(|c| c.len()).max().unwrap_or(0);
        let num_cols = column_cells.len();

        // Get box (with substitution).
        // rich parity: legacy_windows subs are gated by safe (Box.substitute
        // only applies them when safe=True). The outer guard `ascii_only || safe`
        // mirrors that — substitute is only called when at least one mode is active.
        let the_box: Option<&BoxChars> = self.box_chars.map(|b| {
            let safe = self.safe_box.unwrap_or(true);
            let ascii_only = options.ascii_only();
            let legacy_windows = options.legacy_windows;
            let substituted = if ascii_only || safe {
                b.substitute(ascii_only, legacy_windows)
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

            // Item 5 (P3): get_row_style already calls console.get_style() internally and
            // returns a fully resolved Style. The outer get_style(style_obj.to_string())
            // was a redundant round-trip (Style → String → parse again). Removed.
            let row_style = if header_row || footer_row {
                Style::null()
            } else if let Some(idx) = data_row_index {
                self.get_row_style(console, idx)
            } else {
                Style::null()
            };

            // Look up span info for this data row (empty vec = all-1s / no spans).
            let row_col_spans: &[usize] = if let Some(idx) = data_row_index {
                if idx < self.rows.len() {
                    &self.rows[idx].col_spans
                } else {
                    &[]
                }
            } else {
                &[]
            };

            // Helper: effective span for physical column `col_index`.
            // Returns 0 if this column is consumed by a previous span (skip it),
            // ≥1 otherwise.
            let get_span = |col_index: usize| -> usize {
                if row_col_spans.is_empty() {
                    1
                } else {
                    *row_col_spans.get(col_index).unwrap_or(&1)
                }
            };

            // Compute effective width for a spanned cell:
            // sum of the spanned columns' widths plus the (k-1) separator characters.
            // The separator width is 1 when box_chars is Some, else 0.
            let separator_width: usize = if the_box.is_some() { 1 } else { 0 };

            let spanned_width = |col_index: usize, span: usize| -> usize {
                let mut w = 0usize;
                for k in 0..span {
                    let ci = col_index + k;
                    w += if ci < widths.len() { widths[ci] } else { 1 };
                    if k + 1 < span {
                        w += separator_width;
                    }
                }
                w
            };

            // Render all cells for this row
            let mut rendered_cells: Vec<Vec<Vec<Segment>>> = Vec::with_capacity(num_cols);
            let mut max_height: usize = 1;

            #[allow(clippy::needless_range_loop)] // col_index needed for get_span / spanned_width
            for col_index in 0..num_cols {
                let span = get_span(col_index);

                // span == 0 means this column is consumed by the previous span.
                // Push an empty sentinel so indices stay in sync.
                if span == 0 {
                    rendered_cells.push(Vec::new()); // empty = skip in output
                    continue;
                }

                let width = spanned_width(col_index, span);
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

                let cell_combined_style = cell.style.clone() + row_style.clone();

                // Renderable cells (audit gap #19): render at INNER content width
                // (= column width minus the cell's left/right padding), then prepend/
                // append the padding spaces on every line.  This matches the geometry
                // of the plain-cell path, which embeds padding in the Text object.
                let mut lines = if let Some(ref arc) = cell.renderable_arc {
                    let lpad = cell.eff_left_pad;
                    let rpad = cell.eff_right_pad;
                    let inner_width = width.saturating_sub(lpad + rpad).max(1);
                    let inner_opts = options.with_updates(&ConsoleOptionsUpdates {
                        width: Some(inner_width),
                        justify: Some(Some(column.justify)),
                        no_wrap: Some(column.no_wrap),
                        overflow: Some(Some(column.overflow)),
                        height: Some(None),
                        highlight: Some(Some(column.highlight)),
                        ..Default::default()
                    });
                    let inner_lines = console.render_lines(
                        arc.as_ref(),
                        Some(&inner_opts),
                        Some(&cell_combined_style),
                        true,
                        false,
                    );
                    // Apply left/right padding spaces to every line.
                    if lpad == 0 && rpad == 0 {
                        inner_lines
                    } else {
                        let left_seg = if lpad > 0 {
                            Some(Segment::styled(
                                &" ".repeat(lpad),
                                cell_combined_style.clone(),
                            ))
                        } else {
                            None
                        };
                        let right_seg = if rpad > 0 {
                            Some(Segment::styled(
                                &" ".repeat(rpad),
                                cell_combined_style.clone(),
                            ))
                        } else {
                            None
                        };
                        inner_lines
                            .into_iter()
                            .map(|mut line| {
                                if let Some(ref ls) = left_seg {
                                    line.insert(0, ls.clone());
                                }
                                if let Some(ref rs) = right_seg {
                                    line.push(rs.clone());
                                }
                                line
                            })
                            .collect()
                    }
                } else {
                    let render_options = options.with_updates(&ConsoleOptionsUpdates {
                        width: Some(width),
                        justify: Some(Some(column.justify)),
                        no_wrap: Some(column.no_wrap),
                        overflow: Some(Some(column.overflow)),
                        height: Some(None),
                        highlight: Some(Some(column.highlight)),
                        ..Default::default()
                    });
                    console.render_lines(
                        &cell.renderable,
                        Some(&render_options),
                        Some(&cell_combined_style),
                        true,
                        false,
                    )
                };

                // Apply top/bottom cell padding (rich parity: emit blank lines).
                let (pad_top, _pad_right, pad_bottom, _pad_left) = self.padding;
                if pad_top > 0 || pad_bottom > 0 {
                    let blank_line = vec![Segment::styled(
                        &" ".repeat(width),
                        cell_combined_style.clone(),
                    )];
                    let mut padded = Vec::with_capacity(pad_top + lines.len() + pad_bottom);
                    for _ in 0..pad_top {
                        padded.push(blank_line.clone());
                    }
                    padded.append(&mut lines);
                    for _ in 0..pad_bottom {
                        padded.push(blank_line.clone());
                    }
                    lines = padded;
                }

                max_height = max_height.max(lines.len());
                rendered_cells.push(lines);
            }

            // Apply vertical alignment and set shape
            let row_height = rendered_cells.iter().map(|c| c.len()).max().unwrap_or(1);
            let max_height = row_height.max(max_height);

            let mut shaped_cells: Vec<Vec<Vec<Segment>>> = Vec::with_capacity(num_cols);
            for col_index in 0..num_cols {
                let span = get_span(col_index);

                // Skip consumed columns (they were rendered as part of the spanned cell).
                if span == 0 {
                    shaped_cells.push(Vec::new()); // sentinel
                    continue;
                }

                let width = spanned_width(col_index, span);

                let cell_lines = if col_index < rendered_cells.len() {
                    &rendered_cells[col_index]
                } else {
                    shaped_cells.push(vec![vec![Segment::styled(
                        &" ".repeat(width),
                        Style::null(),
                    )]]);
                    continue;
                };

                // Empty sentinel from a span=0 slot: skip.
                if cell_lines.is_empty() && span == 0 {
                    shaped_cells.push(Vec::new());
                    continue;
                }

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

                // Build the row divider once per row (outside the height loop)
                // to avoid repeated re-construction inside O(height × cols).
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
                    // Track which cells are "visible" (span>=1) so we can
                    // insert dividers only between visible cells, not before/
                    // after consumed-by-span slots.
                    let visible_indices: Vec<usize> = shaped_cells
                        .iter()
                        .enumerate()
                        .filter(|(ci, c)| get_span(*ci) != 0 || !c.is_empty())
                        .map(|(ci, _)| ci)
                        .collect();
                    for (vis_pos, &cell_idx) in visible_indices.iter().enumerate() {
                        let last_visible = vis_pos == visible_indices.len() - 1;
                        let cell = &shaped_cells[cell_idx];
                        if line_no < cell.len() {
                            segments.extend(cell[line_no].iter().cloned());
                        }
                        if !last_visible {
                            // Only emit a divider if the NEXT column is not
                            // consumed by the current span (i.e. get_span > 0).
                            let next_ci = visible_indices[vis_pos + 1];
                            let next_span = get_span(next_ci);
                            if next_span != 0 {
                                segments.push(divider.clone());
                            }
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
                    for (ci, cell) in shaped_cells.iter().enumerate() {
                        // Skip consumed-by-span columns.
                        if get_span(ci) == 0 {
                            continue;
                        }
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

            // Common skip condition: don't emit after the last row, after the
            // header row (header sep already emitted above), or before the footer.
            let skip_sep = last
                || (show_footer && row_index >= num_rows.saturating_sub(2))
                || (show_header && header_row);

            if let Some(b) = the_box {
                if (show_lines || leading > 0 || end_section) && !skip_sep {
                    if leading > 0 {
                        // Item 1 (P2): `leading` inserts blank lines, not Mid box
                        // separator rows. A blank line is a space-padded segment of
                        // the total table width (matching rich's Python behavior).
                        let total_width = widths.iter().sum::<usize>()
                            + if show_edge { 2 } else { 0 }
                            + widths.len().saturating_sub(1);
                        let blank = " ".repeat(total_width);
                        for _ in 0..leading {
                            segments.push(Segment::new(&blank, None, None));
                            segments.push(new_line.clone());
                        }
                    } else {
                        // show_lines or end_section: emit a box row separator.
                        segments.push(Segment::styled(
                            &b.get_row(widths, RowLevel::Row, show_edge),
                            border_style.clone(),
                        ));
                        segments.push(new_line.clone());
                    }
                }
            } else if (show_lines || leading > 0) && !skip_sep {
                // Item 2 (P2): when box_chars is None and show_lines/leading is
                // active, emit blank separator lines between data rows.
                let total_width = widths.iter().sum::<usize>() + widths.len().saturating_sub(1);
                let blank = " ".repeat(total_width);
                let count = if leading > 0 { leading } else { 1 };
                for _ in 0..count {
                    segments.push(Segment::new(&blank, None, None));
                    segments.push(new_line.clone());
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
    /// Used by the [`crate::console::Renderable`] trait to determine how much space the table
    /// requires.
    pub fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        // Fixed width of zero collapses the table entirely (matches rich,
        // which treats `Table(width=0)` as a fully-collapsed measurement).
        if self.width == Some(0) {
            return Measurement::new(0, 0);
        }

        let mut max_width = options.max_width;
        if let Some(w) = self.width {
            max_width = w;
        }

        let extra_width = self.extra_width();
        // Build cells once and reuse for both width calculation and measurement.
        let column_cells: Vec<Vec<CellInfo>> = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, col)| self.get_cells(console, i, col))
            .collect();
        let col_widths = self.calculate_column_widths_with_cells(
            console,
            &options.update_width(max_width.saturating_sub(extra_width)),
            &column_cells,
        );
        let total_max: usize = col_widths.iter().sum::<usize>();

        let measurements: Vec<Measurement> = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                self.measure_column_with_cells(
                    console,
                    &options.update_width(total_max),
                    col,
                    &column_cells[i],
                )
            })
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

#[cfg(test)]
mod padding_tests {
    use super::*;

    #[test]
    fn padding_width_collapsed_zeros_left_only_for_non_first_columns() {
        let mut t = Table::new(&["c0", "c1", "c2"]);
        t.padding = (0, 2, 0, 1); // (top, right, bottom, left)
        t.collapse_padding = true;
        t.pad_edge = true;

        // First column keeps its left padding
        assert_eq!(t.get_padding_width(0), 1 + 2);
        // Non-first columns drop the left padding (collapses to a single gap)
        assert_eq!(t.get_padding_width(1), 2);
        assert_eq!(t.get_padding_width(2), 2);
    }

    #[test]
    fn padding_width_no_pad_edge_zeros_outer_padding() {
        let mut t = Table::new(&["c0", "c1", "c2"]);
        t.padding = (0, 2, 0, 3);
        t.collapse_padding = false;
        t.pad_edge = false;

        // First column: left padding zeroed (against table edge)
        assert_eq!(t.get_padding_width(0), 2);
        // Middle column: full padding
        assert_eq!(t.get_padding_width(1), 3 + 2);
        // Last column: right padding zeroed
        assert_eq!(t.get_padding_width(2), 3);
    }

    #[test]
    fn padding_width_grid_default_is_zero_everywhere() {
        // Table::grid() sets padding=(0,0,0,0); collapse/pad_edge interactions
        // are no-ops because there is no padding to remove.
        let t = Table::grid(&["a", "b", "c"]);
        assert_eq!(t.get_padding_width(0), 0);
        assert_eq!(t.get_padding_width(1), 0);
        assert_eq!(t.get_padding_width(2), 0);
    }

    #[test]
    fn padding_width_grid_with_explicit_padding_collapses_correctly() {
        // After setting padding on a grid (collapse_padding=true, pad_edge=false):
        //   first column: pad_edge zeros pad_left   -> 0+pr
        //   middle:       collapse zeros pad_left   -> 0+pr
        //   last column:  pad_edge zeros pad_right  -> 0+0
        let mut t = Table::grid(&["a", "b", "c"]);
        t.padding = (0, 1, 0, 1);
        assert_eq!(t.get_padding_width(0), 1);
        assert_eq!(t.get_padding_width(1), 1);
        assert_eq!(t.get_padding_width(2), 0);
    }

    #[test]
    fn padding_width_single_column_pad_edge_off_zeros_both_sides() {
        let mut t = Table::new(&["only"]);
        t.padding = (0, 4, 0, 5);
        t.pad_edge = false;
        assert_eq!(t.get_padding_width(0), 0);
    }

    #[test]
    fn padding_width_measure_and_render_paths_agree() {
        // Regression: get_padding_width (measure path) and the per-cell
        // padding loop in get_cells (render path) must compute identical
        // totals or text gets truncated.
        //
        // Both paths now use `left.saturating_sub(right)` for collapse_padding
        // (rich parity: max(0, left - right)).
        let mut t = Table::new(&["a", "b", "c", "d"]);
        t.padding = (0, 2, 0, 3);
        t.collapse_padding = true;
        t.pad_edge = false;

        let (_, pad_right, _, pad_left) = t.padding;
        let n = t.columns.len();
        for col_idx in 0..n {
            // Replicate the get_cells render-time logic (rich parity)
            let first = col_idx == 0;
            let last = col_idx + 1 == n;
            let mut left = pad_left;
            let mut right = pad_right;
            if t.collapse_padding && !first {
                left = left.saturating_sub(right); // rich: max(0, left - right)
            }
            if !t.pad_edge {
                if first {
                    left = 0;
                }
                if last {
                    right = 0;
                }
            }
            assert_eq!(
                t.get_padding_width(col_idx),
                left + right,
                "measure/render disagree at column {col_idx}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Task 2 (v1.8): column-span tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod col_span_tests {
    use super::*;

    /// Helper: render a Table to a plain string (no ANSI).
    fn render_plain(table: &Table) -> String {
        format!("{table}")
    }

    /// A 3-column table where row 1 has a cell spanning ALL 3 columns.
    /// The spanned text must appear on ONE line and the line must be as wide
    /// as the full inner table width (not split by vertical borders in the span).
    #[test]
    fn col_span_full_3col_text_on_one_line() {
        let mut table = Table::new(&["A", "B", "C"]);
        table.box_chars = None;
        table.show_header = false;
        table.padding = (0, 0, 0, 0);
        // Fix column widths so the test is deterministic.
        table.columns[0].width = Some(5);
        table.columns[1].width = Some(5);
        table.columns[2].width = Some(5);

        // row 0: normal row
        table.add_row(&["a", "b", "c"]);
        // row 1: first cell spans all 3 columns
        table.add_row_spanned(&["spanned_text"], &[3]);

        let output = render_plain(&table);

        // The spanned text must appear somewhere in the output.
        assert!(
            output.contains("spanned_text"),
            "spanned text should appear in output: {output:?}"
        );

        // Find the line(s) that contain "spanned_text".
        let span_lines: Vec<&str> = output
            .lines()
            .filter(|l| l.contains("spanned_text"))
            .collect();
        // The spanned cell should appear on exactly one line.
        assert_eq!(
            span_lines.len(),
            1,
            "spanned text should be on one line, got: {span_lines:?}"
        );
    }

    /// Partial span: first cell spans 2 of 3 columns, third cell normal.
    #[test]
    fn col_span_partial_2of3_first_cell() {
        let mut table = Table::new(&["X", "Y", "Z"]);
        table.box_chars = None;
        table.show_header = false;
        table.padding = (0, 0, 0, 0);
        table.columns[0].width = Some(4);
        table.columns[1].width = Some(4);
        table.columns[2].width = Some(4);

        // row 0: normal
        table.add_row(&["x", "y", "z"]);
        // row 1: first cell spans 2 cols, second cell is normal (col 2)
        table.add_row_spanned(&["wide", "z2"], &[2, 1]);

        let output = render_plain(&table);
        assert!(
            output.contains("wide"),
            "partial spanned text should appear: {output:?}"
        );
        assert!(
            output.contains("z2"),
            "non-spanned cell in partial span row should appear: {output:?}"
        );
    }

    /// Default (no spans) path is unchanged — all existing rows render correctly.
    #[test]
    fn col_span_default_path_unchanged() {
        let mut table = Table::new(&["Name", "Age"]);
        table.add_row(&["Alice", "30"]);
        table.add_row(&["Bob", "25"]);
        let output = render_plain(&table);
        assert!(output.contains("Alice"), "Alice should appear: {output:?}");
        assert!(output.contains("Bob"), "Bob should appear: {output:?}");
        assert!(output.contains("30"), "30 should appear: {output:?}");
        assert!(output.contains("25"), "25 should appear: {output:?}");
    }
}

#[cfg(test)]
mod renderable_cell_tests {
    use super::*;
    use crate::panel::Panel;
    use crate::text::Text;
    use std::sync::Arc;

    /// Render the table to a plain string via the `Display` impl.
    fn render(table: &Table) -> String {
        format!("{table}")
    }

    #[test]
    fn add_row_renderable_renders_simple_text() {
        // An Arc<Text> implements Renderable.  The cell content should appear
        // in the rendered table output.
        let mut table = Table::new(&["Widget"]);
        table.box_chars = None;
        table.show_header = false;
        table.padding = (0, 0, 0, 0);

        let cell_text: Arc<dyn Renderable + Send + Sync> =
            Arc::new(Text::new("hello world", Style::null()));
        table.add_row_renderable(vec![cell_text]);

        assert_eq!(table.row_count(), 1);
        let output = render(&table);
        assert!(
            output.contains("hello world"),
            "expected 'hello world' in output, got: {output:?}"
        );
    }

    #[test]
    fn add_row_renderable_with_panel_inside_cell() {
        // A Panel rendered inside a cell should contribute its border
        // characters to the rendered output.
        let mut table = Table::new(&["Widget"]);
        table.box_chars = None;
        table.show_header = false;
        table.padding = (0, 0, 0, 0);
        // Give the column enough width to render a small panel
        table.columns[0].width = Some(20);

        let panel_content = Text::new("inner", Style::null());
        let panel = Panel::new(panel_content);
        let cell: Arc<dyn Renderable + Send + Sync> = Arc::new(panel);
        table.add_row_renderable(vec![cell]);

        assert_eq!(table.row_count(), 1);
        let output = render(&table);
        // Panel border characters should appear somewhere in the output
        assert!(
            output.contains('─') || output.contains('│') || output.contains('╭'),
            "expected panel border chars in output, got: {output:?}"
        );
    }

    /// Bug #19 regression: a Renderable cell (Panel) must be rendered at the
    /// COLUMN width, not at the console's default width (80).
    ///
    /// Before the fix: Panel rendered at 80 cols → top border was ~80 `─` chars
    /// → the table re-wrapped that flat text to the 24-col column, destroying
    /// the geometry (truncated border, misaligned corners).
    ///
    /// After the fix: Panel rendered at column width (24) → top border is
    /// exactly 24 chars wide and the corner characters are both intact.
    #[test]
    fn renderable_cell_panel_renders_at_column_width_not_default() {
        // 24-wide column (no table box, no padding for simplicity).
        let col_width = 24usize;
        let mut table = Table::new(&["Widget"]);
        table.box_chars = None;
        table.show_header = false;
        table.padding = (0, 0, 0, 0);
        table.columns[0].width = Some(col_width);

        let panel = Panel::new(Text::new("inner", Style::null()));
        let cell: Arc<dyn Renderable + Send + Sync> = Arc::new(panel);
        table.add_row_renderable(vec![cell]);

        let output = render(&table);

        // The top border of a Panel at width 24 is: "╭" + 22×"─" + "╮"
        // That exact string must appear in the output.
        let expected_top = format!("╭{}╮", "─".repeat(col_width - 2));
        assert!(
            output.contains(&expected_top),
            "expected panel top border '{expected_top}' in output; got:\n{output}"
        );

        // Also verify the bottom border is correct width.
        let expected_bot = format!("╰{}╯", "─".repeat(col_width - 2));
        assert!(
            output.contains(&expected_bot),
            "expected panel bottom border '{expected_bot}' in output; got:\n{output}"
        );
    }

    /// Regression for the geometry bug introduced by the first audit-gap-19 fix:
    /// a Renderable cell was rendered at the FULL column allocation (content +
    /// padding) while a plain cell had padding embedded as spaces in its Text,
    /// so both occupied `column_width` chars — but the renderable content started
    /// at column 0 while the plain content started at column `left_pad`. This
    /// caused the column borders to appear at different visual positions across rows.
    ///
    /// Fix: render the Renderable at `inner_width = column_width - left_pad -
    /// right_pad` and then prepend/append space segments, matching the plain-cell
    /// geometry exactly.
    ///
    /// This test uses a 1-column borderless table with DEFAULT Table::new()
    /// padding (0,1,0,1) and one plain row + one renderable row.  It asserts
    /// that every non-empty line in the rendered output has the same total visual
    /// width — i.e. the column edges align.
    /// Regression for the geometry bug introduced by the first audit-gap-19 fix:
    /// a Renderable cell was rendered at the FULL column allocation (content +
    /// padding) while a plain cell had padding embedded as spaces in its Text,
    /// so both occupied `column_width` chars — but the renderable content started
    /// at column 0 while the plain content started at column `left_pad`. This
    /// caused the column borders to appear at different visual positions across rows.
    ///
    /// Fix: render the Renderable at `inner_width = column_width - left_pad -
    /// right_pad` and then prepend/append space segments, matching the plain-cell
    /// geometry exactly.
    ///
    /// This test uses a 1-column borderless table with DEFAULT Table::new()
    /// padding (0,1,0,1) and one plain row + one renderable row.  It asserts
    /// that every non-empty line in the rendered output has the same total visual
    /// width — i.e. the column edges align.
    #[test]
    fn renderable_cell_aligns_with_plain_cell_under_nonzero_padding() {
        use crate::utils::cells::cell_len;

        // Default padding (0, right=1, 0, left=1) — this is what Table::new() uses.
        // `column.width` is the CONTENT width; the full column allocation is
        // content_width + left_pad + right_pad = 10 + 1 + 1 = 12.
        let content_width = 10usize;
        let left_pad = 1usize;
        let right_pad = 1usize;
        let expected_total_width = content_width + left_pad + right_pad; // 12

        let mut table = Table::new(&["Col"]);
        table.box_chars = None;
        table.show_header = false;
        // padding = (top, right, bottom, left) — Table::new default is (0,1,0,1)
        assert_eq!(
            table.padding,
            (0, 1, 0, 1),
            "test relies on default padding"
        );
        table.columns[0].width = Some(content_width);

        // Row 0: plain text cell
        table.add_row(&["hello"]);
        // Row 1: renderable cell (Arc<Text>)
        let cell: Arc<dyn Renderable + Send + Sync> = Arc::new(Text::new("world", Style::null()));
        table.add_row_renderable(vec![cell]);

        let output = render(&table);

        // Every non-empty line must be exactly `expected_total_width` visual chars wide.
        let line_widths: Vec<usize> = output
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| cell_len(l))
            .collect();

        for (i, &w) in line_widths.iter().enumerate() {
            assert_eq!(
                w,
                expected_total_width,
                "line {i} has width {w}, expected {expected_total_width}; \
                output (each line repr'd with delimiters):\n{}",
                output
                    .lines()
                    .enumerate()
                    .map(|(n, l)| format!("  [{n}] |{l}|"))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        }
    }

    #[test]
    fn add_row_renderable_falls_back_to_text_path_for_text_input() {
        // Using add_row_renderable(Arc<Text>) should produce the same final
        // rendered output as add_row_text(&[Text]).
        let text_value = "compare me";

        let mut table_via_text = Table::new(&["Col"]);
        table_via_text.box_chars = None;
        table_via_text.show_header = false;
        table_via_text.padding = (0, 0, 0, 0);
        table_via_text.add_row_text(&[Text::new(text_value, Style::null())]);

        let mut table_via_renderable = Table::new(&["Col"]);
        table_via_renderable.box_chars = None;
        table_via_renderable.show_header = false;
        table_via_renderable.padding = (0, 0, 0, 0);
        let cell: Arc<dyn Renderable + Send + Sync> =
            Arc::new(Text::new(text_value, Style::null()));
        table_via_renderable.add_row_renderable(vec![cell]);

        let out_text = render(&table_via_text);
        let out_renderable = render(&table_via_renderable);

        assert_eq!(
            out_text, out_renderable,
            "add_row_text and add_row_renderable(Arc<Text>) should produce identical output"
        );
    }
}

#[cfg(test)]
mod parity_batch_7_10_tests {
    use super::*;
    use crate::box_chars::SQUARE;

    fn render_plain(table: &Table) -> String {
        format!("{table}")
    }

    // ---- Item 1: leading inserts blank lines, not Mid box chars ----

    /// Table with a box and `leading=2`: the segments between rows must NOT
    /// contain any Mid-level box separator characters (e.g. '─', '├', '┤', '┼').
    /// Instead each leading line must be a blank space-padded line.
    #[test]
    fn test_leading_inserts_blank_lines_not_box_chars() {
        let mut table = Table::new(&["Col1", "Col2"]);
        table.show_header = false;
        table.padding = (0, 0, 0, 0);
        table.columns[0].width = Some(5);
        table.columns[1].width = Some(5);
        table.leading = 2;
        table.add_row(&["aaaaa", "bbbbb"]);
        table.add_row(&["ccccc", "ddddd"]);

        let output = render_plain(&table);

        // Mid box chars that would appear in a box separator row.
        // If leading incorrectly uses get_row(Mid), these will appear between data rows.
        let mid_chars = ['─', '├', '┤', '┼', '╞', '╡', '╪'];
        // Lines between the two data rows should not contain box chars.
        let lines: Vec<&str> = output.lines().collect();
        // Find the two data lines and check what's in between.
        let row1_pos = lines.iter().position(|l| l.contains("aaaaa"));
        let row2_pos = lines.iter().position(|l| l.contains("ccccc"));
        assert!(
            row1_pos.is_some(),
            "first data row missing from output:\n{output}"
        );
        assert!(
            row2_pos.is_some(),
            "second data row missing from output:\n{output}"
        );
        let r1 = row1_pos.unwrap();
        let r2 = row2_pos.unwrap();
        assert!(r2 > r1, "row2 should come after row1");
        // The lines between r1 and r2 must not contain mid box chars.
        for between_line in &lines[r1 + 1..r2] {
            for ch in mid_chars {
                assert!(
                    !between_line.contains(ch),
                    "leading line '{between_line}' contains Mid box char '{ch}'; \
                    full output:\n{output}"
                );
            }
            // Each leading line should be blank (only spaces).
            assert!(
                between_line.chars().all(|c| c == ' '),
                "leading line '{between_line}' is not a blank space line; \
                full output:\n{output}"
            );
        }
        // There must be exactly 2 blank lines between the data rows.
        let blank_between = lines[r1 + 1..r2].len();
        assert_eq!(
            blank_between, 2,
            "expected 2 blank leading lines; got {blank_between}; output:\n{output}"
        );
    }

    // ---- Item 2: show_lines=true with box_chars=None inserts blank separator ----

    /// Table with `box_chars=None` and `show_lines=true`: a blank separator line
    /// must appear between rows even without a box.
    #[test]
    fn test_show_lines_no_box_inserts_separator() {
        let mut table = Table::new(&["A", "B"]);
        table.box_chars = None;
        table.show_header = false;
        table.padding = (0, 0, 0, 0);
        table.columns[0].width = Some(4);
        table.columns[1].width = Some(4);
        table.show_lines = true;
        table.add_row(&["aaaa", "bbbb"]);
        table.add_row(&["cccc", "dddd"]);

        let output = render_plain(&table);

        let lines: Vec<&str> = output.lines().collect();
        let r1 = lines
            .iter()
            .position(|l| l.contains("aaaa"))
            .expect("row1 missing");
        let r2 = lines
            .iter()
            .position(|l| l.contains("cccc"))
            .expect("row2 missing");
        assert!(
            r2 > r1 + 1,
            "expected at least one blank separator line between data rows; output:\n{output}"
        );
        // All lines between r1 and r2 must be blank (spaces or empty).
        for between_line in &lines[r1 + 1..r2] {
            assert!(
                between_line.chars().all(|c| c == ' '),
                "separator line '{between_line}' is not blank; output:\n{output}"
            );
        }
    }

    // ---- Item 5: row_style resolved once (smoke test) ----

    /// A table with a row_style set should render correctly without panic,
    /// demonstrating that the style is applied with a single get_row_style call.
    #[test]
    fn test_row_style_resolved_once() {
        let mut table = Table::new(&["Name", "Value"]);
        table.show_header = false;
        table.box_chars = Some(&SQUARE);
        table.row_styles = vec!["bold".to_string(), "".to_string()];
        table.add_row(&["alice", "1"]);
        table.add_row(&["bob", "2"]);

        // Should not panic; data should appear in output.
        let output = render_plain(&table);
        assert!(output.contains("alice"), "alice missing: {output:?}");
        assert!(output.contains("bob"), "bob missing: {output:?}");
    }
}
