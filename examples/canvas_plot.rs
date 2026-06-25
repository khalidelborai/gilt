//! `Canvas` function plotter — three math curves on one Braille grid.
//!
//! Braille gives 2×4 sub-cell "pixels", so a modest 64×18 cell `Panel`
//! becomes a 128×72 plotting surface. Each curve is drawn on its own
//! `Canvas` with `line()` between successive samples (no gaps on steep
//! slopes), then the layers are *composited* into a single multicolour
//! `Text`: per cell we OR the dot bits together and colour the cell after
//! the first curve that lit it. The result is one framed plot showing
//!
//!   • a sine wave           (cyan)
//!   • a damped sine         (pink)
//!   • a downward parabola   (yellow)
//!   • the x = 0 / y = 0 axes (grey)
//!
//! all overlaid in true colour — something a single styled `Canvas` can't
//! do on its own, since a `Canvas` carries one `Style` for the whole frame.
//!
//! Run with: `cargo run --example canvas_plot`

use gilt::canvas::Canvas;
use gilt::color::Color;
use gilt::console::Console;
use gilt::gradient::Gradient;
use gilt::panel::Panel;
use gilt::style::Style;
use gilt::text::Text;

/// Empty Braille cell — `frame()` emits this for any all-clear cell.
const BRAILLE_BASE: u32 = 0x2800;

/// Plot `f` (mapped to the value range `-1.0..=1.0`) across the full width of
/// a fresh Braille canvas, joining successive samples with `line()` so steep
/// sections stay connected. `f` receives the normalised x in `0.0..=1.0`.
fn plot(w: usize, h: usize, f: impl Fn(f64) -> f64) -> Canvas {
    let mut c = Canvas::new(w, h);
    let pw = c.pixel_width();
    let ph = c.pixel_height();
    let mut prev: Option<(i32, i32)> = None;
    for x in 0..pw {
        let xn = x as f64 / (pw - 1) as f64;
        let v = f(xn).clamp(-1.0, 1.0);
        // value +1 → top row (y=0), value -1 → bottom row.
        let y = (((1.0 - v) / 2.0) * (ph - 1) as f64).round() as i32;
        let xi = x as i32;
        match prev {
            Some((px, py)) => c.line(px, py, xi, y),
            None => c.set(x, y as usize),
        }
        prev = Some((xi, y));
    }
    c
}

/// A Braille canvas holding just the x-axis (value 0) and the left y-axis.
fn axes(w: usize, h: usize) -> Canvas {
    let mut c = Canvas::new(w, h);
    let pw = c.pixel_width() as i32;
    let ph = c.pixel_height() as i32;
    let zero = (ph - 1) / 2;
    c.line(0, zero, pw - 1, zero); // horizontal x-axis at value 0
    c.line(0, 0, 0, ph - 1); // vertical y-axis at the left edge
    c
}

/// Composite same-sized Braille layers into one multicolour `Text`.
///
/// For every character cell we OR the dot bits of all layers together (so no
/// curve erases another) and colour the cell with the style of the *first*
/// layer that lit any dot there — pass the layers in priority order.
fn overlay(layers: &[(&Canvas, Style)]) -> Text {
    let grids: Vec<Vec<Vec<char>>> = layers
        .iter()
        .map(|(c, _)| c.frame().lines().map(|l| l.chars().collect()).collect())
        .collect();
    let rows = grids[0].len();
    let cols = grids[0].first().map_or(0, Vec::len);

    let mut text = Text::new("", Style::null());
    let mut tokens: Vec<(String, Option<Style>)> = Vec::new();
    for r in 0..rows {
        for col in 0..cols {
            let mut bits: u32 = 0;
            let mut style: Option<Style> = None;
            for (i, grid) in grids.iter().enumerate() {
                let dots = (grid[r][col] as u32).wrapping_sub(BRAILLE_BASE) & 0xFF;
                if dots != 0 {
                    bits |= dots;
                    if style.is_none() {
                        style = Some(layers[i].1.clone());
                    }
                }
            }
            let ch = char::from_u32(BRAILLE_BASE + bits).unwrap_or('\u{2800}');
            tokens.push((ch.to_string(), style));
        }
        if r + 1 < rows {
            tokens.push(("\n".to_string(), None));
        }
    }
    text.append_tokens(&tokens);
    text
}

fn main() {
    let mut console = Console::builder()
        .width(76)
        .force_terminal(true)
        .no_color(false)
        .build();

    console.print(&Gradient::new(
        "gilt · Canvas — three curves, one Braille grid, true-colour overlay",
        vec![
            Color::from_rgb(139, 233, 253),
            Color::from_rgb(189, 147, 249),
            Color::from_rgb(255, 121, 198),
        ],
    ));
    console.line(1);

    // Plotting surface: 64×18 cells → 128×72 Braille pixels.
    let (w, h) = (64usize, 18usize);
    let tau = std::f64::consts::TAU;

    // sin: two full cycles across the width.
    let sine = plot(w, h, |xn| (tau * 2.0 * xn).sin());
    // damped sin: amplitude decays as e^(-3x); three cycles.
    let damped = plot(w, h, |xn| (-3.0 * xn).exp() * (tau * 3.0 * xn).sin());
    // downward parabola peaking at the centre: 1 - 2·u², u ∈ [-1,1].
    let parab = plot(w, h, |xn| {
        let u = (xn - 0.5) * 2.0;
        1.0 - 2.0 * u * u
    });
    let ax = axes(w, h);

    // Composite — curves take colour priority over the axes where they cross.
    let plot_text = overlay(&[
        (&sine, Style::parse("#8be9fd")),
        (&damped, Style::parse("#ff79c6")),
        (&parab, Style::parse("#f1fa8c")),
        (&ax, Style::parse("#6272a4")),
    ]);

    let panel = Panel::new(plot_text)
        .with_title(Text::new(
            " f(x) on a 128×72 Braille canvas ",
            Style::parse("bold #bd93f9"),
        ))
        .with_subtitle(Text::new(
            " sin · damped sin · parabola ",
            Style::parse("italic #6272a4"),
        ))
        .with_border_style(Style::parse("#44475a"));
    console.print(&panel);

    // Legend — colours match the curves above.
    console.line(1);
    console.print_text(
        "  [#8be9fd]●[/] sin(4πx)   [#ff79c6]●[/] e⁻³ˣ·sin(6πx)   \
         [#f1fa8c]●[/] 1−2u²   [#6272a4]●[/] axes",
    );

    // -- Sanity: the overlay must contain lit (non-blank) cells. -------------
    let blank = '\u{2800}';
    let lit = sine.frame().chars().filter(|&c| c != blank).count()
        + damped.frame().chars().filter(|&c| c != blank).count()
        + parab.frame().chars().filter(|&c| c != blank).count();
    assert!(lit > 0, "plot must light up Braille cells");
    console.line(1);
    console.print_text(&format!(
        "  [dim]{lit} Braille cells lit across the three curves — each cell is a 2×4 pixel block.[/]"
    ));
}
