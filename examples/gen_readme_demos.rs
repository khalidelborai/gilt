//! Generate self-contained, Dracula-themed SVG demo files for the README.
//!
//! Run with:
//!   cargo run --example gen_readme_demos --all-features
//!
//! Writes eight SVGs to `assets/demos/`, all exported through gilt's own
//! `export_svg` with the built-in `DRACULA` terminal theme:
//!   hero.svg      — gradient banner + Panel wrapping a Table (2.0 nesting)
//!   styles.svg    — markup / style sampler
//!   table.svg     — styled table
//!   tree.svg      — dependency hierarchy
//!   markdown.svg  — markdown rendering
//!   syntax.svg    — syntax highlighting with line numbers
//!   progress.svg  — static progress snapshot
//!   extras.svg    — Rust-native extras: gradient text + sparklines

use std::fs;
use std::path::Path;

use gilt::color::Color;
use gilt::console::Console;
use gilt::gradient::Gradient;
use gilt::markdown::Markdown;
use gilt::panel::Panel;
use gilt::progress::{BarColumn, Progress, TaskProgressColumn, TextColumn, TimeRemainingColumn};
use gilt::rule::Rule;
use gilt::sparkline::Sparkline;
use gilt::style::Style;
use gilt::syntax::Syntax;
use gilt::table::Table;
use gilt::terminal_theme::DRACULA;
use gilt::text::Text;
use gilt::tree::Tree;

const WIDTH: usize = 72;
const RATIO: f64 = 0.61;

fn recording_console() -> Console {
    Console::builder()
        .width(WIDTH)
        .record(true)
        .force_terminal(true)
        .build()
}

/// Export the recorded buffer as a Dracula-themed SVG and write it.
fn export(c: &mut Console, dir: &Path, name: &str, title: &str, id: &str) {
    let svg = c.export_svg(title, Some(&DRACULA), false, Some(id), RATIO);
    let path = dir.join(name);
    fs::write(&path, &svg).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
    assert!(
        svg.contains("<svg"),
        "{name} does not contain <svg — export failed"
    );
    println!(
        "  wrote {} ({:.1} KB)",
        path.display(),
        svg.len() as f64 / 1024.0
    );
}

// ---------------------------------------------------------------------------
// Scene 0 – hero (gradient banner + Panel wrapping a Table)
// ---------------------------------------------------------------------------

fn scene_hero(dir: &Path) {
    let mut c = recording_console();

    // A gradient banner — Dracula purple → pink → cyan.
    c.print(&Gradient::new(
        "gilt — rich terminal output for Rust",
        vec![
            Color::from_rgb(189, 147, 249), // purple
            Color::from_rgb(255, 121, 198), // pink
            Color::from_rgb(139, 233, 253), // cyan
        ],
    ));
    c.print_text("");

    // A capabilities table — wrapped in a Panel. gilt 2.0 lets any container
    // (Panel here) hold any Renderable, including a full bordered Table.
    let mut caps = Table::new(&["Widget", "What you get"]).with_border_style("magenta");
    caps.add_row(&["[bold]Styles[/]", "[bold]bold[/] [italic]italic[/] [cyan]color[/] [underline]links[/]"]);
    caps.add_row(&["[bold]Table · Tree[/]", "Unicode box-drawing, free nesting"]);
    caps.add_row(&["[bold]Markdown[/]", "headings, lists, [green]code[/], tables"]);
    caps.add_row(&["[bold]Syntax[/]", "[yellow]150+[/] languages"]);
    caps.add_row(&["[bold]Progress · Live[/]", "ETA, speed, lock-free updates"]);

    let panel = Panel::new(caps)
        .with_title(Text::new(" gilt 2.0 ", Style::parse("bold #282a36 on #bd93f9")))
        .with_subtitle(Text::new("a Rust port of rich", Style::parse("italic #6272a4")))
        .with_border_style(Style::parse("bold #bd93f9"));
    c.print(&panel);

    export(&mut c, dir, "hero.svg", "gilt", "gilt-demo-hero");
}

// ---------------------------------------------------------------------------
// Scene 1 – styles
// ---------------------------------------------------------------------------

fn scene_styles(dir: &Path) {
    let mut c = recording_console();
    c.print(&Rule::with_title("gilt — styles"));

    // Basic markup
    c.print_text("[bold magenta]gilt[/] — rich terminal output for Rust");
    c.print_text(
        "[bold]bold[/]  [italic]italic[/]  [underline]underline[/]  [dim]dim[/]  [strike]strike[/]",
    );

    // Named colours (resolved through the Dracula palette on export)
    c.print_text(
        "[red]red[/] [green]green[/] [blue]blue[/] [yellow]yellow[/] [cyan]cyan[/] [magenta]magenta[/]",
    );

    // True-colour
    c.print_text("[#ff5555]red[/]  [#50fa7b]green[/]  [#8be9fd]cyan[/]  [#bd93f9]purple[/]  [#ffb86c]orange[/]");

    // On-colour backgrounds
    c.print_text("[bold #282a36 on #ff5555] ERROR [/]  [bold #282a36 on #f1fa8c] WARN [/]  [bold #282a36 on #50fa7b] OK [/]");

    // Hyperlink (OSC 8)
    c.print_text("[link=https://crates.io/crates/gilt][underline cyan]gilt on crates.io[/]");

    // Dim / nested
    c.print_text("[dim italic]A quiet note tucked at the end.[/]");

    c.print(&Rule::new());

    export(&mut c, dir, "styles.svg", "gilt styles", "gilt-demo-styles");
}

// ---------------------------------------------------------------------------
// Scene 2 – table
// ---------------------------------------------------------------------------

fn scene_table(dir: &Path) {
    let mut c = recording_console();

    let mut table =
        Table::new(&["Crate", "Category", "Rating"]).with_title("[bold]Rust CLI ecosystem[/bold]");
    table.add_row(&["clap", "arg-parsing", "[green]★★★★★[/]"]);
    table.add_row(&["indicatif", "progress-bars", "[green]★★★★[/][dim]★[/]"]);
    table.add_row(&["console", "terminal-utils", "[green]★★★[/][dim]★★[/]"]);
    table.add_row(&["[bold magenta]gilt[/]", "[magenta]rich-output[/]", "[green]★★★★★[/]"]);

    c.print(&table);

    export(&mut c, dir, "table.svg", "gilt table", "gilt-demo-table");
}

// ---------------------------------------------------------------------------
// Scene 3 – tree
// ---------------------------------------------------------------------------

fn scene_tree(dir: &Path) {
    let mut c = recording_console();

    c.print(&Rule::with_title("dependency tree (excerpt)"));

    let bold_blue = Style::parse("bold blue");
    let dim = Style::parse("dim");
    let default = Style::null();

    let mut tree = Tree::new(Text::new("gilt 2.0.0", Style::parse("bold cyan")))
        .with_guide_style(dim.clone());

    let md = tree.add(Text::new("pulldown-cmark 0.12", bold_blue.clone()));
    md.add(Text::new("unicase 2.8", default.clone()));

    let syn = tree.add(Text::new("syntect 5", bold_blue.clone()));
    syn.add(Text::new("regex-fancy 0.x", default.clone()));
    syn.add(Text::new("serde 1", default.clone()));

    tree.add(Text::new("unicode-width 0.2", default.clone()));
    tree.add(Text::new("compact_str 0.8", default.clone()));

    c.print(&tree);

    export(&mut c, dir, "tree.svg", "gilt tree", "gilt-demo-tree");
}

// ---------------------------------------------------------------------------
// Scene 4 – markdown
// ---------------------------------------------------------------------------

fn scene_markdown(dir: &Path) {
    let mut c = recording_console();

    let doc = r#"# gilt

**Fast, beautiful terminal output** for Rust — a *rich* port.

## Quick start

```toml
[dependencies]
gilt = "2.0"
```

## Features

- Styles, colours, hyperlinks
- Tables, trees, panels
- `Syntax` highlighting (150+ languages)
- Markdown rendering (this very output)
- Progress bars with ETA and speed
"#;

    let md = Markdown::new(doc);
    c.print(&md);

    export(&mut c, dir, "markdown.svg", "gilt markdown", "gilt-demo-markdown");
}

// ---------------------------------------------------------------------------
// Scene 5 – syntax highlighting
// ---------------------------------------------------------------------------

fn scene_syntax(dir: &Path) {
    let mut c = recording_console();

    let rust_code = r#"use gilt::prelude::*;

fn main() {
    let mut console = Console::default();
    console.print_text("Hello, [bold magenta]gilt[/]!");

    let mut table = Table::new(&["Lang", "Lines"]);
    table.add_row(&["Rust", "9000"]);
    console.print(&table);
}
"#;

    let syntax = Syntax::new(rust_code, "rs")
        .with_line_numbers(true)
        .with_theme("base16-ocean.dark");
    c.print(&syntax);

    export(&mut c, dir, "syntax.svg", "gilt syntax", "gilt-demo-syntax");
}

// ---------------------------------------------------------------------------
// Scene 6 – progress (static single-frame snapshot)
// ---------------------------------------------------------------------------

fn scene_progress(dir: &Path) {
    let mut c = recording_console();

    // Use `make_tasks_table` to render a static snapshot — no live display.
    let mut progress = Progress::new(vec![
        Box::new(TextColumn::new("{task.description}")),
        Box::new(BarColumn::default()),
        Box::new(TaskProgressColumn::default()),
        Box::new(TimeRemainingColumn::default()),
    ])
    .with_console(
        Console::builder()
            .width(WIDTH)
            .record(false)
            .force_terminal(true)
            .build(),
    )
    .with_auto_refresh(false)
    .with_disable(true); // keep it static — no live display at all

    let t1 = progress.add_task("Compiling crates…", Some(100.0), true);
    let t2 = progress.add_task("Running tests…", Some(200.0), true);
    let t3 = progress.add_task("Generating docs…", Some(50.0), true);

    progress.update(t1, Some(62.0), None, None, None, None, None);
    progress.update(t2, Some(130.0), None, None, None, None, None);
    progress.update(t3, Some(18.0), None, None, None, None, None);

    let table = progress.make_tasks_table();
    c.print(&Rule::with_title("build progress"));
    c.print(&table);

    export(&mut c, dir, "progress.svg", "gilt progress", "gilt-demo-progress");
}

// ---------------------------------------------------------------------------
// Scene 7 – Rust-native extras (gradient text + sparklines)
// ---------------------------------------------------------------------------

fn scene_extras(dir: &Path) {
    let mut c = recording_console();

    c.print(&Rule::with_title("Rust-native extras"));

    // Multi-stop gradient text.
    c.print(&Gradient::new(
        "Gradient text — interpolated across true-colour stops",
        vec![
            Color::from_rgb(255, 85, 85),   // red
            Color::from_rgb(255, 184, 108), // orange
            Color::from_rgb(241, 250, 140), // yellow
        ],
    ));
    c.print_text("");

    // Sparklines — Braille-free block bars driven by raw data.
    let cpu: Vec<f64> = vec![
        12.0, 15.0, 22.0, 35.0, 42.0, 55.0, 68.0, 72.0, 80.0, 95.0, 88.0, 70.0, 60.0, 45.0, 38.0,
        30.0, 25.0, 18.0, 20.0, 28.0, 35.0, 50.0, 62.0, 75.0, 85.0, 78.0, 65.0, 55.0, 40.0, 32.0,
    ];
    c.print_text("[bold]CPU[/]  ");
    c.print(&Sparkline::new(&cpu).with_width(64).with_style(Style::parse("bold green")));

    let mem: Vec<f64> = vec![
        30.0, 32.0, 35.0, 40.0, 55.0, 70.0, 85.0, 92.0, 95.0, 88.0, 75.0, 60.0, 48.0, 40.0,
    ];
    c.print_text("[bold]MEM[/]  ");
    c.print(&Sparkline::new(&mem).with_width(64).with_style(Style::parse("bold yellow")));

    export(&mut c, dir, "extras.svg", "gilt extras", "gilt-demo-extras");
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    let dir = Path::new("assets/demos");
    fs::create_dir_all(dir).expect("could not create assets/demos");

    println!("Generating README demo SVGs → {}/", dir.display());

    scene_hero(dir);
    scene_styles(dir);
    scene_table(dir);
    scene_tree(dir);
    scene_markdown(dir);
    scene_syntax(dir);
    scene_progress(dir);
    scene_extras(dir);

    println!("Done — 8 SVGs written.");
}
