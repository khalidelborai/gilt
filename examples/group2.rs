//! Port of rich's group2.py — nested panel groups.
//!
//! Creates a Group containing two panels with different background
//! colors, then wraps the group in an outer Panel.

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

    // Create inner panels with different background styles
    let panel1 = Panel::new(Text::new("Hello", Style::null()))
        .with_style(Style::parse("on blue").unwrap_or_else(|_| Style::null()));
    let panel2 = Panel::new(Text::new("World", Style::null()))
        .with_style(Style::parse("on red").unwrap_or_else(|_| Style::null()));

    // Render each inner panel to Text so they can go into a Group.
    // We use a temporary console to capture each panel's output as Text.
    let panel1_text = render_to_text(&console, &panel1);
    let panel2_text = render_to_text(&console, &panel2);

    // Create a group of the two rendered panels
    let group = Group::new(vec![panel1_text, panel2_text]);

    // Render the group to text, then wrap in an outer panel
    let group_text = render_to_text(&console, &group);
    let outer = Panel::new(group_text);

    console.print(&outer);
}

/// Render a Renderable to a Text object by capturing console output.
fn render_to_text(console: &Console, renderable: &dyn gilt::console::Renderable) -> Text {
    let segments = console.render(renderable, None);
    let mut text = Text::empty();
    for seg in &segments {
        text.append_str(&seg.text, seg.style.clone());
    }
    text
}
