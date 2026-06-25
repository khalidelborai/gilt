//! `Histogram` — bins raw samples into a vertical block-column distribution.
//!
//! Each bin is a column drawn from the partial-block ladder (`▁▂▃▄▅▆▇█`), scaled
//! to the tallest bin so the busiest bucket reaches full height; sub-cell
//! precision comes from the eighth-block steps within each row. This demo frames
//! each chart in a `Panel` (gilt 2.0 lets a `Panel` hold any `Renderable`) for a
//! dashboard feel, mirroring the `barchart` example.
//!
//! Samples are generated deterministically (a tiny LCG summed via the central
//! limit theorem), so the output is identical on every run.
//!
//! Run with: `cargo run --example histogram`

use gilt::color::Color;
use gilt::console::Console;
use gilt::gradient::Gradient;
use gilt::histogram::Histogram;
use gilt::panel::Panel;
use gilt::style::Style;
use gilt::text::Text;

/// A subtle Dracula-grey panel border.
fn border() -> Style {
    Style::parse("#44475a")
}

fn titled(title: &str, color: &str, hist: Histogram) -> Panel {
    Panel::new(hist)
        .with_title(Text::new(
            &format!(" {title} "),
            Style::parse(&format!("bold {color}")),
        ))
        .with_border_style(border())
}

/// Deterministic [0, 1) pseudo-randoms from a 64-bit LCG (no external deps).
struct Lcg(u64);

impl Lcg {
    fn next_unit(&mut self) -> f64 {
        // Numerical Recipes LCG constants.
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        // Use the top 31 bits for a value in [0, 1).
        ((self.0 >> 33) as f64) / ((1u64 << 31) as f64)
    }
}

/// `n` approximately-normal samples (mean, std) via the central limit theorem:
/// the sum of twelve uniforms has mean 6 and variance 1.
fn gaussian(n: usize, mean: f64, std: f64, seed: u64) -> Vec<f64> {
    let mut rng = Lcg(seed);
    (0..n)
        .map(|_| {
            let sum: f64 = (0..12).map(|_| rng.next_unit()).sum();
            mean + std * (sum - 6.0)
        })
        .collect()
}

/// A right-skewed distribution: square the normal magnitude to stretch the tail.
fn skewed(n: usize, seed: u64) -> Vec<f64> {
    gaussian(n, 0.0, 1.0, seed)
        .into_iter()
        .map(|z| z * z + 0.15 * z)
        .collect()
}

fn main() {
    let mut console = Console::builder()
        .width(72)
        .force_terminal(true)
        .no_color(false)
        .build();

    console.print(&Gradient::new(
        "gilt · Histogram — raw samples binned into a distribution",
        vec![
            Color::from_rgb(189, 147, 249),
            Color::from_rgb(255, 121, 198),
            Color::from_rgb(139, 233, 253),
        ],
    ));
    console.line(1);

    // 1. A crisp bell curve: 4000 samples, 36 bins, with an axis (cyan).
    let bell = Histogram::new(&gaussian(4000, 0.0, 1.0, 0x1234_5678))
        .with_bins(36)
        .with_height(8)
        .with_range(-3.5, 3.5)
        .with_show_axis(true)
        .with_style(Style::parse("#8be9fd"));
    console.print(&titled("normal(0, 1) · 4000 samples", "#8be9fd", bell));
    console.line(1);

    // 2. The same data, fewer/wider bins — a chunkier silhouette (pink).
    let coarse = Histogram::new(&gaussian(4000, 0.0, 1.0, 0x1234_5678))
        .with_bins(14)
        .with_height(6)
        .with_range(-3.5, 3.5)
        .with_show_axis(true)
        .with_style(Style::parse("#ff79c6"));
    console.print(&titled("same data · 14 coarse bins", "#ff79c6", coarse));
    console.line(1);

    // 3. A right-skewed distribution — note the long tail to the right (green).
    let tail = Histogram::new(&skewed(4000, 0xC0FF_EE42))
        .with_bins(36)
        .with_height(8)
        .with_range(0.0, 9.0)
        .with_show_axis(true)
        .with_style(Style::parse("#50fa7b"));
    console.print(&titled("right-skewed · long tail", "#50fa7b", tail));
    console.line(1);

    // 4. A small fixed dataset — exam scores, 10 bins over 0..100 (yellow).
    let scores = [
        52.0, 61.0, 63.0, 67.0, 68.0, 71.0, 72.0, 74.0, 75.0, 76.0, 77.0, 78.0, 79.0, 81.0, 82.0,
        83.0, 84.0, 85.0, 86.0, 88.0, 90.0, 92.0, 95.0, 99.0, 70.0, 73.0, 80.0, 87.0, 66.0, 91.0,
    ];
    let exam = Histogram::new(&scores)
        .with_bins(10)
        .with_height(6)
        .with_range(0.0, 100.0)
        .with_show_axis(true)
        .with_style(Style::parse("#f1fa8c"));
    console.print(&titled("exam scores · 10 bins of 0..100", "#f1fa8c", exam));

    console.line(1);
    console.print(&Text::new(
        "  every histogram is a Renderable — drop one in a Panel, Table cell, or Live display.",
        Style::parse("italic #6272a4"),
    ));
}
