//! Demonstrates the Canvas widget -- Braille dot matrix terminal graphics.
//!
//! Shows: sine wave plot, box with diagonals, circle, scatter pattern.
//!
//! Run with: `cargo run --example canvas`

use gilt::canvas::Canvas;
use gilt::console::Console;
use gilt::rule::Rule;
use gilt::style::Style;

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    // -- Sine wave plot ------------------------------------------------------
    console.print(&Rule::with_title("Sine Wave (40x10 canvas)"));

    let mut wave = Canvas::new(40, 10).with_style(Style::parse("cyan").unwrap());
    let pw = wave.pixel_width();
    let ph = wave.pixel_height();
    for px in 0..pw {
        let t = px as f64 * 4.0 * std::f64::consts::PI / pw as f64;
        let y = ((t.sin() + 1.0) / 2.0 * (ph - 1) as f64) as usize;
        wave.set(px, y);
    }
    console.print(&wave);

    // -- Box with diagonals --------------------------------------------------
    console.print(&Rule::with_title("Box with Diagonals (20x8)"));

    let mut box_canvas = Canvas::new(20, 8).with_style(Style::parse("yellow").unwrap());
    let bw = box_canvas.pixel_width();
    let bh = box_canvas.pixel_height();
    box_canvas.rect(0, 0, bw, bh);
    box_canvas.line(0, 0, (bw - 1) as i32, (bh - 1) as i32);
    box_canvas.line((bw - 1) as i32, 0, 0, (bh - 1) as i32);
    console.print(&box_canvas);

    // -- Circle --------------------------------------------------------------
    console.print(&Rule::with_title("Circle (30x15)"));

    let mut circle_canvas = Canvas::new(30, 15).with_style(Style::parse("green").unwrap());
    let cx = circle_canvas.pixel_width() as i32 / 2;
    let cy = circle_canvas.pixel_height() as i32 / 2;
    let r = cx.min(cy) - 1;
    circle_canvas.circle(cx, cy, r);
    console.print(&circle_canvas);

    // -- Scatter pattern -----------------------------------------------------
    console.print(&Rule::with_title("Deterministic Scatter (25x8)"));

    let mut scatter = Canvas::new(25, 8).with_style(Style::parse("magenta").unwrap());
    let sw = scatter.pixel_width();
    let sh = scatter.pixel_height();
    for i in 0..100 {
        let x = (i * 37 + 13) % sw;
        let y = (i * 53 + 7) % sh;
        scatter.set(x, y);
    }
    console.print(&scatter);

    // -- Filled shapes -------------------------------------------------------
    console.print(&Rule::with_title("Filled Rectangle + Circle"));

    let mut shapes = Canvas::new(30, 8).with_style(Style::parse("red").unwrap());
    shapes.fill_rect(2, 2, 12, 10);
    shapes.circle(40, 16, 12);
    console.print(&shapes);

    // -- Display trait -------------------------------------------------------
    console.print(&Rule::with_title("Display Trait (via println!)"));
    let mut small = Canvas::new(5, 2);
    small.rect(0, 0, 10, 8);
    println!("{small}");
}
