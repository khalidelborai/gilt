//! Demonstrates the Figlet widget -- large ASCII art text using block characters.
//!
//! Run with: `cargo run --example figlet`

use gilt::color::Color;
use gilt::console::Console;
use gilt::figlet::Figlet;
use gilt::gradient::Gradient;
use gilt::rule::Rule;
use gilt::style::Style;

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    // -- "GILT" with gradient coloring ------------------------------------
    console.print(&Rule::with_title("GILT Banner (Gradient)"));

    // Render the figlet text, then apply a gradient across it
    let banner = Figlet::new("GILT");
    let banner_text = format!("{}", banner);
    let gradient = Gradient::new(
        &banner_text,
        vec![
            Color::from_rgb(255, 215, 0), // gold
            Color::from_rgb(255, 140, 0), // dark orange
            Color::from_rgb(255, 69, 0),  // red-orange
        ],
    );
    console.print(&gradient);

    // -- "Hello" in red ---------------------------------------------------
    console.print(&Rule::with_title("Styled Figlet Text"));

    let hello = Figlet::new("Hello").with_style(Style::parse("bold red").unwrap());
    console.print(&hello);

    // -- Numbers ----------------------------------------------------------
    console.print(&Rule::with_title("Numbers"));

    let numbers = Figlet::new("0123456789");
    console.print(&numbers);

    // -- Mixed text -------------------------------------------------------
    console.print(&Rule::with_title("Mixed Text"));

    let mixed = Figlet::new("Hi!");
    console.print(&mixed);

    // -- Width-constrained (wrap) -----------------------------------------
    console.print(&Rule::with_title("Width Constrained (40 cols)"));

    let wide = Figlet::new("ABCDEF").with_width(40);
    console.print(&wide);

    // -- Display trait ----------------------------------------------------
    console.print(&Rule::with_title("Display Trait (println!)"));

    let display_fig = Figlet::new("OK");
    println!("{}", display_fig);
}
