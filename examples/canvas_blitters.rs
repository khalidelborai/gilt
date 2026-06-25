//! `Canvas` blitters — one shape, four resolutions, side by side.
//!
//! The *same* drawing calls (a filled rect, a corner-to-corner diagonal, and
//! the largest circle that fits) are rendered through all four blitters. Only
//! the glyph table changes, so the difference you see is pure sub-cell density:
//!
//!   | Blitter   | cell (w×h) | Unicode block                  |
//!   |-----------|------------|--------------------------------|
//!   | Braille   | 2 × 4      | U+2800 Braille Patterns        |
//!   | Octant    | 2 × 4      | U+1CD00 Octants (Unicode 16)   |
//!   | Sextant   | 2 × 3      | U+1FB00 Legacy Computing (U13) |
//!   | HalfBlock | 1 × 2      | U+2580 Block Elements          |
//!
//! Braille and Octant share the top 2×4 density (Octant draws solid blocks
//! instead of dots); Sextant is 2×3; HalfBlock is the most portable at 1×2.
//! Octant needs a Unicode-16 font and Sextant a Unicode-13 one to render; the
//! pixel *resolution* is identical regardless of what your font can draw.
//!
//! Run with: `cargo run --example canvas_blitters`

use gilt::canvas::{Blitter, Canvas};
use gilt::color::Color;
use gilt::console::Console;
use gilt::gradient::Gradient;
use gilt::panel::Panel;
use gilt::style::Style;
use gilt::text::Text;

/// Draw the identical shape onto a canvas using `blitter`, scaled to whatever
/// pixel resolution that blitter yields for the given cell dimensions.
fn shape(cols: usize, rows: usize, blitter: Blitter, color: &str) -> Canvas {
    let mut c = Canvas::new(cols, rows)
        .with_blitter(blitter)
        .with_style(Style::parse(color));
    let pw = c.pixel_width() as i32;
    let ph = c.pixel_height() as i32;

    // A small filled rectangle anchored top-left.
    c.fill_rect(1, 1, (pw / 5).max(2) as usize, (ph / 5).max(2) as usize);
    // A diagonal spanning the whole grid.
    c.line(0, 0, pw - 1, ph - 1);
    // The largest centred circle that fits.
    c.circle(pw / 2, ph / 2, (pw.min(ph) / 2 - 1).max(1));
    c
}

/// Frame a canvas in a titled `Panel`; the title carries the blitter facts.
fn framed(name: &str, cell: &str, color: &str, canvas: Canvas) -> Panel {
    let (pw, ph) = (canvas.pixel_width(), canvas.pixel_height());
    Panel::new(canvas)
        .with_expand(false) // shrink the frame to the content + title width
        .with_title(Text::new(
            &format!(" {name} · {cell} cells · {pw}×{ph} px "),
            Style::parse(&format!("bold {color}")),
        ))
        .with_subtitle(Text::new(
            &format!(" {} pixels ", pw * ph),
            Style::parse("italic #6272a4"),
        ))
        .with_border_style(Style::parse("#44475a"))
}

fn main() {
    let mut console = Console::builder()
        .width(60)
        .force_terminal(true)
        .no_color(false)
        .build();

    console.print(&Gradient::new(
        "gilt · Canvas blitters — same shape, four sub-cell densities",
        vec![
            Color::from_rgb(139, 233, 253),
            Color::from_rgb(189, 147, 249),
            Color::from_rgb(80, 250, 123),
            Color::from_rgb(255, 184, 108),
        ],
    ));
    console.line(1);

    // Same terminal footprint for every panel — only the glyph density differs.
    let (cols, rows) = (24usize, 12usize);

    let panels = [
        framed(
            "Braille  ",
            "2×4",
            "#8be9fd",
            shape(cols, rows, Blitter::Braille, "#8be9fd"),
        ),
        framed(
            "Octant   ",
            "2×4",
            "#bd93f9",
            shape(cols, rows, Blitter::Octant, "#bd93f9"),
        ),
        framed(
            "Sextant  ",
            "2×3",
            "#50fa7b",
            shape(cols, rows, Blitter::Sextant, "#50fa7b"),
        ),
        framed(
            "HalfBlock",
            "1×2",
            "#ffb86c",
            shape(cols, rows, Blitter::HalfBlock, "#ffb86c"),
        ),
    ];
    for panel in &panels {
        console.print(panel);
    }

    // -- Density check: Braille == Octant > Sextant > HalfBlock. --------------
    let px = |b: Blitter| {
        let c = Canvas::new(cols, rows).with_blitter(b);
        c.pixel_width() * c.pixel_height()
    };
    let (braille, octant, sextant, half) = (
        px(Blitter::Braille),
        px(Blitter::Octant),
        px(Blitter::Sextant),
        px(Blitter::HalfBlock),
    );
    assert_eq!(braille, octant, "Braille and Octant share 2×4 density");
    assert!(
        braille > sextant && sextant > half,
        "density ordering holds"
    );

    console.line(1);
    console.print_text(&format!(
        "  [#8be9fd]Braille[/]/[#bd93f9]Octant[/] {braille}px  ›  \
         [#50fa7b]Sextant[/] {sextant}px  ›  [#ffb86c]HalfBlock[/] {half}px   \
         [dim](same {cols}×{rows} cells)[/]"
    ));
}
