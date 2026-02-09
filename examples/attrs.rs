//! Port of rich's attrs.py — pretty-print complex nested Rust structs.
//!
//! Shows a side-by-side table comparing Pretty-printed output (left)
//! with raw Debug format (right).

use gilt::console::Console;
use gilt::pretty::Pretty;
use gilt::table::Table;
use gilt::text::Text;

#[derive(Debug)]
#[allow(dead_code)]
struct Point3D {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Triangle {
    vertices: [Point3D; 3],
}

#[derive(Debug)]
#[allow(dead_code)]
struct Model {
    name: String,
    triangles: Vec<Triangle>,
}

fn main() {
    let model = Model {
        name: "Alien#1".to_string(),
        triangles: vec![
            Triangle {
                vertices: [
                    Point3D {
                        x: 20.0,
                        y: 50.0,
                        z: 0.0,
                    },
                    Point3D {
                        x: 50.0,
                        y: 15.0,
                        z: -45.34,
                    },
                    Point3D {
                        x: 3.1426,
                        y: 83.2323,
                        z: -16.0,
                    },
                ],
            },
            Triangle {
                vertices: [
                    Point3D {
                        x: -10.5,
                        y: 0.0,
                        z: 33.3,
                    },
                    Point3D {
                        x: 100.0,
                        y: -25.0,
                        z: 7.77,
                    },
                    Point3D {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                ],
            },
        ],
    };

    let mut console = Console::new();

    // Build a table with two columns: Pretty vs raw Debug
    let mut table = Table::new(&["structs *with* Gilt", "structs *without* Gilt"]);

    // Left column: Pretty-printed with syntax highlighting and indent guides
    let pretty = Pretty::from_debug(&model);
    let pretty_text = pretty.text.clone();

    // Right column: raw Debug format
    let raw = format!("{:#?}", model);
    let raw_text = Text::new(&raw, gilt::style::Style::null());

    table.add_row_text(&[pretty_text, raw_text]);

    console.print(&table);
}
