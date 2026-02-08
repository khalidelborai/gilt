//! Demonstrates gilt's Text justification and overflow modes.

use gilt::console::Console;
use gilt::panel::Panel;
use gilt::rule::Rule;
use gilt::style::Style;
use gilt::text::{JustifyMethod, OverflowMethod, Text};

fn main() {
    let mut console = Console::builder()
        .width(40)
        .force_terminal(true)
        .no_color(false)
        .build();

    let sample = "The quick brown fox jumps over the lazy dog near the riverbank.";

    // -- Justification modes -----------------------------------------------

    console.print(&Rule::with_title("Text Justification (width 40)"));

    let modes: &[(&str, JustifyMethod)] = &[
        ("Left", JustifyMethod::Left),
        ("Center", JustifyMethod::Center),
        ("Right", JustifyMethod::Right),
        ("Full", JustifyMethod::Full),
    ];

    for (label, justify) in modes {
        // Print a rule as section header
        let rule = Rule::with_title(label).characters("-");
        console.print(&rule);

        // Create text with the given justification
        let mut text = Text::new(sample, Style::null());
        text.justify = Some(*justify);

        // Wrap it in a panel so alignment is visible against the border
        let panel = Panel::new(text);

        console.print(&panel);
    }

    // -- Overflow modes ----------------------------------------------------

    console.print(&Rule::with_title("Overflow Modes"));

    let long_word = "Supercalifragilisticexpialidocious";

    let overflow_modes: &[(&str, OverflowMethod)] = &[
        ("Fold", OverflowMethod::Fold),
        ("Crop", OverflowMethod::Crop),
        ("Ellipsis", OverflowMethod::Ellipsis),
    ];

    for (label, overflow) in overflow_modes {
        let rule = Rule::with_title(label).characters("-");
        console.print(&rule);

        let mut text = Text::new(long_word, Style::null());
        text.overflow = Some(*overflow);

        // Use a narrow panel to force overflow
        let panel = Panel::new(text).width(25);

        console.print(&panel);
    }

    // -- Combined: justify + styled text -----------------------------------

    console.print(&Rule::with_title("Centered Styled Text"));

    let mut styled = Text::new("", Style::null());
    styled.append_str("gilt", Some(Style::parse("bold magenta").unwrap()));
    styled.append_str(" makes terminal output ", None);
    styled.append_str("beautiful", Some(Style::parse("italic green").unwrap()));
    styled.append_str(" and ", None);
    styled.append_str("easy", Some(Style::parse("bold cyan").unwrap()));
    styled.append_str(".", None);
    styled.justify = Some(JustifyMethod::Center);

    let panel = Panel::new(styled).title(Text::new("Styled & Centered", Style::null()));

    console.print(&panel);
}
