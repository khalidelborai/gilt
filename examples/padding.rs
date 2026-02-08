//! Demonstrates gilt's Padding widget — adding space around content.

use gilt::console::Console;
use gilt::padding::{Padding, PaddingDimensions};
use gilt::rule::Rule;
use gilt::style::Style;
use gilt::text::Text;

fn main() {
    let mut console = Console::builder()
        .width(60)
        .force_terminal(true)
        .no_color(false)
        .build();

    // -- 1. Uniform Padding ---------------------------------------------------

    console.print(&Rule::with_title("Uniform Padding (2)"));

    let content = Text::new("Hello, World!", Style::null());
    let padded = Padding::new(
        content,
        PaddingDimensions::Uniform(2),
        Style::parse("on blue").unwrap(),
        true,
    );
    console.print(&padded);

    // -- 2. Asymmetric Padding (top/bottom, left/right) -----------------------

    console.print(&Rule::with_title("Pair Padding (1, 4)"));

    let content2 =
        Text::from_markup("[bold yellow]Padded text[/bold yellow] with vertical=1, horizontal=4")
            .unwrap();
    let padded2 = Padding::new(
        content2,
        PaddingDimensions::Pair(1, 4),
        Style::parse("on dark_green").unwrap(),
        true,
    );
    console.print(&padded2);

    // -- 3. Full Padding (top, right, bottom, left) ---------------------------

    console.print(&Rule::with_title("Full Padding (0, 8, 0, 4)"));

    let content3 = Text::new("Custom padding on each side.", Style::null());
    let padded3 = Padding::new(
        content3,
        PaddingDimensions::Full(0, 8, 0, 4),
        Style::parse("on red").unwrap(),
        true,
    );
    console.print(&padded3);

    // -- 4. Indent (left padding only) ----------------------------------------

    console.print(&Rule::with_title("Indent (level 8)"));

    let content4 =
        Text::from_markup("[italic]Indented text[/italic] using Padding::indent()").unwrap();
    let indented = Padding::indent(content4, 8);
    console.print(&indented);
}
