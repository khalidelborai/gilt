//! Testable subcommand implementations.
//!
//! Each function takes an explicit `Write` sink instead of stdout so it can be
//! unit-tested without spawning a process.

use std::io::{self, Read, Write};

use gilt::console::Console;
use gilt::csv_table::CsvTable;
use gilt::json::{Json, JsonOptions};
use gilt::markdown::Markdown;
use gilt::panel::Panel;
use gilt::rule::Rule;
use gilt::style::Style;
use gilt::text::Text;

// ---------------------------------------------------------------------------
// Style subcommand options
// ---------------------------------------------------------------------------

/// Styling flags for the `style` subcommand.
#[derive(Debug, Default)]
pub struct StyleOpts {
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub dim: bool,
}

// ---------------------------------------------------------------------------
// Helper: build a capturing console that writes rendered ANSI to `out`
// ---------------------------------------------------------------------------

/// Build a console that captures output and returns the rendered ANSI string.
///
/// Using `force_terminal(true)` so ANSI escape codes are emitted even when
/// the real stdout is not a TTY (e.g. under `cargo test`).
fn make_console() -> Console {
    Console::builder().force_terminal(true).width(80).build()
}

// ---------------------------------------------------------------------------
// Subcommand functions
// ---------------------------------------------------------------------------

/// `gilt print <MARKUP>` — render rich markup to `out`.
pub fn cmd_print(markup: &str, out: &mut impl Write) -> io::Result<()> {
    let mut console = make_console();
    console.begin_capture();
    console.print_text(markup);
    let rendered = console.end_capture();
    out.write_all(rendered.as_bytes())
}

/// `gilt style` — render styled text to `out`.
pub fn cmd_style(opts: &StyleOpts, text: &str, out: &mut impl Write) -> io::Result<()> {
    // Build a style definition string from the opts flags.
    let mut parts: Vec<String> = Vec::new();
    if let Some(ref fg) = opts.fg {
        parts.push(fg.clone());
    }
    if let Some(ref bg) = opts.bg {
        parts.push(format!("on {}", bg));
    }
    if opts.bold {
        parts.push("bold".into());
    }
    if opts.italic {
        parts.push("italic".into());
    }
    if opts.underline {
        parts.push("underline".into());
    }
    if opts.dim {
        parts.push("dim".into());
    }

    let style_def = parts.join(" ");
    let style = if style_def.is_empty() {
        Style::null()
    } else {
        Style::parse(&style_def)
    };

    let content = Text::styled_with(text, style);

    let mut console = make_console();
    console.begin_capture();
    console.print(&content);
    let rendered = console.end_capture();
    out.write_all(rendered.as_bytes())
}

/// `gilt table` — read CSV from `input`, render a table to `out`.
pub fn cmd_table(input: impl Read, out: &mut impl Write) -> io::Result<()> {
    let mut buf = String::new();
    let mut r = input;
    r.read_to_string(&mut buf)?;

    let csv = CsvTable::from_csv_str(&buf)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    let table = csv.to_table();

    let mut console = make_console();
    console.begin_capture();
    console.print(&table);
    let rendered = console.end_capture();
    out.write_all(rendered.as_bytes())
}

/// `gilt rule [TITLE]` — render a horizontal rule to `out`.
pub fn cmd_rule(title: Option<&str>, out: &mut impl Write) -> io::Result<()> {
    let rule = match title {
        Some(t) => Rule::with_title(t),
        None => Rule::new(),
    };

    let mut console = make_console();
    console.begin_capture();
    console.print(&rule);
    let rendered = console.end_capture();
    out.write_all(rendered.as_bytes())
}

/// `gilt panel <TEXT> --title <T>` — render a panel to `out`.
pub fn cmd_panel(text: &str, title: Option<&str>, out: &mut impl Write) -> io::Result<()> {
    let content = Text::new(text, Style::null());
    let panel = match title {
        Some(t) => Panel::new(content).with_title(t),
        None => Panel::new(content),
    };

    let mut console = make_console();
    console.begin_capture();
    console.print(&panel);
    let rendered = console.end_capture();
    out.write_all(rendered.as_bytes())
}

/// `gilt markdown` — read stdin, render Markdown to `out`.
pub fn cmd_markdown(input: impl Read, out: &mut impl Write) -> io::Result<()> {
    let mut buf = String::new();
    let mut r = input;
    r.read_to_string(&mut buf)?;

    let md = Markdown::new(&buf);

    let mut console = make_console();
    console.begin_capture();
    console.print(&md);
    let rendered = console.end_capture();
    out.write_all(rendered.as_bytes())
}

/// `gilt json` — read stdin, pretty-print JSON to `out`.
pub fn cmd_json(input: impl Read, out: &mut impl Write) -> io::Result<()> {
    let mut buf = String::new();
    let mut r = input;
    r.read_to_string(&mut buf)?;

    let json = Json::new(&buf, JsonOptions::default())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    let mut console = make_console();
    console.begin_capture();
    console.print(&json);
    let rendered = console.end_capture();
    out.write_all(rendered.as_bytes())
}

// ---------------------------------------------------------------------------
// Tests (written RED-first, then implemented above)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn output_of(buf: Vec<u8>) -> String {
        String::from_utf8(buf).expect("output is valid UTF-8")
    }

    // -- cmd_print -----------------------------------------------------------

    /// RED: cmd_print of `"[bold red]hi[/]"` writes output containing "hi"
    /// and ANSI bold/red codes (force_terminal → ANSI emitted).
    #[test]
    fn test_cmd_print_contains_text() {
        let mut out = Vec::<u8>::new();
        cmd_print("[bold red]hi[/]", &mut out).unwrap();
        let s = output_of(out);
        assert!(s.contains("hi"), "expected 'hi' in output, got: {:?}", s);
    }

    #[test]
    fn test_cmd_print_contains_ansi_codes() {
        let mut out = Vec::<u8>::new();
        cmd_print("[bold red]hi[/]", &mut out).unwrap();
        let s = output_of(out);
        // force_terminal(true) means ANSI escape codes must be present
        assert!(
            s.contains('\x1b'),
            "expected ANSI escape codes in output, got: {:?}",
            s
        );
    }

    // -- cmd_style -----------------------------------------------------------

    /// RED: cmd_style with bold+red writes "x" with ANSI codes.
    #[test]
    fn test_cmd_style_contains_text() {
        let opts = StyleOpts {
            fg: Some("red".into()),
            bold: true,
            ..Default::default()
        };
        let mut out = Vec::<u8>::new();
        cmd_style(&opts, "x", &mut out).unwrap();
        let s = output_of(out);
        assert!(s.contains('x'), "expected 'x' in output, got: {:?}", s);
    }

    #[test]
    fn test_cmd_style_contains_ansi_codes() {
        let opts = StyleOpts {
            fg: Some("red".into()),
            bold: true,
            ..Default::default()
        };
        let mut out = Vec::<u8>::new();
        cmd_style(&opts, "x", &mut out).unwrap();
        let s = output_of(out);
        assert!(
            s.contains('\x1b'),
            "expected ANSI escape codes in output, got: {:?}",
            s
        );
    }

    // -- cmd_table -----------------------------------------------------------

    /// RED: cmd_table given CSV "a,b\n1,2\n" writes a table containing a, b, 1, 2.
    #[test]
    fn test_cmd_table_contains_headers_and_data() {
        let csv = "a,b\n1,2\n";
        let input = Cursor::new(csv.as_bytes());
        let mut out = Vec::<u8>::new();
        cmd_table(input, &mut out).unwrap();
        let s = output_of(out);
        assert!(s.contains('a'), "expected 'a' in output, got: {:?}", s);
        assert!(s.contains('b'), "expected 'b' in output, got: {:?}", s);
        assert!(s.contains('1'), "expected '1' in output, got: {:?}", s);
        assert!(s.contains('2'), "expected '2' in output, got: {:?}", s);
    }

    // -- cmd_rule ------------------------------------------------------------

    /// RED: cmd_rule with a title writes the title text.
    #[test]
    fn test_cmd_rule_with_title() {
        let mut out = Vec::<u8>::new();
        cmd_rule(Some("Section"), &mut out).unwrap();
        let s = output_of(out);
        assert!(
            s.contains("Section"),
            "expected 'Section' in rule output, got: {:?}",
            s
        );
    }

    /// RED: cmd_rule with no title still writes non-empty output.
    #[test]
    fn test_cmd_rule_no_title() {
        let mut out = Vec::<u8>::new();
        cmd_rule(None, &mut out).unwrap();
        let s = output_of(out);
        assert!(!s.is_empty(), "expected non-empty rule output");
    }

    // -- cmd_panel -----------------------------------------------------------

    /// RED: cmd_panel writes the content text.
    #[test]
    fn test_cmd_panel_contains_text() {
        let mut out = Vec::<u8>::new();
        cmd_panel("hello panel", None, &mut out).unwrap();
        let s = output_of(out);
        assert!(
            s.contains("hello panel"),
            "expected 'hello panel' in panel output, got: {:?}",
            s
        );
    }

    /// RED: cmd_panel with title writes both content and title.
    #[test]
    fn test_cmd_panel_with_title() {
        let mut out = Vec::<u8>::new();
        cmd_panel("body", Some("MyTitle"), &mut out).unwrap();
        let s = output_of(out);
        assert!(
            s.contains("body"),
            "expected 'body' in panel output, got: {:?}",
            s
        );
        assert!(
            s.contains("MyTitle"),
            "expected 'MyTitle' in panel output, got: {:?}",
            s
        );
    }

    // -- cmd_markdown --------------------------------------------------------

    /// RED: cmd_markdown renders markdown containing the input text.
    #[test]
    fn test_cmd_markdown_contains_text() {
        let md = "# Hello\n\nThis is **bold** text.\n";
        let input = Cursor::new(md.as_bytes());
        let mut out = Vec::<u8>::new();
        cmd_markdown(input, &mut out).unwrap();
        let s = output_of(out);
        assert!(
            s.contains("Hello"),
            "expected 'Hello' in markdown output, got: {:?}",
            s
        );
    }

    // -- cmd_json ------------------------------------------------------------

    /// RED: cmd_json pretty-prints JSON containing the key "name".
    #[test]
    fn test_cmd_json_contains_key() {
        let json_input = r#"{"name": "gilt", "version": "1.9"}"#;
        let input = Cursor::new(json_input.as_bytes());
        let mut out = Vec::<u8>::new();
        cmd_json(input, &mut out).unwrap();
        let s = output_of(out);
        assert!(
            s.contains("name"),
            "expected 'name' in json output, got: {:?}",
            s
        );
        assert!(
            s.contains("gilt"),
            "expected 'gilt' in json output, got: {:?}",
            s
        );
    }
}
