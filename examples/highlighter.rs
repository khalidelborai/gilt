//! Demonstrates gilt's highlighter module — regex-based text highlighting.
//!
//! This example shows two approaches:
//! 1. Using `Text::highlight_regex` for simple one-off pattern highlighting
//! 2. Using `ReprHighlighter` for rich repr-style output highlighting

use gilt::console::Console;
use gilt::highlighter::{Highlighter, ReprHighlighter};
use gilt::rule::Rule;
use gilt::style::Style;
use gilt::text::Text;
use regex::Regex;

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    // -- Email Highlighter (manual regex approach) ---------------------------

    console.print(&Rule::with_title("Email Highlighter"));

    let email_re = Regex::new(r"[\w.+-]+@[\w-]+\.[\w.-]+").unwrap();
    let email_style = Style::parse("bold bright_cyan underline").unwrap();

    let samples = [
        "Contact us at support@example.com for help.",
        "Send your resume to hiring@acme-corp.org today!",
        "Reach alice.jones+work@mail.co.uk or bob@dev.io for details.",
        "No email addresses in this line.",
        "Multiple: admin@server.net, info@company.com, test@localhost.dev",
    ];

    for sample in &samples {
        let mut text = Text::new(sample, Style::null());
        let count = text.highlight_regex(&email_re, email_style.clone());
        let label = format!("  ({} match{})", count, if count == 1 { "" } else { "es" });
        text.append_str(&label, Some(Style::parse("dim").unwrap()));
        console.print(&text);
    }

    // -- ReprHighlighter (built-in patterns) ---------------------------------

    console.print(&Rule::with_title("ReprHighlighter"));

    let repr_hl = ReprHighlighter::new();

    let repr_samples = [
        r#"User(name="Alice", age=30, active=True)"#,
        r#"Config(path="/etc/app.conf", debug=False, retries=3)"#,
        "Server listening on 192.168.1.42:8080",
        r#"uuid = "a3f2504e0-4f89-11d3-9a0c-0305e82c3301""#,
        "response_time = 0.042s, status = 200, cached = True",
        r#"Error: FileNotFoundError("config.yaml")"#,
    ];

    for sample in &repr_samples {
        let mut text = Text::new(sample, Style::null());
        repr_hl.highlight(&mut text);
        console.print(&text);
    }

    // -- Combined: Email + Repr highlighting ---------------------------------

    console.print(&Rule::with_title("Combined Highlighting"));

    let combined_text = r#"Contact admin@server.net (IP: 10.0.0.1, port=443, ssl=True)"#;
    let mut text = Text::new(combined_text, Style::null());
    repr_hl.highlight(&mut text);
    text.highlight_regex(&email_re, email_style);
    console.print(&text);
}
