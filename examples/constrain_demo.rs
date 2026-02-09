//! Demonstrates the Constrain widget for limiting content to a maximum width.

use gilt::constrain::Constrain;
use gilt::console::Console;
use gilt::prelude::*;

fn main() {
    let mut console = Console::builder()
        .width(90)
        .force_terminal(true)
        .no_color(false)
        .build();

    let paragraph = "The Constrain widget limits the width of any renderable \
        to a given number of characters. This is useful when you want to \
        prevent content from spanning the full terminal width, making it \
        easier to read long paragraphs or keeping layouts compact. The text \
        will automatically wrap at the constrained boundary.";

    // -- Constrained text at various widths -----------------------------------

    let widths: &[usize] = &[80, 60, 40, 20];

    for &w in widths {
        console.print(&Rule::with_title(&format!("Width = {}", w)));

        let text = Text::new(paragraph, Style::null());
        let constrained = Constrain::new(text, Some(w));
        console.print(&constrained);
    }

    // -- Constraining a Panel -------------------------------------------------

    console.print(&Rule::with_title("Constrained Panel (width = 50)"));

    let panel_text = Text::new(
        "This panel is wrapped inside a Constrain widget set to 50 characters.",
        Style::null(),
    );
    let mut panel = Panel::fit(panel_text);
    panel.title = Some(Text::new("Constrained", Style::parse("bold").unwrap()));

    // Constrain works with Text, so render the panel description inside one
    let inner_text = Text::new(
        "This panel is wrapped inside a Constrain widget set to 50 characters. \
         Notice how the content wraps within the narrower boundary.",
        Style::null(),
    );
    let constrained_panel = Constrain::new(inner_text, Some(50));
    console.print(&constrained_panel);

    // -- Side-by-side width comparison ----------------------------------------

    console.print(&Rule::with_title("No Constraint (full width)"));

    let full_text = Text::new(paragraph, Style::null());
    let unconstrained = Constrain::new(full_text, None);
    console.print(&unconstrained);
}
