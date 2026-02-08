//! Export a styled table to SVG.
//!
//! Run with: `cargo run --example save_table_svg`
//!
//! Demonstrates gilt's SVG export pipeline: create a recording console,
//! print a styled table into it, then export the recorded output as SVG.
//! The SVG is saved to `/tmp/gilt_table.svg`.

use gilt::console::Console;
use gilt::rule::Rule;
use gilt::style::Style;
use gilt::table::Table;
use gilt::text::Text;

fn main() {
    // Create a recording console so output is captured for export.
    let mut console = Console::builder()
        .record(true)
        .width(72)
        .force_terminal(true)
        .no_color(false)
        .build();

    // -- Header rule --
    console.print(&Rule::with_title("Programming Languages"));

    // -- Build a styled table --
    let mut table = Table::new(&[
        "Language", "Paradigm", "Typing", "First Appeared", "TIOBE Rank",
    ]);
    table.title = Some("Top Languages (2025)".to_string());

    table.add_row(&["Rust",       "Multi-paradigm", "Static",  "2010", "#14"]);
    table.add_row(&["Python",     "Multi-paradigm", "Dynamic", "1991", "#1"]);
    table.add_row(&["TypeScript", "Multi-paradigm", "Static",  "2012", "#7"]);
    table.add_row(&["Go",         "Concurrent",     "Static",  "2009", "#8"]);
    table.add_row(&["C",          "Procedural",     "Static",  "1972", "#2"]);
    table.add_row(&["Java",       "Object-oriented","Static",  "1995", "#3"]);
    table.add_row(&["Kotlin",     "Multi-paradigm", "Static",  "2011", "#17"]);
    table.add_row(&["Swift",      "Multi-paradigm", "Static",  "2014", "#16"]);

    console.print(&table);

    // -- Footer --
    let footer = Text::styled(
        "  Source: TIOBE Index, January 2025",
        Style::parse("dim italic").unwrap(),
    );
    console.print(&footer);
    console.print(&Rule::new());

    // -- Print to terminal as well (the record buffer captures it) --
    let plain = console.export_text(false, false);
    print!("{}", plain);

    // -- Export to SVG --
    let svg = console.export_svg("Gilt Table Demo", None, false, None, 0.61);
    let svg_len = svg.len();

    std::fs::write("/tmp/gilt_table.svg", &svg).expect("Failed to write SVG file");

    println!();
    println!("SVG exported successfully!");
    println!("  File: /tmp/gilt_table.svg");
    println!("  Size: {} bytes ({:.1} KB)", svg_len, svg_len as f64 / 1024.0);
    println!();

    // Show first few lines of the SVG as a preview.
    println!("SVG preview (first 5 lines):");
    for line in svg.lines().take(5) {
        println!("  {}", line);
    }
    println!("  ...");
}
