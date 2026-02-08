//! Demonstrates the Gradient widget — smooth color interpolation across text.
//!
//! Shows two-color gradients, rainbow gradients, multi-stop custom gradients,
//! gradients with bold styling, the Display trait, and Renderable integration.
//!
//! Run with: `cargo run --example gradient`

use gilt::color::Color;
use gilt::console::Console;
use gilt::gradient::Gradient;
use gilt::rule::Rule;
use gilt::style::Style;

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    // ── Two-Color Gradient ───────────────────────────────────────────────
    console.print(&Rule::with_title("Two-Color Gradient (Red to Blue)"));

    let red_to_blue = Gradient::two_color(
        "The quick brown fox jumps over the lazy dog",
        Color::from_rgb(255, 0, 0),
        Color::from_rgb(0, 0, 255),
    );
    console.print(&red_to_blue);

    // ── Rainbow Gradient ─────────────────────────────────────────────────
    console.print(&Rule::with_title("Rainbow Gradient"));

    let rainbow = Gradient::rainbow(
        "ROYGBIV: Red Orange Yellow Green Blue Indigo Violet — all the colors of the spectrum!",
    );
    console.print(&rainbow);

    // ── Multi-Stop Custom Gradient ───────────────────────────────────────
    console.print(&Rule::with_title("Multi-Stop Gradient (5 Stops)"));

    let multi_stop = Gradient::new(
        "This text flows through magenta, orange, cyan, lime green, and gold.",
        vec![
            Color::from_rgb(255, 0, 255), // magenta
            Color::from_rgb(255, 165, 0), // orange
            Color::from_rgb(0, 255, 255), // cyan
            Color::from_rgb(0, 255, 64),  // lime green
            Color::from_rgb(255, 215, 0), // gold
        ],
    );
    console.print(&multi_stop);

    // ── Gradient with Bold Style ─────────────────────────────────────────
    console.print(&Rule::with_title("Gradient + Bold Style"));

    let bold_style = Style::parse("bold").unwrap();
    let bold_gradient =
        Gradient::rainbow("Bold rainbow text stands out even more in the terminal!")
            .with_style(bold_style);
    console.print(&bold_gradient);

    // ── Display Trait (println!) ─────────────────────────────────────────
    console.print(&Rule::with_title("Display Trait (via println!)"));

    let display_gradient = Gradient::two_color(
        "Gradient implements Display for use with println!",
        Color::from_rgb(0, 200, 100),
        Color::from_rgb(100, 0, 200),
    );
    // Note: Display uses no_color=true, so this prints plain text.
    // The gradient colors are visible only through Console rendering.
    println!("  println! output: {}", display_gradient);

    // ── Gradient as Renderable ───────────────────────────────────────────
    console.print(&Rule::with_title("Gradient as Renderable (via Console)"));

    // Gradient implements Renderable, so Console.print() renders it
    // with full ANSI color support — each character gets its own color.
    let sunset = Gradient::new(
        "Sunset gradient: deep red through orange to warm yellow",
        vec![
            Color::from_rgb(139, 0, 0),     // dark red
            Color::from_rgb(255, 69, 0),    // red-orange
            Color::from_rgb(255, 140, 0),   // dark orange
            Color::from_rgb(255, 200, 0),   // amber
            Color::from_rgb(255, 255, 100), // warm yellow
        ],
    );
    console.print(&sunset);

    // ── Multi-line Gradient ──────────────────────────────────────────────
    console.print(&Rule::with_title("Multi-line Gradient"));

    let multiline = Gradient::rainbow(
        "Line one of the gradient\nLine two continues the rainbow\nLine three wraps it up",
    );
    console.print(&multiline);
}
