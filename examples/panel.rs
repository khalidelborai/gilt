//! Demonstrates gilt's Panel widget — bordered boxes with titles and styled content.

use gilt::console::Console;
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

    // -- 1. Simple Panel with a Quote -----------------------------------------

    console.print(&Rule::with_title("Simple Panel"));

    let quote = Text::new(
        "The only way to do great work is to love what you do.\n-- Steve Jobs",
        Style::null(),
    );
    let panel = Panel::new(quote);

    console.print(&panel);

    // -- 2. Fit Panel with Title and Subtitle ---------------------------------

    console.print(&Rule::with_title("Fit Panel with Title & Subtitle"));

    let content = Text::new(
        "Gilt is a Rust port of Python's rich library.",
        Style::null(),
    );
    let mut panel = Panel::fit(content);
    panel.title = Some(Text::new("About Gilt", Style::parse("bold").unwrap()));
    panel.subtitle = Some(Text::new("v0.1.0", Style::parse("dim").unwrap()));

    console.print(&panel);

    // -- 3. Panel with Styled Text --------------------------------------------

    console.print(&Rule::with_title("Panel with Styled Text"));

    let mut styled_text = Text::empty();
    styled_text.append_str("Bold text", Some(Style::parse("bold").unwrap()));
    styled_text.append_str(" and ", None);
    styled_text.append_str("italic text", Some(Style::parse("italic").unwrap()));
    styled_text.append_str(" live together.", None);

    let panel = Panel::new(styled_text).with_border_style(Style::parse("green").unwrap());

    console.print(&panel);
}
