//! Verifies gilt **2.3** halfblock alpha compositing: a semi-transparent image is
//! composited over a configurable background via `Image::with_background`.
//!
//! Run it:  cargo run --example image_alpha
//!
//! Uses record mode (→ halfblock) so the compositing is visible in ANY terminal:
//! the left edge is fully transparent (= the background colour) and ramps to fully
//! opaque crimson on the right. Change the background and the transparent side
//! changes with it — that's the alpha blend.

use gilt::color::Color;
use gilt::console::Console;
use gilt::image::Image;
use gilt::rule::Rule;

/// A crimson swatch whose alpha ramps 0 → 255 left → right.
fn ramp(w: u32, h: u32) -> Vec<u8> {
    let mut v = Vec::with_capacity((w * h * 4) as usize);
    for _y in 0..h {
        for x in 0..w {
            let a = if w > 1 {
                (x * 255 / (w - 1)) as u8
            } else {
                255
            };
            v.extend_from_slice(&[220, 40, 60, a]); // crimson, varying alpha
        }
    }
    v
}

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .color_system("truecolor")
        .record(true) // halfblock path — alpha compositing visible, no terminal I/O
        .build();
    let (w, h) = (40u32, 10u32);

    console.print(&Rule::with_title(
        "Semi-transparent crimson over BLACK (default bg)",
    ));
    console.print(&Image::from_rgba(w, h, ramp(w, h)).width(40));

    console.print(&Rule::with_title("…over WHITE (Image::with_background)"));
    console.print(
        &Image::from_rgba(w, h, ramp(w, h))
            .with_background(Color::from_rgb(255, 255, 255))
            .width(40),
    );

    console.print(&Rule::with_title("…over Dracula purple"));
    console.print(
        &Image::from_rgba(w, h, ramp(w, h))
            .with_background(Color::from_rgb(189, 147, 249))
            .width(40),
    );

    println!("Left edge = pure background (alpha 0); right edge = opaque crimson (alpha 255).");
}
