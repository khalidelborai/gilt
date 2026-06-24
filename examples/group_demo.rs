//! Group container demonstration
//!
//! Run: cargo run --example group_demo

use std::sync::Arc;

use gilt::group::Group;
use gilt::panel::Panel;
use gilt::prelude::*;
use gilt::rule::Rule;
use gilt::table::Table;
use gilt::text::Text;
use gilt::RenderableArc;

/// Convenience macro for creating a Group from a list of RenderableArc items.
macro_rules! group {
    ($($item:expr),* $(,)?) => {
        Group::new(vec![$($item),*])
    };
}

/// Convenience macro for creating a fit Group from a list of RenderableArc items.
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
    let items: Vec<RenderableArc> = vec![
        Arc::new(Text::from_markup("[bold blue]First item[/bold blue] in the group").unwrap()),
        Arc::new(Text::from_markup("[bold green]Second item[/bold green] in the group").unwrap()),
        Arc::new(
            Text::from_markup("[bold magenta]Third item[/bold magenta] in the group").unwrap(),
        ),
    ];
    let group = Group::new(items);
    console.print(&group);

    // Using the group![] macro for cleaner syntax
    console.print(&Rule::with_title("Using group![] macro"));
    let macro_group = group![
        Arc::new(Text::from_markup("Item created with [bold]group![][/bold] macro").unwrap())
            as RenderableArc,
        Arc::new(Text::from_markup("[dim]Another item via macro[/dim]").unwrap()),
        Arc::new(Text::from_markup("[italic]Third macro item[/italic]").unwrap()),
    ];
    console.print(&macro_group);

    // ------------------------------------------------------------------------
    // 2. Fit Modes
    // ------------------------------------------------------------------------

    console.print(&Rule::with_title("2. Fit Modes"));

    // Fit mode true (content-sized) - uses Group::fit()
    console.print(&Rule::with_title("Fit mode: true (content-sized)"));
    let fit_group = Group::fit(vec![
        Arc::new(Text::from_markup("[on blue]Short[/on blue]").unwrap()) as RenderableArc,
        Arc::new(Text::from_markup("[on green]Medium length text[/on green]").unwrap()),
        Arc::new(Text::from_markup("[on red]Another short[/on red]").unwrap()),
    ]);
    console.print(&fit_group);
    console.print_text("  ^ This group is sized to its widest content");

    // Using group_fit![] macro
    console.print(&Rule::with_title("Using group_fit![] macro"));
    let fit_macro_group = group_fit![
        Arc::new(Text::from_markup("[on cyan]Macro fit[/on cyan]").unwrap()) as RenderableArc,
        Arc::new(Text::from_markup("[on yellow]Content sized[/on yellow]").unwrap()),
    ];
    console.print(&fit_macro_group);

    // Fit mode false (fills available width) - uses Group::new()
    console.print(&Rule::with_title("Fit mode: false (fills width)"));
    let fill_group = Group::new(vec![
        Arc::new(
            Text::from_markup(
                "[on bright_black]This group fills the available width[/on bright_black]",
            )
            .unwrap(),
        ) as RenderableArc,
        Arc::new(Text::from_markup("Notice how the content expands to fill the terminal").unwrap()),
    ]);
    console.print(&fill_group);

    // ------------------------------------------------------------------------
    // 3. Combining Different Renderables (directly, no pre-render needed)
    // ------------------------------------------------------------------------

    console.print(&Rule::with_title("3. Combining Different Renderables"));

    // Text and Panel combination — Panel goes directly into Group
    console.print(&Rule::with_title("Text with Panel"));

    let panel = Panel::new(Text::from_markup("[bold]Panel content[/bold] inside a group").unwrap())
        .with_title(Text::new("Inner Panel", Style::parse("cyan")));

    let mixed_group = Group::new(vec![
        Arc::new(Text::from_markup("[bold]Header text[/bold] above the panel:").unwrap())
            as RenderableArc,
        Arc::new(panel),
        Arc::new(Text::from_markup("[dim]Footer text below the panel[/dim]").unwrap()),
    ]);
    console.print(&mixed_group);

    // Table and Rule combination
    console.print(&Rule::with_title("Table and Rule"));

    let mut table = Table::grid(&["Key", "Value"]);
    table.add_row(&["Name", "Group Demo"]);
    table.add_row(&["Version", "1.0.0"]);
    table.add_row(&["Language", "Rust"]);

    let table_group = Group::new(vec![
        Arc::new(Text::from_markup("[bold]Configuration:[/bold]").unwrap()) as RenderableArc,
        Arc::new(table),
        Arc::new(Rule::new()),
        Arc::new(Text::from_markup("[green]✓[/green] Setup complete").unwrap()),
    ]);
    console.print(&table_group);

    // Mixed content - various types combined
    console.print(&Rule::with_title("Mixed Content Group"));

    let info_panel = Panel::fit(
        Text::from_markup("[bold]Info[/bold]\nThis demonstrates mixed content").unwrap(),
    )
    .with_border_style(Style::parse("blue"));

    let mixed_content = Group::fit(vec![
        Arc::new(Text::from_markup("[bold underline]Summary[/bold underline]").unwrap())
            as RenderableArc,
        Arc::new(info_panel),
        Arc::new(Text::from_markup("[dim]───────────────[/dim]").unwrap()),
        Arc::new(
            Text::from_markup("Status: [green]Active[/green] | Load: [yellow]Moderate[/yellow]")
                .unwrap(),
        ),
    ]);
    console.print(&mixed_content);

    // ------------------------------------------------------------------------
    // 4. Nesting Groups Within Other Widgets
    // ------------------------------------------------------------------------

    console.print(&Rule::with_title("4. Nesting Groups Within Other Widgets"));

    // Group inside a Panel — directly, no pre-render
    console.print(&Rule::with_title("Group inside a Panel"));

    let inner_group = Group::fit(vec![
        Arc::new(Text::from_markup("[bold]Line 1[/bold] of inner group").unwrap()) as RenderableArc,
        Arc::new(Text::from_markup("[bold]Line 2[/bold] of inner group").unwrap()),
        Arc::new(Text::from_markup("[dim]Line 3 (dimmed)[/dim]").unwrap()),
    ]);

    let panel_with_group = Panel::new(inner_group)
        .with_title(Text::new(
            "Panel Containing Group",
            Style::parse("bold magenta"),
        ))
        .with_border_style(Style::parse("green"));
    console.print(&panel_with_group);

    // Multiple groups combined
    console.print(&Rule::with_title("Multiple Groups in Layout"));

    let group_a = Group::fit(vec![
        Arc::new(Text::from_markup("[on blue][white] Group A [/white][/on blue]").unwrap())
            as RenderableArc,
        Arc::new(Text::new("Item A1", Style::null())),
        Arc::new(Text::new("Item A2", Style::null())),
    ]);

    let group_b = Group::fit(vec![
        Arc::new(Text::from_markup("[on red][white] Group B [/white][/on red]").unwrap())
            as RenderableArc,
        Arc::new(Text::new("Item B1", Style::null())),
        Arc::new(Text::new("Item B2", Style::null())),
    ]);

    let combined = Group::new(vec![
        Arc::new(group_a) as RenderableArc,
        Arc::new(Text::new("", Style::null())), // spacing
        Arc::new(group_b),
    ]);

    let outer_panel = Panel::new(combined).with_title(Text::new(
        "Container with Nested Groups",
        Style::parse("bold cyan"),
    ));
    console.print(&outer_panel);

    // Nested groups (group within a group)
    console.print(&Rule::with_title("Nested Groups"));

    let inner_nested = Group::fit(vec![
        Arc::new(Text::from_markup("[yellow]  → Inner nested item 1[/yellow]").unwrap())
            as RenderableArc,
        Arc::new(Text::from_markup("[yellow]  → Inner nested item 2[/yellow]").unwrap()),
    ]);

    let outer_nested = Group::new(vec![
        Arc::new(Text::from_markup("[bold]Outer group start[/bold]").unwrap()) as RenderableArc,
        Arc::new(inner_nested),
        Arc::new(Text::from_markup("[bold]Outer group end[/bold]").unwrap()),
    ]);
    console.print(&outer_nested);

    // ------------------------------------------------------------------------
    // 5. Practical Use Case: Status Display
    // ------------------------------------------------------------------------

    console.print(&Rule::with_title("5. Practical Example: Status Display"));

    let status_group = Group::fit(vec![
        Arc::new(Text::from_markup("[bold underline]System Status[/bold underline]").unwrap())
            as RenderableArc,
        Arc::new(Text::from_markup("[green]●[/green] Database: Connected").unwrap()),
        Arc::new(Text::from_markup("[green]●[/green] Cache: Operational").unwrap()),
        Arc::new(Text::from_markup("[yellow]●[/yellow] Queue: 12 pending").unwrap()),
        Arc::new(Text::from_markup("[red]●[/red] Backup: Overdue").unwrap()),
        Arc::new(Text::from_markup("[dim]─────────────────[/dim]").unwrap()),
        Arc::new(Text::from_markup("Last update: [italic]Just now[/italic]").unwrap()),
    ]);

    let status_panel = Panel::new(status_group)
        .with_title(Text::new("Dashboard", Style::parse("bold white")))
        .with_border_style(Style::parse("bright_black"));
    console.print(&status_panel);

    console.rule(Some("End of Demo"));
}
