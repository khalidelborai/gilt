//! Group container demonstration
//!
//! Run: cargo run --example group_demo

use gilt::prelude::*;
use gilt::group::Group;
use gilt::panel::Panel;
use gilt::rule::Rule;
use gilt::table::Table;
use gilt::text::Text;

/// Convenience macro for creating a Group from a list of text items.
///
/// Similar to `vec![]` but wraps items in a Group.
macro_rules! group {
    ($($item:expr),* $(,)?) => {
        Group::new(vec![$($item),*])
    };
}

/// Convenience macro for creating a fit Group from a list of text items.
macro_rules! group_fit {
    ($($item:expr),* $(,)?) => {
        Group::fit(vec![$($item),*])
    };
}

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    console.rule(Some("Group Demo"));

    // ------------------------------------------------------------------------
    // 1. Basic Group Usage
    // ------------------------------------------------------------------------

    console.print(&Rule::with_title("1. Basic Group Usage"));

    // Creating a group with multiple Text items using Group::new()
    let items = vec![
        Text::from_markup("[bold blue]First item[/bold blue] in the group").unwrap(),
        Text::from_markup("[bold green]Second item[/bold green] in the group").unwrap(),
        Text::from_markup("[bold magenta]Third item[/bold magenta] in the group").unwrap(),
    ];
    let group = Group::new(items);
    console.print(&group);

    // Using the group![] macro for cleaner syntax
    console.print(&Rule::with_title("Using group![] macro"));
    let macro_group = group![
        Text::from_markup("Item created with [bold]group![][/bold] macro").unwrap(),
        Text::from_markup("[dim]Another item via macro[/dim]").unwrap(),
        Text::from_markup("[italic]Third macro item[/italic]").unwrap(),
    ];
    console.print(&macro_group);

    // ------------------------------------------------------------------------
    // 2. Fit Modes
    // ------------------------------------------------------------------------

    console.print(&Rule::with_title("2. Fit Modes"));

    // Fit mode true (content-sized) - uses Group::fit()
    console.print(&Rule::with_title("Fit mode: true (content-sized)"));
    let fit_group = Group::fit(vec![
        Text::from_markup("[on blue]Short[/on blue]").unwrap(),
        Text::from_markup("[on green]Medium length text[/on green]").unwrap(),
        Text::from_markup("[on red]Another short[/on red]").unwrap(),
    ]);
    console.print(&fit_group);
    console.print_text("  ^ This group is sized to its widest content");

    // Using group_fit![] macro
    console.print(&Rule::with_title("Using group_fit![] macro"));
    let fit_macro_group = group_fit![
        Text::from_markup("[on cyan]Macro fit[/on cyan]").unwrap(),
        Text::from_markup("[on yellow]Content sized[/on yellow]").unwrap(),
    ];
    console.print(&fit_macro_group);

    // Fit mode false (fills available width) - uses Group::new()
    console.print(&Rule::with_title("Fit mode: false (fills width)"));
    let fill_group = Group::new(vec![
        Text::from_markup("[on bright_black]This group fills the available width[/on bright_black]").unwrap(),
        Text::from_markup("Notice how the content expands to fill the terminal").unwrap(),
    ]);
    console.print(&fill_group);

    // ------------------------------------------------------------------------
    // 3. Combining Different Renderables (via Text conversion)
    // ------------------------------------------------------------------------

    console.print(&Rule::with_title("3. Combining Different Renderables"));

    // Text and Panel combination
    console.print(&Rule::with_title("Text with Panel (rendered to text)"));

    let panel = Panel::new(Text::from_markup("[bold]Panel content[/bold] inside a group").unwrap())
        .with_title(Text::new("Inner Panel", Style::parse("cyan").unwrap()));

    // Render panel to text for inclusion in group
    let panel_text = render_to_text(&console, &panel);

    let mixed_group = Group::new(vec![
        Text::from_markup("[bold]Header text[/bold] above the panel:").unwrap(),
        panel_text,
        Text::from_markup("[dim]Footer text below the panel[/dim]").unwrap(),
    ]);
    console.print(&mixed_group);

    // Table and Rule combination
    console.print(&Rule::with_title("Table and Rule (rendered to text)"));

    let mut table = Table::grid(&["Key", "Value"]);
    table.add_row(&["Name", "Group Demo"]);
    table.add_row(&["Version", "1.0.0"]);
    table.add_row(&["Language", "Rust"]);

    let table_text = render_to_text(&console, &table);
    let rule_text = render_to_text(&console, &Rule::new());

    let table_group = Group::new(vec![
        Text::from_markup("[bold]Configuration:[/bold]").unwrap(),
        table_text,
        rule_text,
        Text::from_markup("[green]✓[/green] Setup complete").unwrap(),
    ]);
    console.print(&table_group);

    // Mixed content - various types combined
    console.print(&Rule::with_title("Mixed Content Group"));

    let info_panel = Panel::fit(Text::from_markup("[bold]Info[/bold]\nThis demonstrates mixed content").unwrap())
        .with_border_style(Style::parse("blue").unwrap());
    let info_text = render_to_text(&console, &info_panel);

    let mixed_content = Group::fit(vec![
        Text::from_markup("[bold underline]Summary[/bold underline]").unwrap(),
        info_text,
        Text::from_markup("[dim]───────────────[/dim]").unwrap(),
        Text::from_markup("Status: [green]Active[/green] | Load: [yellow]Moderate[/yellow]").unwrap(),
    ]);
    console.print(&mixed_content);

    // ------------------------------------------------------------------------
    // 4. Nesting Groups Within Other Widgets
    // ------------------------------------------------------------------------

    console.print(&Rule::with_title("4. Nesting Groups Within Other Widgets"));

    // Group inside a Panel
    console.print(&Rule::with_title("Group inside a Panel"));

    let inner_group = Group::fit(vec![
        Text::from_markup("[bold]Line 1[/bold] of inner group").unwrap(),
        Text::from_markup("[bold]Line 2[/bold] of inner group").unwrap(),
        Text::from_markup("[dim]Line 3 (dimmed)[/dim]").unwrap(),
    ]);
    let inner_group_text = render_to_text(&console, &inner_group);

    let panel_with_group = Panel::new(inner_group_text)
        .with_title(Text::new("Panel Containing Group", Style::parse("bold magenta").unwrap()))
        .with_border_style(Style::parse("green").unwrap());
    console.print(&panel_with_group);

    // Multiple groups in a Columns-like layout (using a container panel)
    console.print(&Rule::with_title("Multiple Groups in Layout"));

    let group_a = Group::fit(vec![
        Text::from_markup("[on blue][white] Group A [/white][/on blue]").unwrap(),
        Text::new("Item A1", Style::null()),
        Text::new("Item A2", Style::null()),
    ]);

    let group_b = Group::fit(vec![
        Text::from_markup("[on red][white] Group B [/white][/on red]").unwrap(),
        Text::new("Item B1", Style::null()),
        Text::new("Item B2", Style::null()),
    ]);

    // Combine groups vertically in a container
    let combined = Group::new(vec![
        render_to_text(&console, &group_a),
        Text::new("", Style::null()), // spacing
        render_to_text(&console, &group_b),
    ]);

    let outer_panel = Panel::new(render_to_text(&console, &combined))
        .with_title(Text::new("Container with Nested Groups", Style::parse("bold cyan").unwrap()));
    console.print(&outer_panel);

    // Nested groups (group within a group)
    console.print(&Rule::with_title("Nested Groups"));

    let inner_nested = Group::fit(vec![
        Text::from_markup("[yellow]  → Inner nested item 1[/yellow]").unwrap(),
        Text::from_markup("[yellow]  → Inner nested item 2[/yellow]").unwrap(),
    ]);

    let outer_nested = Group::new(vec![
        Text::from_markup("[bold]Outer group start[/bold]").unwrap(),
        render_to_text(&console, &inner_nested),
        Text::from_markup("[bold]Outer group end[/bold]").unwrap(),
    ]);
    console.print(&outer_nested);

    // ------------------------------------------------------------------------
    // 5. Practical Use Case: Status Display
    // ------------------------------------------------------------------------

    console.print(&Rule::with_title("5. Practical Example: Status Display"));

    let status_group = Group::fit(vec![
        Text::from_markup("[bold underline]System Status[/bold underline]").unwrap(),
        Text::from_markup("[green]●[/green] Database: Connected").unwrap(),
        Text::from_markup("[green]●[/green] Cache: Operational").unwrap(),
        Text::from_markup("[yellow]●[/yellow] Queue: 12 pending").unwrap(),
        Text::from_markup("[red]●[/red] Backup: Overdue").unwrap(),
        Text::from_markup("[dim]─────────────────[/dim]").unwrap(),
        Text::from_markup("Last update: [italic]Just now[/italic]").unwrap(),
    ]);

    let status_panel = Panel::new(render_to_text(&console, &status_group))
        .with_title(Text::new("Dashboard", Style::parse("bold white").unwrap()))
        .with_border_style(Style::parse("bright_black").unwrap());
    console.print(&status_panel);

    console.rule(Some("End of Demo"));
}

/// Render a Renderable to a Text object by capturing console output.
fn render_to_text(console: &Console, renderable: &dyn Renderable) -> Text {
    let segments = console.render(renderable, None);
    let mut text = Text::empty();
    for seg in &segments {
        text.append_str(&seg.text, seg.style.clone());
    }
    text
}
