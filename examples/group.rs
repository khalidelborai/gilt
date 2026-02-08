//! Demonstrates gilt's Group widget — rendering multiple items in sequence.

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
        Text::from_markup("[bold blue]Hello[/bold blue] from item one").unwrap(),
        Text::from_markup("[bold red]World[/bold red] from item two").unwrap(),
        Text::from_markup("[bold green]Goodbye[/bold green] from item three").unwrap(),
    ];
    let group = Group::new(items);

    console.print(&group);

    // -- 2. Group inside a Panel (via Text) -----------------------------------

    console.print(&Rule::with_title("Group in a Panel"));

    // Panel accepts Text, so we build a multi-line Text that mimics the group.
    let mut combined = Text::from_markup("[bold blue]First[/bold blue] line of the group").unwrap();
    combined.append_str("\n", None);
    combined.append_str("Second line with ", None);
    combined.append_str("red emphasis", Some(Style::parse("bold red").unwrap()));
    combined.append_str("\n", None);
    combined.append_str("Third line with ", None);
    combined.append_str("green flair", Some(Style::parse("bold green").unwrap()));

    let panel = Panel::new(combined)
        .title(Text::new("Grouped Content", Style::parse("bold").unwrap()))
        .border_style(Style::parse("cyan").unwrap());
    console.print(&panel);

    // -- 3. Fit Group ---------------------------------------------------------

    console.print(&Rule::with_title("Fit Group"));

    let items2 = vec![
        Text::from_markup("[bold magenta]Short").unwrap(),
        Text::from_markup("[dim]A slightly longer line here").unwrap(),
    ];
    let group2 = Group::fit(items2);

    console.print(&group2);
}
