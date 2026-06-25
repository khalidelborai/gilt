//! Verifies gilt **2.3** Markdown inline images: a Markdown `![alt](path)` whose
//! `src` is a local file renders as a REAL inline image (Kitty / iTerm2 / Sixel /
//! halfblock, auto-selected), not a text placeholder.
//!
//! Requires the `inline-images` feature (PNG decoding):
//!   cargo run --example markdown_image --features inline-images
//!
//! Without the feature, Markdown keeps the text placeholder — run with it to see
//! the image.

#[cfg(not(feature = "inline-images"))]
fn main() {
    eprintln!(
        "This example needs the `inline-images` feature:\n  \
         cargo run --example markdown_image --features inline-images"
    );
}

#[cfg(feature = "inline-images")]
fn main() {
    use gilt::console::Console;
    use gilt::markdown::Markdown;

    // Generate a small gradient PNG to a temp path so the example is self-contained.
    let (w, h) = (64u32, 32u32);
    let mut img = ::image::RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            img.put_pixel(
                x,
                y,
                ::image::Rgba([(x * 255 / w) as u8, (y * 255 / h) as u8, 180, 255]),
            );
        }
    }
    let path = std::env::temp_dir().join("gilt_markdown_image_demo.png");
    img.save(&path).expect("write temp png");

    let doc = format!(
        "# Inline images in Markdown\n\n\
         gilt 2.3 renders a local-file image reference as a real inline image:\n\n\
         ![a gradient]({})\n\n\
         …and normal Markdown continues after it.\n",
        path.display()
    );

    // Record mode renders the inline image as halfblock (`▀`) — pure styled text
    // that prints correctly in ANY terminal (and won't block on a graphics-protocol
    // handshake). For a crisp Kitty / iTerm2 / Sixel image, drop `.record(true)` and
    // run this in a graphics-capable terminal.
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .color_system("truecolor")
        .record(true)
        .build();
    console.print(&Markdown::new(&doc));
    println!("(generated {})", path.display());
}
