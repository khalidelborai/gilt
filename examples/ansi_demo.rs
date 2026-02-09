//! Demonstrates converting raw ANSI escape codes into gilt-styled Text.

use gilt::ansi::AnsiDecoder;
use gilt::console::Console;
use gilt::prelude::*;

fn main() {
    let mut console = Console::builder()
        .width(90)
        .force_terminal(true)
        .no_color(false)
        .build();

    console.print(&Rule::with_title("ANSI Escape Code Parsing"));

    // A collection of raw ANSI strings to demonstrate various escape codes.
    let samples: &[(&str, &str)] = &[
        ("Red text", "\x1b[31mred text\x1b[0m"),
        ("Bold green", "\x1b[1;32mbold green\x1b[0m"),
        ("Italic cyan", "\x1b[3;36mitalic cyan\x1b[0m"),
        ("Underline yellow", "\x1b[4;33munderline yellow\x1b[0m"),
        ("Bold + italic magenta", "\x1b[1;3;35mbold italic magenta\x1b[0m"),
        ("Dim blue", "\x1b[2;34mdim blue\x1b[0m"),
        (
            "256-color (orange, index 208)",
            "\x1b[38;5;208morange 256-color\x1b[0m",
        ),
        (
            "True color (RGB 100,200,50)",
            "\x1b[38;2;100;200;50mtrue color green\x1b[0m",
        ),
        (
            "Background (white on red)",
            "\x1b[37;41mwhite on red\x1b[0m",
        ),
        (
            "Strikethrough",
            "\x1b[9mstrikethrough text\x1b[0m",
        ),
    ];

    // -- "Before" vs "After" table -------------------------------------------

    console.print(&Rule::with_title("Before & After"));

    let mut table = Table::new(&["Description", "Raw Bytes", "Parsed Output"]);

    let mut decoder = AnsiDecoder::new();

    for (description, raw) in samples {
        // "Raw bytes" column: show the escape codes as visible text
        let raw_display = raw
            .replace('\x1b', "\\x1b")
            .replace('\n', "\\n");

        // Parse the ANSI string into gilt Text
        let parsed_lines = decoder.decode(raw);

        // Collect the plain text from parsed output (the rendering is styled
        // when printed via Console, but for the table cell we show the plain
        // text to demonstrate that parsing succeeded).
        let plain: String = parsed_lines.iter().map(|t| t.plain().to_string()).collect();

        table.add_row(&[description, &raw_display, &plain]);
    }

    console.print(&table);

    // -- Direct rendering of parsed ANSI output ------------------------------

    console.print(&Rule::with_title("Rendered Output (with colors)"));

    let mut decoder = AnsiDecoder::new();

    for (_description, raw) in samples {
        let parsed_lines = decoder.decode(raw);
        for text in &parsed_lines {
            console.print(text);
        }
    }

    // -- Multi-line ANSI string -----------------------------------------------

    console.print(&Rule::with_title("Multi-line ANSI Input"));

    let multiline = "\x1b[1;34m=== Build Report ===\x1b[0m\n\
                     \x1b[32m  [PASS]\x1b[0m Unit tests: 142 passed\n\
                     \x1b[32m  [PASS]\x1b[0m Integration tests: 38 passed\n\
                     \x1b[33m  [WARN]\x1b[0m 3 deprecation warnings\n\
                     \x1b[31m  [FAIL]\x1b[0m Linting: 1 error";

    let mut decoder = AnsiDecoder::new();
    let lines = decoder.decode(multiline);
    for line in &lines {
        console.print(line);
    }
}
