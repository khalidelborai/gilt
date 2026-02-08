//! Comprehensive showcase of gilt features as a continuous live demo.
//!
//! Run with: `cargo run --example showcase --all-features`
//!
//! Cycles through every major feature group with animated pacing,
//! demonstrating text styling, widgets, highlighting, progress bars,
//! status spinners, and more.

use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use gilt::bar::Bar;
use gilt::color::Color;
use gilt::color_triplet::ColorTriplet;
use gilt::columns::Columns;
use gilt::console::Console;
use gilt::emoji::Emoji;
use gilt::emoji_replace::emoji_replace;
use gilt::filesize;
use gilt::gradient::Gradient;
use gilt::highlighter::*;
use gilt::inspect::Inspect;
use gilt::panel::Panel;
use gilt::prelude::*;
use gilt::pretty::Pretty;
use gilt::progress::Progress;
use gilt::rule::Rule;
use gilt::status::Status;
use gilt::tree::Tree;

use serde_json::json;

/// Pause between sections for a live-demo feel.
fn pause() {
    thread::sleep(Duration::from_millis(300));
}

fn main() {
    let mut console = Console::builder()
        .width(90)
        .force_terminal(true)
        .no_color(false)
        .build();

    // =========================================================================
    // 1. Welcome Banner
    // =========================================================================
    console.line(1);
    let banner = Gradient::rainbow("  gilt -- Rich Terminal Formatting for Rust  ")
        .with_style(Style::parse("bold").unwrap());
    console.print(&banner);
    console.line(1);
    console.rule(Some("Welcome"));
    pause();

    // =========================================================================
    // 2. Text Styling (Stylize trait)
    // =========================================================================
    console.rule(Some("Text Styling"));

    console.print(&"Bold text".bold());
    console.print(&"Italic text".italic());
    console.print(&"Underlined text".underline());
    console.print(&"Strikethrough text".strikethrough());
    console.print(&"Dim text".dim());
    console.print(&"Bold + Italic + Underline".bold().italic().underline());
    console.line(1);

    // Standard colors
    console.print(&"Red".red());
    console.print(&"Green".green());
    console.print(&"Blue".blue());
    console.print(&"Yellow".yellow());
    console.print(&"Magenta".magenta());
    console.print(&"Cyan".cyan());
    console.print(&"White".white());
    console.print(&"Black on white".black().on_white());
    console.print(&"Bright Red".bright_red());
    console.print(&"Bright Green".bright_green());
    console.print(&"Bright Blue".bright_blue());
    console.print(&"Bright Yellow".bright_yellow());
    console.print(&"Bright Magenta".bright_magenta());
    console.print(&"Bright Cyan".bright_cyan());
    console.print(&"Bright White".bright_white());
    console.line(1);

    // RGB / TrueColor
    console.print(&"TrueColor: #ff6600 (orange)".fg("#ff6600"));
    console.print(&"TrueColor: #00ccff (sky blue)".fg("#00ccff"));
    console.print(
        &"TrueColor: bold + #00ff88 fg + #222222 bg"
            .bold()
            .fg("#00ff88")
            .bg("#222222"),
    );
    pause();

    // =========================================================================
    // 3. Markup
    // =========================================================================
    console.rule(Some("Markup"));

    console.print_text("[bold magenta]This text uses markup[/bold magenta] for [italic cyan]inline styling[/italic cyan].");
    console.print_text("[red]Error:[/red] something went wrong in [bold]module.rs[/bold]");
    console.print_text("[dim]Dim text[/dim], [underline]underlined[/underline], and [bold green]bold green[/bold green].");
    pause();

    // =========================================================================
    // 4. Panel
    // =========================================================================
    console.rule(Some("Panel"));

    let content = Text::new(
        "Gilt is a Rust port of Python's rich library.\nIt brings beautiful terminal formatting to the Rust ecosystem.",
        Style::null(),
    );
    let panel = Panel::fit(content)
        .title(Text::new("About Gilt", Style::parse("bold cyan").unwrap()))
        .subtitle(Text::new("v0.1.0", Style::parse("dim").unwrap()))
        .border_style(Style::parse("bright_blue").unwrap());
    console.print(&panel);
    pause();

    // =========================================================================
    // 5. Table
    // =========================================================================
    console.rule(Some("Table"));

    let mut table = Table::new(&["Language", "Paradigm", "Year", "Typing"]);
    table.title = Some("Programming Languages".to_string());
    table.title_style = "bold".to_string();
    table.header_style = "bold magenta".to_string();
    table.border_style = "bright_green".to_string();
    table.add_row(&["Rust", "Systems / Multi", "2010", "Static, Strong"]);
    table.add_row(&["Python", "Multi-paradigm", "1991", "Dynamic, Strong"]);
    table.add_row(&["Haskell", "Functional", "1990", "Static, Strong"]);
    table.add_row(&["Go", "Concurrent / Imperative", "2009", "Static, Strong"]);
    table.add_row(&["TypeScript", "Multi-paradigm", "2012", "Static, Gradual"]);
    console.print(&table);
    pause();

    // =========================================================================
    // 6. Tree
    // =========================================================================
    console.rule(Some("Tree"));

    let bold_blue = Style::parse("bold blue").unwrap();
    let green = Style::parse("green").unwrap();
    let default = Style::null();

    let mut tree = Tree::new(Text::new("my_project/", bold_blue.clone()));
    tree.guide_style = Style::parse("dim").unwrap();

    {
        let src = tree.add(Text::new("src/", bold_blue.clone()));
        src.add(Text::new("main.rs", green.clone()));
        src.add(Text::new("lib.rs", green.clone()));
        let models = src.add(Text::new("models/", bold_blue.clone()));
        models.add(Text::new("user.rs", default.clone()));
        models.add(Text::new("post.rs", default.clone()));
    }
    {
        let tests = tree.add(Text::new("tests/", bold_blue.clone()));
        tests.add(Text::new("integration.rs", default.clone()));
        tests.add(Text::new("unit.rs", default.clone()));
    }
    tree.add(Text::new("Cargo.toml", green.clone()));
    tree.add(Text::new("README.md", default.clone()));

    console.print(&tree);
    pause();

    // =========================================================================
    // 7. Columns
    // =========================================================================
    console.rule(Some("Columns"));

    let items = [
        "Rust",
        "Python",
        "Go",
        "TypeScript",
        "Java",
        "C++",
        "Ruby",
        "Swift",
        "Kotlin",
        "Haskell",
        "Elixir",
        "Zig",
        "Scala",
        "Clojure",
        "Erlang",
        "OCaml",
    ];

    let mut cols = Columns::new().with_equal(true);
    for item in &items {
        cols.add_renderable(item);
    }
    console.print(&cols);
    pause();

    // =========================================================================
    // 8. Rule
    // =========================================================================
    console.rule(Some("Rules"));

    console.print(&Rule::new());
    console.print(&Rule::with_title("Centered Title"));
    console.print(
        &Rule::with_title("Heavy Rule")
            .characters("\u{2501}")
            .style(Style::parse("bold red").unwrap()),
    );
    console.print(
        &Rule::with_title("Double Line")
            .characters("=")
            .style(Style::parse("green").unwrap()),
    );
    console.print(
        &Rule::with_title("Dotted")
            .characters(".")
            .style(Style::parse("dim").unwrap()),
    );
    pause();

    // =========================================================================
    // 9. Emoji
    // =========================================================================
    console.rule(Some("Emoji"));

    let emoji_names = ["heart", "rocket", "star", "fire", "sparkles", "thumbs_up"];
    for name in &emoji_names {
        match Emoji::new(name) {
            Ok(emoji) => {
                let line = Text::new(&format!("  :{name}:  =>  {emoji}"), Style::null());
                console.print(&line);
            }
            Err(_) => {
                let line = Text::new(&format!("  :{name}:  =>  (not found)"), Style::null());
                console.print(&line);
            }
        }
    }
    console.line(1);

    let replaced = emoji_replace("I :heart: Rust! :rocket: :sparkles:", None);
    console.print(&Text::new(
        &format!("  Replaced: {replaced}"),
        Style::null(),
    ));
    pause();

    // =========================================================================
    // 10. Gradient Text
    // =========================================================================
    console.rule(Some("Gradient Text"));

    let rainbow =
        Gradient::rainbow("ROYGBIV: Red Orange Yellow Green Blue Indigo Violet - full spectrum!");
    console.print(&rainbow);

    let blue_to_green = Gradient::two_color(
        "Smooth transition from ocean blue to forest green",
        Color::from_rgb(0, 100, 255),
        Color::from_rgb(0, 200, 80),
    );
    console.print(&blue_to_green);

    let sunset = Gradient::new(
        "Sunset gradient: deep red through orange to warm gold",
        vec![
            Color::from_rgb(139, 0, 0),
            Color::from_rgb(255, 69, 0),
            Color::from_rgb(255, 200, 0),
        ],
    );
    console.print(&sunset);
    pause();

    // =========================================================================
    // 11. Syntax Highlighting (feature-gated)
    // =========================================================================
    #[cfg(feature = "syntax")]
    {
        console.rule(Some("Syntax Highlighting"));

        let rust_code = r#"use std::collections::HashMap;

fn main() {
    let mut scores: HashMap<&str, i32> = HashMap::new();
    scores.insert("Alice", 100);
    scores.insert("Bob", 85);

    for (name, score) in &scores {
        println!("{name}: {score}");
    }
}"#;

        let syntax = gilt::syntax::Syntax::new(rust_code, "rs")
            .with_line_numbers(true)
            .with_theme("base16-ocean.dark");
        console.print(&syntax);
        pause();
    }

    // =========================================================================
    // 12. Markdown (feature-gated)
    // =========================================================================
    #[cfg(feature = "markdown")]
    {
        console.rule(Some("Markdown"));

        let md_source = r#"# Gilt Features

Gilt supports **bold**, *italic*, and `inline code` in markdown.

## Bullet List

- Rich text rendering
- Progress bars and spinners
- Tables, trees, and panels

> The best way to predict the future is to invent it. -- Alan Kay
"#;

        let md = gilt::markdown::Markdown::new(md_source);
        console.print(&md);
        pause();
    }

    // =========================================================================
    // 13. JSON (feature-gated)
    // =========================================================================
    #[cfg(feature = "json")]
    {
        console.rule(Some("JSON"));

        let json_str = r#"{
    "name": "Gilt",
    "version": "0.1.0",
    "features": ["syntax", "markdown", "json"],
    "metadata": {
        "stars": 42,
        "active": true
    }
}"#;

        let json_widget =
            gilt::json::Json::new(json_str, gilt::json::JsonOptions::default()).unwrap();
        console.print(&json_widget);
        pause();
    }

    // =========================================================================
    // 14. Highlighters
    // =========================================================================
    console.rule(Some("Highlighters"));

    let url_hl = URLHighlighter::new();
    let text = url_hl.apply("Visit https://example.com or http://localhost:8080/api");
    console.print_text("[dim]URL:[/dim]");
    console.print(&text);

    let uuid_hl = UUIDHighlighter::new();
    let text = uuid_hl.apply("Request ID: 550e8400-e29b-41d4-a716-446655440000");
    console.print_text("[dim]UUID:[/dim]");
    console.print(&text);

    let iso_hl = ISODateHighlighter::new();
    let text = iso_hl.apply("Created: 2024-01-15T10:30:00Z  Updated: 2024-06-20");
    console.print_text("[dim]ISO Date:[/dim]");
    console.print(&text);

    let jp_hl = JSONPathHighlighter::new();
    let text = jp_hl.apply("Access $.config.database.host or .users[0].name");
    console.print_text("[dim]JSONPath:[/dim]");
    console.print(&text);
    pause();

    // =========================================================================
    // 15. Inspect
    // =========================================================================
    console.rule(Some("Inspect"));

    let numbers = vec![1, 2, 3, 42, 100];
    let inspect = Inspect::new(&numbers).with_label("numbers");
    console.print(&inspect);

    let mut config: HashMap<String, String> = HashMap::new();
    config.insert("host".into(), "localhost".into());
    config.insert("port".into(), "8080".into());
    config.insert("debug".into(), "true".into());
    let inspect = Inspect::new(&config)
        .with_label("config")
        .with_doc("Application configuration map");
    console.print(&inspect);
    pause();

    // =========================================================================
    // 16. Pretty Printing
    // =========================================================================
    console.rule(Some("Pretty Printing"));

    let nested = json!({
        "server": {
            "host": "0.0.0.0",
            "port": 8080,
            "tls": { "enabled": true, "cert": "/etc/ssl/cert.pem" }
        },
        "database": {
            "url": "postgres://localhost:5432/myapp",
            "pool_size": 10
        }
    });
    let pretty = Pretty::from_json(&nested);
    console.print(&pretty);
    pause();

    // =========================================================================
    // 17. Accessibility
    // =========================================================================
    console.rule(Some("Accessibility"));

    let pairs: &[(&str, ColorTriplet, &str, ColorTriplet)] = &[
        (
            "Black",
            ColorTriplet::new(0, 0, 0),
            "White",
            ColorTriplet::new(255, 255, 255),
        ),
        (
            "Dark Blue",
            ColorTriplet::new(0, 0, 139),
            "Light Yellow",
            ColorTriplet::new(255, 255, 224),
        ),
        (
            "Red",
            ColorTriplet::new(255, 0, 0),
            "White",
            ColorTriplet::new(255, 255, 255),
        ),
        (
            "Gray",
            ColorTriplet::new(128, 128, 128),
            "Black",
            ColorTriplet::new(0, 0, 0),
        ),
    ];

    for (fg_name, fg, bg_name, bg) in pairs {
        let ratio = contrast_ratio(fg, bg);
        let aa = if meets_aa(fg, bg) { "PASS" } else { "FAIL" };
        let aaa = if meets_aaa(fg, bg) { "PASS" } else { "FAIL" };
        let line = format!("  {fg_name} on {bg_name}: ratio={ratio:.1}:1  AA={aa}  AAA={aaa}");
        console.print(&Text::new(&line, Style::null()));
    }
    pause();

    // =========================================================================
    // 18. Progress Bars (Animated)
    // =========================================================================
    console.rule(Some("Progress Bars (Animated)"));

    {
        let progress_console = Console::builder()
            .width(90)
            .force_terminal(true)
            .no_color(false)
            .build();

        let mut progress = Progress::new(Progress::default_columns())
            .with_console(progress_console)
            .with_auto_refresh(false);

        let task1 = progress.add_task("Downloading dataset.tar.gz", Some(1000.0));
        let task2 = progress.add_task("Processing model-weights.bin", Some(500.0));
        let task3 = progress.add_task("Compiling config.json", Some(200.0));

        progress.start();

        let mut done1 = false;
        let mut done2 = false;
        let mut done3 = false;

        loop {
            if !done1 {
                progress.advance(task1, 20.0);
                if let Some(t) = progress.get_task(task1) {
                    if t.finished() {
                        done1 = true;
                    }
                }
            }
            if !done2 {
                progress.advance(task2, 8.0);
                if let Some(t) = progress.get_task(task2) {
                    if t.finished() {
                        done2 = true;
                    }
                }
            }
            if !done3 {
                progress.advance(task3, 5.0);
                if let Some(t) = progress.get_task(task3) {
                    if t.finished() {
                        done3 = true;
                    }
                }
            }

            progress.refresh();

            if done1 && done2 && done3 {
                break;
            }

            thread::sleep(Duration::from_millis(50));
        }

        progress.stop();
    }
    pause();

    // =========================================================================
    // 19. Status Spinner (Animated)
    // =========================================================================
    console.rule(Some("Status Spinner (Animated)"));

    {
        let status_console = Console::builder()
            .force_terminal(true)
            .no_color(false)
            .build();

        let messages = [
            "Connecting to server...",
            "Authenticating...",
            "Fetching data...",
            "Almost done...",
        ];

        let mut status = Status::new(messages[0]).with_console(status_console);
        status.start();

        for msg in &messages[1..] {
            thread::sleep(Duration::from_millis(500));
            status.update().status(msg).apply().unwrap();
        }

        thread::sleep(Duration::from_millis(500));
        status.stop();
    }
    pause();

    // =========================================================================
    // 20. Bar
    // =========================================================================
    console.rule(Some("Bar Widgets"));

    let bar_width: usize = 40;
    let levels: &[(&str, f64)] = &[
        ("  0% ", 0.0),
        (" 25% ", 10.0),
        (" 50% ", 20.0),
        (" 75% ", 30.0),
        ("100% ", 40.0),
    ];

    for (label, end) in levels {
        let label_text = Text::new(label, Style::parse("bold").unwrap());
        console.print(&label_text);
        let bar = Bar::new(40.0, 0.0, *end).with_width(bar_width);
        console.print(&bar);
    }

    console.line(1);

    let colors: &[(&str, &str, f64)] = &[
        ("Red   ", "red", 15.0),
        ("Green ", "green", 25.0),
        ("Blue  ", "blue", 35.0),
        ("Yellow", "yellow", 40.0),
    ];

    for (label, color_name, end) in colors {
        let label_text = Text::new(&format!("{label} "), Style::null());
        console.print(&label_text);
        let bar = Bar::new(40.0, 0.0, *end)
            .with_width(bar_width)
            .with_color(Color::parse(color_name).unwrap());
        console.print(&bar);
    }
    pause();

    // =========================================================================
    // 21. Filesize
    // =========================================================================
    console.rule(Some("Filesize"));

    let sizes: &[(&str, u64)] = &[
        ("Empty file", 0),
        ("Small file", 1),
        ("Text file", 4_096),
        ("Photo", 3_500_000),
        ("Video", 1_200_000_000),
        ("Dataset", 5_000_000_000_000),
    ];

    for (name, size) in sizes {
        let decimal = filesize::decimal(*size, 1, " ");
        let line = format!("  {name:.<20} {decimal:>12}");
        console.print(&Text::new(&line, Style::null()));
    }
    pause();

    // =========================================================================
    // Farewell
    // =========================================================================
    console.line(1);
    let farewell = Gradient::rainbow("  Thank you for exploring gilt!  ")
        .with_style(Style::parse("bold").unwrap());
    console.print(&farewell);
    console.rule(None);
}
