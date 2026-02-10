//! Row and cell content types for the table module.

use crate::console::Console;
use crate::text::Text;

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
    pub(crate) fn resolve(&self, console: &Console) -> Text {
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

/// Information regarding a row.
#[derive(Debug, Clone, Default)]
pub struct Row {
    /// Optional style to apply to this row.
    pub style: Option<String>,
    /// Whether this row ends a section (draws a line after it).
    pub end_section: bool,
}
