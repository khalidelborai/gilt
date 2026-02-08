//! Demonstrates gilt's Bar widget — solid block-character progress bars.

use gilt::bar::Bar;
use gilt::color::Color;
use gilt::console::Console;
use gilt::rule::Rule;

fn main() {
    let mut console = Console::builder()
        .width(60)
        .force_terminal(true)
        .no_color(false)
        .build();

    console.print(&Rule::with_title("Progress Dashboard"));

    // Each bar uses a total size of 40.0 and fills from 0.0 to a fraction
    // of that size.  Bar::new(size, begin, end) where begin and end are
    // absolute positions within [0, size].
    let bar_width: usize = 40;

    let levels: &[(&str, f64)] = &[
        ("  0% ", 0.0),
        (" 25% ", 10.0),
        (" 50% ", 20.0),
        (" 75% ", 30.0),
        ("100% ", 40.0),
    ];

    for (label, end) in levels {
        print!("{}", label);
        let bar = Bar::new(40.0, 0.0, *end).with_width(bar_width);
        console.print(&bar);
    }

    // -- Colored bars -------------------------------------------------------

    console.print(&Rule::with_title("Colored Bars"));

    let colors: &[(&str, &str, f64)] = &[
        ("Red   ", "red", 15.0),
        ("Green ", "green", 25.0),
        ("Blue  ", "blue", 35.0),
        ("Yellow", "yellow", 40.0),
    ];

    for (label, color_name, end) in colors {
        print!("{} ", label);
        let bar = Bar::new(40.0, 0.0, *end)
            .with_width(bar_width)
            .with_color(Color::parse(color_name).unwrap());
        console.print(&bar);
    }

    // -- Offset bars (begin > 0) -------------------------------------------

    console.print(&Rule::with_title("Offset Bars (partial fill)"));

    let offsets: &[(&str, f64, f64)] = &[("A ", 0.0, 20.0), ("B ", 10.0, 30.0), ("C ", 20.0, 40.0)];

    for (label, begin, end) in offsets {
        print!("{}", label);
        let bar = Bar::new(40.0, *begin, *end).with_width(bar_width);
        console.print(&bar);
    }
}
