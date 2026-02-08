//! Demonstrates gilt's ergonomic `From` / `Into` conversions — create `Text`
//! from string types without specifying a style, and combine with `Stylize`.
//!
//! Run with: `cargo run --example from_conversions`

use gilt::console::Console;
use gilt::panel::Panel;
use gilt::rule::Rule;
use gilt::style::Style;
use gilt::styled_str::Stylize;
use gilt::text::Text;

/// A function that accepts `impl Into<Text>` — callers can pass `&str`,
/// `String`, or `Text` directly.
fn print_in_panel(console: &mut Console, content: impl Into<Text>, title: &str) {
    let text: Text = content.into();
    let mut panel = Panel::fit(text);
    panel.title = Some(Text::new(title, Style::parse("bold").unwrap()));
    console.print(&panel);
}

fn main() {
    let mut console = Console::builder()
        .width(72)
        .force_terminal(true)
        .no_color(false)
        .build();

    // ── Text::from(&str) ─────────────────────────────────────────────────
    console.print(&Rule::with_title("Text::from(&str)"));

    let text = Text::from("Created with Text::from(&str) — no style needed");
    console.print(&text);

    // ── &str .into() ─────────────────────────────────────────────────────
    console.print(&Rule::with_title("let t: Text = str.into()"));

    let text: Text = "Created with .into() — even shorter".into();
    console.print(&text);

    // ── String .into() ───────────────────────────────────────────────────
    console.print(&Rule::with_title("String .into()"));

    let owned = String::from("Owned String converted to Text via .into()");
    let text: Text = owned.into();
    console.print(&text);

    // ── &String .into() ──────────────────────────────────────────────────
    console.print(&Rule::with_title("&String .into()"));

    let owned = String::from("Borrowed &String also converts to Text");
    let text: Text = (&owned).into();
    console.print(&text);

    // ── Using .into() in function arguments ──────────────────────────────
    console.print(&Rule::with_title("impl Into<Text> in function args"));

    // All three of these work with the same function:
    print_in_panel(&mut console, "A &str argument", "From &str");

    let s = String::from("A String argument");
    print_in_panel(&mut console, s, "From String");

    let styled_text = Text::styled("A pre-styled Text", Style::parse("italic green").unwrap());
    print_in_panel(&mut console, styled_text, "From Text");

    // ── Comparison: verbose vs ergonomic ─────────────────────────────────
    console.print(&Rule::with_title("Verbose vs Ergonomic"));

    // Verbose: explicit style
    let verbose = Text::new("The verbose way: Text::new(s, Style::null())", Style::null());
    console.print(&verbose);

    // Ergonomic: just use From
    let ergonomic = Text::from("The ergonomic way: Text::from(s)");
    console.print(&ergonomic);

    // Even shorter
    let shortest: Text = "The shortest way: .into()".into();
    console.print(&shortest);

    // ── StyledStr is Renderable ──────────────────────────────────────────
    console.print(&Rule::with_title("StyledStr: Stylize + Renderable"));

    // StyledStr can be printed directly — it implements Renderable
    let styled = "Bold green text from Stylize trait".bold().green();
    console.print(&styled);

    // Or convert to Text for further manipulation
    let text = "Styled, then converted to Text".bold().cyan().to_text();
    console.print(&text);
}
