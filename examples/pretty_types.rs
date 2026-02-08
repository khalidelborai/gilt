//! Demonstrates Pretty printer with type annotations.

use gilt::prelude::*;
use gilt::pretty::Pretty;

fn main() {
    let mut console = Console::new();

    console.print_text("[bold underline]Pretty Type Annotations[/bold underline]\n");

    // JSON object with type annotation
    let json: serde_json::Value = serde_json::json!({
        "name": "gilt",
        "version": "0.1.0",
        "features": ["tracing", "derive"]
    });
    let pretty = Pretty::from_json(&json).with_type_annotation(true);
    console.print_text("[dim]JSON object:[/dim]");
    console.print(&pretty);
    console.line(1);

    // JSON array with type annotation
    let arr: serde_json::Value = serde_json::json!([1, 2, 3, 4, 5]);
    let pretty = Pretty::from_json(&arr).with_type_annotation(true);
    console.print_text("[dim]JSON array:[/dim]");
    console.print(&pretty);
    console.line(1);

    // Debug struct
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Config { host: String, port: u16, debug: bool }
    let cfg = Config { host: "localhost".into(), port: 8080, debug: true };
    let pretty = Pretty::from_debug(&cfg).with_type_annotation(true);
    console.print_text("[dim]Debug struct:[/dim]");
    console.print(&pretty);
    console.line(1);

    // Without type annotation (default)
    let pretty = Pretty::from_debug(&cfg);
    console.print_text("[dim]Without annotation:[/dim]");
    console.print(&pretty);
}
