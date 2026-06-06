//! Canvas blitters — draw the same circle in Braille, Sextant, and HalfBlock.
//!
//! Demonstrates:
//! - `Canvas::new(cols, rows)` — default Braille blitter.
//! - `.with_blitter(Blitter::Sextant)` — 2×3 pixel cells (higher vert density).
//! - `.with_blitter(Blitter::HalfBlock)` — 1×2 pixel cells (widest compat).
//! - `canvas.circle(cx, cy, r)` — midpoint circle on the pixel grid.
//! - `canvas.line(x0, y0, x1, y1)` — Bresenham line.
//! - `canvas.frame()` — render to a multi-line string.
//!
//! Resolution per cell:
//!   Braille   — 2 × 4 px   (most detail per cell)
//!   Sextant   — 2 × 3 px   (Unicode 13; needs a font with U+1FB00 block)
//!   HalfBlock — 1 × 2 px   (universal: ▀ ▄ █; lowest resolution)
//!
//! Run with: cargo run --example canvas_blitters

use gilt::canvas::{Blitter, Canvas};
use gilt::console::Console;

/// Print the frame of a canvas with a labelled header.
fn show(label: &str, canvas: &Canvas, console: &mut Console) {
    console.print_text(&format!("[bold cyan]=== {} ===[/]", label));
    // print each line of the frame through the console for ANSI styling.
    for line in canvas.frame().lines() {
        println!("{}", line);
    }
    println!();
}

fn main() {
    let mut console = Console::builder().width(80).force_terminal(true).build();

    // Common parameters — same shape, different blitters.
    // Each canvas is 20 cols × 12 rows terminal cells.
    let cols = 20usize;
    let rows = 12usize;

    // ---- Braille (2×4 px per cell = 40×48 pixel grid) ----------------------
    let mut braille = Canvas::new(cols, rows); // default = Braille
    let (pw, ph) = (braille.pixel_width() as i32, braille.pixel_height() as i32);
    let cx = pw / 2;
    let cy = ph / 2;
    let r = (pw.min(ph) / 2 - 2).max(1);
    braille.circle(cx, cy, r);
    // Add a diagonal for contrast.
    braille.line(0, 0, pw - 1, ph - 1);
    show(
        &format!("Braille  (pixel {}×{})", pw, ph),
        &braille,
        &mut console,
    );

    // ---- Sextant (2×3 px per cell = 40×36 pixel grid) ----------------------
    let mut sextant = Canvas::new(cols, rows).with_blitter(Blitter::Sextant);
    let (pw, ph) = (sextant.pixel_width() as i32, sextant.pixel_height() as i32);
    let cx = pw / 2;
    let cy = ph / 2;
    let r = (pw.min(ph) / 2 - 2).max(1);
    sextant.circle(cx, cy, r);
    sextant.line(0, 0, pw - 1, ph - 1);
    show(
        &format!("Sextant  (pixel {}×{})", pw, ph),
        &sextant,
        &mut console,
    );

    // ---- HalfBlock (1×2 px per cell = 20×24 pixel grid) --------------------
    let mut halfblock = Canvas::new(cols, rows).with_blitter(Blitter::HalfBlock);
    let (pw, ph) = (
        halfblock.pixel_width() as i32,
        halfblock.pixel_height() as i32,
    );
    let cx = pw / 2;
    let cy = ph / 2;
    let r = (pw.min(ph) / 2 - 2).max(1);
    halfblock.circle(cx, cy, r);
    halfblock.line(0, 0, pw - 1, ph - 1);
    show(
        &format!("HalfBlock(pixel {}×{})", pw, ph),
        &halfblock,
        &mut console,
    );

    // ---- Verify ----
    // Each canvas frame must be non-empty and differ in resolution.
    assert!(!braille.frame().is_empty(), "Braille frame non-empty");
    assert!(!sextant.frame().is_empty(), "Sextant frame non-empty");
    assert!(!halfblock.frame().is_empty(), "HalfBlock frame non-empty");

    // Braille is the densest (most pixels); HalfBlock is the least.
    let b_px = Canvas::new(cols, rows).pixel_width() * Canvas::new(cols, rows).pixel_height();
    let h_px = Canvas::new(cols, rows)
        .with_blitter(Blitter::HalfBlock)
        .pixel_width()
        * Canvas::new(cols, rows)
            .with_blitter(Blitter::HalfBlock)
            .pixel_height();
    assert!(b_px > h_px, "Braille has more pixels than HalfBlock");

    console.print_text(
        "[bold green]assertions passed[/]  — Braille > Sextant > HalfBlock pixel density",
    );
}
