//! Demonstrates gilt's inspect() for debugging values.

use gilt::prelude::*;
use std::collections::HashMap;

fn main() {
    let mut console = Console::new();

    // Inspect a vector
    let numbers = vec![1, 2, 3, 42, 100];
    let inspect = Inspect::new(&numbers).with_label("numbers");
    console.print(&inspect);

    // Inspect a HashMap
    let mut config: HashMap<String, String> = HashMap::new();
    config.insert("host".into(), "localhost".into());
    config.insert("port".into(), "8080".into());
    config.insert("debug".into(), "true".into());
    let inspect = Inspect::new(&config)
        .with_label("config")
        .with_doc("Application configuration");
    console.print(&inspect);

    // Inspect a custom struct
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Point { x: f64, y: f64, z: f64 }

    let origin = Point { x: 0.0, y: 0.0, z: 0.0 };
    let inspect = Inspect::new(&origin)
        .with_label("origin")
        .with_title("3D Point");
    console.print(&inspect);

    // Inspect an Option
    let maybe: Option<&str> = None;
    let inspect = Inspect::new(&maybe).with_label("maybe");
    console.print(&inspect);

    // Use the global convenience function
    gilt::inspect(&vec!["hello", "world"]);
}
