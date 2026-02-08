//! Demonstrates gilt's style system — parsing, rendering ANSI, merging, and HTML export.

use gilt::color::ColorSystem;
use gilt::style::{Style, StyleStack};

fn main() {
    println!("=== Style Parsing & Rendering ===\n");

    let definitions = [
        "bold",
        "bold red",
        "italic green on black",
        "bold underline #ff6600",
        "not bold dim cyan",
        "strike red on white",
    ];

    for def in definitions {
        let style = Style::parse(def).unwrap();
        let rendered = style.render(def, Some(ColorSystem::TrueColor));
        println!("  {:<35} → {}", def, rendered);
    }

    println!("\n=== Style Merging (operator +) ===\n");

    let base = Style::parse("bold red").unwrap();
    let overlay = Style::parse("italic on blue").unwrap();
    let merged = base.clone() + overlay.clone();
    println!("  base:    {}", base);
    println!("  overlay: {}", overlay);
    println!("  merged:  {}", merged);
    println!(
        "  render:  {}",
        merged.render("Hello, gilt!", Some(ColorSystem::TrueColor))
    );

    println!("\n=== Tri-State Attributes ===\n");

    let style = Style::parse("bold not italic").unwrap();
    println!("  bold:      {:?}", style.bold()); // Some(true)
    println!("  italic:    {:?}", style.italic()); // Some(false)
    println!("  underline: {:?}", style.underline()); // None (not set)

    println!("\n=== Style Stack ===\n");

    let mut stack = StyleStack::new(Style::parse("white").unwrap());
    println!("  base:   {}", stack.current());

    stack.push(Style::parse("bold").unwrap());
    println!("  +bold:  {}", stack.current());

    stack.push(Style::parse("italic red").unwrap());
    println!("  +ital:  {}", stack.current());

    stack.pop().unwrap();
    println!("  pop:    {}", stack.current());

    stack.pop().unwrap();
    println!("  pop:    {}", stack.current());

    println!("\n=== HTML Export ===\n");

    let style = Style::parse("bold italic #e06c75 on #282c34").unwrap();
    println!("  style: {}", style);
    println!("  css:   {}", style.get_html_style(None));

    println!("\n=== Null Style ===\n");

    let null = Style::null();
    println!("  is_null: {}", null.is_null());
    println!(
        "  render:  \"{}\"",
        null.render("plain text", Some(ColorSystem::TrueColor))
    );
}
