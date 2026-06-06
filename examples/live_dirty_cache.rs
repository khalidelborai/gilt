//! Layout dirty cache — per-region hash-based skip for unchanged panes.
//!
//! Demonstrates:
//! - `Layout` with two named leaf panes: a static header and a changing body.
//! - `Renderable::content_hash()` — `Text` implements this; returning
//!   `Some(hash)` lets the cache skip re-rendering when the hash is the same.
//! - `LayoutCache::new()` — an empty cache to pass across render calls.
//! - `Layout::render_with_cache(console, options, cache)` — renders leaf panes,
//!   reusing cached segments for unchanged panes (hash match) and re-rendering
//!   only changed ones.
//!
//! The example renders the layout twice.  The body text changes between renders,
//! so the body pane is rendered both times.  The header text stays constant, so
//! its `content_hash` matches on the second call — the cache skips it.
//!
//! Run with: cargo run --example live_dirty_cache

use gilt::console::{Console, ConsoleOptions, Renderable};
use gilt::layout::{Layout, LayoutCache};
use gilt::segment::Segment;
use gilt::style::Style;
use gilt::text::Text;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Instrumented renderable: counts calls and forwards to a fixed Text.
// ---------------------------------------------------------------------------

struct CountingText {
    text: Text,
    calls: Arc<AtomicUsize>,
}

impl CountingText {
    fn new(content: &str, calls: Arc<AtomicUsize>) -> Self {
        CountingText {
            text: Text::new(content, Style::null()),
            calls,
        }
    }
}

impl Renderable for CountingText {
    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.text.gilt_console(console, options)
    }

    // Delegate to Text's hash — same content = same hash = cache hit.
    fn content_hash(&self) -> Option<u64> {
        self.text.content_hash()
    }
}

fn main() {
    let console = Console::builder().width(60).height(10).build();
    let options: ConsoleOptions = console.options().update_dimensions(60, 10);

    // --- Build a two-pane column layout: header (fixed 1 row) + body ---
    let header_calls = Arc::new(AtomicUsize::new(0));
    let body_calls = Arc::new(AtomicUsize::new(0));

    let mut root = Layout::default_layout();
    root.name = Some("root".to_string());

    let mut header_pane = Layout::new(None, Some("header".to_string()), Some(1), None, None, None);
    header_pane.update_renderable(CountingText::new(
        "=== Static Header ===",
        Arc::clone(&header_calls),
    ));

    let mut body_pane = Layout::new(None, Some("body".to_string()), None, None, Some(1), None);
    body_pane.update_renderable(CountingText::new(
        "Frame 1 body content",
        Arc::clone(&body_calls),
    ));

    root.split_column(vec![header_pane, body_pane]);

    // --- First render --------------------------------------------------------
    let mut cache = LayoutCache::new();
    let render1 = root.render_with_cache(&console, &options, &mut cache);
    println!("=== Render 1 ===");
    for (name, (_region, lines)) in &render1 {
        let flat: String = lines
            .iter()
            .flat_map(|row| row.iter().map(|s| s.text.as_str()))
            .collect();
        println!("  pane {:12} → {:?}", name, flat.trim());
    }
    println!(
        "  header renders: {}  body renders: {}",
        header_calls.load(Ordering::SeqCst),
        body_calls.load(Ordering::SeqCst)
    );

    // --- Update only the body pane, keep header the same ---

    if let Some(body) = root.get_mut("body") {
        body.update_renderable(CountingText::new(
            "Frame 2 body content (CHANGED)",
            Arc::clone(&body_calls),
        ));
    }

    // --- Second render — header should be a cache hit ----------------------
    let render2 = root.render_with_cache(&console, &options, &mut cache);
    println!("\n=== Render 2 ===");
    for (name, (_region, lines)) in &render2 {
        let flat: String = lines
            .iter()
            .flat_map(|row| row.iter().map(|s| s.text.as_str()))
            .collect();
        println!("  pane {:12} → {:?}", name, flat.trim());
    }
    let h2 = header_calls.load(Ordering::SeqCst);
    let b2 = body_calls.load(Ordering::SeqCst);
    println!("  header renders: {}  body renders: {}", h2, b2);

    // Header content is unchanged: hash matches → still only 1 render.
    assert_eq!(
        h2, 1,
        "header should have been rendered only once (cache hit)"
    );
    // Body changed: must have been rendered both times.
    assert_eq!(
        b2, 2,
        "body should have been rendered twice (dirty each frame)"
    );

    println!("\n[assertions passed] dirty cache skipped header on frame 2");
}
