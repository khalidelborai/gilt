//! Inline terminal image demo.
//!
//! gilt auto-selects the best protocol your terminal supports:
//!   Kitty graphics  →  iTerm2 (OSC 1337)  →  halfblock `▀` (works everywhere)
//!
//! Run it:
//!   cargo run --example image
//!       Procedural gradient via halfblock or Kitty — no extra features needed.
//!
//!   cargo run --example image --features inline-images
//!       Also enables the iTerm2 protocol (PNG-encoded) and loading real files.
//!
//!   cargo run --example image --features inline-images -- path/to/photo.png
//!       Display an actual image file.

use gilt::console::Console;
use gilt::image::Image;
use gilt::rule::Rule;

/// Linear interpolation across three Dracula stops: purple → pink → cyan.
fn gradient_stop(t: f32) -> (u8, u8, u8) {
    let stops = [(189u8, 147u8, 249u8), (255, 121, 198), (139, 233, 253)];
    let (seg, local) = if t < 0.5 {
        (0, t / 0.5)
    } else {
        (1, (t - 0.5) / 0.5)
    };
    let (a, b) = (stops[seg], stops[seg + 1]);
    let lerp = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * local).round() as u8;
    (lerp(a.0, b.0), lerp(a.1, b.1), lerp(a.2, b.2))
}

/// Build a colourful `w × h` RGBA image with no external file:
/// a diagonal Dracula gradient with a bright disc in the centre.
fn gradient_image(w: u32, h: u32) -> Image {
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);
    let radius = w.min(h) as f32 * 0.34;
    for y in 0..h {
        for x in 0..w {
            let t = (x as f32 + y as f32) / (w as f32 + h as f32);
            let (mut r, mut g, mut b) = gradient_stop(t);
            // bright yellow disc (#f1fa8c) in the middle
            let (dx, dy) = (x as f32 - cx, y as f32 - cy);
            if (dx * dx + dy * dy).sqrt() < radius {
                (r, g, b) = (241, 250, 140);
            }
            rgba.extend_from_slice(&[r, g, b, 255]);
        }
    }
    Image::from_rgba(w, h, rgba)
}

fn main() {
    let mut console = Console::default();

    // Which protocol will this terminal use? (read caps before borrowing mutably)
    let (kitty, iterm) = {
        let caps = console.capabilities();
        (caps.kitty, caps.iterm)
    };
    let protocol = if kitty {
        "Kitty graphics protocol"
    } else if iterm {
        "iTerm2 inline images (OSC 1337)"
    } else {
        "halfblock (▀) — the universal fallback"
    };

    console.print(&Rule::with_title("gilt — inline images"));
    console.print_text(&format!("This terminal will use: [bold cyan]{protocol}[/]"));
    console.print_text("");

    // A procedural image — always works, no file or feature required.
    console.print_text("[bold]Procedural gradient[/] (Image::from_rgba):");
    console.print(&gradient_image(120, 120).width(32));
    console.print_text("");

    // Optionally load a real image file (needs the `inline-images` feature).
    #[cfg(feature = "inline-images")]
    {
        match std::env::args().nth(1) {
            Some(path) => {
                console.print(&Rule::with_title("file"));
                match Image::from_path(&path) {
                    Ok(img) => console.print(&img.width(48)),
                    Err(e) => console.print_text(&format!("[bold red]could not load {path}:[/] {e}")),
                }
            }
            None => {
                console.print_text(
                    "[dim]tip: pass a path to display a real image file:[/]\n\
                     [dim]  cargo run --example image --features inline-images -- photo.png[/]",
                );
            }
        }
    }
    #[cfg(not(feature = "inline-images"))]
    {
        console.print_text(
            "[dim]build with [/][dim cyan]--features inline-images[/][dim] to load image files \
             and enable the iTerm2 protocol.[/]",
        );
    }
}
