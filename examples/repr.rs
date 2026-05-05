//! Port of rich/examples/repr.py — pretty-printed Debug representations.

use gilt::console::Console;
use gilt::pretty::Pretty;
use std::collections::HashMap;

#[derive(Debug)]
#[allow(dead_code)]
struct Bird {
    name: &'static str,
    eats: &'static [&'static str],
    fly: bool,
    extinct: bool,
}

fn main() {
    let birds = HashMap::from([
        (
            "gull",
            Bird {
                name: "gull",
                eats: &["fish", "chips", "ice cream"],
                fly: true,
                extinct: false,
            },
        ),
        (
            "penguin",
            Bird {
                name: "penguin",
                eats: &["fish"],
                fly: false,
                extinct: false,
            },
        ),
        (
            "dodo",
            Bird {
                name: "dodo",
                eats: &["fruit"],
                fly: false,
                extinct: true,
            },
        ),
    ]);
    Console::default().print(&Pretty::from_debug(&birds));
}
