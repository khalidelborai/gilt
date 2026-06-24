//! Port of rich's group2.py — nested panel groups.
//!
//! Creates a Group containing two panels with different background
//! colors, then wraps the group in an outer Panel.

use std::sync::Arc;

use gilt::console::Console;
use gilt::group::Group;
use gilt::panel::Panel;
use gilt::style::Style;
use gilt::text::Text;

fn main() {
    let mut console = Console::builder()
        .width(60)
        .force_terminal(true)
        .no_color(false)
        .build();

    // Create inner panels with different background styles.
    // Group now accepts any Renderable directly — no need to pre-render to Text.
    let panel1 = Panel::new(Text::new("Hello", Style::null())).with_style(Style::parse("on blue"));
    let panel2 = Panel::new(Text::new("World", Style::null())).with_style(Style::parse("on red"));

    // Create a group that holds the panels directly.
    let group = Group::new(vec![Arc::new(panel1), Arc::new(panel2)]);

    // Wrap the group in an outer panel.
    let outer = Panel::new(group);

    console.print(&outer);
}
