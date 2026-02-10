//! Nested Panels demonstration with generic Renderable content
//!
//! This example shows how Panels can contain different types of content:
//! - Text content
//! - Table content (rendered as text)
//! - Tree content (rendered as text)
//! - Columns content (rendered as text)
//!
//! It also demonstrates nested panels (Panel inside Panel) for creating
//! complex dashboard layouts.
//!
//! Run: cargo run --example panel_nested

use gilt::box_chars::{ASCII, DOUBLE, HEAVY, ROUNDED, SQUARE};
use gilt::prelude::*;

fn main() {
    let mut console = Console::builder()
        .width(100)
        .force_terminal(true)
        .no_color(false)
        .build();

    console.rule(Some("Nested Panels Demo"));

    // ── Section 1: Basic Panel with Text content ───────────────────────────
    console.print_text("\n[bold cyan]1. Basic Panel with Text content[/]");

    let text_panel = Panel::new(Text::new(
        "This is a simple text panel with some content inside.",
        Style::null(),
    ));
    console.print(&text_panel);

    // ── Section 2: Panel with styled Text ──────────────────────────────────
    console.print_text("\n[bold cyan]2. Panel with styled Text content[/]");

    let styled_text = Text::styled(
        "This text has [bold]bold[/] and [italic green]styled[/] content using markup!",
        Style::null(),
    );
    let styled_panel = Panel::new(styled_text)
        .with_title("Styled Content")
        .with_border_style(Style::parse("bright_blue").unwrap());
    console.print(&styled_panel);

    // ── Section 3: Panel containing a Table ────────────────────────────────
    console.print_text("\n[bold cyan]3. Panel containing a Table[/]");

    let mut table = Table::new(&["Language", "Type", "Year"]);
    table.add_row(&["Rust", "Systems", "2010"]);
    table.add_row(&["Python", "Scripting", "1991"]);
    table.add_row(&["Go", "Systems", "2009"]);
    table.add_row(&["TypeScript", "Web", "2012"]);

    // Convert table to Text by capturing its rendered output
    let table_text = Text::from(format!("{}", table));
    let table_panel = Panel::new(table_text)
        .with_title("Programming Languages")
        .with_box_chars(&DOUBLE)
        .with_border_style(Style::parse("magenta").unwrap());
    console.print(&table_panel);

    // ── Section 4: Panel containing a Tree ─────────────────────────────────
    console.print_text("\n[bold cyan]4. Panel containing a Tree[/]");

    let bold_green = Style::parse("bold green").unwrap();
    let dim = Style::parse("dim").unwrap();

    let mut tree = Tree::new(Text::new("project/", bold_green.clone()));
    {
        let src = tree.add(Text::new("src/", bold_green.clone()));
        src.add(Text::new("main.rs", dim.clone()));
        src.add(Text::new("lib.rs", dim.clone()));
        let utils = src.add(Text::new("utils/", bold_green.clone()));
        utils.add(Text::new("helpers.rs", dim.clone()));
        utils.add(Text::new("mod.rs", dim.clone()));
    }
    {
        let tests = tree.add(Text::new("tests/", bold_green.clone()));
        tests.add(Text::new("integration.rs", dim.clone()));
        tests.add(Text::new("unit.rs", dim.clone()));
    }
    tree.add(Text::new("Cargo.toml", dim.clone()));
    tree.add(Text::new("README.md", dim.clone()));

    // Convert tree to Text
    let tree_text = Text::from(format!("{}", tree));
    let tree_panel = Panel::new(tree_text)
        .with_title("Project Structure")
        .with_box_chars(&HEAVY)
        .with_border_style(Style::parse("cyan").unwrap());
    console.print(&tree_panel);

    // ── Section 5: Panel containing Columns ────────────────────────────────
    console.print_text("\n[bold cyan]5. Panel containing Columns[/]");

    let mut columns = Columns::new();
    columns.add_renderable("Option A");
    columns.add_renderable("Option B");
    columns.add_renderable("Option C");
    columns.add_renderable("Option D");
    columns.add_renderable("Option E");
    columns.add_renderable("Option F");

    // Convert columns to Text
    let columns_text = Text::from(format!("{}", columns));
    let columns_panel = Panel::new(columns_text)
        .with_title("Available Options")
        .with_box_chars(&ROUNDED)
        .with_border_style(Style::parse("yellow").unwrap());
    console.print(&columns_panel);

    // ── Section 6: Nested Panels (Panel inside Panel) ──────────────────────
    console.print_text("\n[bold cyan]6. Nested Panels (Panel inside Panel)[/]");

    let inner_panel = Panel::new(Text::new(
        "This is the innermost panel!",
        Style::parse("italic").unwrap(),
    ))
    .with_title("Inner")
    .with_box_chars(&ASCII)
    .with_border_style(Style::parse("dim").unwrap());

    // Convert inner panel to text for nesting
    let inner_text = Text::from(format!("{}", inner_panel));
    let middle_panel = Panel::new(inner_text)
        .with_title("Middle Layer")
        .with_box_chars(&SQUARE)
        .with_border_style(Style::parse("bright_green").unwrap());

    // Convert middle panel to text for final nesting
    let middle_text = Text::from(format!("{}", middle_panel));
    let outer_panel = Panel::new(middle_text)
        .with_title("Outer Layer")
        .with_box_chars(&DOUBLE)
        .with_border_style(Style::parse("bright_red").unwrap());

    console.print(&outer_panel);

    // ── Section 7: Dashboard Layout with Multiple Nested Panels ────────────
    console.print_text("\n[bold cyan]7. Practical Dashboard Layout[/]");

    // System Info Panel
    let sys_info_text = Text::from(
        "[bold]Hostname:[/] server-01\n\
         [bold]OS:[/] Linux 6.1.0\n\
         [bold]Uptime:[/] 45 days, 3 hours\n\
         [bold]Load:[/] 0.45 0.38 0.32",
    );
    let sys_info_panel = Panel::new(sys_info_text)
        .with_title("System Info")
        .with_box_chars(&ROUNDED)
        .with_border_style(Style::parse("bright_cyan").unwrap());

    // Resources Panel with mini table
    let mut resources_table = Table::grid(&["Resource", "Usage"]);
    resources_table.add_row(&["CPU", "32%"]);
    resources_table.add_row(&["Memory", "58%"]);
    resources_table.add_row(&["Disk", "71%"]);
    resources_table.add_row(&["Network", "12%"]);

    let resources_text = Text::from(format!("{}", resources_table));
    let resources_panel = Panel::new(resources_text)
        .with_title("Resources")
        .with_box_chars(&ROUNDED)
        .with_border_style(Style::parse("bright_green").unwrap());

    // Status Panel
    let status_text = Text::from(
        "[bold green]●[/] Web Server: Running\n\
         [bold green]●[/] Database: Running\n\
         [bold yellow]●[/] Cache: Warning\n\
         [bold red]●[/] Queue: Error",
    );
    let status_panel = Panel::new(status_text)
        .with_title("Services")
        .with_box_chars(&ROUNDED)
        .with_border_style(Style::parse("bright_magenta").unwrap());

    // Print dashboard panels side by side conceptually
    console.print(&Rule::with_title("Dashboard").with_style(Style::parse("bright_white").unwrap()));
    console.print(&sys_info_panel);
    console.print(&resources_panel);
    console.print(&status_panel);

    // ── Section 8: Deep Nesting Example ────────────────────────────────────
    console.print_text("\n[bold cyan]8. Deep Nesting (4 levels)[/]");

    let level4 = Panel::new(Text::new("Level 4: Core", Style::null()))
        .with_title("L4")
        .with_box_chars(&ASCII);

    let level3 = Panel::new(Text::from(format!("{}\nLevel 3: Wrapped", level4)))
        .with_title("L3")
        .with_box_chars(&SQUARE);

    let level2 = Panel::new(Text::from(format!("{}\nLevel 2: Container", level3)))
        .with_title("L2")
        .with_box_chars(&ROUNDED);

    let level1 = Panel::new(Text::from(format!("{}\nLevel 1: Outer", level2)))
        .with_title("L1 - Deep Nest")
        .with_box_chars(&DOUBLE)
        .with_border_style(Style::parse("bright_yellow").unwrap());

    console.print(&level1);

    // ── Footer ─────────────────────────────────────────────────────────────
    console.rule(Some("End of Demo"));
}
