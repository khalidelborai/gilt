//! `Heatmap` — a 2-D grid of values painted as coloured cells.
//!
//! Each value is normalized into `[0,1]` and mapped through a configurable
//! colour gradient to a background-coloured cell. This demo shows three very
//! different uses — a contributions calendar, a smooth interference field, and a
//! diverging correlation matrix — each framed in a `Panel`.
//!
//! Run with: `cargo run --example heatmap`

use gilt::color::Color;
use gilt::console::Console;
use gilt::gradient::Gradient;
use gilt::heatmap::Heatmap;
use gilt::panel::Panel;
use gilt::style::Style;
use gilt::text::Text;

fn border() -> Style {
    Style::parse("#44475a")
}

fn titled(title: &str, color: &str, hm: Heatmap) -> Panel {
    Panel::new(hm)
        .with_title(Text::new(
            &format!(" {title} "),
            Style::parse(&format!("bold {color}")),
        ))
        .with_border_style(border())
}

fn main() {
    let mut console = Console::builder()
        .width(96)
        .force_terminal(true)
        .no_color(false)
        .build();

    console.print(&Gradient::new(
        "gilt · Heatmap — values painted as colour",
        vec![
            Color::from_rgb(139, 233, 253),
            Color::from_rgb(80, 250, 123),
            Color::from_rgb(241, 250, 140),
            Color::from_rgb(255, 85, 85),
        ],
    ));
    console.line(1);

    // 1. GitHub-style contributions calendar: 7 days × 30 weeks, green ramp.
    let (days, weeks) = (7usize, 30usize);
    let calendar: Vec<Vec<f64>> = (0..days)
        .map(|d| {
            (0..weeks)
                .map(|w| {
                    let wave = (w as f64 * 0.45 + d as f64 * 0.8).sin() * 0.5 + 0.5;
                    let weekday = if d == 0 || d == 6 { 0.35 } else { 1.0 };
                    (wave * weekday).clamp(0.0, 1.0)
                })
                .collect()
        })
        .collect();
    console.print(&titled(
        "contributions · last 30 weeks",
        "#50fa7b",
        Heatmap::new(calendar)
            .with_min(0.0)
            .with_max(1.0)
            .with_cell_width(2)
            .with_gradient(vec![
                Color::from_rgb(40, 42, 54),
                Color::from_rgb(14, 68, 41),
                Color::from_rgb(33, 110, 57),
                Color::from_rgb(64, 196, 99),
                Color::from_rgb(57, 211, 83),
            ]),
    ));
    console.line(1);

    // 2. A smooth interference field — cool→hot, 1-cell resolution.
    let (rows, cols) = (12usize, 44usize);
    let field: Vec<Vec<f64>> = (0..rows)
        .map(|r| {
            (0..cols)
                .map(|c| {
                    let x = c as f64 / cols as f64 * std::f64::consts::TAU;
                    let y = r as f64 / rows as f64 * std::f64::consts::TAU;
                    ((x * 1.5).sin() + (y * 1.2).cos() + (x + y).sin()) / 3.0 * 0.5 + 0.5
                })
                .collect()
        })
        .collect();
    console.print(&titled(
        "interference field · sin/cos",
        "#8be9fd",
        Heatmap::new(field).with_cell_width(1).with_gradient(vec![
            Color::from_rgb(40, 42, 54),
            Color::from_rgb(98, 114, 164),
            Color::from_rgb(139, 233, 253),
            Color::from_rgb(241, 250, 140),
            Color::from_rgb(255, 85, 85),
        ]),
    ));
    console.line(1);

    // 3. Correlation matrix — diverging blue↔red gradient over -1..1.
    let corr = vec![
        vec![1.00, 0.82, -0.30, 0.10, -0.65],
        vec![0.82, 1.00, -0.12, 0.25, -0.40],
        vec![-0.30, -0.12, 1.00, -0.55, 0.20],
        vec![0.10, 0.25, -0.55, 1.00, 0.05],
        vec![-0.65, -0.40, 0.20, 0.05, 1.00],
    ];
    console.print(&titled(
        "correlation matrix · diverging -1..1",
        "#ff79c6",
        Heatmap::new(corr)
            .with_min(-1.0)
            .with_max(1.0)
            .with_cell_width(5)
            .with_gradient(vec![
                Color::from_rgb(68, 138, 255),
                Color::from_rgb(40, 42, 54),
                Color::from_rgb(255, 85, 85),
            ]),
    ));

    console.line(1);
    console.print(&Text::new(
        "  same widget, three stories — calendars, fields, matrices, all from Vec<Vec<f64>>.",
        Style::parse("italic #6272a4"),
    ));
}
