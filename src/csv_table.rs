//! CSV to Table — load CSV data directly into a gilt [`Table`].
//!
//! The basic [`CsvTable::from_csv_str`] method works with zero extra dependencies
//! by implementing a simple CSV parser that handles quoted fields. For full CSV
//! support (file reading, streaming), enable the `csv` feature which uses the
//! [`csv`](https://docs.rs/csv) crate.
//!
//! # Example
//!
//! ```rust
//! use gilt::csv_table::CsvTable;
//!
//! let data = "Name,Age,City\nAlice,30,NYC\nBob,25,LA";
//! let csv = CsvTable::from_csv_str(data).unwrap();
//! let table = csv.to_table();
//! ```

use std::fmt;

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::Style;
use crate::table::Table;

#[cfg(feature = "csv")]
use csv::Reader;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur when parsing CSV data.
#[derive(Debug, thiserror::Error)]
pub enum CsvTableError {
    /// The input CSV string was empty.
    #[error("empty CSV data")]
    Empty,
    /// No header row was found.
    #[error("no header row")]
    NoHeader,
    /// An I/O error occurred.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// A CSV parsing error from the `csv` crate (feature-gated).
    #[cfg(feature = "csv")]
    #[error("CSV parse error: {0}")]
    Csv(#[from] csv::Error),
}

// ---------------------------------------------------------------------------
// Basic CSV parser (no dependencies)
// ---------------------------------------------------------------------------

/// Parse a single CSV line into fields, handling quoted fields.
///
/// Rules:
/// - Fields are separated by commas
/// - Fields may be enclosed in double quotes
/// - Within a quoted field, a literal `"` is represented as `""`
/// - Commas inside quoted fields do not split
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '"' {
                // Check for escaped quote ""
                if chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next(); // consume the second "
                } else {
                    // End of quoted field
                    in_quotes = false;
                }
            } else {
                current.push(ch);
            }
        } else {
            match ch {
                '"' => {
                    in_quotes = true;
                }
                ',' => {
                    fields.push(std::mem::take(&mut current));
                }
                _ => {
                    current.push(ch);
                }
            }
        }
    }
    // Push the last field
    fields.push(current);
    fields
}

/// Parse CSV text into headers and rows using the built-in parser.
fn parse_csv_text(text: &str) -> Result<(Vec<String>, Vec<Vec<String>>), CsvTableError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(CsvTableError::Empty);
    }

    let mut lines = text.lines();

    let header_line = lines.next().ok_or(CsvTableError::NoHeader)?;
    let headers = parse_csv_line(header_line);
    if headers.is_empty() || (headers.len() == 1 && headers[0].is_empty()) {
        return Err(CsvTableError::NoHeader);
    }

    let mut rows = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        rows.push(parse_csv_line(line));
    }

    Ok((headers, rows))
}

// ---------------------------------------------------------------------------
// CsvTable struct
// ---------------------------------------------------------------------------

/// A CSV dataset that can be converted to a gilt [`Table`] for rendering.
///
/// The simplest way to create one is from a CSV string:
///
/// ```rust
/// use gilt::csv_table::CsvTable;
///
/// let csv = CsvTable::from_csv_str("Name,Age\nAlice,30").unwrap();
/// let table = csv.to_table();
/// ```
#[derive(Debug, Clone)]
pub struct CsvTable {
    /// Column headers.
    headers: Vec<String>,
    /// Data rows (each row is a vec of field values).
    rows: Vec<Vec<String>>,
    /// Optional limit on the number of rows shown.
    max_rows: Option<usize>,
    /// Optional style applied to header cells.
    header_style: Option<Style>,
    /// Optional table title.
    title: Option<String>,
}

impl CsvTable {
    /// Create a `CsvTable` from raw headers and rows.
    fn from_parts(headers: Vec<String>, rows: Vec<Vec<String>>) -> Self {
        Self {
            headers,
            rows,
            max_rows: None,
            header_style: None,
            title: None,
        }
    }

    /// Load from a CSV string using the built-in parser (no extra dependencies).
    ///
    /// The first line is treated as the header row.
    ///
    /// # Errors
    ///
    /// Returns [`CsvTableError::Empty`] if the input is empty, or
    /// [`CsvTableError::NoHeader`] if no valid header row is found.
    pub fn from_csv_str(csv_text: &str) -> Result<Self, CsvTableError> {
        let (headers, rows) = parse_csv_text(csv_text)?;
        Ok(Self::from_parts(headers, rows))
    }

    /// Load from a CSV file path using the `csv` crate.
    ///
    /// Requires the `csv` feature.
    #[cfg(feature = "csv")]
    pub fn from_path(path: &str) -> Result<Self, CsvTableError> {
        let reader = Reader::from_path(path)?;
        Self::from_reader(reader)
    }

    /// Load from a `csv::Reader`.
    ///
    /// Requires the `csv` feature.
    #[cfg(feature = "csv")]
    pub fn from_reader<R: std::io::Read>(mut reader: Reader<R>) -> Result<Self, CsvTableError> {
        let headers: Vec<String> = reader.headers()?.iter().map(|h| h.to_string()).collect();

        if headers.is_empty() {
            return Err(CsvTableError::NoHeader);
        }

        let mut rows = Vec::new();
        for result in reader.records() {
            let record = result?;
            let row: Vec<String> = record.iter().map(|f| f.to_string()).collect();
            rows.push(row);
        }

        Ok(Self::from_parts(headers, rows))
    }

    /// Limit the number of data rows displayed.
    #[must_use]
    pub fn with_max_rows(mut self, max: usize) -> Self {
        self.max_rows = Some(max);
        self
    }

    /// Set a style for the header row.
    #[must_use]
    pub fn with_header_style(mut self, style: Style) -> Self {
        self.header_style = Some(style);
        self
    }

    /// Set a table title.
    #[must_use]
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Return the column headers.
    pub fn headers(&self) -> &[String] {
        &self.headers
    }

    /// Return all data rows.
    pub fn rows(&self) -> &[Vec<String>] {
        &self.rows
    }

    /// Return the number of data rows (before any max_rows limit).
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Convert this CSV data into a gilt [`Table`].
    pub fn to_table(&self) -> Table {
        let header_refs: Vec<&str> = self.headers.iter().map(|s| s.as_str()).collect();
        let mut table = Table::new(&header_refs);

        if let Some(title) = &self.title {
            table.title = Some(title.clone());
        }

        if let Some(style) = &self.header_style {
            let style_str = format!("{}", style);
            table.header_style = style_str;
        }

        let row_limit = self.max_rows.unwrap_or(self.rows.len());
        for row in self.rows.iter().take(row_limit) {
            let cells: Vec<&str> = row.iter().map(|s| s.as_str()).collect();
            table.add_row(&cells);
        }

        table
    }

    /// Return the measurement for this CSV table (delegated to the inner table).
    pub fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        let table = self.to_table();
        table.measure(console, options)
    }
}

// ---------------------------------------------------------------------------
// Renderable
// ---------------------------------------------------------------------------

impl Renderable for CsvTable {
    fn rich_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let table = self.to_table();
        table.rich_console(console, options)
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl fmt::Display for CsvTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

    // -- Simple CSV string --------------------------------------------------

    #[test]
    fn test_simple_csv() {
        let csv = CsvTable::from_csv_str("Name,Age\nAlice,30\nBob,25").unwrap();
        assert_eq!(csv.headers(), &["Name", "Age"]);
        assert_eq!(csv.row_count(), 2);
    }

    // -- CSV with headers ---------------------------------------------------

    #[test]
    fn test_headers_only() {
        let csv = CsvTable::from_csv_str("A,B,C").unwrap();
        assert_eq!(csv.headers(), &["A", "B", "C"]);
        assert_eq!(csv.row_count(), 0);
    }

    // -- Quoted fields ------------------------------------------------------

    #[test]
    fn test_quoted_fields() {
        let csv = CsvTable::from_csv_str("Name,Bio\nAlice,\"Likes coding\"").unwrap();
        assert_eq!(csv.rows()[0], vec!["Alice", "Likes coding"]);
    }

    // -- Commas in quotes ---------------------------------------------------

    #[test]
    fn test_commas_in_quotes() {
        let csv = CsvTable::from_csv_str("City,Pop\n\"New York, NY\",8000000").unwrap();
        assert_eq!(csv.rows()[0][0], "New York, NY");
        assert_eq!(csv.rows()[0][1], "8000000");
    }

    // -- Escaped quotes -----------------------------------------------------

    #[test]
    fn test_escaped_quotes() {
        let csv = CsvTable::from_csv_str("Name,Quote\nAlice,\"She said \"\"hi\"\"\"").unwrap();
        assert_eq!(csv.rows()[0][1], "She said \"hi\"");
    }

    // -- Empty CSV ----------------------------------------------------------

    #[test]
    fn test_empty_csv() {
        let result = CsvTable::from_csv_str("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CsvTableError::Empty));
    }

    #[test]
    fn test_whitespace_only_csv() {
        let result = CsvTable::from_csv_str("   \n  \n  ");
        assert!(result.is_err());
    }

    // -- Single column ------------------------------------------------------

    #[test]
    fn test_single_column() {
        let csv = CsvTable::from_csv_str("Name\nAlice\nBob").unwrap();
        assert_eq!(csv.headers().len(), 1);
        assert_eq!(csv.row_count(), 2);
    }

    // -- Single row ---------------------------------------------------------

    #[test]
    fn test_single_row() {
        let csv = CsvTable::from_csv_str("A,B\n1,2").unwrap();
        assert_eq!(csv.row_count(), 1);
        assert_eq!(csv.rows()[0], vec!["1", "2"]);
    }

    // -- Max rows limit -----------------------------------------------------

    #[test]
    fn test_max_rows() {
        let csv = CsvTable::from_csv_str("A\n1\n2\n3\n4\n5")
            .unwrap()
            .with_max_rows(3);
        let table = csv.to_table();
        assert_eq!(table.row_count(), 3);
    }

    // -- Header style -------------------------------------------------------

    #[test]
    fn test_header_style() {
        let style = Style::parse("bold").unwrap();
        let csv = CsvTable::from_csv_str("A,B\n1,2")
            .unwrap()
            .with_header_style(style.clone());
        assert!(csv.header_style.is_some());
    }

    // -- Title --------------------------------------------------------------

    #[test]
    fn test_title() {
        let csv = CsvTable::from_csv_str("A,B\n1,2")
            .unwrap()
            .with_title("My Data");
        let table = csv.to_table();
        assert_eq!(table.title.as_deref(), Some("My Data"));
    }

    // -- To table conversion ------------------------------------------------

    #[test]
    fn test_to_table_conversion() {
        let csv = CsvTable::from_csv_str("Name,Age\nAlice,30\nBob,25").unwrap();
        let table = csv.to_table();
        assert_eq!(table.columns.len(), 2);
        assert_eq!(table.row_count(), 2);
    }

    #[test]
    fn test_to_table_with_title_and_limit() {
        let csv = CsvTable::from_csv_str("X,Y\n1,2\n3,4\n5,6")
            .unwrap()
            .with_title("Test")
            .with_max_rows(2);
        let table = csv.to_table();
        assert_eq!(table.title.as_deref(), Some("Test"));
        assert_eq!(table.row_count(), 2);
    }

    // -- Renderable output --------------------------------------------------

    #[test]
    fn test_renderable_output() {
        let csv = CsvTable::from_csv_str("Name,Age\nAlice,30").unwrap();
        let console = make_console(60);
        let opts = console.options();
        let segments = csv.rich_console(&console, &opts);
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains("Name"));
        assert!(text.contains("Alice"));
        assert!(text.contains("30"));
    }

    // -- Display trait ------------------------------------------------------

    #[test]
    fn test_display_trait() {
        let csv = CsvTable::from_csv_str("A,B\n1,2").unwrap();
        let s = format!("{}", csv);
        assert!(s.contains("A"));
        assert!(s.contains("B"));
        assert!(s.contains("1"));
        assert!(s.contains("2"));
    }

    // -- Measure ------------------------------------------------------------

    #[test]
    fn test_measure() {
        let csv = CsvTable::from_csv_str("Name,Age\nAlice,30").unwrap();
        let console = make_console(80);
        let opts = console.options();
        let m = csv.measure(&console, &opts);
        assert!(m.minimum > 0);
        assert!(m.maximum > 0);
    }

    // -- parse_csv_line unit tests ------------------------------------------

    #[test]
    fn test_parse_csv_line_simple() {
        let fields = parse_csv_line("a,b,c");
        assert_eq!(fields, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_parse_csv_line_quoted() {
        let fields = parse_csv_line("\"hello, world\",foo");
        assert_eq!(fields, vec!["hello, world", "foo"]);
    }

    #[test]
    fn test_parse_csv_line_escaped_quote() {
        let fields = parse_csv_line("\"a \"\"b\"\" c\",d");
        assert_eq!(fields, vec!["a \"b\" c", "d"]);
    }

    #[test]
    fn test_parse_csv_line_empty_fields() {
        let fields = parse_csv_line(",a,,b,");
        assert_eq!(fields, vec!["", "a", "", "b", ""]);
    }

    // -- Builder chain ------------------------------------------------------

    #[test]
    fn test_builder_chain() {
        let csv = CsvTable::from_csv_str("A\n1")
            .unwrap()
            .with_max_rows(10)
            .with_header_style(Style::parse("bold").unwrap())
            .with_title("Title");
        assert_eq!(csv.max_rows, Some(10));
        assert!(csv.header_style.is_some());
        assert_eq!(csv.title.as_deref(), Some("Title"));
    }

    // -- Blank lines in CSV -------------------------------------------------

    #[test]
    fn test_blank_lines_skipped() {
        let csv = CsvTable::from_csv_str("A,B\n1,2\n\n3,4\n").unwrap();
        assert_eq!(csv.row_count(), 2);
    }

    // -- CSV feature-gated tests --------------------------------------------

    #[cfg(feature = "csv")]
    mod csv_crate_tests {
        use super::*;
        use std::io::Cursor;

        #[test]
        fn test_from_reader() {
            let data = "Name,Age\nAlice,30\nBob,25";
            let reader = csv::Reader::from_reader(Cursor::new(data));
            let csv_table = CsvTable::from_reader(reader).unwrap();
            assert_eq!(csv_table.headers(), &["Name", "Age"]);
            assert_eq!(csv_table.row_count(), 2);
        }

        #[test]
        fn test_from_reader_single_column() {
            let data = "Name\nAlice\nBob";
            let reader = csv::Reader::from_reader(Cursor::new(data));
            let csv_table = CsvTable::from_reader(reader).unwrap();
            assert_eq!(csv_table.headers().len(), 1);
            assert_eq!(csv_table.row_count(), 2);
        }

        #[test]
        fn test_from_path_nonexistent() {
            let result = CsvTable::from_path("/tmp/gilt_nonexistent_csv_file.csv");
            assert!(result.is_err());
        }
    }
}
