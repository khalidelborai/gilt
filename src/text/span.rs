//! Span type for styled ranges within Text.

use std::cmp::Ordering;

use crate::style::Style;

/// A styled range within a [`Text`](super::Text) object.
///
/// A span associates a [`Style`] with a half-open character range `[start, end)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    /// Start character offset (inclusive).
    pub start: usize,
    /// End character offset (exclusive).
    pub end: usize,
    /// Style applied to this range.
    pub style: Style,
}

impl Span {
    /// Create a new span covering the character range `[start, end)` with the given style.
    pub fn new(start: usize, end: usize, style: Style) -> Self {
        Span { start, end, style }
    }

    /// Return `true` if the span covers zero or negative characters.
    pub fn is_empty(&self) -> bool {
        self.end <= self.start
    }

    /// Split span at `offset` (char index).
    /// If offset is outside the span, returns (self, None).
    /// Otherwise returns (left, Some(right)).
    pub fn split(&self, offset: usize) -> (Span, Option<Span>) {
        if offset < self.start || offset >= self.end {
            return (self.clone(), None);
        }
        let left = Span::new(self.start, offset, self.style.clone());
        let right = Span::new(offset, self.end, self.style.clone());
        (left, Some(right))
    }

    /// Shift span by `offset` positions.
    pub fn move_span(&self, offset: usize) -> Span {
        Span::new(
            self.start.saturating_add(offset),
            self.end.saturating_add(offset),
            self.style.clone(),
        )
    }

    /// Crop the end to `min(offset, self.end)`.
    pub fn right_crop(&self, offset: usize) -> Span {
        Span::new(
            self.start,
            std::cmp::min(offset, self.end),
            self.style.clone(),
        )
    }

    /// Extend end by `cells`.
    pub fn extend(&self, cells: usize) -> Span {
        Span::new(self.start, self.end + cells, self.style.clone())
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
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.start.hash(state);
        self.end.hash(state);
        self.style.hash(state);
    }
}
