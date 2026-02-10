//! Table widget module -- rich table rendering with columns, rows, and box borders.
//!
//! Port of Python's `rich/table.py`.
//!
//! # Example
//!
//! ```
//! use gilt::table::Table;
//!
//! let mut table = Table::new(&["Name", "Age"]);
//! table.add_row(&["Alice", "30"]);
//! table.add_row(&["Bob", "25"]);
//! println!("{}", table);
//! ```

mod column;
mod render;
mod row;
mod core;

// Re-exports for backward compatibility
pub use column::{Column, ColumnOptions};
pub use row::{CellContent, Row};
pub use core::Table;
