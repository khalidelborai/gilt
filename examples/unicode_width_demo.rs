//! Unicode cell width demonstration
//!
//! This example demonstrates the improved Unicode cell width calculation
//! including ZWJ sequences, variation selectors, and regional indicators.
//!
//! Run: cargo run --example unicode_width_demo

use gilt::cells::cell_len;
use gilt::prelude::*;

fn main() {
    let mut console = Console::new();

    console.rule(Some("Unicode Cell Width Demo"));
    console.print_text("Demonstrating advanced Unicode width handling\n");

    // ========================================================================
    // 1. Basic Characters
    // ========================================================================
    console.rule(Some("Basic Characters"));

    let basics = [
        ("A", "ASCII letter", 1),
        ("ñ", "Latin accented", 1),
        ("中", "CJK character", 2),
        ("あ", "Japanese hiragana", 2),
        ("한", "Korean hangul", 2),
    ];

    for (text, desc, expected) in &basics {
        let actual = cell_len(text);
        let status = if actual == *expected {
            "[green]✓[/green]"
        } else {
            "[red]✗[/red]"
        };
        console.print_text(&format!(
            "{} {:6} {:20} expected={}, actual={}",
            status, text, desc, expected, actual
        ));
    }

    // ========================================================================
    // 2. Emoji
    // ========================================================================
    console.rule(Some("Emoji"));

    let emojis = [
        ("💩", "Single emoji", 2),
        ("😀", "Smiley", 2),
        ("🚀", "Rocket", 2),
        ("❤️", "Heart with VS16", 2),
        ("©️", "Copyright with VS16", 2),
    ];

    for (text, desc, expected) in &emojis {
        let actual = cell_len(text);
        let status = if actual == *expected {
            "[green]✓[/green]"
        } else {
            "[red]✗[/red]"
        };
        console.print_text(&format!(
            "{} {:6} {:25} expected={}, actual={}",
            status, text, desc, expected, actual
        ));
    }

    // ========================================================================
    // 3. ZWJ Sequences (Family emojis, etc.)
    // ========================================================================
    console.rule(Some("ZWJ Sequences (Family Emojis)"));

    let zwj = [
        ("👨‍👩‍👧‍👦", "Family (ZWJ)", 2),
        ("👨‍⚕️", "Man doctor", 2),
        ("👩‍🚀", "Woman astronaut", 2),
        ("🏳️‍🌈", "Rainbow flag", 2),
    ];

    for (text, desc, expected) in &zwj {
        let actual = cell_len(text);
        let status = if actual == *expected {
            "[green]✓[/green]"
        } else {
            "[red]✗[/red]"
        };
        console.print_text(&format!(
            "{} {:10} {:25} expected={}, actual={}",
            status, text, desc, expected, actual
        ));
    }

    // ========================================================================
    // 4. Regional Indicators (Flags)
    // ========================================================================
    console.rule(Some("Regional Indicators (Flags)"));

    let flags = [
        ("🇺🇸", "US flag", 2),
        ("🇬🇧", "UK flag", 2),
        ("🇯🇵", "Japan flag", 2),
        ("🇩🇪", "Germany flag", 2),
        ("🇫🇷", "France flag", 2),
    ];

    for (text, desc, expected) in &flags {
        let actual = cell_len(text);
        let status = if actual == *expected {
            "[green]✓[/green]"
        } else {
            "[red]✗[/red]"
        };
        console.print_text(&format!(
            "{} {:6} {:20} expected={}, actual={}",
            status, text, desc, expected, actual
        ));
    }

    // ========================================================================
    // 5. Box Drawing Characters (Tree guides)
    // ========================================================================
    console.rule(Some("Box Drawing (Tree Guides)"));

    let guides = [
        ("│   ", "Tree guide (thin)", 4),
        ("├── ", "Tree fork (thin)", 4),
        ("└── ", "Tree end (thin)", 4),
        ("┃   ", "Tree guide (bold)", 4),
        ("┣━━ ", "Tree fork (bold)", 4),
        ("┗━━ ", "Tree end (bold)", 4),
    ];

    for (text, desc, expected) in &guides {
        let actual = cell_len(text);
        let status = if actual == *expected {
            "[green]✓[/green]"
        } else {
            "[red]✗[/red]"
        };
        console.print_text(&format!(
            "{} {:10} {:25} expected={}, actual={}",
            status, text, desc, expected, actual
        ));
    }

    // ========================================================================
    // 6. Combined Unicode Text
    // ========================================================================
    console.rule(Some("Combined Unicode Text"));

    let text = "a👨‍👩‍👧‍👦🇺🇸💩";
    console.print_text(&format!("Text: {}", text));
    console.print_text(&format!("Total width: {} cells", cell_len(text)));
    console.print_text("(ASCII + ZWJ family + Flag + Emoji)");

    // ========================================================================
    // 7. Practical: Table with mixed content
    // ========================================================================
    console.rule(Some("Practical: Mixed Content Table"));

    let mut table = Table::new(&["Content", "Type", "Width"]);
    table.add_row(&["Hello", "ASCII", "5"]);
    table.add_row(&["💩", "Emoji", "2"]);
    table.add_row(&["👨‍👩‍👧‍👦", "ZWJ Family", "2"]);
    table.add_row(&["🇺🇸", "Flag", "2"]);
    table.add_row(&["中", "CJK", "2"]);
    console.print(&table);

    console.line(1);
    console.print_text("[green]✓[/green] Unicode width demo complete!");
}
