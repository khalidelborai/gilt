//! Convenience re-exports for common gilt types.
//!
//! ```
//! use gilt::prelude::*;
//! ```

// Accessibility
pub use crate::accessibility::{contrast_ratio, meets_aa, meets_aa_large, meets_aaa};

// Core engine
pub use crate::console::{Console, ConsoleBuilder, ConsoleOptions, Renderable};

// Text and styling
pub use crate::color::{Color, ColorSystem};
pub use crate::segment::Segment;
pub use crate::style::Style;
pub use crate::text::{JustifyMethod, OverflowMethod, Text};
pub use crate::utils::styled_str::{StyledStr, Stylize};

// Widgets
pub use crate::canvas::Canvas;
pub use crate::columns::Columns;
pub use crate::csv_table::CsvTable;
pub use crate::diff::{Diff, DiffStyle};
pub use crate::figlet::Figlet;
pub use crate::gradient::Gradient;
pub use crate::group::Group;
pub use crate::inspect::Inspect;
#[cfg(feature = "markdown")]
pub use crate::markdown::Markdown;
pub use crate::panel::Panel;
pub use crate::progress::Progress;
pub use crate::progress::ProgressIteratorExt;
pub use crate::progress_bar::ProgressBar;
pub use crate::rule::Rule;
pub use crate::sparkline::Sparkline;
#[cfg(feature = "syntax")]
pub use crate::syntax::Syntax;
pub use crate::table::Table;
pub use crate::tree::Tree;

// Interactive
pub use crate::bar::Bar;
#[cfg(feature = "json")]
pub use crate::json::Json;
pub use crate::layout::Layout;
pub use crate::live::Live;
pub use crate::prompt::{MultiSelect, Prompt, Select};
pub use crate::status::Status;

// Markup
pub use crate::markup::render as render_markup;
