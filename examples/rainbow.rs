//! Demonstrates rainbow text coloring — each character gets a different color.
//!
//! Colors cycle through the 8-bit color palette (indices 16..231), which
//! covers the 6x6x6 color cube. This produces a smooth rainbow gradient
//! across the text.

use gilt::color::Color;
use gilt::console::Console;
use gilt::rule::Rule;
use gilt::style::Style;
use gilt::text::Text;

/// Apply a rainbow gradient to each character of a string.
fn make_rainbow(content: &str, spread: f64) -> Text {
    let mut text = Text::empty();
    let chars: Vec<char> = content.chars().collect();
    let color_range = 231 - 16; // 6x6x6 cube: indices 16..=231

    for (i, ch) in chars.iter().enumerate() {
        let offset = ((i as f64 * spread) as usize) % color_range;
        let color_index = (16 + offset) as u8;
        let color = Color::from_ansi(color_index);
        let style = Style::from_color(Some(color), None);
        text.append_str(&ch.to_string(), Some(style));
    }
    text
}

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    console.print(&Rule::with_title("Rainbow Text"));

    // Classic rainbow text
    let message =
        "The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs!";
    let rainbow = make_rainbow(message, 1.0);
    console.print(&rainbow);

    // Wider color spread
    let message2 =
        "Rust is a systems programming language focused on safety, speed, and concurrency.";
    let rainbow2 = make_rainbow(message2, 3.0);
    console.print(&rainbow2);

    // Narrow spread — subtle gradient
    let message3 =
        "gilt brings Rich-style terminal formatting to Rust with full ANSI color support.";
    let rainbow3 = make_rainbow(message3, 0.5);
    console.print(&rainbow3);

    // Box-drawing rainbow
    console.print(&Rule::with_title("Rainbow Box"));

    let box_art = [
        "+----------------------------------------------+",
        "|  Every character gets its own color from the  |",
        "|  6x6x6 ANSI color cube (indices 16 to 231).  |",
        "+----------------------------------------------+",
    ];

    for (row, line) in box_art.iter().enumerate() {
        // Offset each row so the gradient shifts downward
        let mut text = Text::empty();
        let chars: Vec<char> = line.chars().collect();
        let color_range = 231 - 16;
        for (i, ch) in chars.iter().enumerate() {
            let offset = ((i + row * 8) * 2) % color_range;
            let color_index = (16 + offset) as u8;
            let color = Color::from_ansi(color_index);
            let style = Style::from_color(Some(color), None);
            text.append_str(&ch.to_string(), Some(style));
        }
        console.print(&text);
    }
}
