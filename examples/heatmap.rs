//! Demonstrates the Heatmap widget — a 2-D grid of values rendered as coloured
//! cell blocks.
//!
//! Shows the default gradient (dark-blue → cyan → yellow → red), a custom
//! gradient, fixed value bounds, and a wider cell size.
//!
//! Run with: `cargo run --example heatmap`

use gilt::color::Color;
use gilt::console::Console;
use gilt::heatmap::Heatmap;
use gilt::rule::Rule;

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    // ── Default gradient ─────────────────────────────────────────────────
    console.print(&Rule::with_title(
        "Default Heatmap (dark-blue → cyan → yellow → red)",
    ));

    // A smooth 8x16 ramp so the whole gradient is visible.
    let ramp: Vec<Vec<f64>> = (0..8)
        .map(|r| (0..16).map(|c| (r * 16 + c) as f64).collect())
        .collect();
    console.print(&Heatmap::new(ramp));

    // ── A radial-ish field ───────────────────────────────────────────────
    console.print(&Rule::with_title("Radial Field"));

    let (rows, cols) = (10usize, 24usize);
    let field: Vec<Vec<f64>> = (0..rows)
        .map(|r| {
            (0..cols)
                .map(|c| {
                    let dy = r as f64 - rows as f64 / 2.0;
                    let dx = (c as f64 - cols as f64 / 2.0) / 2.0;
                    -(dx * dx + dy * dy)
                })
                .collect()
        })
        .collect();
    console.print(&Heatmap::new(field));

    // ── Custom gradient (greyscale), fixed bounds ────────────────────────
    console.print(&Rule::with_title("Greyscale, bounds 0..1"));

    let grid = vec![
        vec![0.0, 0.25, 0.5, 0.75, 1.0],
        vec![1.0, 0.75, 0.5, 0.25, 0.0],
    ];
    console.print(
        &Heatmap::new(grid)
            .with_min(0.0)
            .with_max(1.0)
            .with_gradient(vec![
                Color::from_rgb(0, 0, 0),
                Color::from_rgb(255, 255, 255),
            ])
            .with_cell_width(4),
    );

    // ── Ragged rows (padded to the widest with the minimum) ──────────────
    console.print(&Rule::with_title("Ragged rows (padded)"));

    let ragged = vec![vec![3.0, 6.0, 9.0, 12.0], vec![1.0, 2.0], vec![5.0]];
    console.print(&Heatmap::new(ragged).with_cell_width(3));
}
