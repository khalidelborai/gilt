//! Per-`Console` style interner. **Dormant — types only, nothing interns yet.**
//!
//! Foundation for the deferred L2 perf work: deduplicate `Style` instances
//! inside a `Console` so `Segment::style` could shrink from `Option<Style>`
//! (measured **48 B**, not the 136 B once claimed — `Color` was compacted to
//! 4 B and `Option<Style>` is niche-optimized) to a 4-byte `StyleId`, taking
//! `Segment` from 96 B → ~52 B. This file lands the *types* — interner, ID,
//! dedup map, NULL sentinel — but no callers wire through it.
//!
//! Activation is **deferred** (semver-major): it requires changing the
//! `Renderable` trait return type so segment construction has an interner
//! context, which ripples through every widget and test. See
//! `.review/v2-structural-decision-2026-06-06.md` for the full decision and
//! the benchmark trigger that would justify doing it.
//!
//! # Design notes
//!
//! - `StyleId(u32)` is `Copy`. `StyleId::NULL == StyleId(0)` is reserved
//!   for `Style::null()` and is pre-seeded by [`StyleInterner::new`].
//! - Dedup is content-based: `HashMap<Arc<Style>, StyleId>` works because
//!   `Arc<T>: Hash + Eq` delegates to `T`'s impls (and `Style` has manual
//!   `Hash + Eq`). Two distinct `Arc` instances with equal `Style` content
//!   hash to the same bucket and compare equal.
//! - `Arc<Style>` (not raw `Style`) so `get` can return a cheap clone
//!   without copying the underlying style data.
//! - **No cap yet.** Per the design doc, an LRU eviction at 64K ids will
//!   land in PR3 alongside interner activation. Until then, an unbounded
//!   `Vec` is fine because nothing actually interns.

use std::collections::HashMap;
use std::sync::Arc;

use crate::style::Style;

/// Stable identifier for an interned [`Style`] within a single
/// [`StyleInterner`]. Cross-interner IDs are not interchangeable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StyleId(u32);

impl StyleId {
    /// Reserved id for `Style::null()`. Always present immediately after
    /// [`StyleInterner::new`] returns.
    pub const NULL: StyleId = StyleId(0);

    /// Numeric value of the id. Useful for serialization and
    /// stable-equality assertions in tests.
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

impl Default for StyleId {
    fn default() -> Self {
        Self::NULL
    }
}

/// Content-deduplicating interner for [`Style`] values. One interner per
/// [`crate::console::Console`] (renderables on different consoles must
/// not share ids).
#[derive(Debug)]
pub struct StyleInterner {
    forward: Vec<Arc<Style>>,
    reverse: HashMap<Arc<Style>, StyleId>,
}

impl StyleInterner {
    /// Create an interner pre-seeded with `Style::null()` at
    /// [`StyleId::NULL`].
    pub fn new() -> Self {
        let null_style = Arc::new(Style::null());
        let mut interner = Self {
            forward: Vec::new(),
            reverse: HashMap::new(),
        };
        interner.forward.push(Arc::clone(&null_style));
        interner.reverse.insert(null_style, StyleId::NULL);
        interner
    }

    /// Intern a style and return its id. If an equal style is already
    /// interned, the existing id is returned.
    pub fn intern(&mut self, style: Style) -> StyleId {
        let arc = Arc::new(style);
        if let Some(&id) = self.reverse.get(&arc) {
            return id;
        }
        let id = StyleId(self.forward.len() as u32);
        self.forward.push(Arc::clone(&arc));
        self.reverse.insert(arc, id);
        id
    }

    /// Look up an interned style by id. Returns `None` for ids from
    /// other interners or invalid ids.
    pub fn get(&self, id: StyleId) -> Option<&Arc<Style>> {
        self.forward.get(id.0 as usize)
    }

    /// Number of distinct styles interned (including the pre-seeded
    /// null). Always >= 1.
    pub fn len(&self) -> usize {
        self.forward.len()
    }

    /// Whether the interner holds nothing beyond the pre-seeded null.
    pub fn is_empty(&self) -> bool {
        self.forward.len() <= 1
    }
}

impl Default for StyleInterner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_is_pre_seeded() {
        let interner = StyleInterner::new();
        assert_eq!(interner.len(), 1);
        assert!(interner.is_empty()); // empty = "nothing beyond null"
        let arc = interner.get(StyleId::NULL).expect("null present");
        assert!(arc.is_null());
    }

    #[test]
    fn dedup_returns_same_id() {
        let mut interner = StyleInterner::new();
        let s = Style::parse("bold red");
        let id1 = interner.intern(s.clone());
        let id2 = interner.intern(s);
        assert_eq!(id1, id2);
        assert_eq!(interner.len(), 2); // null + bold red
    }

    #[test]
    fn distinct_styles_get_distinct_ids() {
        let mut interner = StyleInterner::new();
        let bold = Style::parse("bold");
        let italic = Style::parse("italic");
        let id1 = interner.intern(bold);
        let id2 = interner.intern(italic);
        assert_ne!(id1, id2);
        assert_eq!(interner.len(), 3);
    }

    #[test]
    fn null_intern_returns_null_id() {
        let mut interner = StyleInterner::new();
        let id = interner.intern(Style::null());
        assert_eq!(id, StyleId::NULL);
        assert_eq!(interner.len(), 1); // no growth
    }

    #[test]
    fn get_unknown_id_returns_none() {
        let interner = StyleInterner::new();
        assert!(interner.get(StyleId(999)).is_none());
    }

    #[test]
    fn ids_are_stable() {
        let mut interner = StyleInterner::new();
        let s = Style::parse("bold");
        let id = interner.intern(s.clone());
        // Intern many other styles; original id must not move.
        for i in 0..10 {
            interner.intern(Style::parse(&format!("color({i})")));
        }
        assert_eq!(interner.intern(s), id);
    }
}
