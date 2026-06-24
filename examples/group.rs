//! Demonstrates gilt's Group widget — rendering multiple items in sequence.

use std::sync::Arc;

use gilt::console::Console;
use gilt::group::Group;
use gilt::panel::Panel;
use gilt::rule::Rule;
use gilt::style::Style;
use gilt::text::Text;

fn main() {
    let mut console = Console::builder()
        .width(60)
        .force_terminal(true)
        .no_color(false)
        .build();

    // -- 1. Group rendered directly -------------------------------------------

    console.print(&Rule::with_title("Group — Direct Rendering"));

    let items = vec![
        Arc::new(Text::from_markup("[bold blue]Hello[/bold blue] from item one").unwrap()),
        Arc::new(Text::from_markup("[bold red]World[/bold red] from item two").unwrap()),
        Arc::new(Text::from_markup("[bold green]Goodbye[/bold green] from item three").unwrap()),
    ];
    let group = Group::new(items);

    console.print(&group);

    // -- 2. Group inside a Panel ----------------------------------------------

    console.print(&Rule::with_title("Group in a Panel"));

    let mut combined = Text::from_markup("[bold blue]First[/bold blue] line of the group").unwrap();
    combined.append_str("\n", None);
    combined.append_str("Second line with ", None);
    combined.append_str("red emphasis", Some(Style::parse("bold red")));
    combined.append_str("\n", None);
    combined.append_str("Third line with ", None);
    combined.append_str("green flair", Some(Style::parse("bold green")));

    let panel = Panel::new(combined)
        .with_title(Text::new("Grouped Content", Style::parse("bold")))
        .with_border_style(Style::parse("cyan"));
    console.print(&panel);

    // -- 3. Fit Group ---------------------------------------------------------

    console.print(&Rule::with_title("Fit Group"));

    let items2 = vec![
        Arc::new(Text::from_markup("[bold magenta]Short").unwrap()),
        Arc::new(Text::from_markup("[dim]A slightly longer line here").unwrap()),
    ];
    let group2 = Group::fit(items2);

    console.print(&group2);
}
