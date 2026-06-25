//! `LineChart` — Braille line plots with axes, multiple series, and a legend.
//!
//! The flagship complex chart: each y-series is drawn as a connected Braille
//! line (2×4 sub-cell resolution) over a shared plot area, with a left y-axis
//! gutter (min/mid/max labels), a bottom axis line, and a colour-coded legend.
//! Series overlay in the same plot; each keeps its own colour. This demo frames
//! every chart in a `Panel` (gilt 2.0 lets a `Panel` hold any `Renderable`).
//!
//! Run with: `cargo run --example linechart`

use std::f64::consts::PI;

use gilt::color::Color;
use gilt::console::Console;
use gilt::gradient::Gradient;
use gilt::linechart::LineChart;
use gilt::panel::Panel;
use gilt::style::Style;
use gilt::text::Text;

/// A subtle Dracula-grey panel border.
fn border() -> Style {
    Style::parse("#44475a")
}

fn titled(title: &str, color: &str, chart: LineChart) -> Panel {
    Panel::new(chart)
        .with_title(Text::new(
            &format!(" {title} "),
            Style::parse(&format!("bold {color}")),
        ))
        .with_border_style(border())
}

fn main() {
    let mut console = Console::builder()
        .width(78)
        .force_terminal(true)
        .no_color(false)
        .build();

    console.print(&Gradient::new(
        "gilt · LineChart — Braille lines, multiple series, axes & legend",
        vec![
            Color::from_rgb(189, 147, 249),
            Color::from_rgb(255, 121, 198),
            Color::from_rgb(139, 233, 253),
        ],
    ));
    console.line(1);

    // 1. Trig trio — sine, cosine, and a half-amplitude sine, overlaid.
    let n = 120;
    let sine: Vec<f64> = (0..n)
        .map(|i| (i as f64 * 2.0 * PI / n as f64).sin())
        .collect();
    let cosine: Vec<f64> = (0..n)
        .map(|i| (i as f64 * 2.0 * PI / n as f64).cos())
        .collect();
    let half: Vec<f64> = (0..n)
        .map(|i| 0.5 * (i as f64 * 4.0 * PI / n as f64).sin())
        .collect();

    let trig = LineChart::new()
        .with_width(60)
        .with_height(14)
        .with_y_range(-1.1, 1.1)
        .series("sin", &sine, Style::parse("#8be9fd"))
        .series("cos", &cosine, Style::parse("#ff79c6"))
        .series("½·sin 2x", &half, Style::parse("#f1fa8c"));
    console.print(&titled(
        "trigonometric waves · y ∈ [-1.1, 1.1]",
        "#8be9fd",
        trig,
    ));
    console.line(1);

    // 2. Two "stock" curves — auto-ranged, with a legend.
    let bull: Vec<f64> = (0..48)
        .map(|i| 100.0 + 18.0 * (i as f64 / 9.0).sin() + i as f64 * 0.9)
        .collect();
    let bear: Vec<f64> = (0..48)
        .map(|i| 150.0 - i as f64 * 0.7 + 10.0 * (i as f64 / 5.0).cos())
        .collect();

    let market = LineChart::new()
        .with_width(60)
        .with_height(12)
        .series("ACME ▲", &bull, Style::parse("bold #50fa7b"))
        .series("WILE ▼", &bear, Style::parse("bold #ff5555"));
    console.print(&titled("share price · auto-ranged", "#50fa7b", market));
    console.line(1);

    // 3. A noisy ramp — a deterministic "random" walk climbing over time.
    let mut val = 0.0_f64;
    let noisy: Vec<f64> = (0..90)
        .map(|i| {
            val += 0.6 + (((i * 13 + 7) % 17) as f64 - 8.0) * 0.35;
            val
        })
        .collect();

    let ramp = LineChart::new().with_width(60).with_height(10).series(
        "throughput",
        &noisy,
        Style::parse("#bd93f9"),
    );
    console.print(&titled("noisy ramp · req/s over time", "#bd93f9", ramp));
    console.line(1);

    console.print(&Text::new(
        "  every series is overlaid on one Braille canvas — drop the chart in a Panel, Table cell, or Live display.",
        Style::parse("italic #6272a4"),
    ));
}
