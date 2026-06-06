//! Inline terminal images with `Image::from_rgba`.
//!
//! Demonstrates:
//! - `Image::from_rgba(w, h, pixels)` — always available, no extra feature.
//! - `.width(cols)` — set target display width in terminal columns.
//! - `console.print(&image)` — the halfblock renderer fires in recording mode,
//!   producing `▀` half-block characters styled with TrueColor SGR codes.
//! - `from_path` / `from_bytes` require `--features inline-images` (note below).
//!
//! Run with: cargo run --example inline_image
//! For file/bytes loading: cargo run --example inline_image --features inline-images

use gilt::console::Console;
use gilt::image::Image;

fn main() {
    // Build a 32×16 horizontal gradient: left = red, right = blue.
    let width: u32 = 32;
    let height: u32 = 16;
    let mut pixels: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);
    for _row in 0..height {
        for col in 0..width {
            let t = col as f32 / (width - 1) as f32;
            let r = (255.0 * (1.0 - t)) as u8;
            let g = 0u8;
            let b = (255.0 * t) as u8;
            pixels.extend_from_slice(&[r, g, b, 255]);
        }
    }

    // Render at 40 terminal columns; aspect ratio is preserved automatically.
    let image = Image::from_rgba(width, height, pixels).width(40);

    // Use record(true) + force_terminal(true) + truecolor to guarantee halfblock
    // output regardless of the actual terminal type (safe in CI/pipes).
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .color_system("truecolor")
        .record(true) // forces halfblock path (skips kitty/sixel)
        .build();

    println!("=== Halfblock gradient (32×16 px → 40 cols) ===");
    console.print(&image);

    // Verify halfblock characters were produced.
    let exported = console.export_html(None, false, false);
    assert!(
        exported.contains('\u{2580}'),
        "halfblock ▀ must appear in the export"
    );
    println!("\n[assertion passed] ▀ halfblock character found in HTML export");

    // --- Feature note --------------------------------------------------------
    // Add --features inline-images to enable:
    //   let img = Image::from_path("photo.png").unwrap();
    //   let img = Image::from_bytes(include_bytes!("../assets/logo.png")).unwrap();
    println!("\n[tip] Use --features inline-images to enable Image::from_path / from_bytes.");
}
