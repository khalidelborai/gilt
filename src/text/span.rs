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
/// compares equal; the manual `Hash` impl still hashes only `start`, `end`, and `style`
/// so that meta-only differences may collide (which is permitted by the `Hash` contract).
#[derive(Clone, Debug, PartialEq, Eq)]
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
}

impl Span {
    /// Create a new span covering `[start, end)` with the given style and no metadata.
    pub fn new(start: usize, end: usize, style: Style) -> Self {
        Span {
            start,
            end,
            style,
            meta: None,
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
        }
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
    pub fn split(&self, offset: usize) -> (Span, Option<Span>) {
        if offset < self.start || offset >= self.end {
            return (self.clone(), None);
        }
        let left = Span::with_meta(self.start, offset, self.style.clone(), self.meta.clone());
        let right = Span::with_meta(offset, self.end, self.style.clone(), self.meta.clone());
        (left, Some(right))
    }

    /// Shift span by `offset` positions.  Metadata is preserved.
    pub fn move_span(&self, offset: usize) -> Span {
        Span::with_meta(
            self.start.saturating_add(offset),
            self.end.saturating_add(offset),
            self.style.clone(),
            self.meta.clone(),
        )
    }

    /// Crop the end to `min(offset, self.end)`.  Metadata is preserved.
    pub fn right_crop(&self, offset: usize) -> Span {
        Span::with_meta(
            self.start,
            std::cmp::min(offset, self.end),
            self.style.clone(),
            self.meta.clone(),
        )
    }

    /// Extend end by `cells`.  Metadata is preserved.
    pub fn extend(&self, cells: usize) -> Span {
        Span::with_meta(
            self.start,
            self.end + cells,
            self.style.clone(),
            self.meta.clone(),
        )
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
    /// Hashes only `start`, `end`, and `style`.
    ///
    /// `meta` is intentionally excluded: `HashMap` is not `Hash`, and the
    /// contract only requires that equal values produce the same hash — spans
    /// that differ solely in `meta` may collide, which is permitted.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.start.hash(state);
        self.end.hash(state);
        self.style.hash(state);
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
}
