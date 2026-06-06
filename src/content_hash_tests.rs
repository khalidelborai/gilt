//! TDD RED tests for Task 1: Renderable::content_hash + Layout dirty-tracking cache.
//!
//! These tests compile only AFTER the feature is implemented (they fail to compile
//! before `content_hash` is added to `Renderable`).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::layout::{Layout, LayoutCache};
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A wrapper renderable that counts how many times `gilt_console` is called.
/// Returns a fixed `content_hash` so the cache can kick in.
struct CountingRenderable {
    call_count: Arc<AtomicUsize>,
    fixed_hash: u64,
}

impl CountingRenderable {
    fn new(call_count: Arc<AtomicUsize>, fixed_hash: u64) -> Self {
        CountingRenderable {
            call_count,
            fixed_hash,
        }
    }
}

impl Renderable for CountingRenderable {
    fn gilt_console(&self, _console: &Console, _options: &ConsoleOptions) -> Vec<Segment> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        vec![Segment::text("hello"), Segment::line()]
    }

    fn content_hash(&self) -> Option<u64> {
        Some(self.fixed_hash)
    }
}

/// A renderable that returns `None` for content_hash — must always re-render.
struct AlwaysDirty {
    call_count: Arc<AtomicUsize>,
}

impl Renderable for AlwaysDirty {
    fn gilt_console(&self, _console: &Console, _options: &ConsoleOptions) -> Vec<Segment> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        vec![Segment::text("dirty"), Segment::line()]
    }
    // content_hash returns None (default) — always re-render
}

// ---------------------------------------------------------------------------
// Test: cached child is rendered only ONCE across two layout renders
// ---------------------------------------------------------------------------

#[test]
fn test_content_hash_caches_second_render() {
    let console = Console::builder().width(40).height(5).build();
    let options = console.options().update_dimensions(40, 5);

    let call_count = Arc::new(AtomicUsize::new(0));
    let counter_arc = Arc::clone(&call_count);

    let child = CountingRenderable::new(call_count, 0xDEAD_BEEF);
    let mut layout = Layout::default_layout();
    layout.name = Some("main".to_string());
    layout.update_renderable(child);

    // First pass: cache is empty — must render
    let mut cache = LayoutCache::new();
    layout.render_with_cache(&console, &options, &mut cache);

    // Second pass: hash is same — must NOT call gilt_console again
    layout.render_with_cache(&console, &options, &mut cache);

    let calls = counter_arc.load(Ordering::SeqCst);
    assert!(
        calls < 2,
        "expected cached render to skip second gilt_console call, but got {} calls",
        calls
    );
    assert_eq!(
        calls, 1,
        "expected exactly 1 gilt_console call (first render only)"
    );
}

// ---------------------------------------------------------------------------
// Test: None-hash child is re-rendered every pass
// ---------------------------------------------------------------------------

#[test]
fn test_none_hash_always_rerenders() {
    let console = Console::builder().width(40).height(5).build();
    let options = console.options().update_dimensions(40, 5);

    let call_count = Arc::new(AtomicUsize::new(0));
    let counter_arc = Arc::clone(&call_count);

    let child = AlwaysDirty { call_count };
    let mut layout = Layout::default_layout();
    layout.name = Some("dirty_pane".to_string());
    layout.update_renderable(child);

    let mut cache = LayoutCache::new();
    layout.render_with_cache(&console, &options, &mut cache);
    layout.render_with_cache(&console, &options, &mut cache);

    let calls = counter_arc.load(Ordering::SeqCst);
    assert_eq!(
        calls, 2,
        "None-hash child must re-render every time, got {} calls",
        calls
    );
}

// ---------------------------------------------------------------------------
// Test: default content_hash on Renderable returns None
// ---------------------------------------------------------------------------

#[test]
fn test_default_content_hash_is_none() {
    // A plain struct that only implements the required method.
    struct Minimal;
    impl Renderable for Minimal {
        fn gilt_console(&self, _: &Console, _: &ConsoleOptions) -> Vec<Segment> {
            vec![]
        }
    }
    let m = Minimal;
    assert_eq!(m.content_hash(), None, "default content_hash must be None");
}

// ---------------------------------------------------------------------------
// Test: Text implements content_hash (returns Some)
// ---------------------------------------------------------------------------

#[test]
fn test_text_has_content_hash() {
    let t = Text::new("hello", Style::null());
    assert!(t.content_hash().is_some(), "Text should return Some(hash)");
}

// ---------------------------------------------------------------------------
// Test: same Text produces same hash, different Text produces different hash
// ---------------------------------------------------------------------------

#[test]
fn test_text_content_hash_stable_and_distinct() {
    let t1 = Text::new("hello", Style::null());
    let t2 = Text::new("hello", Style::null());
    let t3 = Text::new("world", Style::null());
    assert_eq!(
        t1.content_hash(),
        t2.content_hash(),
        "same content → same hash"
    );
    assert_ne!(
        t1.content_hash(),
        t3.content_hash(),
        "different content → different hash"
    );
}

// ---------------------------------------------------------------------------
// Test: Rule implements content_hash (returns Some)
// ---------------------------------------------------------------------------

#[test]
fn test_rule_has_content_hash() {
    use crate::rule::Rule;
    let r = Rule::new();
    assert!(r.content_hash().is_some(), "Rule should return Some(hash)");
}

// ---------------------------------------------------------------------------
// Test: existing layout tests still compile (smoke check)
// ---------------------------------------------------------------------------

#[test]
fn test_layout_render_unchanged_smoke() {
    let console = Console::builder().width(40).height(5).build();
    let options = console.options();
    let mut layout = Layout::default_layout();
    layout.update("hello".to_string());
    layout.name = Some("main".to_string());
    let render_map = layout.render(&console, &options);
    assert!(render_map.contains_key("main"));
}
