//! Demonstrates all highlighter types including URL, UUID, ISO date, and JSONPath.

use gilt::prelude::*;
use gilt::highlighter::*;

fn main() {
    let mut console = Console::new();

    console.print_text("[bold underline]Highlighter Types Demo[/bold underline]\n");

    // ReprHighlighter (existing)
    let repr_hl = ReprHighlighter::new();
    let text = repr_hl.apply(r#"User { name: "Alice", age: 30, active: true, score: 99.5 }"#);
    console.print_text("[dim]ReprHighlighter:[/dim]");
    console.print(&text);
    console.line(1);

    // URL Highlighter
    let url_hl = URLHighlighter::new();
    let text = url_hl.apply("Visit https://example.com or http://localhost:8080/api for details");
    console.print_text("[dim]URLHighlighter:[/dim]");
    console.print(&text);
    console.line(1);

    // UUID Highlighter
    let uuid_hl = UUIDHighlighter::new();
    let text = uuid_hl.apply("Request ID: 550e8400-e29b-41d4-a716-446655440000");
    console.print_text("[dim]UUIDHighlighter:[/dim]");
    console.print(&text);
    console.line(1);

    // ISO Date Highlighter
    let iso_hl = ISODateHighlighter::new();
    let text = iso_hl.apply("Created: 2024-01-15T10:30:00Z  Updated: 2024-06-20");
    console.print_text("[dim]ISODateHighlighter:[/dim]");
    console.print(&text);
    console.line(1);

    // JSONPath Highlighter
    let jp_hl = JSONPathHighlighter::new();
    let text = jp_hl.apply("Access $.config.database.host or .users[0].name");
    console.print_text("[dim]JSONPathHighlighter:[/dim]");
    console.print(&text);
    console.line(1);

    // JSON Highlighter (existing)
    let json_hl = JSONHighlighter::new();
    let text = json_hl.apply(r#"{"name": "gilt", "version": "0.1.0", "stable": true}"#);
    console.print_text("[dim]JSONHighlighter:[/dim]");
    console.print(&text);
}
