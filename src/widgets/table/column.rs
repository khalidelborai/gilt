//! Column and column options types for the table module.

use crate::text::{JustifyMethod, OverflowMethod};
use crate::utils::align_widget::VerticalAlign;
use crate::widgets::table::CellContent;

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
