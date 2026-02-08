//! Convenience re-exports for common gilt types.
//!
//! ```
//! use gilt::prelude::*;
//! ```

// Core engine
pub use crate::console::{Console, ConsoleBuilder, ConsoleOptions, Renderable};

// Text and styling
pub use crate::color::{Color, ColorSystem};
pub use crate::segment::Segment;
pub use crate::style::Style;
pub use crate::styled_str::{StyledStr, Stylize};
pub use crate::text::{JustifyMethod, OverflowMethod, Text};

// Widgets
pub use crate::columns::Columns;
pub use crate::gradient::Gradient;
pub use crate::group::Group;
pub use crate::inspect::Inspect;
pub use crate::markdown::Markdown;
pub use crate::panel::Panel;
pub use crate::progress::Progress;
pub use crate::progress::ProgressIteratorExt;
pub use crate::progress_bar::ProgressBar;
pub use crate::rule::Rule;
pub use crate::syntax::Syntax;
pub use crate::table::Table;
pub use crate::tree::Tree;

// Markup
pub use crate::markup::render as render_markup;
