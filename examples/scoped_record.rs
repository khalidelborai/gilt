//! Scoped recording — capture a snippet of console output as HTML / SVG / text.
//!
//! Demonstrates:
//! - `console.scoped_record(|c| { … })` — capture only the output produced
//!   inside the closure, leaving the outer record buffer untouched.
//! - `Recording::to_text()` — plain-text export (no ANSI codes).
//! - `Recording::to_html()` — full HTML document with inline styles.
//! - `Recording::to_svg("title")` — SVG image suitable for README badges.
//!
//! Run with: cargo run --example scoped_record

use gilt::console::Console;

fn main() {
    let mut console = Console::builder()
        .width(72)
        .force_terminal(true)
        .record(true)
        .build();

    // Print something *before* the scoped section to show it is excluded.
    console.print_text("[dim]pre-scope line (not in recording)[/]");

    // Capture just the output produced inside the closure.
    let rec = console.scoped_record(|c| {
        c.print_text("[bold green]  ok [/]  all 47 tests passed");
        c.print_text("[bold red] err [/]  1 test failed: test_foo");
        c.print_text("[bold yellow]warn[/]  2 tests skipped");
    });

    // Print something *after* the scoped section — also excluded.
    console.print_text("[dim]post-scope line (not in recording)[/]");

    // --- Exports ---

    let plain = rec.to_text();
    println!("=== to_text() ({} bytes) ===", plain.len());
    // to_text strips ANSI codes; print it directly.
    print!("{}", plain);

    let html = rec.to_html();
    println!(
        "\n=== to_html() ({} bytes, first 120 chars) ===",
        html.len()
    );
    println!("{}", &html[..html.len().min(120)]);

    let svg = rec.to_svg("Test Results");
    println!("\n=== to_svg() ({} bytes, first 120 chars) ===", svg.len());
    println!("{}", &svg[..svg.len().min(120)]);

    // Verify that the outer console record buffer only has the pre/post lines.
    let outer = console.export_text(true, false);
    assert!(
        outer.contains("pre-scope line"),
        "outer buffer should have pre-scope line"
    );
    assert!(
        outer.contains("post-scope line"),
        "outer buffer should have post-scope line"
    );
    assert!(
        !outer.contains("all 47 tests"),
        "outer buffer must NOT contain the scoped content"
    );

    println!("\n[assertion passed] outer buffer does not contain scoped content");
}
