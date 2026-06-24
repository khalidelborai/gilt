//! Generate self-contained SVG demo files for the README.
//!
//! Run with:
//!   cargo run --example gen_readme_demos --all-features
//!
//! Writes five SVGs to `assets/demos/`:
//!   styles.svg    — markup/style sampler
//!   table.svg     — styled table
//!   tree.svg      — file/dep hierarchy
//!   markdown.svg  — markdown rendering
//!   progress.svg  — static progress snapshot

use std::fs;
use std::path::Path;

use gilt::console::Console;
use gilt::markdown::Markdown;
use gilt::progress::{BarColumn, Progress, TaskProgressColumn, TextColumn, TimeRemainingColumn};
use gilt::rule::Rule;
use gilt::style::Style;
use gilt::table::Table;
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

fn save(dir: &Path, name: &str, svg: String) {
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

    // Named colours
    c.print_text(
        "[red]red[/] [green]green[/] [blue]blue[/] [yellow]yellow[/] [cyan]cyan[/] [magenta]magenta[/]",
    );

    // True-colour
    c.print_text("[#e06c75]salmon[/]  [#98c379]sage[/]  [#61afef]sky[/]  [#c678dd]plum[/]");

    // On-colour backgrounds
    c.print_text("[bold white on red] ERROR [/]  [bold black on yellow] WARN [/]  [bold white on green] OK [/]");

    // Hyperlink (OSC 8)
    c.print_text("[link=https://crates.io/crates/gilt][underline cyan]gilt on crates.io[/]");

    // Dim / nested
    c.print_text("[dim italic]A quiet note tucked at the end.[/]");

    c.print(&Rule::new());

    let svg = c.export_svg("gilt styles", None, false, Some("gilt-demo-styles"), RATIO);
    save(dir, "styles.svg", svg);
}

// ---------------------------------------------------------------------------
// Scene 2 – table
// ---------------------------------------------------------------------------

fn scene_table(dir: &Path) {
    let mut c = recording_console();

    let mut table =
        Table::new(&["Crate", "Category", "Stars"]).with_title("[bold]Rust CLI ecosystem[/bold]");
    table.add_row(&["clap", "arg-parsing", "★★★★★"]);
    table.add_row(&["indicatif", "progress-bars", "★★★★☆"]);
    table.add_row(&["console", "terminal-utils", "★★★☆☆"]);
    table.add_row(&["gilt", "rich-output", "★★★★★"]);

    c.print(&table);

    let svg = c.export_svg("gilt table", None, false, Some("gilt-demo-table"), RATIO);
    save(dir, "table.svg", svg);
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

    let mut tree = Tree::new(Text::new("gilt 1.10.0", Style::parse("bold cyan")))
        .with_guide_style(dim.clone());

    let md = tree.add(Text::new("pulldown-cmark 0.12", bold_blue.clone()));
    md.add(Text::new("unicase 2.8", default.clone()));

    let syn = tree.add(Text::new("syntect 5", bold_blue.clone()));
    syn.add(Text::new("regex-fancy 0.x", default.clone()));
    syn.add(Text::new("serde 1", default.clone()));

    tree.add(Text::new("unicode-width 0.2", default.clone()));
    tree.add(Text::new("compact_str 0.8", default.clone()));

    c.print(&tree);

    let svg = c.export_svg("gilt tree", None, false, Some("gilt-demo-tree"), RATIO);
    save(dir, "tree.svg", svg);
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
gilt = "1.10"
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

    let svg = c.export_svg(
        "gilt markdown",
        None,
        false,
        Some("gilt-demo-markdown"),
        RATIO,
    );
    save(dir, "markdown.svg", svg);
}

// ---------------------------------------------------------------------------
// Scene 5 – progress (static single-frame snapshot)
// ---------------------------------------------------------------------------

fn scene_progress(dir: &Path) {
    // Build a Progress with a fixed clock so times are deterministic.
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

    let svg = c.export_svg(
        "gilt progress",
        None,
        false,
        Some("gilt-demo-progress"),
        RATIO,
    );
    save(dir, "progress.svg", svg);
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    let dir = Path::new("assets/demos");
    fs::create_dir_all(dir).expect("could not create assets/demos");

    println!("Generating README demo SVGs → {}/", dir.display());

    scene_styles(dir);
    scene_table(dir);
    scene_tree(dir);
    scene_markdown(dir);
    scene_progress(dir);

    println!("Done — {} SVGs written.", 5);
}
