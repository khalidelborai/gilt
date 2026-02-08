//! Demonstrates gilt's export pipeline — record console output then export
//! to plain text, HTML, and SVG.

use gilt::console::Console;
use gilt::rule::Rule;
use gilt::style::Style;
use gilt::table::Table;
use gilt::text::Text;

fn main() {
    // Create a recording console
    let mut console = Console::builder()
        .record(true)
        .force_terminal(true)
        .width(60)
        .no_color(false)
        .build();

    // -- Print several things into the recording buffer --------------------

    // 1. A rule
    let rule = Rule::with_title("Gilt Export Demo");
    console.print(&rule);

    // 2. Some styled text
    let mut text = Text::new("Hello from ", Style::null());
    text.append_str("gilt", Some(Style::parse("bold green").unwrap()));
    text.append_str("! This text is ", None);
    text.append_str("recorded", Some(Style::parse("italic cyan").unwrap()));
    text.append_str(" for export.", None);
    console.print(&text);

    // 3. A small table
    let mut table = Table::new(&["Language", "Typing", "Speed"]);
    table.add_row(&["Rust",   "Static",  "Fast"]);
    table.add_row(&["Python", "Dynamic", "Moderate"]);
    table.add_row(&["Go",     "Static",  "Fast"]);
    console.print(&table);

    // Another rule to close
    let rule = Rule::new();
    console.print(&rule);

    // -- Export to plain text ----------------------------------------------

    println!("=== Exported Plain Text ===\n");
    let plain = console.export_text(false, false);
    print!("{}", plain);

    // -- Export to HTML ----------------------------------------------------

    let html = console.export_html(None, false, true);
    std::fs::write("/tmp/gilt_demo.html", &html).expect("Failed to write HTML");

    // -- Export to SVG -----------------------------------------------------

    let svg = console.export_svg("gilt demo", None, false, None, 0.61);
    std::fs::write("/tmp/gilt_demo.svg", &svg).expect("Failed to write SVG");

    println!("\n=== Export Complete ===");
    println!("HTML saved to: /tmp/gilt_demo.html");
    println!("SVG  saved to: /tmp/gilt_demo.svg");
}
