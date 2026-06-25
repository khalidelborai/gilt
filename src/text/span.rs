//! Span type for styled ranges within Text.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;

use crate::style::Style;

/// A styled range within a [`Text`](super::Text) object.
///
/// A span associates a [`Style`] with a half-open character range `[start, end)`.
/// The optional `meta` field carries arbitrary string key/value metadata (e.g. from
/// `[@key=val]...[/]` markup tags).  Two spans are equal only when their `meta` also
/// compares equal; the manual `Hash` impl still hashes only `start`, `end`, `style`, and `style_name`
/// so that meta-only differences may collide (which is permitted by the `Hash` contract).
///
/// The optional `style_name` field carries a theme token (e.g. `"warning"`,
/// `"repr.number"`) for deferred resolution — the name is kept alongside (or instead of)
/// a resolved [`Style`] so the render pipeline can resolve it against the active theme
/// at the last moment.  Named spans are created with [`Span::named`] /
/// [`Span::named_with_meta`]; unnamed spans (created with [`Span::new`] /
/// [`Span::with_meta`]) leave this field as `None`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Span {
    /// Start character offset (inclusive).
    pub start: usize,
    /// End character offset (exclusive).
    pub end: usize,
    /// Style applied to this range.
    pub style: Style,
    /// Arbitrary key/value metadata attached to this span (e.g. from `[@key=val]` markup).
    ///
    /// `None` when no metadata is present, which is the common case and costs nothing.
    /// When present, the `Arc` allows the same metadata to be shared across cloned spans
    /// without copying the `HashMap`.
    pub meta: Option<Arc<HashMap<String, String>>>,
    /// Optional theme token (e.g. `"warning"`, `"repr.number"`) for deferred resolution.
    ///
    /// When `Some`, the span carries a symbolic style name that the render pipeline can
    /// resolve against the active theme at render time.  `None` for all ordinary spans
    /// created with [`Span::new`] or [`Span::with_meta`].
    pub style_name: Option<String>,
}

impl Span {
    /// Create a new span covering `[start, end)` with the given style and no metadata.
    pub fn new(start: usize, end: usize, style: Style) -> Self {
        Span {
            start,
            end,
            style,
            meta: None,
            style_name: None,
        }
    }

    /// Create a new span with explicit metadata.
    ///
    /// Pass `None` for `meta` to get the same result as [`Span::new`].
    pub fn with_meta(
        start: usize,
        end: usize,
        style: Style,
        meta: Option<Arc<HashMap<String, String>>>,
    ) -> Self {
        Span {
            start,
            end,
            style,
            meta,
            style_name: None,
        }
    }

    /// Create a named span covering `[start, end)` with a deferred theme token.
    ///
    /// The resolved `style` is left as [`Style::null()`]; the pipeline resolves the
    /// name against the active theme at render time.
    pub fn named(start: usize, end: usize, name: impl Into<String>) -> Self {
        Span {
            start,
            end,
            style: Style::null(),
            meta: None,
            style_name: Some(name.into()),
        }
    }

    /// Create a named span with explicit metadata.
    ///
    /// Combines [`Span::named`] with the `meta` field from [`Span::with_meta`].
    pub fn named_with_meta(
        start: usize,
        end: usize,
        name: impl Into<String>,
        meta: Option<Arc<HashMap<String, String>>>,
    ) -> Self {
        Span {
            start,
            end,
            style: Style::null(),
            meta,
            style_name: Some(name.into()),
        }
    }

    /// Return the theme token for this span, if it was created with [`Span::named`].
    pub fn style_name(&self) -> Option<&str> {
        self.style_name.as_deref()
    }

    /// Return `true` if this span carries a deferred theme name (i.e. was created with
    /// [`Span::named`] or [`Span::named_with_meta`]).
    pub fn is_named(&self) -> bool {
        self.style_name.is_some()
    }

    /// Return `true` if the span covers zero or negative characters.
    pub fn is_empty(&self) -> bool {
        self.end <= self.start
    }

    /// Split span at `offset` (char index).
    /// If offset is outside the span, returns (self, None).
    /// Otherwise returns (left, Some(right)).
    ///
    /// Both halves inherit `self.meta` via `Arc::clone` (zero allocation).
    /// `style_name` is also cloned into both halves.
    pub fn split(&self, offset: usize) -> (Span, Option<Span>) {
        if offset < self.start || offset >= self.end {
            return (self.clone(), None);
        }
        let left = Span {
            start: self.start,
            end: offset,
            style: self.style.clone(),
            meta: self.meta.clone(),
            style_name: self.style_name.clone(),
        };
        let right = Span {
            start: offset,
            end: self.end,
            style: self.style.clone(),
            meta: self.meta.clone(),
            style_name: self.style_name.clone(),
        };
        (left, Some(right))
    }

    /// Shift span by `offset` positions.  Metadata and `style_name` are preserved.
    pub fn move_span(&self, offset: usize) -> Span {
        Span {
            start: self.start.saturating_add(offset),
            end: self.end.saturating_add(offset),
            style: self.style.clone(),
            meta: self.meta.clone(),
            style_name: self.style_name.clone(),
        }
    }

    /// Crop the end to `min(offset, self.end)`.  Metadata and `style_name` are preserved.
    pub fn right_crop(&self, offset: usize) -> Span {
        Span {
            start: self.start,
            end: std::cmp::min(offset, self.end),
            style: self.style.clone(),
            meta: self.meta.clone(),
            style_name: self.style_name.clone(),
        }
    }

    /// Extend end by `cells`.  Metadata and `style_name` are preserved.
    pub fn extend(&self, cells: usize) -> Span {
        Span {
            start: self.start,
            end: self.end + cells,
            style: self.style.clone(),
            meta: self.meta.clone(),
            style_name: self.style_name.clone(),
        }
    }
}

impl PartialOrd for Span {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Span {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.start, self.end).cmp(&(other.start, other.end))
    }
}

impl std::hash::Hash for Span {
    /// Hashes `start`, `end`, `style`, and `style_name`.
    ///
    /// `meta` is intentionally excluded: `HashMap` is not `Hash`, and the
    /// contract only requires that equal values produce the same hash — spans
    /// that differ solely in `meta` may collide, which is permitted.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.start.hash(state);
        self.end.hash(state);
        self.style.hash(state);
        self.style_name.hash(state);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn null_style() -> Style {
        Style::null()
    }

    #[test]
    fn span_new_has_no_meta() {
        let s = Span::new(0, 5, null_style());
        assert!(s.meta.is_none());
    }

    #[test]
    fn span_with_meta_stores_meta() {
        let mut m = HashMap::new();
        m.insert("key".to_string(), "val".to_string());
        let arc = Arc::new(m);
        let s = Span::with_meta(0, 5, null_style(), Some(arc.clone()));
        assert!(s.meta.is_some());
        assert_eq!(
            s.meta.as_ref().unwrap().get("key").map(|v| v.as_str()),
            Some("val")
        );
    }

    #[test]
    fn span_meta_equality() {
        let mut m = HashMap::new();
        m.insert("k".to_string(), "v".to_string());
        let s1 = Span::with_meta(0, 5, null_style(), Some(Arc::new(m.clone())));
        let s2 = Span::with_meta(0, 5, null_style(), Some(Arc::new(m.clone())));
        let s3 = Span::with_meta(0, 5, null_style(), None);
        assert_eq!(s1, s2, "spans with equal meta must be equal");
        assert_ne!(s1, s3, "span with meta != span without meta");
    }

    #[test]
    fn span_clone_shares_meta_arc() {
        let mut m = HashMap::new();
        m.insert("x".to_string(), "y".to_string());
        let arc = Arc::new(m);
        let s = Span::with_meta(1, 4, null_style(), Some(arc.clone()));
        let c = s.clone();
        // Both should point to the same Arc (same allocation).
        assert!(Arc::ptr_eq(
            s.meta.as_ref().unwrap(),
            c.meta.as_ref().unwrap()
        ));
    }

    #[test]
    fn split_propagates_meta() {
        let mut m = HashMap::new();
        m.insert("a".to_string(), "b".to_string());
        let s = Span::with_meta(0, 6, null_style(), Some(Arc::new(m)));
        let (left, right) = s.split(3);
        assert!(left.meta.is_some());
        assert!(right.unwrap().meta.is_some());
    }

    #[test]
    fn move_span_propagates_meta() {
        let mut m = HashMap::new();
        m.insert("p".to_string(), "q".to_string());
        let s = Span::with_meta(0, 3, null_style(), Some(Arc::new(m)));
        let shifted = s.move_span(10);
        assert!(shifted.meta.is_some());
        assert_eq!(shifted.start, 10);
        assert_eq!(shifted.end, 13);
    }

    #[test]
    fn right_crop_propagates_meta() {
        let mut m = HashMap::new();
        m.insert("r".to_string(), "s".to_string());
        let s = Span::with_meta(0, 10, null_style(), Some(Arc::new(m)));
        let cropped = s.right_crop(5);
        assert!(cropped.meta.is_some());
        assert_eq!(cropped.end, 5);
    }

    #[test]
    fn named_span_carries_style_name() {
        let s = Span::named(0, 5, "warning");
        assert_eq!(s.style_name(), Some("warning"));
        assert!(s.is_named());
        assert!(
            s.style.is_null(),
            "named span has null resolved style initially"
        );
    }

    #[test]
    fn regular_span_has_no_style_name() {
        let s = Span::new(0, 5, Style::parse("bold"));
        assert_eq!(s.style_name(), None);
        assert!(!s.is_named());
    }

    #[test]
    fn named_span_split_preserves_name() {
        let (left, right) = Span::named(0, 6, "repr.number").split(3);
        assert_eq!(left.style_name(), Some("repr.number"));
        assert_eq!(right.unwrap().style_name(), Some("repr.number"));
    }

    #[test]
    fn named_span_move_preserves_name() {
        let moved = Span::named(0, 3, "repr.bool_true").move_span(10);
        assert_eq!(moved.style_name(), Some("repr.bool_true"));
        assert_eq!(moved.start, 10);
    }
}
