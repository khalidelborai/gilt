//! Pretty-printing demo — shows gilt's Pretty widget for structured data.
//!
//! Demonstrates pretty-printing of Debug-formatted Rust values and JSON data
//! with syntax highlighting and indent guides.

use gilt::console::Console;
use gilt::pretty::Pretty;
use gilt::rule::Rule;
use serde_json::json;
use std::collections::BTreeMap;

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    // ── 1. Pretty-print a Rust struct via Debug ────────────────────────────
    console.print(&Rule::with_title("Debug: Vec of tuples"));

    let data: Vec<(&str, i32, bool)> = vec![
        ("Alice", 30, true),
        ("Bob", 25, false),
        ("Charlie", 35, true),
        ("Diana", 28, false),
    ];
    let pretty = Pretty::from_debug(&data);
    console.print(&pretty);

    // ── 2. Pretty-print a BTreeMap ─────────────────────────────────────────
    console.print(&Rule::with_title("Debug: BTreeMap"));

    let mut settings: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    settings.insert("colors", vec!["red", "green", "blue", "cyan", "magenta"]);
    settings.insert("fonts", vec!["Fira Code", "JetBrains Mono", "Cascadia Code"]);
    settings.insert("themes", vec!["monokai", "dracula", "solarized", "nord"]);
    let pretty = Pretty::from_debug(&settings);
    console.print(&pretty);

    // ── 3. Pretty-print JSON ───────────────────────────────────────────────
    console.print(&Rule::with_title("JSON: Configuration"));

    let config = json!({
        "server": {
            "host": "0.0.0.0",
            "port": 8080,
            "tls": {
                "enabled": true,
                "cert": "/etc/ssl/cert.pem",
                "key": "/etc/ssl/key.pem"
            }
        },
        "database": {
            "url": "postgres://localhost:5432/myapp",
            "pool_size": 10,
            "timeout_ms": 5000
        },
        "logging": {
            "level": "info",
            "format": "json",
            "outputs": ["stdout", "file:///var/log/app.log"]
        }
    });
    let pretty = Pretty::from_json(&config);
    console.print(&pretty);

    // ── 4. Pretty-print nested JSON array ──────────────────────────────────
    console.print(&Rule::with_title("JSON: Nested Array"));

    let users = json!([
        {
            "id": 1,
            "name": "Alice Wonderland",
            "email": "alice@example.com",
            "roles": ["admin", "user"],
            "active": true
        },
        {
            "id": 2,
            "name": "Bob Builder",
            "email": "bob@example.com",
            "roles": ["user"],
            "active": false
        },
        {
            "id": 3,
            "name": "Charlie Chaplin",
            "email": "charlie@example.com",
            "roles": ["moderator", "user"],
            "active": true
        }
    ]);
    let pretty = Pretty::from_json(&users);
    console.print(&pretty);

    // ── 5. Pretty-print from a plain string ────────────────────────────────
    console.print(&Rule::with_title("Plain String with Repr Highlighting"));

    let repr_text = r#"{
    "count": 42,
    "pi": 3.14159,
    "enabled": true,
    "label": "Hello, World!",
    "nothing": null
}"#;
    let pretty = Pretty::from_str(repr_text);
    console.print(&pretty);

    // ── 6. Customised indent ───────────────────────────────────────────────
    console.print(&Rule::with_title("Custom Indent Size (2 spaces)"));

    let nested = json!({
        "a": {
            "b": {
                "c": {
                    "d": "deeply nested value"
                }
            }
        }
    });
    let pretty = Pretty::from_json(&nested).with_indent_size(2);
    console.print(&pretty);

    // ── 7. No indent guides ────────────────────────────────────────────────
    console.print(&Rule::with_title("Without Indent Guides"));

    let pretty = Pretty::from_json(&nested).with_indent_guides(false);
    console.print(&pretty);
}
