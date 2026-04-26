//! Row and cell content types for the table module.

use crate::console::{Console, Renderable};
use crate::text::Text;
use std::sync::Arc;

/// Content of a table cell -- either a plain string (parsed with markup),
/// a pre-styled [`Text`] object, or any [`Renderable`] widget (Panel, Tree,
/// nested Table, etc.).
// Variant-size disparity (Plain=24 B, Styled=240 B) is intentional for now.
// L2 (StyleId interner) in v0.11.0 will shrink `Text` enough to drop this
// allow; boxing `Styled(Box<Text>)` here would be a public API break.
#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum CellContent {
    // Note: PartialEq is implemented manually below (Plain compares string).
    /// A plain string, optionally containing markup tags.
    Plain(String),
    /// A pre-styled [`Text`] value (styles are preserved as-is).
    Styled(Text),
    /// Any renderable widget wrapped in an [`Arc`] for cheap cloning.
    ///
    /// The `Send + Sync` bounds are required so that `CellContent` (and
    /// therefore `Table`) remains `Send + Sync` — important for async
    /// contexts.  All built-in widgets (`Panel`, `Tree`, nested `Table`, …)
    /// are automatically `Send + Sync` because they contain no `Rc` / raw
    /// pointer fields.  If you need a non-`Send` renderable, wrap it in a
    /// `Mutex` before boxing.
    Renderable(Arc<dyn Renderable + Send + Sync>),
}

// Manual Debug — Arc<dyn Renderable> doesn't implement Debug, so we print a
// placeholder rather than deriving.
impl std::fmt::Debug for CellContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CellContent::Plain(s) => f.debug_tuple("Plain").field(s).finish(),
            CellContent::Styled(t) => f.debug_tuple("Styled").field(t).finish(),
            CellContent::Renderable(_) => f.write_str("Renderable(<dyn Renderable>)"),
        }
    }
}

impl CellContent {
    /// Resolve into a [`Text`] using the given console for markup parsing.
    ///
    /// For the [`CellContent::Renderable`] variant the widget is rendered at
    /// the console's default width, the resulting segments are reassembled
    /// into a [`Text`] that preserves per-segment styling, and the outer
    /// table renderer then re-renders that [`Text`] at the correct column
    /// width via `render_lines`.
    pub(crate) fn resolve(&self, console: &Console) -> Text {
        match self {
            CellContent::Plain(s) => console.render_str(s, None, None, None),
            CellContent::Styled(t) => t.clone(),
            CellContent::Renderable(r) => {
                // Render to segments using the console's default options, then
                // rebuild a Text that preserves per-segment styling.  The
                // outer table renderer will re-render this Text at the correct
                // column width via render_lines.
                let segs = console.render(r.as_ref(), None);
                let mut t = Text::empty();
                for seg in &segs {
                    t.append_str(&seg.text, seg.style.clone());
                }
                t
            }
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
            CellContent::Renderable(_) => false,
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
