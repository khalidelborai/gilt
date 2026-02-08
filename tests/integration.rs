//! End-to-end integration tests that verify the full rendering pipeline.
//!
//! Each test creates a Console in capture mode and renders widgets through
//! the full pipeline: Renderable -> Segments -> ANSI output.

use gilt::prelude::*;
use gilt::style::Style;
use gilt::text::Text;

// ---------------------------------------------------------------------------
// Console + Text + Style
// ---------------------------------------------------------------------------

#[test]
fn text_with_markup_renders() {
    let mut c = Console::builder().width(60).force_terminal(true).build();
    c.begin_capture();
    c.print_text("[bold red]Error:[/bold red] something went wrong");
    let output = c.end_capture();
    assert!(output.contains("Error:"));
    assert!(output.contains("something went wrong"));
}

#[test]
fn text_from_markup_roundtrip() {
    let text = Text::from_markup("[italic]hello[/italic] world").unwrap();
    assert_eq!(text.plain(), "hello world");
    assert!(!text.spans().is_empty());
}

#[test]
fn text_from_ansi_roundtrip() {
    let text = Text::from_ansi("\x1b[1mBold\x1b[0m Normal");
    assert_eq!(text.plain(), "Bold Normal");
}

// ---------------------------------------------------------------------------
// Table
// ---------------------------------------------------------------------------

#[test]
fn table_renders_with_data() {
    let mut table = Table::new(&["Name", "Age"]);
    table.add_row(&["Alice", "30"]);
    table.add_row(&["Bob", "25"]);

    let mut c = Console::builder().width(40).force_terminal(true).build();
    c.begin_capture();
    c.print(&table);
    let output = c.end_capture();
    assert!(output.contains("Alice"));
    assert!(output.contains("Bob"));
    assert!(output.contains("30"));
    assert!(output.contains("25"));
}

// ---------------------------------------------------------------------------
// Panel
// ---------------------------------------------------------------------------

#[test]
fn panel_wraps_text() {
    let text = Text::new("Inside the panel", Style::null());
    let panel = Panel::new(text);

    let mut c = Console::builder().width(40).force_terminal(true).build();
    c.begin_capture();
    c.print(&panel);
    let output = c.end_capture();
    assert!(output.contains("Inside the panel"));
}

// ---------------------------------------------------------------------------
// Rule
// ---------------------------------------------------------------------------

#[test]
fn rule_renders_with_title() {
    let mut c = Console::builder().width(40).force_terminal(true).build();
    c.begin_capture();
    c.rule(Some("Section"));
    let output = c.end_capture();
    assert!(output.contains("Section"));
}

#[test]
fn rule_renders_without_title() {
    let mut c = Console::builder().width(40).force_terminal(true).build();
    c.begin_capture();
    c.rule(None);
    let output = c.end_capture();
    assert!(!output.trim().is_empty());
}

// ---------------------------------------------------------------------------
// Markdown
// ---------------------------------------------------------------------------

#[test]
fn markdown_renders_heading() {
    let md = Markdown::new("# Hello World\n\nSome paragraph text.");

    let mut c = Console::builder().width(60).force_terminal(true).build();
    c.begin_capture();
    c.print(&md);
    let output = c.end_capture();
    assert!(output.contains("Hello World"));
    assert!(output.contains("Some paragraph text"));
}

// ---------------------------------------------------------------------------
// Syntax
// ---------------------------------------------------------------------------

#[test]
fn syntax_highlights_rust() {
    let code = r#"fn main() {
    println!("Hello, world!");
}"#;
    let syn = Syntax::new(code, "rust");

    let mut c = Console::builder().width(60).force_terminal(true).build();
    c.begin_capture();
    c.print(&syn);
    let output = c.end_capture();
    assert!(output.contains("main"));
    assert!(output.contains("Hello, world!"));
}

// ---------------------------------------------------------------------------
// Tree
// ---------------------------------------------------------------------------

#[test]
fn tree_renders_hierarchy() {
    let mut tree = Tree::new(Text::new("Root", Style::null()));
    tree.add(Text::new("Child 1", Style::null()));
    let subtree = tree.add(Text::new("Child 2", Style::null()));
    subtree.add(Text::new("Grandchild", Style::null()));

    let mut c = Console::builder().width(40).force_terminal(true).build();
    c.begin_capture();
    c.print(&tree);
    let output = c.end_capture();
    assert!(output.contains("Root"));
    assert!(output.contains("Child 1"));
    assert!(output.contains("Child 2"));
    assert!(output.contains("Grandchild"));
}

// ---------------------------------------------------------------------------
// JSON
// ---------------------------------------------------------------------------

#[test]
fn json_pretty_prints() {
    let mut c = Console::builder().width(60).force_terminal(true).build();
    c.begin_capture();
    c.print_json(r#"{"name": "gilt", "version": 1}"#);
    let output = c.end_capture();
    assert!(output.contains("name"));
    assert!(output.contains("gilt"));
}

// ---------------------------------------------------------------------------
// Columns
// ---------------------------------------------------------------------------

#[test]
fn columns_renders_grid() {
    let mut cols = Columns::new();
    for i in 1..=6 {
        cols.add_renderable(&format!("Item {}", i));
    }

    let mut c = Console::builder().width(80).force_terminal(true).build();
    c.begin_capture();
    c.print(&cols);
    let output = c.end_capture();
    assert!(output.contains("Item 1"));
    assert!(output.contains("Item 6"));
}

// ---------------------------------------------------------------------------
// Console convenience methods
// ---------------------------------------------------------------------------

#[test]
fn console_log_includes_timestamp() {
    let mut c = Console::builder().width(60).force_terminal(true).build();
    c.begin_capture();
    c.log("test message");
    let output = c.end_capture();
    assert!(output.contains("test message"));
}

#[test]
fn console_line_outputs_newlines() {
    let mut c = Console::builder().width(40).build();
    c.begin_capture();
    c.line(3);
    let output = c.end_capture();
    let newline_count = output.chars().filter(|&ch| ch == '\n').count();
    assert!(newline_count >= 3);
}

// ---------------------------------------------------------------------------
// Export: text, HTML, SVG
// ---------------------------------------------------------------------------

#[test]
fn export_text_captures_output() {
    let mut c = Console::builder()
        .width(40)
        .record(true)
        .force_terminal(true)
        .build();
    c.print_text("Hello export");
    let text = c.export_text(false, false);
    assert!(text.contains("Hello export"));
}

#[test]
fn export_html_contains_markup() {
    let mut c = Console::builder()
        .width(40)
        .record(true)
        .force_terminal(true)
        .build();
    c.print_text("[bold]Bold text[/bold]");
    let html = c.export_html(None, false, true);
    assert!(html.contains("Bold text"));
}

#[test]
fn export_svg_contains_svg_tags() {
    let mut c = Console::builder()
        .width(40)
        .record(true)
        .force_terminal(true)
        .build();
    c.print_text("SVG content");
    let svg = c.export_svg("gilt", None, false, None, 0.61);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("SVG content"));
}

// ---------------------------------------------------------------------------
// Style parsing
// ---------------------------------------------------------------------------

#[test]
fn style_parse_complex() {
    let style = Style::parse("bold italic red on blue").unwrap();
    assert_eq!(style.bold(), Some(true));
    assert_eq!(style.italic(), Some(true));
}

// ---------------------------------------------------------------------------
// Measure
// ---------------------------------------------------------------------------

#[test]
fn measure_text_widget() {
    let text = Text::new("Hello World", Style::null());
    let c = Console::builder().width(80).build();
    let m = c.measure(&text);
    assert!(m.minimum > 0);
    assert!(m.maximum > 0);
    assert!(m.maximum >= m.minimum);
}

// ---------------------------------------------------------------------------
// ProgressBar standalone
// ---------------------------------------------------------------------------

#[test]
fn progress_bar_renders() {
    let bar = ProgressBar::new()
        .with_total(Some(100.0))
        .with_completed(50.0);

    let mut c = Console::builder().width(40).force_terminal(true).build();
    c.begin_capture();
    c.print(&bar);
    let output = c.end_capture();
    assert!(!output.is_empty());
}

// ---------------------------------------------------------------------------
// Global convenience functions
// ---------------------------------------------------------------------------

#[test]
fn global_print_text_does_not_panic() {
    gilt::with_console(|c| {
        c.begin_capture();
        c.print_text("global test");
        let output = c.end_capture();
        assert!(output.contains("global test"));
    });
}
