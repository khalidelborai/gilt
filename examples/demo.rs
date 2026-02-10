//! Gilt CLI Demo — Comprehensive showcase of all gilt features.
//!
//! Run with: `cargo run --example demo`
//!
//! This demo cycles through all major gilt features, demonstrating:
//! - Text styling (colors, attributes, markup)
//! - Widgets (panels, tables, trees, rules, columns)
//! - New features (badges, breadcrumbs, accordions, toasts)
//! - Advanced features (gradients, markdown, syntax highlighting, live display)
//! - Performance (cold/warm cache timing)
//! - Export (HTML and SVG)
//!
//! Similar to Python's `python -m rich`

use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

use gilt::accordion::Accordion;
use gilt::badge::Badge;
use gilt::box_chars::{DOUBLE, HEAVY, ROUNDED, SIMPLE};
use gilt::breadcrumbs::Breadcrumbs;
use gilt::color::Color;
use gilt::emoji_replace::emoji_replace;
use gilt::figlet::Figlet;
use gilt::highlighter::{Highlighter, ISODateHighlighter, URLHighlighter, UUIDHighlighter};
use gilt::prelude::*;
use gilt::style::Style;
use gilt::table::ColumnOptions;
use gilt::text::Text;
use gilt::toast::Toast;

// ---------------------------------------------------------------------------
// Demo Configuration
// ---------------------------------------------------------------------------

/// Pause between sections (set to 0 to run without pausing)
static PAUSE_MS: u64 = 2000;

/// Width of the console for demos
const CONSOLE_WIDTH: usize = 100;

// ---------------------------------------------------------------------------
// Main Entry Point
// ---------------------------------------------------------------------------

fn main() {
    let mut console = Console::builder()
        .width(CONSOLE_WIDTH)
        .force_terminal(true)
        .no_color(false)
        .build();

    // Introduction
    show_intro(&mut console);
    pause_or_wait();

    // Text Styling
    show_text_styling(&mut console);
    pause_or_wait();

    // Widgets Showcase
    show_widgets(&mut console);
    pause_or_wait();

    // New Features (v0.9.0)
    show_new_features(&mut console);
    pause_or_wait();

    // Advanced Features
    show_advanced_features(&mut console);
    pause_or_wait();

    // Performance Test Card
    show_performance(&mut console);
    pause_or_wait();

    // Export Demo
    show_export(&mut console);
    pause_or_wait();

    // Farewell
    show_farewell(&mut console);
}

// ---------------------------------------------------------------------------
// Pause / Wait for Input
// ---------------------------------------------------------------------------

fn pause_or_wait() {
    if PAUSE_MS > 0 {
        thread::sleep(Duration::from_millis(PAUSE_MS));
    } else {
        pause_for_enter();
    }
}

fn pause_for_enter() {
    print!("\n[Press Enter to continue...]");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
}

// Small pause for visual pacing
fn _pause() {
    thread::sleep(Duration::from_millis(300));
}

// ---------------------------------------------------------------------------
// Section 1: Introduction
// ---------------------------------------------------------------------------

fn show_intro(console: &mut Console) {
    console.clear();
    console.line(2);

    // Banner with figlet-style text
    let banner = Figlet::new("GILT");
    console.print(&banner);

    console.line(1);

    // Rainbow subtitle
    let subtitle = Gradient::rainbow("  Rich Terminal Formatting for Rust  ");
    console.print(&subtitle);

    console.line(2);

    // Version info panel
    let version_text = Text::from_markup(
        "[bold]Version:[/bold]     0.8.0\n\
         [bold]Repository:[/bold]  https://github.com/khalidelborai/gilt\n\
         [bold]License:[/bold]     MIT\n\
         [bold]Features:[/bold]    40+ widgets, syntax highlighting, markdown, gradients",
    )
    .unwrap();

    let info_panel = Panel::new(version_text)
        .with_title(Text::styled("About", Style::parse("bold cyan").unwrap()))
        .with_box_chars(&ROUNDED)
        .with_border_style(Style::parse("cyan").unwrap());
    console.print(&info_panel);

    console.line(1);

    // Quick description
    console.print_text(
        "[dim]A Rust port of Python's rich library — beautiful terminal output with styles, \
         tables, trees, syntax highlighting, progress bars, and more.[/dim]",
    );

    console.line(2);

    // Section navigation hint
    let nav = Text::from_markup(
        "[bold green]▶[/bold green] This demo will showcase: Text Styling • Widgets • New Features • \
         Advanced Features • Performance • Export",
    )
    .unwrap();
    console.print(&nav);

    console.line(1);
}

// ---------------------------------------------------------------------------
// Section 2: Text Styling
// ---------------------------------------------------------------------------

fn show_text_styling(console: &mut Console) {
    console.clear();
    console.print(&Rule::with_title("Text Styling"));
    console.line(1);

    // Text Attributes
    console.print_text("[bold magenta]Text Attributes[/bold magenta]");
    console.print_text("  [bold]Bold[/bold] | [dim]Dim[/dim] | [italic]Italic[/italic] | [underline]Underline[/underline] | \
                      [strike]Strikethrough[/strike] | [reverse]Reverse[/reverse]");
    console.line(1);

    // Standard Colors
    console.print_text("[bold magenta]Standard Colors[/bold magenta]");
    console.print(&"  Black".black());
    console.print(&"  Red".red());
    console.print(&"  Green".green());
    console.print(&"  Yellow".yellow());
    console.print(&"  Blue".blue());
    console.print(&"  Magenta".magenta());
    console.print(&"  Cyan".cyan());
    console.print(&"  White".white());
    console.line(1);

    // Bright Colors
    console.print_text("[bold magenta]Bright Colors[/bold magenta]");
    console.print(&"  Bright Red".bright_red());
    console.print(&"  Bright Green".bright_green());
    console.print(&"  Bright Blue".bright_blue());
    console.print(&"  Bright Yellow".bright_yellow());
    console.print(&"  Bright Magenta".bright_magenta());
    console.print(&"  Bright Cyan".bright_cyan());
    console.print(&"  Bright White".bright_white());
    console.line(1);

    // Background Colors
    console.print_text("[bold magenta]Background Colors[/bold magenta]");
    console.print(&"  Red on Black".red().on_black());
    console.print(&"  White on Blue".white().on_blue());
    console.print(&"  Black on Yellow".black().on_yellow());
    console.print(&"  Black on Cyan".black().on_cyan());
    console.print(&"  White on Magenta".white().on_magenta());
    console.line(1);

    // TrueColor
    console.print_text("[bold magenta]TrueColor (RGB)[/bold magenta]");
    console.print(&"  Orange (#ff6600)".fg("#ff6600"));
    console.print(&"  Sky Blue (#00ccff)".fg("#00ccff"));
    console.print(
        &"  Lime (#00ff88) on Dark"
            .bold()
            .fg("#00ff88")
            .bg("#222222"),
    );
    console.line(1);

    // Combinations
    console.print_text("[bold magenta]Combinations[/bold magenta]");
    console.print(&"  Bold + Italic + Underline".bold().italic().underline());
    console.print(&"  Dim + Italic".dim().italic());
    console.print(&"  Bold Red on Yellow".bold().red().on_yellow());
    console.line(1);

    // Markup Examples
    console.print_text("[bold magenta]BBCode-style Markup[/bold magenta]");
    console.print_text("  [bold red]Error:[/bold red] Something went wrong");
    console.print_text("  [green]✓[/green] Success! [dim](took 42ms)[/dim]");
    console.print_text(
        "  [bold cyan]Info:[/bold cyan] Use [code][tag][/code] for [italic]inline styling[/italic]",
    );
    console.line(1);
}

// ---------------------------------------------------------------------------
// Section 3: Widgets Showcase
// ---------------------------------------------------------------------------

fn show_widgets(console: &mut Console) {
    console.clear();
    console.print(&Rule::with_title("Widgets Showcase"));
    console.line(1);

    // Panel
    console.print_text("[bold blue]Panel[/bold blue] — Bordered containers with titles");
    let panel_content = Text::from_markup(
        "This is a [bold]Panel[/bold] widget. It wraps content in a \
         customizable border with optional titles and subtitles.",
    )
    .unwrap();
    let panel = Panel::new(panel_content)
        .with_title(Text::styled("My Panel", Style::parse("bold cyan").unwrap()))
        .with_subtitle(Text::styled("v0.8.0", Style::parse("dim").unwrap()))
        .with_box_chars(&ROUNDED)
        .with_border_style(Style::parse("blue").unwrap());
    console.print(&panel);
    console.line(1);

    // Different border styles
    console.print_text("[bold blue]Border Styles[/bold blue]");
    let simple_panel = Panel::new(Text::new("SIMPLE borders", Style::null()))
        .with_box_chars(&SIMPLE)
        .with_border_style(Style::parse("green").unwrap());
    console.print(&simple_panel);

    let heavy_panel = Panel::new(Text::new("HEAVY borders", Style::null()))
        .with_box_chars(&HEAVY)
        .with_border_style(Style::parse("red").unwrap());
    console.print(&heavy_panel);

    let double_panel = Panel::new(Text::new("DOUBLE borders", Style::null()))
        .with_box_chars(&DOUBLE)
        .with_border_style(Style::parse("magenta").unwrap());
    console.print(&double_panel);
    console.line(1);

    // Table
    console.print_text("[bold blue]Table[/bold blue] — Structured data display");
    let mut table = Table::new(&["Language", "Paradigm", "Year", "Typing"]);
    table.title = Some("Programming Languages".to_string());
    table.title_style = "bold cyan".to_string();
    table.header_style = "bold".to_string();
    table.border_style = "green".to_string();
    table.row_styles = vec!["".to_string(), "dim".to_string()];

    table.add_row(&["Rust", "Systems / Multi", "2010", "Static, Strong"]);
    table.add_row(&["Python", "Multi-paradigm", "1991", "Dynamic, Strong"]);
    table.add_row(&["Haskell", "Functional", "1990", "Static, Strong"]);
    table.add_row(&["Go", "Concurrent", "2009", "Static, Strong"]);
    table.add_row(&["TypeScript", "Multi-paradigm", "2012", "Static, Gradual"]);
    console.print(&table);
    console.line(1);

    // Tree
    console.print_text("[bold blue]Tree[/bold blue] — Hierarchical data display");
    let bold_blue = Style::parse("bold blue").unwrap();
    let green = Style::parse("green").unwrap();
    let default = Style::null();

    let mut tree = Tree::new(Text::new("my_project/", bold_blue.clone()))
        .with_guide_style(Style::parse("dim green").unwrap());

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
    console.line(1);

    // Progress Bar (static display)
    console.print_text("[bold blue]Progress Bar[/bold blue] — Task completion indicators");
    let levels: &[(f64, &str, &str)] = &[
        (0.0, "dim", "  0% "),
        (25.0, "yellow", " 25% "),
        (50.0, "blue", " 50% "),
        (75.0, "cyan", " 75% "),
        (100.0, "bold green", "100% "),
    ];

    for (completed, style, label) in levels {
        let label_text = Text::from_markup(&format!("[{style}]{label}[/{style}]")).unwrap();
        console.print(&label_text);
        let bar = gilt::progress_bar::ProgressBar::new()
            .with_total(Some(100.0))
            .with_completed(*completed)
            .with_width(Some(40));
        console.print(&bar);
    }
    console.line(1);

    // Spinner
    console.print_text("[bold blue]Spinner[/bold blue] — Loading indicators");
    let spinners = ["dots", "line", "circle", "bouncing", "arrow"];
    for name in spinners {
        let spinner_text = Text::from_markup(&format!(
            "  [cyan]{name:12}[/cyan] [dim]⏵[/dim] [italic]animated {name} spinner[/italic]",
            name = name,
        ))
        .unwrap();
        console.print(&spinner_text);
    }
    console.line(1);

    // Rule
    console.print_text("[bold blue]Rule[/bold blue] — Horizontal dividers");
    console.print(&Rule::new());
    console.print(&Rule::with_title("Centered Title"));
    console.print(
        &Rule::with_title("Heavy Rule")
            .with_characters("━")
            .with_style(Style::parse("red").unwrap()),
    );
    console.print(
        &Rule::with_title("Double Line")
            .with_characters("=")
            .with_style(Style::parse("green").unwrap()),
    );
    console.line(1);

    // Columns
    console.print_text("[bold blue]Columns[/bold blue] — Multi-column layouts");
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
    ];

    let mut cols = Columns::new().with_equal(true);
    for item in &items {
        cols.add_renderable(item);
    }
    console.print(&cols);
    console.line(1);
}

// ---------------------------------------------------------------------------
// Section 4: New Features (v0.9.0)
// ---------------------------------------------------------------------------

fn show_new_features(console: &mut Console) {
    console.clear();
    console.print(&Rule::with_title("New Features in v0.9.0"));
    console.line(1);

    // Badges
    console.print_text("[bold green]Badges[/bold green] — Status indicators");

    let badge_row1 = Panel::new(Text::new("", Style::null()))
        .with_title(Text::from_markup("[bold]Standard[/bold]").unwrap());
    console.print(&badge_row1);

    console.print(&Badge::success("Build Passed"));
    console.print(&Badge::error("Tests Failed"));
    console.print(&Badge::warning("Deprecated"));
    console.print(&Badge::info("New Feature"));
    console.line(1);

    // Rounded badges
    console.print_text("[dim]Rounded variants:[/dim]");
    console.print(&Badge::success("Published").rounded(true));
    console.print(&Badge::info("Beta").rounded(true));
    console.line(1);

    // Breadcrumbs
    console.print_text("[bold green]Breadcrumbs[/bold green] — Navigation paths");

    let default_crumbs = Breadcrumbs::new(vec!["Home".into(), "Settings".into(), "Profile".into()]);
    console.print(&default_crumbs);

    let file_path = Breadcrumbs::slash(vec![
        "home".into(),
        "user".into(),
        "projects".into(),
        "myapp".into(),
        "src".into(),
        "main.rs".into(),
    ]);
    console.print(&file_path);

    let workflow = Breadcrumbs::arrow(vec![
        "Input".into(),
        "Process".into(),
        "Validate".into(),
        "Output".into(),
    ]);
    console.print(&workflow);
    console.line(1);

    // Accordion
    console.print_text("[bold green]Accordion[/bold green] — Collapsible content panels");

    let accordion_expanded = Accordion::new(
        "Getting Started (expanded)",
        Text::new(
            "Welcome to the accordion widget! This content is visible because \
             the accordion is expanded by default.",
            Style::null(),
        ),
    )
    .title_style(Style::parse("bold cyan").unwrap());
    console.print(&accordion_expanded);
    console.line(1);

    let accordion_collapsed = Accordion::new(
        "Advanced Options (collapsed)",
        Text::new(
            "You shouldn't see this content because the accordion is collapsed.",
            Style::null(),
        ),
    )
    .collapsed(true)
    .title_style(Style::parse("dim").unwrap());
    console.print(&accordion_collapsed);
    console.line(1);

    // Toast Notifications
    console.print_text("[bold green]Toast Notifications[/bold green] — Brief status messages");

    Toast::success("Operation completed successfully").show(console);
    Toast::info("Processing...").show(console);
    Toast::warning("Disk space low").show(console);
    Toast::error("Failed to save file").show(console);
    console.line(1);

    // Theme Showcase
    console.print_text("[bold green]Built-in Themes[/bold green] — Pre-defined color palettes");
    console.print_text(
        "  Available themes: [cyan]monokai[/cyan], [cyan]solarized_dark[/cyan], \
                      [cyan]solarized_light[/cyan], [cyan]dracula[/cyan], [cyan]nord[/cyan], \
                      [cyan]one_dark[/cyan], [cyan]github_dark[/cyan], [cyan]github_light[/cyan]",
    );
    console.line(1);
}

// ---------------------------------------------------------------------------
// Section 5: Advanced Features
// ---------------------------------------------------------------------------

fn show_advanced_features(console: &mut Console) {
    console.clear();
    console.print(&Rule::with_title("Advanced Features"));
    console.line(1);

    // Gradient Text
    console.print_text("[bold yellow]Gradient Text[/bold yellow] — Smooth color transitions");
    let rainbow =
        Gradient::rainbow("ROYGBIV: Red Orange Yellow Green Blue Indigo Violet — full spectrum!");
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
    console.line(1);

    // Markdown (feature-gated)
    #[cfg(feature = "markdown")]
    {
        console.print_text("[bold yellow]Markdown Rendering[/bold yellow] — Terminal markdown");
        let md_source = r#"# Markdown Support

Gilt renders **bold**, *italic*, and `inline code`.

## Features

- Headers at multiple levels
- Bullet and numbered lists
- Code blocks with syntax highlighting
- Block quotes

> The quick brown fox jumps over the lazy dog.
"#;
        let md = gilt::markdown::Markdown::new(md_source);
        console.print(&md);
        console.line(1);
    }

    // Syntax Highlighting (feature-gated)
    #[cfg(feature = "syntax")]
    {
        console.print_text("[bold yellow]Syntax Highlighting[/bold yellow] — Code with style");
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
        console.line(1);
    }

    // JSON Pretty-Printing (feature-gated)
    #[cfg(feature = "json")]
    {
        console.print_text("[bold yellow]JSON Pretty-Printing[/bold yellow] — Structured data");
        let json_str = r#"{"name": "gilt", "version": "0.8.0", "features": ["syntax", "markdown", "json"], "metadata": {"stars": 42, "active": true}}"#;
        let json_widget =
            gilt::json::Json::new(json_str, gilt::json::JsonOptions::default()).unwrap();
        console.print(&json_widget);
        console.line(1);
    }

    // Live Display
    console.print_text("[bold yellow]Live Display Demo[/bold yellow] — Updating content in-place");
    console.print_text("[dim]Watch the counter update below...[/dim]");

    {
        let live_console = Console::builder()
            .width(CONSOLE_WIDTH)
            .force_terminal(true)
            .build();

        let mut live =
            Live::new(Text::new("Starting...", Style::null())).with_console(live_console);
        live.start();

        for i in 0..=5 {
            let text = Text::new(
                &format!("Processing... {}/5", i),
                Style::parse("cyan").unwrap(),
            );
            live.update_renderable(text, true);
            thread::sleep(Duration::from_millis(300));
        }

        let done = Text::new("✓ Complete!", Style::parse("bold green").unwrap());
        live.update_renderable(done, true);
        thread::sleep(Duration::from_millis(500));
        live.stop();
    }
    console.line(1);

    // Status Spinner
    console.print_text("[bold yellow]Status Spinner[/bold yellow] — Animated task status");
    console.print_text("[dim]Simulating async tasks...[/dim]");

    {
        let status_console = Console::builder().force_terminal(true).build();

        let messages = [
            "Connecting to server...",
            "Authenticating...",
            "Fetching data...",
            "Processing results...",
            "Done!",
        ];

        let mut status = Status::new(messages[0]).with_console(status_console);
        status.start();

        for msg in &messages[1..] {
            thread::sleep(Duration::from_millis(600));
            status.update().status(msg).apply().unwrap();
        }

        thread::sleep(Duration::from_millis(300));
        status.stop();
    }
    console.line(1);

    // Highlighters
    console.print_text("[bold yellow]Highlighters[/bold yellow] — Automatic pattern detection");

    let url_hl = URLHighlighter::new();
    let text = url_hl.apply("Visit https://example.com or http://localhost:8080/api");
    console.print_text("[dim]URLs:[/dim]");
    console.print(&text);

    let uuid_hl = UUIDHighlighter::new();
    let text = uuid_hl.apply("Request ID: 550e8400-e29b-41d4-a716-446655440000");
    console.print_text("[dim]UUIDs:[/dim]");
    console.print(&text);

    let iso_hl = ISODateHighlighter::new();
    let text = iso_hl.apply("Created: 2024-01-15T10:30:00Z  Updated: 2024-06-20");
    console.print_text("[dim]ISO Dates:[/dim]");
    console.print(&text);
    console.line(1);

    // Emoji
    console.print_text("[bold yellow]Emoji Support[/bold yellow] — Shortcode replacement");
    let emoji_text = emoji_replace(
        ":rocket: Launching :sparkles: :thumbs_up: \
         :heart: :star: :fire: :white_check_mark: :warning:",
        None,
    );
    console.print(&Text::new(&emoji_text, Style::null()));
    console.line(1);
}

// ---------------------------------------------------------------------------
// Section 6: Performance (Test Card)
// ---------------------------------------------------------------------------

fn show_performance(console: &mut Console) {
    console.clear();
    console.print(&Rule::with_title("Performance — Test Card"));
    console.line(1);

    // Build capture console for timing
    let (term_width, _) = Console::detect_terminal_size();
    let width = (term_width as usize).min(CONSOLE_WIDTH);

    let mut capture_console = Console::builder()
        .width(width)
        .force_terminal(true)
        .color_system("truecolor")
        .build();

    // Build the test card
    let test_card = make_test_card(&mut capture_console);

    // Cold render (first render, populating caches)
    let cold_start = Instant::now();
    capture_console.begin_capture();
    capture_console.print(&test_card);
    let _ = capture_console.end_capture();
    let cold_ms = cold_start.elapsed().as_secs_f64() * 1000.0;

    // Warm render (caches populated)
    let warm_start = Instant::now();
    capture_console.begin_capture();
    capture_console.print(&test_card);
    let _ = capture_console.end_capture();
    let warm_ms = warm_start.elapsed().as_secs_f64() * 1000.0;

    // Render to screen
    console.print(&test_card);

    // Timing results
    console.line(1);
    console.print_text(&format!(
        "[dim]Rendered in [bold]{cold_ms:.2}ms[/bold] (cold) / [bold]{warm_ms:.2}ms[/bold] (warm)[/dim]"
    ));
    console.line(1);
}

/// Make a comprehensive test card like Python Rich's test card
fn make_test_card(console: &mut Console) -> Table {
    let mut table = Table::grid(&[]);
    table.title = Some("Gilt Test Card".to_string());
    table.padding = (1, 1, 0, 1);
    table.pad_edge = true;
    table.set_expand(true);
    table.add_column(
        "Feature",
        "",
        ColumnOptions {
            no_wrap: true,
            justify: Some(JustifyMethod::Center),
            style: Some("bold red".to_string()),
            ..Default::default()
        },
    );
    table.add_column("Demonstration", "", Default::default());

    // Row 1: Colors
    {
        let color_demo = Text::from_markup(
            "[bold green]✓[/bold green] 4-bit color\n\
             [bold blue]✓[/bold blue] 8-bit color\n\
             [bold magenta]✓[/bold magenta] Truecolor (16.7 million)\n\
             [bold yellow]✓[/bold yellow] Automatic color conversion",
        )
        .unwrap();
        table.add_row_text(&[
            Text::from_markup("[bold red]Colors[/]").unwrap(),
            color_demo,
        ]);
    }

    // Row 2: Styles
    table.add_row(&[
        "Styles",
        "All ANSI styles: [bold]bold[/], [dim]dim[/], [italic]italic[/], \
         [underline]underline[/], [strike]strikethrough[/], [reverse]reverse[/]",
    ]);

    // Row 3: Text Justification
    {
        let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.";

        let mut t_left = Text::new(lorem, Style::parse("green").unwrap());
        t_left.justify = Some(JustifyMethod::Left);
        let mut t_center = Text::new(lorem, Style::parse("yellow").unwrap());
        t_center.justify = Some(JustifyMethod::Center);
        let mut t_right = Text::new(lorem, Style::parse("blue").unwrap());
        t_right.justify = Some(JustifyMethod::Right);

        let mut justify_table = Table::grid(&[]);
        justify_table.padding = (0, 1, 0, 0);
        justify_table.collapse_padding = true;
        justify_table.set_expand(true);
        for _ in 0..3 {
            justify_table.add_column(
                "",
                "",
                ColumnOptions {
                    ratio: Some(1),
                    ..Default::default()
                },
            );
        }
        justify_table.add_row_text(&[t_left, t_center, t_right]);

        let header =
            Text::from_markup("Justify: [green]left[/], [yellow]center[/], [blue]right[/]")
                .unwrap();

        let mut combined = header;
        combined.append_str("\n", None);
        let rendered = render_to_text(console, &justify_table);
        combined.append_text(&rendered);

        table.add_row_text(&[Text::from_markup("[bold red]Text[/]").unwrap(), combined]);
    }

    // Row 4: CJK Support
    {
        let cjk_text = emoji_replace(
            ":flag_for_china: 该库支持中文，日文和韩文文本！\n\
             :flag_for_japan:  このライブラリは多言語をサポートしています\n\
             :flag_for_south_korea:  이 라이브러리는 한국어 텍스트를 지원합니다",
            None,
        );
        table.add_row_text(&[
            Text::new("CJK\nSupport", Style::null()),
            Text::new(&cjk_text, Style::null()),
        ]);
    }

    // Row 5: Markup
    {
        let markup_text = emoji_replace(
            "[bold magenta]gilt[/] supports [i]bbcode[/i]-like [b]markup[/b] \
             for [yellow]color[/], [underline]style[/], and emoji! \
             :thumbs_up: :apple: :star: :fire:",
            None,
        );
        table.add_row(&["Markup", &markup_text]);
    }

    // Row 6: Tables
    {
        let mut movie_table = Table::new(&[]);
        movie_table.show_edge = false;
        movie_table.show_header = true;
        movie_table.set_expand(false);
        movie_table.row_styles = vec!["".to_string(), "dim".to_string()];
        movie_table.box_chars = Some(&SIMPLE);

        movie_table.add_column(
            "[green]Date",
            "",
            ColumnOptions {
                style: Some("green".to_string()),
                no_wrap: true,
                ..Default::default()
            },
        );
        movie_table.add_column(
            "[blue]Title",
            "",
            ColumnOptions {
                style: Some("blue".to_string()),
                ..Default::default()
            },
        );
        movie_table.add_column(
            "[cyan]Budget",
            "",
            ColumnOptions {
                style: Some("cyan".to_string()),
                justify: Some(JustifyMethod::Right),
                no_wrap: true,
                ..Default::default()
            },
        );

        movie_table.add_row(&[
            "Dec 20, 2019",
            "Star Wars: The Rise of Skywalker",
            "$275,000,000",
        ]);
        movie_table.add_row(&[
            "Dec 15, 2017",
            "[b]Star Wars:[/b] The Last Jedi",
            "$262,000,000",
        ]);

        let demo = render_to_text(console, &movie_table);
        table.add_row_text(&[Text::from_markup("[bold red]Tables[/]").unwrap(), demo]);
    }

    // Row 7: Syntax & Pretty
    #[cfg(feature = "syntax")]
    {
        use gilt::pretty::Pretty;
        use gilt::syntax::Syntax;

        let code = "fn hello() -> &'static str { \"Hello, World!\" }";
        let syntax = Syntax::new(code, "rs").with_line_numbers(true);

        let data = serde_json::json!({
            "foo": [3.142, ["Alice", "Bob"]],
            "active": true
        });
        let pretty = Pretty::from_json(&data);

        let demo = comparison(console, &syntax, &pretty);
        table.add_row_text(&[
            Text::from_markup("[bold red]Syntax &\nPretty[/]").unwrap(),
            demo,
        ]);
    }

    // Row 8: Markdown
    #[cfg(feature = "markdown")]
    {
        use gilt::markdown::Markdown;

        let markdown_source = "# Hello\n\nSupports **bold**, *italic*, and `code`.\n\n- Lists\n- Quotes\n\n> Block quote";
        let left = Text::new(markdown_source, Style::parse("cyan").unwrap());
        let md = Markdown::new(markdown_source);

        let demo = comparison(console, &left, &md);
        table.add_row_text(&[Text::from_markup("[bold red]Markdown[/]").unwrap(), demo]);
    }

    // Row 9: Tree
    {
        let bold_blue = Style::parse("bold blue").unwrap();
        let default = Style::null();

        let mut tree = Tree::new(Text::new("gilt/", bold_blue.clone()))
            .with_guide_style(Style::parse("dim green").unwrap());

        let src = tree.add(Text::new("src/", bold_blue.clone()));
        src.add(Text::new("console.rs", default.clone()));
        src.add(Text::new("table.rs", default.clone()));
        let widgets = src.add(Text::new("widgets/", bold_blue.clone()));
        widgets.add(Text::new("panel.rs", default.clone()));

        let demo = render_to_text(console, &tree);
        table.add_row_text(&[Text::from_markup("[bold red]Tree[/]").unwrap(), demo]);
    }

    // Row 10: Progress
    {
        let mut bar_text = Text::empty();
        let levels: &[(f64, &str)] = &[
            (0.0, "  0%"),
            (30.0, " 30%"),
            (70.0, " 70%"),
            (100.0, "100%"),
        ];

        for (completed, label) in levels {
            let label_t = Text::from_markup(&format!("[dim]{label}[/dim] ")).unwrap();
            bar_text.append_text(&label_t);

            let bar = gilt::progress_bar::ProgressBar::new()
                .with_total(Some(100.0))
                .with_completed(*completed)
                .with_width(Some(25));
            let bar_rendered = render_to_text(console, &bar);
            bar_text.append_text(&bar_rendered);
            bar_text.append_str("\n", None);
        }

        table.add_row_text(&[
            Text::from_markup("[bold red]Progress[/]").unwrap(),
            bar_text,
        ]);
    }

    // Row 11: Gradient
    {
        let rainbow = Gradient::rainbow("The quick brown fox jumps over the lazy dog!");
        let demo = render_to_text(console, &rainbow);
        table.add_row_text(&[Text::from_markup("[bold red]Gradient[/]").unwrap(), demo]);
    }

    // Row 12: More features
    table.add_row(&[
        "+more!",
        "Badges, breadcrumbs, accordions, toasts, columns, layouts, \
         live display, spinners, accessibility, and more...",
    ]);

    table
}

// Helper: render a widget to Text
fn render_to_text(console: &mut Console, widget: &dyn Renderable) -> Text {
    console.begin_capture();
    console.print(widget);
    let captured = console.end_capture();
    let trimmed = captured.trim_end_matches('\n');
    Text::from_ansi(trimmed)
}

// Helper: create side-by-side comparison
fn comparison(console: &mut Console, left: &dyn Renderable, right: &dyn Renderable) -> Text {
    let left_text = render_to_text(console, left);
    let right_text = render_to_text(console, right);

    let mut table = Table::grid(&[]);
    table.set_expand(true);
    table.padding = (0, 1, 0, 0);
    table.add_column(
        "",
        "",
        ColumnOptions {
            ratio: Some(1),
            ..Default::default()
        },
    );
    table.add_column(
        "",
        "",
        ColumnOptions {
            ratio: Some(1),
            ..Default::default()
        },
    );
    table.add_row_text(&[left_text, right_text]);

    render_to_text(console, &table)
}

// ---------------------------------------------------------------------------
// Section 7: Export Demo
// ---------------------------------------------------------------------------

fn show_export(console: &mut Console) {
    console.clear();
    console.print(&Rule::with_title("Export Capabilities"));
    console.line(1);

    // Create a recording console
    let mut recording_console = Console::builder()
        .record(true)
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    // Record some content
    let rule = Rule::with_title("Export Demo");
    recording_console.print(&rule);

    let mut text = Text::new("Hello from ", Style::null());
    text.append_str("gilt", Some(Style::parse("bold green").unwrap()));
    text.append_str("! This content is recorded for export.", None);
    recording_console.print(&text);

    let mut table = Table::new(&["Format", "Description", "Use Case"]);
    table.border_style = "cyan".to_string();
    table.add_row(&["Text", "Plain text output", "Logging, file output"]);
    table.add_row(&["HTML", "Styled HTML with CSS", "Web display, sharing"]);
    table.add_row(&["SVG", "Scalable vector graphic", "Documentation, images"]);
    recording_console.print(&table);

    // Export to different formats
    let plain = recording_console.export_text(false, false);
    let html = recording_console.export_html(None, true, true);
    let svg = recording_console.export_svg("Gilt Demo", None, true, None, 0.61);

    // Display results
    console.print_text("[bold green]Exported Content Preview[/bold green]");
    console.line(1);

    console.print_text("[bold]Plain Text:[/bold]");
    console.print(
        &Panel::new(Text::new(&plain, Style::null()))
            .with_box_chars(&SIMPLE)
            .with_border_style(Style::parse("dim").unwrap()),
    );

    console.line(1);
    console.print_text(&format!(
        "[bold]HTML Export:[/bold] [dim]{} bytes[/dim]",
        html.len()
    ));
    console.print_text(&format!(
        "[bold]SVG Export:[/bold]  [dim]{} bytes[/dim]",
        svg.len()
    ));

    // Try to save to temp directory
    let html_path = "/tmp/gilt_demo_export.html";
    let svg_path = "/tmp/gilt_demo_export.svg";

    if std::fs::write(html_path, &html).is_ok() && std::fs::write(svg_path, &svg).is_ok() {
        console.line(1);
        console.print_text(&format!("[green]✓[/green] Files saved to:"));
        console.print_text(&format!("  HTML: [cyan]{}[/cyan]", html_path));
        console.print_text(&format!("  SVG:  [cyan]{}[/cyan]", svg_path));
    }

    console.line(1);
}

// ---------------------------------------------------------------------------
// Section 8: Farewell
// ---------------------------------------------------------------------------

fn show_farewell(console: &mut Console) {
    console.clear();
    console.line(2);

    // Large banner
    let banner = Figlet::new("GILT");
    console.print(&banner);

    console.line(1);

    // Rainbow thank you
    let thanks = Gradient::rainbow("  Thank you for trying gilt!  ");
    console.print(&thanks);

    console.line(2);

    // Final info panel
    let farewell_content = Text::from_markup(
        "[bold]gilt[/bold] — Rich terminal formatting for Rust\n\n\
         [cyan]https://github.com/khalidelborai/gilt[/cyan]\n\
         [dim]Install: cargo add gilt[/dim]\n\n\
         Features: [green]✓[/green] 40+ widgets  [green]✓[/green] Syntax highlighting  \
         [green]✓[/green] Markdown  [green]✓[/green] Gradients  [green]✓[/green] Export",
    )
    .unwrap();

    let farewell_panel = Panel::new(farewell_content)
        .with_title(Text::styled(
            " gilt v0.8.0 ",
            Style::parse("bold green").unwrap(),
        ))
        .with_box_chars(&ROUNDED)
        .with_border_style(Style::parse("green").unwrap());

    console.print(&farewell_panel);

    console.line(2);
}
