//! Integration tests proving the Phase-4 container generalization works end-to-end.
//!
//! Each test builds a composition of widgets (Panel, Table, Group, Tree, Live)
//! using the new `RenderableArc`-based API and asserts that the rendered output
//! contains the expected content.

use std::sync::Arc;

use gilt::live::Live;
use gilt::panel::Panel;
use gilt::prelude::*;
use gilt::rule::Rule;
use gilt::style::Style;
use gilt::table::Table;
use gilt::text::Text;
use gilt::tree::Tree;
use gilt::RenderableArc;

fn test_console() -> Console {
    Console::builder()
        .width(60)
        .force_terminal(true)
        .no_color(true)
        .build()
}

// ---------------------------------------------------------------------------
// 1. Panel wrapping a Table
// ---------------------------------------------------------------------------

#[test]
fn panel_wrapping_table_renders_headers_row_and_title() {
    let mut table = Table::new(&["Col", "Val"]);
    table.add_row(&["alpha", "1"]);

    let panel = Panel::new(table).with_title("Dashboard");

    let mut c = test_console();
    c.begin_capture();
    c.print(&panel);
    let output = c.end_capture();

    assert!(output.contains("Dashboard"), "panel title missing");
    assert!(output.contains("Col"), "table header 'Col' missing");
    assert!(output.contains("Val"), "table header 'Val' missing");
    assert!(output.contains("alpha"), "table row cell 'alpha' missing");
}

// ---------------------------------------------------------------------------
// 2. Group of mixed widgets
// ---------------------------------------------------------------------------

#[test]
fn group_of_mixed_widgets_renders_all() {
    let text: RenderableArc = Arc::new(Text::new("hello from text", Style::null()));
    let rule: RenderableArc = Arc::new(Rule::new());
    let panel: RenderableArc = Arc::new(Panel::new(Text::new("inside panel", Style::null())));

    let group = Group::new(vec![text, rule, panel]);

    let mut c = test_console();
    c.begin_capture();
    c.print(&group);
    let output = c.end_capture();

    assert!(output.contains("hello from text"), "text item missing");
    assert!(output.contains("inside panel"), "panel content missing");
    // Rule renders as a line of dashes/box chars — just check non-empty output
    assert!(!output.trim().is_empty(), "group output is empty");
}

// ---------------------------------------------------------------------------
// 3. Tree whose label is a Panel
// ---------------------------------------------------------------------------

#[test]
fn tree_with_panel_label_renders_panel_content() {
    let label_panel = Panel::new(Text::new("root label", Style::null()));
    let mut tree = Tree::new(label_panel);
    tree.add(Text::new("child node", Style::null()));

    let mut c = test_console();
    c.begin_capture();
    c.print(&tree);
    let output = c.end_capture();

    assert!(output.contains("root label"), "panel label content missing");
    assert!(output.contains("child node"), "child node missing");
}

// ---------------------------------------------------------------------------
// 4. Live::new(Panel::new(Table)) constructs without panic
// ---------------------------------------------------------------------------

#[test]
fn live_wrapping_panel_wrapping_table_constructs() {
    let mut table = Table::new(&["Col"]);
    table.add_row(&["row1"]);

    let panel = Panel::new(table);
    // Just constructing (not starting) must not panic.
    let _live = Live::new(panel);
}
