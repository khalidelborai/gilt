//! Comprehensive showcase of gilt features as a continuous live demo.
//!
//! Run with: `cargo run --example showcase --all-features`
//!
//! Cycles through every major feature group with animated pacing,
//! demonstrating text styling, widgets, highlighting, progress bars,
//! status spinners, and more.

use std::collections::HashMap;
use std::io;
use std::thread;
use std::time::Duration;

use gilt::align_widget::Align;
use gilt::ansi::AnsiDecoder;
use gilt::bar::Bar;
use gilt::canvas::Canvas;
use gilt::color::Color;
use gilt::color_triplet::ColorTriplet;
use gilt::columns::Columns;
use gilt::console::Console;
use gilt::constrain::Constrain;
use gilt::csv_table::CsvTable;
use gilt::diff::Diff;
use gilt::emoji::Emoji;
use gilt::emoji_replace::emoji_replace;
use gilt::figlet::Figlet;
use gilt::filesize;
use gilt::gradient::Gradient;
use gilt::highlighter::*;
use gilt::inspect::Inspect;
use gilt::layout::Layout;
use gilt::padding::{Padding, PaddingDimensions};
use gilt::panel::Panel;
use gilt::prelude::*;
use gilt::pretty::Pretty;
use gilt::progress::Progress;
use gilt::rule::Rule;
use gilt::scope::Scope;
use gilt::sparkline::Sparkline;
use gilt::spinners::SPINNERS;
use gilt::status::Status;
use gilt::styled::Styled;
use gilt::theme::Theme;
use gilt::traceback::{Frame, Traceback};
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
        .with_title(Text::new("About Gilt", Style::parse("bold cyan").unwrap()))
        .with_subtitle(Text::new("v0.5.0", Style::parse("dim").unwrap()))
        .with_border_style(Style::parse("bright_blue").unwrap());
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

    let mut tree = Tree::new(Text::new("my_project/", bold_blue.clone()))
        .with_guide_style(Style::parse("dim").unwrap());

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
            .with_characters("\u{2501}")
            .with_style(Style::parse("bold red").unwrap()),
    );
    console.print(
        &Rule::with_title("Double Line")
            .with_characters("=")
            .with_style(Style::parse("green").unwrap()),
    );
    console.print(
        &Rule::with_title("Dotted")
            .with_characters(".")
            .with_style(Style::parse("dim").unwrap()),
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
    // 21. Filesize (Decimal & Binary)
    // =========================================================================
    console.rule(Some("Filesize (Decimal & Binary)"));

    let sizes: &[(&str, u64)] = &[
        ("Empty file", 0),
        ("Small file", 1),
        ("Text file", 4_096),
        ("Photo", 3_500_000),
        ("Video", 1_200_000_000),
        ("Dataset", 5_000_000_000_000),
    ];

    console.print(&Text::new(
        &format!(
            "  {:.<20} {:>14}  {:>14}",
            "Name", "Decimal (SI)", "Binary (IEC)"
        ),
        Style::parse("bold").unwrap(),
    ));
    for (name, size) in sizes {
        let dec = filesize::decimal(*size, 1, " ");
        let bin = filesize::binary(*size, 1, " ");
        let line = format!("  {name:.<20} {dec:>14}  {bin:>14}");
        console.print(&Text::new(&line, Style::null()));
    }
    pause();

    // =========================================================================
    // 22. Layout
    // =========================================================================
    console.rule(Some("Layout"));

    {
        let mut layout_console = Console::builder()
            .width(90)
            .height(12)
            .force_terminal(true)
            .no_color(false)
            .build();

        let mut layout = Layout::new(None, Some("root".to_string()), None, None, None, None);
        let header = Layout::new(
            Some("HEADER: gilt layout system".to_string()),
            Some("header".to_string()),
            Some(3),
            None,
            None,
            None,
        );
        let mut body = Layout::new(None, Some("body".to_string()), None, None, Some(1), None);
        let sidebar = Layout::new(
            Some("Sidebar".to_string()),
            Some("sidebar".to_string()),
            Some(20),
            None,
            None,
            None,
        );
        let main = Layout::new(
            Some("Main content area".to_string()),
            Some("main".to_string()),
            None,
            None,
            Some(1),
            None,
        );
        body.split_row(vec![sidebar, main]);
        let footer = Layout::new(
            Some("FOOTER".to_string()),
            Some("footer".to_string()),
            Some(3),
            None,
            None,
            None,
        );
        layout.split_column(vec![header, body, footer]);
        layout_console.print(&layout);
    }
    pause();

    // =========================================================================
    // 23. Align
    // =========================================================================
    console.rule(Some("Alignment"));

    let left = Align::left(Text::new("Left-aligned text", Style::null()));
    console.print(&left);

    let center = Align::center(Text::new("Center-aligned text", Style::null()));
    console.print(&center);

    let right = Align::right(Text::new("Right-aligned text", Style::null()));
    console.print(&right);
    pause();

    // =========================================================================
    // 24. Padding
    // =========================================================================
    console.rule(Some("Padding"));

    let padded = Padding::new(
        Text::new(
            "This text has padding: top=1, right=4, bottom=1, left=8",
            Style::null(),
        ),
        PaddingDimensions::Full(1, 4, 1, 8),
        Style::null(),
        true,
    );
    console.print(&padded);

    let indented = Padding::indent(Text::new("Indented text (left=6)", Style::null()), 6);
    console.print(&indented);
    pause();

    // =========================================================================
    // 25. Constrain
    // =========================================================================
    console.rule(Some("Constrain"));

    let wide_text = Text::new(
        "This text would normally fill the full 90-column width, but Constrain limits it to 50 characters, causing it to wrap earlier.",
        Style::null(),
    );
    let constrained = Constrain::new(wide_text, Some(50));
    console.print(&constrained);
    pause();

    // =========================================================================
    // 26. Styled Containers
    // =========================================================================
    console.rule(Some("Styled Containers"));

    let inner = Text::new("Bold + italic overlay via Styled container", Style::null());
    let styled_widget = Styled::new(inner, Style::parse("bold italic cyan").unwrap());
    console.print(&styled_widget);

    let inner2 = Text::new("Red on dark background", Style::parse("red").unwrap());
    let styled_widget2 = Styled::new(inner2, Style::parse("on grey11").unwrap());
    console.print(&styled_widget2);
    pause();

    // =========================================================================
    // 27. Text Justify Modes
    // =========================================================================
    console.rule(Some("Text Justification"));

    let justify_modes = [
        ("Left", JustifyMethod::Left),
        ("Center", JustifyMethod::Center),
        ("Right", JustifyMethod::Right),
        ("Full", JustifyMethod::Full),
    ];

    for (label, justify) in &justify_modes {
        let mut text = Text::new(
            &format!("{label}: The quick brown fox jumps over the lazy dog near the riverbank."),
            Style::null(),
        );
        text.justify = Some(*justify);
        let panel = Panel::fit(text)
            .with_title(Text::new(label, Style::parse("bold").unwrap()))
            .with_border_style(Style::parse("dim").unwrap());
        console.print(&panel);
    }
    pause();

    // =========================================================================
    // 28. Text Overflow
    // =========================================================================
    console.rule(Some("Text Overflow"));

    let overflow_modes = [
        ("Fold", OverflowMethod::Fold),
        ("Crop", OverflowMethod::Crop),
        ("Ellipsis", OverflowMethod::Ellipsis),
    ];

    for (label, overflow) in &overflow_modes {
        let mut text = Text::new(
            "Superlongwordwithoutanyspacesthatexceedsthenormalwidth_and_continues_going_forever",
            Style::null(),
        );
        text.overflow = Some(*overflow);
        let constrained_overflow = Constrain::new(text, Some(40));
        console.print(&Text::new(
            &format!("  {label}:"),
            Style::parse("bold").unwrap(),
        ));
        console.print(&constrained_overflow);
    }
    pause();

    // =========================================================================
    // 29. Scope
    // =========================================================================
    console.rule(Some("Scope"));

    let scope = Scope::from_pairs(&[
        ("host", "localhost"),
        ("port", "8080"),
        ("debug", "true"),
        ("workers", "4"),
        ("database_url", "postgres://localhost/myapp"),
    ])
    .title("Server Config");
    console.print(&scope);
    pause();

    // =========================================================================
    // 30. Logging
    // =========================================================================
    console.rule(Some("Logging (console.log)"));

    console.log("Application started");
    console.log("[bold green]Server[/bold green] listening on port 8080");
    console.log("[yellow]Warning:[/yellow] cache miss for key 'user:42'");
    console.log("[red]Error:[/red] connection timeout after 30s");
    pause();

    // =========================================================================
    // 31. Color Systems
    // =========================================================================
    console.rule(Some("Color Systems"));

    let color_systems = [
        ("truecolor", "TrueColor (16M)"),
        ("256", "256 colors"),
        ("standard", "Standard (16)"),
    ];

    for (cs, label) in &color_systems {
        let mut cs_console = Console::builder()
            .width(90)
            .force_terminal(true)
            .color_system(cs)
            .build();
        cs_console.begin_capture();
        cs_console.print(&Text::styled(
            &format!("  {label}: Hello from rgb(255,102,0) on rgb(0,51,102)"),
            Style::parse("rgb(255,102,0) on rgb(0,51,102) bold").unwrap(),
        ));
        let captured = cs_console.end_capture();
        console.print(&Text::new(captured.trim_end(), Style::null()));
    }

    // No color
    {
        let mut nc_console = Console::builder()
            .width(90)
            .force_terminal(true)
            .no_color(true)
            .build();
        nc_console.begin_capture();
        nc_console.print(&Text::styled(
            "  No Color: Hello (styles stripped)",
            Style::parse("bold red").unwrap(),
        ));
        let captured = nc_console.end_capture();
        console.print(&Text::new(captured.trim_end(), Style::null()));
    }
    pause();

    // =========================================================================
    // 32. Theme Push/Pop
    // =========================================================================
    console.rule(Some("Theme Push/Pop"));

    console.print_text("[bold]Default theme:[/bold] [info]info style[/info]");

    let mut custom_styles = HashMap::new();
    custom_styles.insert(
        "info".to_string(),
        Style::parse("bold magenta on grey15").unwrap(),
    );
    let custom_theme = Theme::new(Some(custom_styles), true);
    console.push_theme(custom_theme);

    console.print_text("[bold]Custom theme:[/bold] [info]info is now magenta on grey[/info]");

    console.pop_theme();
    console.print_text("[bold]After pop:[/bold] [info]info reverted to default[/info]");
    pause();

    // =========================================================================
    // 33. Console Capture & Export
    // =========================================================================
    console.rule(Some("Console Capture"));

    {
        let mut cap_console = Console::builder()
            .width(60)
            .force_terminal(true)
            .no_color(true)
            .build();

        cap_console.begin_capture();
        cap_console.print(&Text::new("First captured line", Style::null()));
        cap_console.print(&Text::new("Second captured line", Style::null()));
        cap_console.print(&Text::new("Third captured line", Style::null()));
        let captured = cap_console.end_capture();

        console.print(&Text::new(
            "  Captured output (3 lines):",
            Style::parse("bold").unwrap(),
        ));
        for line in captured.lines() {
            console.print(&Text::new(
                &format!("    | {line}"),
                Style::parse("dim").unwrap(),
            ));
        }
    }
    pause();

    // =========================================================================
    // 34. Synchronized Output
    // =========================================================================
    console.rule(Some("Synchronized Output"));

    console.synchronized(|c| {
        c.print(&Text::new(
            "  These lines are rendered atomically",
            Style::parse("bold green").unwrap(),
        ));
        c.print(&Text::new(
            "  inside a DEC Mode 2026 sync block.",
            Style::parse("green").unwrap(),
        ));
        c.print(&Text::new(
            "  The terminal buffers until the block ends.",
            Style::parse("dim green").unwrap(),
        ));
    });
    pause();

    // =========================================================================
    // 35. Traceback
    // =========================================================================
    console.rule(Some("Traceback"));

    // Error chain traceback
    let inner_err = io::Error::new(io::ErrorKind::ConnectionRefused, "connection refused");
    let outer_err = io::Error::other(format!("failed to connect to database: {}", inner_err));
    let tb = Traceback::from_error(&outer_err);
    console.print(&tb);

    // Custom traceback with source lines
    let frames = vec![
        Frame::new("src/database.rs", Some(87), "Database::connect")
            .with_source_line("    let conn = TcpStream::connect(&self.addr)?;"),
        Frame::new("src/main.rs", Some(28), "main")
            .with_source_line("    server.run(handle_request).await?;"),
    ];
    let tb = Traceback {
        title: "ConnectionError".to_string(),
        message: "failed to establish connection".to_string(),
        frames,
        ..Traceback::new()
    };
    console.print(&tb);
    pause();

    // =========================================================================
    // 36. Wrap Modes
    // =========================================================================
    console.rule(Some("Text Wrapping"));

    let long_text = "word1 word2 word3 word4 word5 word6 word7 word8 word9 word10 word11 word12 word13 word14 word15";
    let wrapped = Text::new(long_text, Style::null());
    let lines = wrapped.wrap(
        40,
        Some(JustifyMethod::Left),
        Some(OverflowMethod::Fold),
        8,
        false,
    );
    console.print(&Text::new(
        "  Wrapped at 40 cols:",
        Style::parse("bold").unwrap(),
    ));
    for line in lines.iter() {
        console.print(&Text::new(
            &format!("    {}", line.plain()),
            Style::parse("dim").unwrap(),
        ));
    }

    let tab_text = Text::new("col1\tcol2\tcol3\tcol4", Style::null());
    let tab_lines = tab_text.wrap(60, Some(JustifyMethod::Left), None, 8, false);
    console.print(&Text::new(
        "  Tab stops (tab_size=8):",
        Style::parse("bold").unwrap(),
    ));
    for line in tab_lines.iter() {
        console.print(&Text::new(
            &format!("    {}", line.plain()),
            Style::parse("dim").unwrap(),
        ));
    }
    pause();

    // =========================================================================
    // 37. #[derive(Table)] (feature-gated)
    // =========================================================================
    #[cfg(feature = "derive")]
    {
        console.rule(Some("Derive Table"));

        use gilt::Table as DeriveTable;

        #[derive(DeriveTable)]
        struct Planet {
            name: String,
            distance_au: f64,
            moons: u32,
        }

        let planets = vec![
            Planet {
                name: "Mercury".into(),
                distance_au: 0.39,
                moons: 0,
            },
            Planet {
                name: "Venus".into(),
                distance_au: 0.72,
                moons: 0,
            },
            Planet {
                name: "Earth".into(),
                distance_au: 1.00,
                moons: 1,
            },
            Planet {
                name: "Mars".into(),
                distance_au: 1.52,
                moons: 2,
            },
        ];

        let table = Planet::to_table(&planets);
        console.print(&table);
        pause();
    }

    // =========================================================================
    // 38. Spinners Gallery
    // =========================================================================
    console.rule(Some("Spinner Gallery"));

    let spinner_names = [
        "dots",
        "dots2",
        "dots3",
        "line",
        "pipe",
        "simpleDots",
        "star",
        "arc",
        "bouncingBar",
        "moon",
    ];

    for name in &spinner_names {
        if let Some(data) = SPINNERS.get(name) {
            let frames_preview: String = data
                .frames
                .iter()
                .take(10)
                .cloned()
                .collect::<Vec<_>>()
                .join(" ");
            let line = format!("  {:<20} {}", name, frames_preview);
            console.print(&Text::new(&line, Style::null()));
        }
    }
    pause();

    // =========================================================================
    // 39. ANSI Parsing
    // =========================================================================
    console.rule(Some("ANSI Parsing"));

    let ansi_input = "\x1b[1mBold\x1b[0m \x1b[31mRed\x1b[0m \x1b[32mGreen\x1b[0m \x1b[1;34mBold Blue\x1b[0m Normal";
    let mut decoder = AnsiDecoder::new();
    let decoded_lines = decoder.decode(ansi_input);
    console.print(&Text::new(
        "  Raw ANSI input parsed into styled Text:",
        Style::parse("bold").unwrap(),
    ));
    for line in &decoded_lines {
        console.print(line);
    }
    pause();

    // =========================================================================
    // 40. Color Palette
    // =========================================================================
    console.rule(Some("Standard Color Palette"));

    let color_names = [
        ("black", "color(0)"),
        ("red", "color(1)"),
        ("green", "color(2)"),
        ("yellow", "color(3)"),
        ("blue", "color(4)"),
        ("magenta", "color(5)"),
        ("cyan", "color(6)"),
        ("white", "color(7)"),
        ("bright_black", "color(8)"),
        ("bright_red", "color(9)"),
        ("bright_green", "color(10)"),
        ("bright_yellow", "color(11)"),
        ("bright_blue", "color(12)"),
        ("bright_magenta", "color(13)"),
        ("bright_cyan", "color(14)"),
        ("bright_white", "color(15)"),
    ];

    for (name, color_spec) in &color_names {
        let combined = format!("  \u{2588}\u{2588}  {name}");
        let combined_style = Style::parse(color_spec).unwrap_or_else(|_| Style::null());
        console.print(&Text::styled(&combined, combined_style));
    }
    pause();

    // =========================================================================
    // 41. Sparkline
    // =========================================================================
    console.rule(Some("Sparkline"));

    // Simulated CPU usage over time (%)
    let cpu_data: Vec<f64> = vec![
        12.0, 15.0, 22.0, 35.0, 42.0, 55.0, 68.0, 72.0, 80.0, 95.0, 88.0, 70.0, 60.0, 45.0, 38.0,
        30.0, 25.0, 18.0, 20.0, 28.0, 35.0, 50.0, 62.0, 75.0, 85.0, 78.0, 65.0, 55.0, 40.0, 32.0,
        22.0, 15.0, 10.0, 18.0, 30.0, 45.0, 58.0, 70.0, 82.0, 90.0,
    ];
    let spark = Sparkline::new(&cpu_data)
        .with_width(70)
        .with_style(Style::parse("bold green").unwrap());
    console.print(&Text::new(
        "  CPU usage over time:",
        Style::parse("bold").unwrap(),
    ));
    console.print(&spark);

    // Memory pressure — shorter data, no resample
    let mem_data: Vec<f64> = vec![
        30.0, 32.0, 35.0, 40.0, 55.0, 70.0, 85.0, 92.0, 95.0, 88.0, 75.0, 60.0,
    ];
    let mem_spark = Sparkline::new(&mem_data).with_style(Style::parse("bold yellow").unwrap());
    console.print(&Text::new(
        "  Memory pressure:",
        Style::parse("bold").unwrap(),
    ));
    console.print(&mem_spark);
    pause();

    // =========================================================================
    // 42. Canvas (Braille Dot-Matrix Graphics)
    // =========================================================================
    console.rule(Some("Canvas (Braille Dot-Matrix)"));

    // 30 cols x 8 rows => 60x32 pixel grid
    let mut canvas = Canvas::new(30, 8).with_style(Style::parse("cyan").unwrap());

    // Draw a rectangle border
    canvas.rect(0, 0, 59, 31);

    // Draw diagonal lines forming an X
    canvas.line(2, 2, 56, 28);
    canvas.line(56, 2, 2, 28);

    // Draw a circle in the center
    canvas.circle(30, 16, 12);

    console.print(&canvas);
    pause();

    // =========================================================================
    // 43. Diff (Text Diff)
    // =========================================================================
    console.rule(Some("Text Diff"));

    let old_code = r#"fn greet(name: &str) {
    println!("Hello, {}!", name);
}

fn main() {
    greet("world");
}"#;

    let new_code = r#"fn greet(name: &str, excited: bool) {
    if excited {
        println!("Hello, {}!!", name);
    } else {
        println!("Hello, {}.", name);
    }
}

fn main() {
    greet("world", true);
}"#;

    let diff = Diff::new(old_code, new_code)
        .with_labels("a/greet.rs", "b/greet.rs")
        .with_context(2);
    console.print(&diff);
    pause();

    // =========================================================================
    // 44. Figlet (ASCII Art)
    // =========================================================================
    console.rule(Some("Figlet (ASCII Art)"));

    let banner = Figlet::new("GILT").with_style(Style::parse("bold bright_magenta").unwrap());
    console.print(&banner);
    console.line(1);

    let sub_banner = Figlet::new("v0.5")
        .with_style(Style::parse("dim cyan").unwrap())
        .with_width(90);
    console.print(&sub_banner);
    pause();

    // =========================================================================
    // 45. CSV Table
    // =========================================================================
    console.rule(Some("CSV Table"));

    let csv_data = "\
City,Country,Population,Area (km2)
Tokyo,Japan,13960000,2194
Delhi,India,11030000,1484
Shanghai,China,24870000,6341
Sao Paulo,Brazil,12330000,1521
Mumbai,India,12440000,603";

    let csv = CsvTable::from_csv_str(csv_data)
        .unwrap()
        .with_title("World Cities")
        .with_header_style(Style::parse("bold magenta").unwrap());
    console.print(&csv);
    pause();

    // =========================================================================
    // 46. Iterator .progress() (non-animated summary)
    // =========================================================================
    console.rule(Some("Iterator Progress (.progress())"));

    console.print(&Text::new(
        "  The ProgressIteratorExt trait adds .progress() to any iterator:",
        Style::null(),
    ));
    console.print(&Text::new(
        "    for item in (0..100).progress(\"Processing\") { ... }",
        Style::parse("bold green").unwrap(),
    ));
    console.print(&Text::new(
        "    for item in data.iter().progress_with_total(\"Loading\", 500.0) { ... }",
        Style::parse("bold green").unwrap(),
    ));
    console.print(&Text::new(
        "  Total is inferred from size_hint() or set explicitly.",
        Style::parse("dim").unwrap(),
    ));
    pause();

    // =========================================================================
    // 47. Derive Macros (feature-gated)
    // =========================================================================
    #[cfg(feature = "derive")]
    {
        console.rule(Some("Derive Macros"));

        console.print(&Text::new(
            "  gilt-derive provides: #[derive(Table)], #[derive(Panel)],",
            Style::null(),
        ));
        console.print(&Text::new(
            "  #[derive(Tree)], #[derive(Columns)], #[derive(Rule)],",
            Style::null(),
        ));
        console.print(&Text::new(
            "  #[derive(Inspect)], and #[derive(Renderable)]",
            Style::null(),
        ));
        console.line(1);

        // Demonstrate #[derive(Panel)]
        use gilt::Panel as PanelDerive;

        #[derive(PanelDerive)]
        #[panel(
            title = "Server Status",
            box_style = "ROUNDED",
            border_style = "bright_green",
            title_style = "bold white"
        )]
        struct ServerStatus {
            #[field(label = "Host", style = "bold cyan")]
            host: String,
            #[field(label = "Port")]
            port: u16,
            #[field(label = "Status", style = "bold green")]
            status: String,
            #[field(label = "Uptime")]
            uptime: String,
        }

        let server = ServerStatus {
            host: "prod-web-01.example.com".into(),
            port: 443,
            status: "HEALTHY".into(),
            uptime: "47 days, 13:22:09".into(),
        };
        console.print(&server.to_panel());

        // Demonstrate #[derive(Tree)]
        use gilt::Tree as TreeDerive;

        #[derive(TreeDerive)]
        #[tree(style = "bold blue", guide_style = "dim")]
        struct MenuItem {
            #[tree(label)]
            name: String,
            #[tree(children)]
            items: Vec<MenuItem>,
        }

        let menu = MenuItem {
            name: "Application".into(),
            items: vec![
                MenuItem {
                    name: "File".into(),
                    items: vec![
                        MenuItem {
                            name: "New".into(),
                            items: vec![],
                        },
                        MenuItem {
                            name: "Open".into(),
                            items: vec![],
                        },
                    ],
                },
                MenuItem {
                    name: "Edit".into(),
                    items: vec![
                        MenuItem {
                            name: "Undo".into(),
                            items: vec![],
                        },
                        MenuItem {
                            name: "Redo".into(),
                            items: vec![],
                        },
                    ],
                },
            ],
        };
        console.print(&menu.to_tree());

        // Demonstrate #[derive(Columns)]
        use gilt::DeriveColumns;

        #[derive(DeriveColumns)]
        #[columns(equal, padding = 1)]
        struct Framework {
            #[field(label = "Name", style = "bold")]
            name: String,
            #[field(label = "Language", style = "cyan")]
            language: String,
            #[field(label = "Stars")]
            stars: String,
        }

        let frameworks = vec![
            Framework {
                name: "Axum".into(),
                language: "Rust".into(),
                stars: "19k".into(),
            },
            Framework {
                name: "Actix".into(),
                language: "Rust".into(),
                stars: "22k".into(),
            },
            Framework {
                name: "Rocket".into(),
                language: "Rust".into(),
                stars: "24k".into(),
            },
        ];
        console.print(&Framework::to_columns(&frameworks));
        pause();
    }

    // =========================================================================
    // Farewell
    // =========================================================================
    console.line(1);
    let farewell = Gradient::rainbow("  Thank you for exploring gilt!  ")
        .with_style(Style::parse("bold").unwrap());
    console.print(&farewell);
    console.rule(None);
}
