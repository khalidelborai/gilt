//! `Canvas` Lissajous curves — `x = sin(a·t + δ)`, `y = sin(b·t)`.
//!
//! A Lissajous figure is the path traced by two perpendicular sine waves. The
//! integer ratio `a : b` sets the number of lobes and the phase `δ` rotates the
//! knot. We sample `t` over one full period and join successive points with
//! `Canvas::line()` (Bresenham), so the curve stays continuous even where it
//! sweeps fast — a single styled `Canvas` is all it takes.
//!
//! Shown: one hero 3 : 2 knot on a 100×80 Braille grid, then a side-by-side
//! gallery of ratio/phase variations composited into one multicolour `Text`.
//!
//! Run with: `cargo run --example canvas_lissajous`

use gilt::canvas::Canvas;
use gilt::color::Color;
use gilt::console::Console;
use gilt::gradient::Gradient;
use gilt::panel::Panel;
use gilt::style::Style;
use gilt::text::Text;

/// Trace a Lissajous curve onto a fresh Braille canvas.
///
/// `a`/`b` are the angular-frequency multipliers, `delta` the phase offset.
/// Successive samples are connected with `line()` so the knot is unbroken.
fn lissajous(w: usize, h: usize, a: f64, b: f64, delta: f64) -> Canvas {
    const SAMPLES: usize = 2400;
    let mut c = Canvas::new(w, h);
    let pw = c.pixel_width() as i32;
    let ph = c.pixel_height() as i32;
    let (sx, sy) = ((pw - 1) as f64 / 2.0, (ph - 1) as f64 / 2.0);

    let mut prev: Option<(i32, i32)> = None;
    for i in 0..=SAMPLES {
        let t = i as f64 / SAMPLES as f64 * std::f64::consts::TAU;
        // sin ∈ [-1,1] → pixel ∈ [0, dim-1].
        let x = ((a * t + delta).sin() + 1.0) * sx;
        let y = ((b * t).sin() + 1.0) * sy;
        let (xi, yi) = (x.round() as i32, y.round() as i32);
        match prev {
            Some((px, py)) => c.line(px, py, xi, yi),
            None => c.set(xi.max(0) as usize, yi.max(0) as usize),
        }
        prev = Some((xi, yi));
    }
    c
}

/// Centre `label` within `width` cells (left-biased on odd remainders).
fn centre(label: &str, width: usize) -> String {
    let len = label.chars().count();
    if len >= width {
        return label.chars().take(width).collect();
    }
    let pad = width - len;
    let left = pad / 2;
    format!("{}{}{}", " ".repeat(left), label, " ".repeat(pad - left))
}

/// Composite same-height canvases into one multicolour `Text`, laid out
/// left-to-right with a labelled header row above each knot.
fn gallery(items: &[(&str, &str, Canvas)]) -> Text {
    const GAP: usize = 3;
    let rows = items[0].2.frame().lines().count();
    let gap = " ".repeat(GAP);

    // Row-major grid: `grid[r]` holds each knot's styled line for row `r`.
    let mut grid: Vec<Vec<(String, Style)>> = vec![Vec::new(); rows];
    for (_, color, canvas) in items {
        let style = Style::parse(color);
        for (line, slot) in canvas.frame().lines().zip(grid.iter_mut()) {
            slot.push((line.to_string(), style.clone()));
        }
    }

    let mut text = Text::new("", Style::null());
    let mut tokens: Vec<(String, Option<Style>)> = Vec::new();

    // Header row: a centred, bold, colour-matched label over each knot.
    for (i, (label, color, canvas)) in items.iter().enumerate() {
        let width = canvas
            .frame()
            .lines()
            .next()
            .map_or(0, |l| l.chars().count());
        tokens.push((
            centre(label, width),
            Some(Style::parse(&format!("bold {color}"))),
        ));
        if i + 1 < items.len() {
            tokens.push((gap.clone(), None));
        }
    }
    tokens.push(("\n".to_string(), None));

    // Knot rows: each column keeps its own colour.
    for (r, row) in grid.iter().enumerate() {
        for (i, (line, style)) in row.iter().enumerate() {
            tokens.push((line.clone(), Some(style.clone())));
            if i + 1 < row.len() {
                tokens.push((gap.clone(), None));
            }
        }
        if r + 1 < grid.len() {
            tokens.push(("\n".to_string(), None));
        }
    }
    text.append_tokens(&tokens);
    text
}

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();
    let pi = std::f64::consts::PI;

    console.print(&Gradient::new(
        "gilt · Canvas — Lissajous knots traced with Bresenham lines",
        vec![
            Color::from_rgb(139, 233, 253),
            Color::from_rgb(189, 147, 249),
            Color::from_rgb(255, 121, 198),
        ],
    ));
    console.line(1);

    // -- Hero knot: classic 3 : 2 with a quarter-turn phase. -----------------
    let hero = lissajous(50, 20, 3.0, 2.0, pi / 2.0);
    let lit = hero.frame().chars().filter(|&ch| ch != '\u{2800}').count();
    let panel = Panel::new(hero.with_style(Style::parse("#8be9fd")))
        .with_title(Text::new(
            " a:b = 3:2 · δ = π/2 · 100×80 Braille px ",
            Style::parse("bold #8be9fd"),
        ))
        .with_subtitle(Text::new(
            " x = sin(3t + π/2),  y = sin(2t) ",
            Style::parse("italic #6272a4"),
        ))
        .with_border_style(Style::parse("#bd93f9"));
    console.print(&panel);

    // -- Gallery: ratio + phase variations, side by side in one Panel. -------
    console.line(1);
    let knots = [
        ("3:4 · δ=0", "#ff79c6", lissajous(17, 9, 3.0, 4.0, 0.0)),
        (
            "5:4 · δ=π/2",
            "#50fa7b",
            lissajous(17, 9, 5.0, 4.0, pi / 2.0),
        ),
        (
            "5:6 · δ=π/4",
            "#f1fa8c",
            lissajous(17, 9, 5.0, 6.0, pi / 4.0),
        ),
    ];
    let gallery_panel = Panel::new(gallery(&knots))
        .with_expand(false)
        .with_title(Text::new(
            " Gallery · same recipe, new a:b and δ ",
            Style::parse("bold #f8f8f2"),
        ))
        .with_border_style(Style::parse("#44475a"));
    console.print(&gallery_panel);

    // -- Sanity: the hero knot must trace a non-trivial closed curve. --------
    assert!(lit > 50, "Lissajous knot should light many Braille cells");
    console.line(1);
    console.print_text(&format!(
        "  [dim]hero knot lit {lit} Braille cells — one closed path of \
         2400 Bresenham segments.[/]"
    ));
}
