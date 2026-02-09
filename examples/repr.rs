//! Port of rich's repr.py — pretty-printed Debug representations.
//!
//! Defines a Bird struct, creates a HashMap of birds, and pretty-prints
//! the collection using gilt's Pretty widget.

use std::collections::HashMap;

use gilt::console::Console;
use gilt::pretty::Pretty;

#[derive(Debug)]
#[allow(dead_code)]
struct Bird {
    name: String,
    eats: Vec<String>,
    fly: bool,
    extinct: bool,
}

impl Bird {
    fn new(name: &str, eats: &[&str], fly: bool, extinct: bool) -> Self {
        Bird {
            name: name.to_string(),
            eats: eats.iter().map(|s| s.to_string()).collect(),
            fly,
            extinct,
        }
    }
}

fn main() {
    let mut birds = HashMap::new();
    birds.insert(
        "gull".to_string(),
        Bird::new(
            "gull",
            &["fish", "chips", "ice cream", "sausage rolls"],
            true,
            false,
        ),
    );
    birds.insert(
        "penguin".to_string(),
        Bird::new("penguin", &["fish"], false, false),
    );
    birds.insert(
        "dodo".to_string(),
        Bird::new("dodo", &["fruit"], false, true),
    );

    let mut console = Console::new();

    // Pretty-print the entire HashMap with syntax highlighting
    let pretty = Pretty::from_debug(&birds);
    console.print(&pretty);
}
