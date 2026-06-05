//! Runtime integration tests for the `text!` proc macro.
//!
//! Confirms that `text!(...)` produces a `Text` identical to what
//! `Text::from_markup(...)` would return at runtime.

#![cfg(feature = "derive")]

use gilt::text;
use gilt::text::Text;

/// The macro must produce the same `Text` as `Text::from_markup`.
#[test]
fn test_text_macro_equals_from_markup() {
    let markup = "[bold red]hi[/]";
    let via_macro = text!("[bold red]hi[/]");
    let via_runtime = Text::from_markup(markup).unwrap();

    assert_eq!(
        via_macro.plain(),
        via_runtime.plain(),
        "plain text must match"
    );
    assert_eq!(
        via_macro.spans().len(),
        via_runtime.spans().len(),
        "span count must match"
    );
    for (i, (ms, rs)) in via_macro
        .spans()
        .iter()
        .zip(via_runtime.spans())
        .enumerate()
    {
        assert_eq!(ms.start, rs.start, "span[{i}] start");
        assert_eq!(ms.end, rs.end, "span[{i}] end");
        assert_eq!(ms.style, rs.style, "span[{i}] style");
    }
}

/// Plain text (no tags) must round-trip correctly.
#[test]
fn test_text_macro_plain_text() {
    let t = text!("hello world");
    assert_eq!(t.plain(), "hello world");
    assert_eq!(t.spans().len(), 0);
}

/// Empty string is valid.
#[test]
fn test_text_macro_empty() {
    let t = text!("");
    assert_eq!(t.plain(), "");
    assert_eq!(t.spans().len(), 0);
}

/// Unclosed tags are treated the same way as the runtime (apply to rest of text).
#[test]
fn test_text_macro_unclosed_tag() {
    let via_macro = text!("[bold]hello");
    let via_runtime = Text::from_markup("[bold]hello").unwrap();
    assert_eq!(via_macro.plain(), via_runtime.plain());
    assert_eq!(via_macro.spans().len(), via_runtime.spans().len());
}

/// Nested tags must produce the same spans.
#[test]
fn test_text_macro_nested() {
    let via_macro = text!("[green]X[blue]Y[/blue]Z[/green]");
    let via_runtime = Text::from_markup("[green]X[blue]Y[/blue]Z[/green]").unwrap();
    assert_eq!(via_macro.plain(), via_runtime.plain());
    assert_eq!(via_macro.spans().len(), via_runtime.spans().len());
}

/// Anti-false-positive: all these valid markup strings must compile AND
/// produce the same output as the runtime parser.
#[test]
fn test_text_macro_anti_false_positive() {
    macro_rules! check {
        ($lit:literal) => {{
            let via_macro = text!($lit);
            let via_runtime = Text::from_markup($lit).unwrap();
            assert_eq!(
                via_macro.plain(),
                via_runtime.plain(),
                "plain text mismatch for markup {:?}",
                $lit
            );
            assert_eq!(
                via_macro.spans().len(),
                via_runtime.spans().len(),
                "span count mismatch for markup {:?}",
                $lit
            );
        }};
    }

    check!("[bold]text[/bold]");
    check!("[bold red]text[/]");
    check!("[italic]x[/italic]");
    check!("[underline]y[/underline]");
    check!("[on green]bg[/]");
    check!("[bright_blue]hi[/]");
    check!("[#aabbcc]hex[/]");
    check!("[color(200)]indexed[/]");
    check!("[rgb(10,20,30)]rgb[/]");
    check!("[not bold]normal[/]");
    check!("[link https://example.com]url[/]");
    check!("[@meta=val]tagged[/]");
    check!("[bold italic red]combined[/]");
    check!("[green]X[blue]Y[/blue]Z[/green]");
    check!("[bold]outer[italic]inner[/italic][/bold]");
    check!("no tags at all");
    check!("");
    check!("[dim]faded[/dim]");
    check!("[strike]struck[/strike]");
    check!("[reverse]flipped[/reverse]");
    check!("[blink]blinky[/blink]");
    check!("[on bright_red]error bg[/]");
    check!("[cyan]text[/cyan]");
    check!("[magenta]m[/magenta]");
}
