//! Emoji demo — shows gilt's emoji name-to-symbol replacement.
//!
//! Demonstrates looking up individual emoji by name and performing bulk
//! replacement of `:name:` codes in text strings.

use gilt::console::Console;
use gilt::emoji::Emoji;
use gilt::emoji_replace::emoji_replace;
use gilt::rule::Rule;
use gilt::style::Style;
use gilt::text::Text;

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    // ── 1. Individual emoji lookups ────────────────────────────────────────
    console.print(&Rule::with_title("Individual Emoji Lookups"));

    let names = [
        "heart",
        "thumbs_up",
        "rocket",
        "star",
        "fire",
        "sparkles",
        "white_check_mark",
        "warning",
        "cross_mark",
        "hourglass",
    ];

    for name in &names {
        match Emoji::new(name) {
            Ok(emoji) => {
                let line = Text::new(&format!("  :{name}:  =>  {emoji}"), Style::null());
                console.print(&line);
            }
            Err(_) => {
                let line = Text::new(
                    &format!("  :{name}:  =>  (not found)"),
                    Style::parse("dim red").unwrap_or_else(|_| Style::null()),
                );
                console.print(&line);
            }
        }
    }

    // ── 2. Bulk emoji replacement in text ──────────────────────────────────
    console.print(&Rule::with_title("Emoji Replacement in Strings"));

    let samples = [
        "I :heart: Rust! :rocket:",
        "Great job! :thumbs_up: :sparkles:",
        ":warning: Careful with that :fire: you might get burned",
        "Status: :white_check_mark: passed, :cross_mark: failed",
        ":star: :star: :star: :star: :star: Five stars!",
        "Time is running out :hourglass:",
    ];

    for sample in &samples {
        let replaced = emoji_replace(sample, None);
        let before = Text::new(
            &format!("  Before: {sample}"),
            Style::parse("dim").unwrap_or_else(|_| Style::null()),
        );
        let after = Text::new(&format!("  After:  {replaced}"), Style::null());
        console.print(&before);
        console.print(&after);
        // blank line between pairs
        console.print(&Text::new("", Style::null()));
    }

    // ── 3. Using Emoji::replace (convenience method) ───────────────────────
    console.print(&Rule::with_title("Emoji::replace Convenience Method"));

    let message = "Deploy :rocket: complete :white_check_mark: — zero :bug: issues!";
    let result = Emoji::replace(message);
    let line = Text::new(&format!("  {result}"), Style::null());
    console.print(&line);

    // ── 4. Unknown emoji names are left unchanged ──────────────────────────
    console.print(&Rule::with_title("Unknown Names Left Unchanged"));

    let text_with_unknown = "Hello :nonexistent_emoji_xyz: world :heart:";
    let result = emoji_replace(text_with_unknown, None);
    let line = Text::new(
        &format!("  Input:  {text_with_unknown}"),
        Style::parse("dim").unwrap_or_else(|_| Style::null()),
    );
    console.print(&line);
    let line = Text::new(&format!("  Output: {result}"), Style::null());
    console.print(&line);
}
