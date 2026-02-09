//! Demonstrates the Sparkline widget -- inline Unicode sparkline charts.
//!
//! Shows various sparklines: stock prices, CPU usage, sine wave, with
//! different styles and widths.
//!
//! Run with: `cargo run --example sparkline`

use gilt::console::Console;
use gilt::rule::Rule;
use gilt::sparkline::Sparkline;
use gilt::style::Style;

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    // -- Ascending sequence --------------------------------------------------
    console.print(&Rule::with_title("Ascending (1-8)"));

    let ascending = Sparkline::new(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
    console.print(&ascending);

    // -- Simulated stock prices ----------------------------------------------
    console.print(&Rule::with_title("Stock Prices"));

    let prices: Vec<f64> = vec![
        100.0, 102.5, 101.0, 105.0, 107.0, 106.5, 110.0, 108.0, 112.0, 115.0, 113.0, 118.0, 120.0,
        117.0, 122.0, 125.0, 123.0, 128.0, 130.0, 127.0,
    ];
    let stock_spark = Sparkline::new(&prices).with_style(Style::parse("green").unwrap());
    console.print(&stock_spark);

    // -- CPU usage -----------------------------------------------------------
    console.print(&Rule::with_title("CPU Usage (red, width=40)"));

    let cpu: Vec<f64> = vec![
        12.0, 15.0, 45.0, 78.0, 92.0, 88.0, 55.0, 30.0, 22.0, 18.0, 25.0, 60.0, 85.0, 95.0, 70.0,
        40.0,
    ];
    let cpu_spark = Sparkline::new(&cpu)
        .with_width(40)
        .with_min(0.0)
        .with_max(100.0)
        .with_style(Style::parse("red").unwrap());
    console.print(&cpu_spark);

    // -- Sine wave -----------------------------------------------------------
    console.print(&Rule::with_title("Sine Wave (cyan, 60 columns)"));

    let sine: Vec<f64> = (0..120)
        .map(|i| (i as f64 * std::f64::consts::PI / 15.0).sin())
        .collect();
    let sine_spark = Sparkline::new(&sine)
        .with_width(60)
        .with_style(Style::parse("cyan").unwrap());
    console.print(&sine_spark);

    // -- Random walk ---------------------------------------------------------
    console.print(&Rule::with_title("Random Walk (magenta)"));

    let mut walk = vec![0.0f64; 50];
    let mut val = 50.0;
    for (i, item) in walk.iter_mut().enumerate().take(50) {
        // Deterministic "random" walk for reproducibility
        val += ((i * 7 + 3) % 11) as f64 - 5.0;
        *item = val;
    }
    let walk_spark = Sparkline::new(&walk).with_style(Style::parse("magenta").unwrap());
    console.print(&walk_spark);

    // -- Gradient style sparkline --------------------------------------------
    console.print(&Rule::with_title("Bold Yellow on Blue"));

    let triangle: Vec<f64> = (0..20)
        .map(|i| if i < 10 { i as f64 } else { (20 - i) as f64 })
        .collect();
    let styled_spark =
        Sparkline::new(&triangle).with_style(Style::parse("bold yellow on blue").unwrap());
    console.print(&styled_spark);

    // -- Display trait -------------------------------------------------------
    console.print(&Rule::with_title("Display Trait (via println!)"));
    let display_spark = Sparkline::new(&[1.0, 4.0, 7.0, 2.0, 5.0, 8.0, 3.0, 6.0]);
    println!("  println! output: {display_spark}");
}
