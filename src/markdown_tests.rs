//! Tests extracted from markdown.rs for readability.
//! Wired in via `#[path = ...] mod tests;`.

use super::*;
use crate::cells::cell_len;

fn make_console(width: usize) -> Console {
    Console::builder()
        .width(width)
        .force_terminal(true)
        .no_color(true)
        .markup(false)
        .build()
}

fn render_markdown(console: &Console, md: &Markdown) -> String {
    let opts = console.options();
    let segments = md.gilt_console(console, &opts);
    segments.iter().map(|s| s.text.as_str()).collect()
}

fn render_segments(console: &Console, md: &Markdown) -> Vec<Segment> {
    let opts = console.options();
    md.gilt_console(console, &opts)
}

// -- Simple paragraph ---------------------------------------------------

#[test]
fn test_simple_paragraph() {
    let console = make_console(80);
    let md = Markdown::new("Hello, world!");
    let output = render_markdown(&console, &md);
    assert!(output.contains("Hello, world!"));
}

#[test]
fn test_two_paragraphs() {
    let console = make_console(80);
    let md = Markdown::new("First paragraph.\n\nSecond paragraph.");
    let output = render_markdown(&console, &md);
    assert!(output.contains("First paragraph."));
    assert!(output.contains("Second paragraph."));
}

// -- Headings -----------------------------------------------------------

#[test]
fn test_heading_h1() {
    let console = make_console(80);
    let md = Markdown::new("# Heading 1");
    let output = render_markdown(&console, &md);
    assert!(output.contains("Heading 1"));
}

#[test]
fn test_heading_h2() {
    let console = make_console(80);
    let md = Markdown::new("## Heading 2");
    let output = render_markdown(&console, &md);
    assert!(output.contains("Heading 2"));
}

#[test]
fn test_heading_h3() {
    let console = make_console(80);
    let md = Markdown::new("### Heading 3");
    let output = render_markdown(&console, &md);
    assert!(output.contains("Heading 3"));
}

#[test]
fn test_heading_h4() {
    let console = make_console(80);
    let md = Markdown::new("#### Heading 4");
    let output = render_markdown(&console, &md);
    assert!(output.contains("Heading 4"));
}

#[test]
fn test_heading_h5() {
    let console = make_console(80);
    let md = Markdown::new("##### Heading 5");
    let output = render_markdown(&console, &md);
    assert!(output.contains("Heading 5"));
}

#[test]
fn test_heading_h6() {
    let console = make_console(80);
    let md = Markdown::new("###### Heading 6");
    let output = render_markdown(&console, &md);
    assert!(output.contains("Heading 6"));
}

#[test]
fn test_headings_have_appropriate_styles() {
    let console = make_console(80);

    // H1 should produce a rule underline
    let md = Markdown::new("# Title");
    let output = render_markdown(&console, &md);
    // H1 and H2 get underlines (Rule characters)
    assert!(output.contains("Title"));
    // The rule character should be present (─ U+2500 light, rich parity; '-' for ascii)
    assert!(output.contains('\u{2500}') || output.contains('-'));
}

#[test]
fn test_h1_has_rule_underline() {
    let console = make_console(40);
    let md = Markdown::new("# Big Title");
    let segments = render_segments(&console, &md);
    let text: String = segments.iter().map(|s| s.text.as_str()).collect();
    // Should contain the heading text and a rule line
    assert!(text.contains("Big Title"));
    // Rule default is now ─ (U+2500, light) for rich parity
    assert!(text.contains('\u{2500}'));
}

#[test]
fn test_h2_has_rule_underline() {
    let console = make_console(40);
    let md = Markdown::new("## Sub Title");
    let segments = render_segments(&console, &md);
    let text: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(text.contains("Sub Title"));
    // Rule default is now ─ (U+2500, light) for rich parity
    assert!(text.contains('\u{2500}'));
}

// -- Bold text ----------------------------------------------------------

#[test]
fn test_bold_text() {
    let console = make_console(80);
    let md = Markdown::new("This is **bold** text.");
    let segments = render_segments(&console, &md);
    let text: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(text.contains("bold"));
    assert!(text.contains("This is"));
    assert!(text.contains("text."));

    // Check that "bold" segment has a style with bold attribute
    let bold_seg = segments.iter().find(|s| s.text == "bold");
    assert!(bold_seg.is_some(), "Should have a segment with text 'bold'");
    if let Some(seg) = bold_seg {
        assert!(seg.style().is_some(), "Bold segment should have a style");
    }
}

// -- Italic text --------------------------------------------------------

#[test]
fn test_italic_text() {
    let console = make_console(80);
    let md = Markdown::new("This is *italic* text.");
    let segments = render_segments(&console, &md);
    let text: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(text.contains("italic"));

    let italic_seg = segments.iter().find(|s| s.text == "italic");
    assert!(
        italic_seg.is_some(),
        "Should have a segment with text 'italic'"
    );
    if let Some(seg) = italic_seg {
        assert!(seg.style().is_some(), "Italic segment should have a style");
    }
}

// -- Bold + italic combined ---------------------------------------------

#[test]
fn test_bold_italic_combined() {
    let console = make_console(80);
    let md = Markdown::new("This is ***bold and italic*** text.");
    let segments = render_segments(&console, &md);
    let text: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(text.contains("bold and italic"));

    // The combined text should have a style
    let combined_seg = segments.iter().find(|s| s.text.contains("bold and italic"));
    assert!(combined_seg.is_some());
    if let Some(seg) = combined_seg {
        assert!(
            seg.style().is_some(),
            "Bold+italic segment should have a style"
        );
    }
}

// -- Inline code --------------------------------------------------------

#[test]
fn test_inline_code() {
    let console = make_console(80);
    let md = Markdown::new("Use `println!` to print.");
    let segments = render_segments(&console, &md);
    let text: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(text.contains("println!"));

    let code_seg = segments.iter().find(|s| s.text == "println!");
    assert!(
        code_seg.is_some(),
        "Should have a segment with inline code text"
    );
    if let Some(seg) = code_seg {
        assert!(seg.style().is_some(), "Inline code should have a style");
    }
}

// -- Code blocks (fenced) -----------------------------------------------

#[test]
fn test_code_block() {
    let console = make_console(80);
    let md = Markdown::new("```\nfn main() {\n    println!(\"hello\");\n}\n```");
    let output = render_markdown(&console, &md);
    assert!(output.contains("fn main()"));
    assert!(output.contains("println!"));
}

#[test]
fn test_code_block_with_language() {
    let console = make_console(80);
    let md = Markdown::new("```rust\nlet x = 42;\n```");
    let output = render_markdown(&console, &md);
    assert!(output.contains("let x = 42;"));
}

// -- Links with URLs ----------------------------------------------------

#[test]
fn test_link_with_url() {
    // P1 parity: when hyperlinks=true (default), the link text is shown but the
    // URL is NOT appended inline — rich renders the text as a clickable OSC-8
    // hyperlink instead.  Only the link text should appear.
    let console = make_console(80);
    let md = Markdown::new("[Rust](https://www.rust-lang.org)");
    let output = render_markdown(&console, &md);
    assert!(output.contains("Rust"));
    // URL should NOT appear as plain text when hyperlinks are enabled
    assert!(!output.contains("https://www.rust-lang.org"));
}

#[test]
fn test_link_without_url_display() {
    // P1 parity: when hyperlinks=false, the URL IS shown inline as "(url)".
    let console = make_console(80);
    let md = Markdown::new("[Rust](https://www.rust-lang.org)").with_hyperlinks(false);
    let output = render_markdown(&console, &md);
    assert!(output.contains("Rust"));
    assert!(output.contains("https://www.rust-lang.org"));
}

// -- Unordered lists (bullets) ------------------------------------------

#[test]
fn test_unordered_list() {
    let console = make_console(80);
    let md = Markdown::new("- Item 1\n- Item 2\n- Item 3");
    let output = render_markdown(&console, &md);
    assert!(output.contains("Item 1"));
    assert!(output.contains("Item 2"));
    assert!(output.contains("Item 3"));
    // Should have bullet characters
    assert!(output.contains('\u{2022}'));
}

#[test]
fn test_flat_list_items_share_indent_not_progressive() {
    // Regression: flat sibling bullets must all share the SAME indent.
    // Previously each item's trailing continuation-indent leaked onto the
    // next sibling's bullet, making item 2/3 look nested under item 1.
    let console = make_console(80);
    let md = Markdown::new("- Item 1\n- Item 2\n- Item 3");
    let output = render_markdown(&console, &md);
    let leads: Vec<usize> = output
        .lines()
        .filter(|l| l.contains('\u{2022}') && l.contains("Item"))
        .map(|l| l.chars().take_while(|c| *c == ' ').count())
        .collect();
    assert_eq!(leads.len(), 3, "expected 3 bullet lines, got {leads:?}");
    assert!(
        leads.iter().all(|&n| n == leads[0]),
        "flat sibling bullets must share the same indent, got {leads:?}"
    );
}

// -- Ordered lists (numbers) --------------------------------------------

#[test]
fn test_ordered_list() {
    let console = make_console(80);
    let md = Markdown::new("1. First\n2. Second\n3. Third");
    let output = render_markdown(&console, &md);
    assert!(output.contains("First"));
    assert!(output.contains("Second"));
    assert!(output.contains("Third"));
    assert!(output.contains("1."));
    assert!(output.contains("2."));
    assert!(output.contains("3."));
}

// -- Nested lists -------------------------------------------------------

#[test]
fn test_nested_list() {
    let console = make_console(80);
    let md = Markdown::new("- Outer\n  - Inner 1\n  - Inner 2\n- Outer 2");
    let output = render_markdown(&console, &md);
    assert!(output.contains("Outer"));
    assert!(output.contains("Inner 1"));
    assert!(output.contains("Inner 2"));
    assert!(output.contains("Outer 2"));
}

// -- Block quotes -------------------------------------------------------

#[test]
fn test_block_quote() {
    let console = make_console(80);
    let md = Markdown::new("> This is a quote.");
    let output = render_markdown(&console, &md);
    assert!(output.contains("This is a quote."));
    // P2 parity: rich uses ▌ (U+258C left half block), NOT │ (U+2502)
    assert!(output.contains('\u{258C}'));
}

// -- Horizontal rules ---------------------------------------------------

#[test]
fn test_horizontal_rule() {
    let console = make_console(40);
    let md = Markdown::new("Above\n\n---\n\nBelow");
    let output = render_markdown(&console, &md);
    assert!(output.contains("Above"));
    assert!(output.contains("Below"));
    // Should contain rule characters (─ U+2500 light, rich parity default)
    assert!(output.contains('\u{2500}'));
}

// -- Mixed content ------------------------------------------------------

#[test]
fn test_mixed_content() {
    let console = make_console(80);
    let md = Markdown::new("# Title\n\nA paragraph.\n\n- Item 1\n- Item 2\n\n```\ncode\n```");
    let output = render_markdown(&console, &md);
    assert!(output.contains("Title"));
    assert!(output.contains("A paragraph."));
    assert!(output.contains("Item 1"));
    assert!(output.contains("Item 2"));
    assert!(output.contains("code"));
}

// -- Empty markdown -----------------------------------------------------

#[test]
fn test_empty_markdown() {
    let console = make_console(80);
    let md = Markdown::new("");
    let output = render_markdown(&console, &md);
    assert!(output.is_empty() || output.trim().is_empty());
}

// -- Tables -------------------------------------------------------------

#[test]
fn test_table() {
    let console = make_console(80);
    let md = Markdown::new("| Name | Age |\n|------|-----|\n| Alice | 30 |\n| Bob | 25 |");
    let output = render_markdown(&console, &md);
    assert!(output.contains("Name"));
    assert!(output.contains("Age"));
    assert!(output.contains("Alice"));
    assert!(output.contains("30"));
    assert!(output.contains("Bob"));
    assert!(output.contains("25"));
}

#[test]
fn test_table_with_alignment() {
    let console = make_console(80);
    let md = Markdown::new("| Left | Center | Right |\n|:-----|:------:|------:|\n| L | C | R |");
    let output = render_markdown(&console, &md);
    assert!(output.contains("Left"));
    assert!(output.contains("Center"));
    assert!(output.contains("Right"));
}

// -- Renderable trait integration ---------------------------------------

#[test]
fn test_renderable_integration() {
    let console = Console::builder()
        .width(60)
        .force_terminal(true)
        .no_color(true)
        .markup(false)
        .build();

    let md = Markdown::new("Hello, **world**!");
    let opts = console.options();
    let segments = md.gilt_console(&console, &opts);

    // Should produce non-empty output
    assert!(!segments.is_empty());

    let text: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(text.contains("Hello,"));
    assert!(text.contains("world"));
}

#[test]
fn test_renderable_through_console_render() {
    let console = Console::builder()
        .width(60)
        .force_terminal(true)
        .no_color(true)
        .markup(false)
        .build();

    let md = Markdown::new("# Title\n\nParagraph text.");
    let segments = console.render(&md, None);
    let text: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(text.contains("Title"));
    assert!(text.contains("Paragraph text."));
}

// -- Constructor / builder pattern --------------------------------------

#[test]
fn test_constructor_defaults() {
    let md = Markdown::new("test");
    assert_eq!(md.markup, "test");
    assert_eq!(md.code_theme, "monokai");
    assert!(md.inline_code_lexer.is_none());
    assert!(md.inline_code_theme.is_none());
    assert!(md.hyperlinks);
    assert!(md.justify.is_none());
}

#[test]
fn test_builder_code_theme() {
    let md = Markdown::new("test").with_code_theme("dracula");
    assert_eq!(md.code_theme, "dracula");
}

#[test]
fn test_builder_hyperlinks() {
    let md = Markdown::new("test").with_hyperlinks(false);
    assert!(!md.hyperlinks);
}

#[test]
fn test_builder_justify() {
    let md = Markdown::new("test").with_justify(JustifyMethod::Center);
    assert_eq!(md.justify, Some(JustifyMethod::Center));
}

// -- Strikethrough text -------------------------------------------------

#[test]
fn test_strikethrough() {
    let console = make_console(80);
    let md = Markdown::new("This is ~~deleted~~ text.");
    let segments = render_segments(&console, &md);
    let text: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(text.contains("deleted"));
}

// -- Soft and hard breaks -----------------------------------------------

#[test]
fn test_soft_break() {
    let console = make_console(80);
    let md = Markdown::new("Line one\nLine two");
    let output = render_markdown(&console, &md);
    // Soft break should become a space
    assert!(output.contains("Line one"));
    assert!(output.contains("Line two"));
}

#[test]
fn test_hard_break() {
    let console = make_console(80);
    let md = Markdown::new("Line one  \nLine two");
    let output = render_markdown(&console, &md);
    assert!(output.contains("Line one"));
    assert!(output.contains("Line two"));
}

// -- Multiple headings --------------------------------------------------

#[test]
fn test_all_heading_levels() {
    let console = make_console(80);
    let md = Markdown::new("# H1\n\n## H2\n\n### H3\n\n#### H4\n\n##### H5\n\n###### H6");
    let output = render_markdown(&console, &md);
    assert!(output.contains("H1"));
    assert!(output.contains("H2"));
    assert!(output.contains("H3"));
    assert!(output.contains("H4"));
    assert!(output.contains("H5"));
    assert!(output.contains("H6"));
}

// -- Width constraints --------------------------------------------------

#[test]
fn test_narrow_width() {
    let console = make_console(20);
    let md = Markdown::new("This is a paragraph with enough text to wrap.");
    let output = render_markdown(&console, &md);
    assert!(output.contains("This"));
    // Check that output lines are within the width constraint
    for line in output.split('\n') {
        if !line.is_empty() {
            assert!(
                cell_len(line) <= 20,
                "Line exceeds width: '{}' ({} cells)",
                line,
                cell_len(line)
            );
        }
    }
}

// -- Code block in panel ------------------------------------------------

#[test]
fn test_code_block_has_panel_border() {
    let console = make_console(40);
    let md = Markdown::new("```\nhello\n```");
    let output = render_markdown(&console, &md);
    // P1 parity: code blocks now use Syntax (when feature = "syntax") rather
    // than a HEAVY-bordered Panel, so we check for the code content instead.
    // The HEAVY box chars (┏/━/┃) are no longer present in the default config.
    assert!(
        output.contains("hello"),
        "Code block content should be present in output, got: {:?}",
        output
    );
}

// -- Multiple items with inline formatting ------------------------------

#[test]
fn test_list_with_inline_formatting() {
    let console = make_console(80);
    let md = Markdown::new("- **Bold item**\n- *Italic item*\n- `Code item`");
    let output = render_markdown(&console, &md);
    assert!(output.contains("Bold item"));
    assert!(output.contains("Italic item"));
    assert!(output.contains("Code item"));
}

// -- Edge case: only whitespace -----------------------------------------

#[test]
fn test_whitespace_only() {
    let console = make_console(80);
    let md = Markdown::new("   \n\n   ");
    let output = render_markdown(&console, &md);
    assert!(output.trim().is_empty());
}

// -- Blockquote with multiple paragraphs --------------------------------

#[test]
fn test_blockquote_multiple_paragraphs() {
    let console = make_console(80);
    let md = Markdown::new("> First quote.\n>\n> Second quote.");
    let output = render_markdown(&console, &md);
    assert!(output.contains("First quote."));
    assert!(output.contains("Second quote."));
}

// -- Image as link-like -------------------------------------------------

#[test]
fn test_image() {
    // P2 parity: images render with 🌆 prefix; when hyperlinks=true (default)
    // the URL is not shown as plain text (same logic as links).
    let console = make_console(80);
    let md = Markdown::new("![Alt text](https://example.com/image.png)");
    let output = render_markdown(&console, &md);
    assert!(output.contains("Alt text"));
    // With hyperlinks=true, URL not appended inline
    assert!(!output.contains("https://example.com/image.png"));
    // 🌆 prefix should appear
    assert!(output.contains('\u{1F306}'));
}

#[test]
fn test_image_hyperlinks_disabled_shows_url() {
    // When hyperlinks=false, image URL should appear inline as "(url)"
    let console = make_console(80);
    let md = Markdown::new("![Alt text](https://example.com/image.png)").with_hyperlinks(false);
    let output = render_markdown(&console, &md);
    assert!(output.contains("Alt text"));
    assert!(output.contains("https://example.com/image.png"));
}

// -- Verify output ends with newline ------------------------------------

#[test]
fn test_output_has_trailing_content() {
    let console = make_console(80);
    let md = Markdown::new("Hello");
    let segments = render_segments(&console, &md);
    assert!(!segments.is_empty());
}

#[test]
fn test_display_trait() {
    let md = Markdown::new("# Hello\n\nWorld");
    let s = format!("{}", md);
    assert!(!s.is_empty());
}

// -- Inline code in table cells (Rich v15.0.0 regression guard) ----------

#[test]
fn inline_code_in_table_cell_is_rendered() {
    // Regression guard: inline code inside a table cell must not be dropped.
    let console = make_console(80);
    let md = Markdown::new("| Col | Code |\n|---|---|\n| a | `foo` |");
    let output = render_markdown(&console, &md);
    assert!(
        output.contains("foo"),
        "Expected 'foo' to appear in table output, got: {:?}",
        output
    );
}

#[test]
fn inline_code_in_table_cell_keeps_style() {
    // The inline-code segment inside a table cell must carry the
    // markdown.code style (rendered as a styled segment, not plain text).
    let console = make_console(80);
    let md = Markdown::new("| Col | Code |\n|---|---|\n| a | `foo` |");
    let segments = render_segments(&console, &md);

    let text: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(text.contains("foo"), "Expected 'foo' in output");

    let code_seg = segments.iter().find(|s| s.text.contains("foo"));
    assert!(code_seg.is_some(), "Expected a segment containing 'foo'");
    assert!(
        code_seg.unwrap().style.is_some(),
        "Inline code in table cell must carry a style"
    );
}

#[test]
fn inline_code_outside_table_still_works() {
    // Regression guard: inline code outside tables must remain unaffected.
    let console = make_console(80);
    let md = Markdown::new("Use `println!` to print.");
    let segments = render_segments(&console, &md);
    let text: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(text.contains("println!"));

    let code_seg = segments.iter().find(|s| s.text == "println!");
    assert!(
        code_seg.is_some(),
        "Should have a segment with text 'println!'"
    );
    assert!(
        code_seg.unwrap().style.is_some(),
        "Inline code outside table should still have a style"
    );
}

#[test]
fn inline_code_between_text_in_table_cell_preserves_order() {
    // Ordering guard: inline code sandwiched between plain text in a cell
    // must appear at its correct position, not appended after surrounding text.
    let console = make_console(80);
    let md = Markdown::new("| x |\n|---|\n| pre `mid` post |");
    let output = render_markdown(&console, &md);
    let pre_pos = output.find("pre");
    let mid_pos = output.find("mid");
    let post_pos = output.find("post");
    assert!(
        pre_pos.is_some() && mid_pos.is_some() && post_pos.is_some(),
        "Expected 'pre', 'mid', and 'post' in output, got: {:?}",
        output
    );
    assert!(
        pre_pos.unwrap() < mid_pos.unwrap() && mid_pos.unwrap() < post_pos.unwrap(),
        "Expected order pre < mid < post, got positions: pre={:?} mid={:?} post={:?}",
        pre_pos,
        mid_pos,
        post_pos
    );
}

// -- Task 2: GFM Task-list rendering ----------------------------------------

/// Task-list markdown is enabled via `ENABLE_TASKLISTS`.  pulldown-cmark
/// emits `Event::TaskListMarker(checked)` events which we render as ☑/☐.
#[test]
fn test_task_list_checked_item_renders_checked_box() {
    let console = make_console(80);
    let md = Markdown::new("- [x] done");
    let output = render_markdown(&console, &md);
    assert!(
        output.contains('\u{2611}'),
        "expected ☑ (U+2611) for checked item; got: {:?}",
        output
    );
    assert!(
        output.contains("done"),
        "expected 'done' text; got: {:?}",
        output
    );
}

#[test]
fn test_task_list_unchecked_item_renders_unchecked_box() {
    let console = make_console(80);
    let md = Markdown::new("- [ ] todo");
    let output = render_markdown(&console, &md);
    assert!(
        output.contains('\u{2610}'),
        "expected ☐ (U+2610) for unchecked item; got: {:?}",
        output
    );
    assert!(
        output.contains("todo"),
        "expected 'todo' text; got: {:?}",
        output
    );
}

#[test]
fn test_task_list_mixed_items() {
    let console = make_console(80);
    let md = Markdown::new("- [x] done\n- [ ] todo");
    let output = render_markdown(&console, &md);
    assert!(
        output.contains('\u{2611}'),
        "expected ☑ in mixed list; got: {:?}",
        output
    );
    assert!(
        output.contains('\u{2610}'),
        "expected ☐ in mixed list; got: {:?}",
        output
    );
    assert!(output.contains("done"));
    assert!(output.contains("todo"));
}

#[test]
fn test_task_list_checked_comes_before_unchecked() {
    let console = make_console(80);
    let md = Markdown::new("- [x] first\n- [ ] second");
    let output = render_markdown(&console, &md);
    let checked_pos = output.find('\u{2611}').expect("☑ not found");
    let unchecked_pos = output.find('\u{2610}').expect("☐ not found");
    assert!(
        checked_pos < unchecked_pos,
        "☑ should precede ☐; checked={}, unchecked={}",
        checked_pos,
        unchecked_pos
    );
}

// ---------------------------------------------------------------------------
// Phase 7.2: 7 parity fixes
// ---------------------------------------------------------------------------

// Item 1: OSC-8 hyperlinks
#[test]
fn test_markdown_osc8_link() {
    // When hyperlinks=true (default), link text spans must carry an OSC-8 link style.
    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(true)
        .markup(false)
        .build();
    let md = Markdown::new("[Click me](https://example.com)");
    let segments = render_segments(&console, &md);
    // Find a segment containing the link text
    let link_seg = segments.iter().find(|s| s.text.contains("Click me"));
    assert!(link_seg.is_some(), "link text segment should be present");
    let seg = link_seg.unwrap();
    assert!(
        seg.style().is_some(),
        "link text should have a style (carries OSC-8)"
    );
    let style = seg.style().unwrap();
    assert!(
        style.link().is_some(),
        "link style must have OSC-8 URL set; got style: {:?}",
        style
    );
    assert_eq!(
        style.link().as_deref(),
        Some("https://example.com"),
        "OSC-8 URL must match the href"
    );
}

#[test]
fn test_markdown_osc8_link_disabled() {
    // When hyperlinks=false, no OSC-8 link should be set on segments.
    let console = make_console(80);
    let md = Markdown::new("[Click me](https://example.com)").with_hyperlinks(false);
    let segments = render_segments(&console, &md);
    let link_seg = segments.iter().find(|s| s.text.contains("Click me"));
    assert!(link_seg.is_some(), "link text should appear");
    // No OSC-8 link on segment when hyperlinks=false
    if let Some(seg) = link_seg {
        if let Some(ref style) = seg.style {
            assert!(
                style.link().is_none(),
                "hyperlinks=false must not emit OSC-8; got link: {:?}",
                style.link()
            );
        }
    }
}

// Item 2: inline_code_lexer (syntax-feature gated)
#[test]
#[cfg(feature = "syntax")]
fn test_markdown_inline_code_lexer() {
    // When inline_code_lexer is set, inline code should produce multiple styled spans
    // (one per token) rather than a single plain-styled span.
    let console = make_console(80);
    let mut md = Markdown::new("Use `let x = 1` here.");
    md.inline_code_lexer = Some("rust".to_string());
    let segments = render_segments(&console, &md);
    // At minimum, the code text should appear somewhere in the output.
    let output: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(
        output.contains("let") || output.contains("x") || output.contains("1"),
        "syntax-highlighted inline code should appear in output; got: {:?}",
        output
    );
    // There should be multiple styled segments for the code (more than 1 token).
    let code_segs: Vec<_> = segments
        .iter()
        .filter(|s| {
            (s.text.contains("let") || s.text.contains("x") || s.text.contains("1"))
                && s.style().is_some()
        })
        .collect();
    // syntax highlighting must produce ≥2 styled token segments
    assert!(
        code_segs.len() >= 2,
        "inline code with lexer should produce ≥2 styled spans (real highlighting vs plain fallback); got: {}",
        code_segs.len()
    );
}

// Item 3: blockquote non-paragraph blocks miss ▌ prefix
#[test]
fn test_markdown_blockquote_heading_prefix() {
    let console = make_console(80);
    let md = Markdown::new("> # Heading in quote");
    let output = render_markdown(&console, &md);
    assert!(
        output.contains("Heading in quote"),
        "heading text should appear; got: {:?}",
        output
    );
    // The blockquote prefix ▌ (U+258C) must appear in the same output as the heading.
    assert!(
        output.contains('\u{258C}'),
        "blockquote prefix ▌ should appear before heading; got: {:?}",
        output
    );
}

#[test]
fn test_markdown_blockquote_hr_prefix() {
    let console = make_console(80);
    let md = Markdown::new("> ---");
    let output = render_markdown(&console, &md);
    // ▌ prefix must appear alongside the horizontal rule inside a blockquote.
    assert!(
        output.contains('\u{258C}'),
        "blockquote prefix ▌ should appear before HR; got: {:?}",
        output
    );
}

// Item 4: table pad_edge + collapse_padding
// (structural rendering test — just verify no panic and output is correct)
#[test]
fn test_markdown_table_pad_edge() {
    let console = make_console(80);
    let md = Markdown::new("| A | B |\n|---|---|\n| 1 | 2 |");
    let output = render_markdown(&console, &md);
    // Table must still render correctly after applying pad_edge(false)+collapse_padding(true).
    assert!(output.contains("A"), "header A should appear");
    assert!(output.contains("B"), "header B should appear");
    assert!(output.contains("1"), "cell 1 should appear");
    assert!(output.contains("2"), "cell 2 should appear");
}

// Item 5: table header cells preserve styled Text
#[test]
fn test_markdown_table_header_styled() {
    // Headers with bold/italic markdown should preserve inline styling.
    let console = make_console(80);
    let md = Markdown::new("| **Bold** | *Italic* |\n|---|---|\n| a | b |");
    let segments = render_segments(&console, &md);
    let output: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(
        output.contains("Bold"),
        "bold header text should appear; got: {:?}",
        output
    );
    assert!(
        output.contains("Italic"),
        "italic header text should appear; got: {:?}",
        output
    );
    // The "Bold" segment should carry a bold style.
    let bold_seg = segments.iter().find(|s| s.text.contains("Bold"));
    assert!(bold_seg.is_some(), "Bold segment should be present");
    if let Some(seg) = bold_seg {
        assert!(
            seg.style().is_some(),
            "Bold header should have a style applied"
        );
    }
}

// Item 6: ordered list right-alignment
#[test]
fn test_markdown_ordered_list_alignment() {
    let console = make_console(80);
    let md = Markdown::new("1. First\n2. Second\n3. Third");
    let segments = render_segments(&console, &md);
    // Each item number should appear with a "." after it.
    let output: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(output.contains("1."), "item 1 should appear");
    assert!(output.contains("2."), "item 2 should appear");
    assert!(output.contains("3."), "item 3 should appear");
    // All number prefixes should have the same display width.
    let prefix_segs: Vec<&str> = segments
        .iter()
        .filter_map(|s| {
            if s.text.contains(". ")
                && (s.text.contains('1') || s.text.contains('2') || s.text.contains('3'))
            {
                Some(s.text.as_str())
            } else {
                None
            }
        })
        .collect();
    // Each prefix segment (e.g. "1. ") should have the same length for single-digit lists.
    if prefix_segs.len() >= 2 {
        let first_len = prefix_segs[0].len();
        for seg in &prefix_segs {
            assert_eq!(
                seg.len(),
                first_len,
                "all item number prefixes should have equal width; got {:?}",
                prefix_segs
            );
        }
    }
}

// Item 7: image no-alt blank fallback
#[test]
fn test_markdown_image_filename_fallback() {
    // An image with empty alt text should display the filename stem.
    let console = make_console(80);
    let md = Markdown::new("![](logo.png)");
    let output = render_markdown(&console, &md);
    assert!(
        output.contains("logo"),
        "filename stem 'logo' should appear as alt-text fallback; got: {:?}",
        output
    );
}

#[test]
fn test_markdown_image_filename_fallback_path() {
    // Should strip directory and extension from a URL path.
    let console = make_console(80);
    let md = Markdown::new("![](/images/photos/sunset.jpg)");
    let output = render_markdown(&console, &md);
    assert!(
        output.contains("sunset"),
        "filename stem 'sunset' should appear as fallback; got: {:?}",
        output
    );
}

#[test]
fn test_markdown_image_with_alt_not_overridden() {
    // When alt text is present, it should be used (not the filename).
    let console = make_console(80);
    let md = Markdown::new("![Beautiful sunset](sunset.jpg)");
    let output = render_markdown(&console, &md);
    assert!(
        output.contains("Beautiful sunset"),
        "explicit alt text should appear; got: {:?}",
        output
    );
}

// ---------------------------------------------------------------------------
// Reviewer fixes (task-7.2 post-review pass)
// ---------------------------------------------------------------------------

// Item 3 follow-up: list items inside blockquotes must get the ▌ prefix.
#[test]
fn test_markdown_blockquote_list_prefix() {
    // A bullet list nested inside a blockquote must have ▌ prefixed on each
    // item line, matching rich's behaviour for block elements in blockquotes.
    let console = make_console(80);
    let md = Markdown::new("> - alpha\n> - beta\n> - gamma");
    let output = render_markdown(&console, &md);
    assert!(
        output.contains("alpha"),
        "list item 'alpha' should appear; got: {:?}",
        output
    );
    assert!(
        output.contains("beta"),
        "list item 'beta' should appear; got: {:?}",
        output
    );
    // Every item line must be prefixed with ▌ (U+258C).
    assert!(
        output.contains('\u{258C}'),
        "blockquote prefix ▌ must appear for list items inside a blockquote; got: {:?}",
        output
    );
    // The bullet character (•) must also be present.
    assert!(
        output.contains('\u{2022}'),
        "bullet • must appear for unordered list items; got: {:?}",
        output
    );
}

// Item 6 follow-up: ordered list right-alignment uses list-max-width, not per-item width.
#[test]
fn test_markdown_ordered_list_alignment_10_items() {
    // A 10-item ordered list must right-align all numbers to 2 digits.
    // Items 1–9 must render as " 1." … " 9." and item 10 as "10.".
    let console = make_console(80);
    let md_src = (1u32..=10)
        .map(|i| format!("{}. item{}", i, i))
        .collect::<Vec<_>>()
        .join("\n");
    let md = Markdown::new(&md_src);
    let segments = render_segments(&console, &md);
    let output: String = segments.iter().map(|s| s.text.as_str()).collect();

    // All ten items must appear in the output.
    for i in 1u32..=10 {
        assert!(
            output.contains(&format!("item{}", i)),
            "item{} should appear in output; got: {:?}",
            i,
            output
        );
    }

    // Collect the ordered-list prefix segments (those that contain ". ").
    // In the no-color console these are plain-text styled segments.
    let prefix_segs: Vec<&str> = segments
        .iter()
        .filter_map(|s| {
            if s.text.contains(". ")
                && s.text
                    .trim()
                    .split('.')
                    .next()
                    .map_or(false, |n| n.trim().parse::<u32>().is_ok())
            {
                Some(s.text.as_str())
            } else {
                None
            }
        })
        .collect();

    assert!(
        prefix_segs.len() >= 10,
        "expected at least 10 number-prefix segments; got {:?}",
        prefix_segs
    );

    // All prefix segments must have the same display width (right-aligned to 2 digits).
    let first_len = prefix_segs[0].len();
    for seg in &prefix_segs {
        assert_eq!(
            seg.len(),
            first_len,
            "all number prefixes must have equal width for right-alignment; got {:?}",
            prefix_segs
        );
    }

    // Item 1 must be padded: " 1. " (leading space) not "1. ".
    let item1_prefix = prefix_segs
        .iter()
        .find(|s| s.contains("1."))
        .copied()
        .unwrap_or("");
    assert!(
        item1_prefix.starts_with(' '),
        "item 1 prefix should be right-padded to width 2 (e.g. ' 1. '); got: {:?}",
        item1_prefix
    );
}

// ---------------------------------------------------------------------------
// Nested ordered list alignment (pre-pass order bug fix)
// ---------------------------------------------------------------------------

/// Regression guard: for nested ordered lists the pre-pass must push
/// max_digits in list-OPEN order so the render pass (which consumes in
/// open-order) reads the correct value for each list.
///
/// Outer list: 2 items → max_digits = 1 (numbers fit in 1 char)
/// Inner list (nested inside outer item 1): 10 items → max_digits = 2
///
/// Before the fix the pre-pass pushed in End-order (inner closes first),
/// so ordered_list_max_digits = [inner=2, outer=1].  The render pass then
/// used 2 for the outer list (wrong — outer items got leading spaces) and
/// 1 for the inner list (wrong — "10." rendered with max_digits=1, so items
/// 1–9 were not padded to width 2 with a leading space).
///
/// NOTE: The pre-existing rendering bug (outer item text lost when a nested
/// list immediately follows) is separate and not tested here. This test
/// focuses solely on the max_digits assignment for the inner list.
#[test]
fn test_markdown_nested_ordered_list_alignment() {
    // Build markdown with:
    //   - An outer list with 2 items (max_digits should be 1)
    //   - The first outer item contains a nested 10-item list (max_digits should be 2)
    //
    // With the buggy pre-pass (End-order push), outer gets max_digits=2 and
    // inner gets max_digits=1. With the correct fix (Start-order push), the
    // inner list correctly gets max_digits=2.
    //
    // We detect the bug by checking whether inner item "1." is right-padded
    // to 2 digits (i.e. the prefix contains " 1." with a leading space).
    let inner_items: String = (1u32..=10)
        .map(|i| format!("   {}. Inner {}", i, i))
        .collect::<Vec<_>>()
        .join("\n");
    let md_src = format!("1. Outer item one\n{}\n2. Outer item two", inner_items);

    let console = make_console(120);
    let md = Markdown::new(&md_src);
    let segments = render_segments(&console, &md);
    let output: String = segments.iter().map(|s| s.text.as_str()).collect();

    // Inner items must appear in output (outer item text loss is a separate bug).
    for i in 1u32..=10 {
        assert!(
            output.contains(&format!("Inner {}", i)),
            "Inner {} missing from output; output: {:?}",
            i,
            output
        );
    }

    // Collect the number-prefix segments emitted by the ordered list renderer.
    // These are styled segments of the form "<indent><digits>. " (e.g. "    1. ").
    let prefix_segs: Vec<&str> = segments
        .iter()
        .filter_map(|s| {
            if s.text.contains(". ")
                && s.text
                    .trim()
                    .split('.')
                    .next()
                    .map_or(false, |n| n.trim().parse::<u32>().is_ok())
            {
                Some(s.text.as_str())
            } else {
                None
            }
        })
        .collect();

    // We expect at least 10 inner prefix segments.
    assert!(
        prefix_segs.len() >= 10,
        "expected ≥10 prefix segments (10 inner); got {:?}",
        prefix_segs
    );

    // --- Inner list: max_digits = 2 ---
    // The inner list has 10 items, so max_digits = 2.
    // Items 1–9 must be padded: prefix contains " 1." not "1." in the number field.
    // The inner items are at indent_level=1 (4 spaces indent).
    // The styled prefix segment for inner item N looks like "    <num>. "
    // where <num> is right-aligned in a max_digits=2 field: " 1", " 2", …, "10".
    let inner_prefix_segs: Vec<&str> = prefix_segs
        .iter()
        .filter(|s| s.starts_with("    "))
        .copied()
        .collect();

    assert!(
        inner_prefix_segs.len() >= 10,
        "expected ≥10 inner prefix segments (with 4-space indent); got {:?}",
        inner_prefix_segs
    );

    // All inner prefix segments must have the same total length (right-aligned to 2 digits).
    let first_inner_len = inner_prefix_segs[0].len();
    for seg in &inner_prefix_segs {
        assert_eq!(
            seg.len(),
            first_inner_len,
            "inner list prefix widths must all be equal for right-alignment; got {:?}",
            inner_prefix_segs
        );
    }

    // Inner item 1 must be padded to max_digits=2: the number field is " 1" not "1".
    // After stripping the 4-space indent, the next char must be a space (the padding).
    let inner1 = inner_prefix_segs
        .iter()
        .find(|s| {
            // The segment for item 1 is "    <pad>1. " — find it by the trailing "1. "
            s.trim_end().ends_with("1.")
        })
        .copied()
        .unwrap_or("");
    let after_indent = inner1.trim_start_matches("    ");
    assert!(
        after_indent.starts_with(' '),
        "inner item 1 must be right-padded to max_digits=2 (e.g. '     1. '); \
         got prefix {:?}; all inner prefixes: {:?}",
        inner1,
        inner_prefix_segs
    );
}

// Item 2 strengthened: inline_code_lexer must produce ≥2 styled token segments.
#[test]
#[cfg(feature = "syntax")]
fn test_markdown_inline_code_lexer_multi_token() {
    // When inline_code_lexer is set, inline code should produce multiple styled spans
    // (one per token) rather than a single plain-styled span.
    let console = make_console(80);
    let mut md = Markdown::new("Use `let x = 1` here.");
    md.inline_code_lexer = Some("rust".to_string());
    let segments = render_segments(&console, &md);
    // At minimum, the code text should appear somewhere in the output.
    let output: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(
        output.contains("let") || output.contains("x") || output.contains("1"),
        "syntax-highlighted inline code should appear in output; got: {:?}",
        output
    );
    // There should be multiple styled segments for the code (more than 1 token).
    // syntax highlighting must produce ≥2 styled token segments
    let code_segs: Vec<_> = segments
        .iter()
        .filter(|s| {
            (s.text.contains("let") || s.text.contains("x") || s.text.contains("1"))
                && s.style().is_some()
        })
        .collect();
    assert!(
        code_segs.len() >= 2,
        "inline code with lexer should produce ≥2 styled spans (real highlighting vs plain fallback); got: {}",
        code_segs.len()
    );
}

// -- Trailing blockquote prefix -----------------------------------------

/// The blockquote prefix `▌ ` must only appear before content lines, never
/// as a dangling trailing prefix after the block's final newline.
fn blockquote_output(md: &Markdown) -> String {
    let console = make_console(80);
    render_markdown(&console, md)
}

/// Check if the *last* non-empty line in `output` is a dangling blockquote
/// prefix — i.e. it consists solely of the ▌ glyph and whitespace, with no
/// content following it.  This is the rich-parity bug: the prefix is emitted
/// after the block's final newline, producing a trailing `▌ ` line.
fn last_line_is_dangling_prefix(output: &str) -> bool {
    let last_non_empty = output.lines().rev().find(|line| !line.trim().is_empty());
    match last_non_empty {
        Some(line) => {
            let trimmed = line.trim_end();
            !trimmed.is_empty()
                && trimmed
                    .chars()
                    .all(|c| c.is_whitespace() || c == '\u{258C}')
                && trimmed.contains('\u{258C}')
        }
        None => false,
    }
}

#[test]
fn test_blockquote_paragraph_no_trailing_prefix() {
    let output = blockquote_output(&Markdown::new("> This is a quote."));
    assert!(
        output.contains("This is a quote."),
        "content should appear; got: {:?}",
        output
    );
    assert!(
        !last_line_is_dangling_prefix(&output),
        "no trailing dangling ▌ prefix after paragraph; got: {:?}",
        output
    );
}

#[test]
fn test_blockquote_heading_no_trailing_prefix() {
    let output = blockquote_output(&Markdown::new("> # Heading in quote"));
    assert!(
        output.contains("Heading in quote"),
        "heading text should appear; got: {:?}",
        output
    );
    assert!(
        !last_line_is_dangling_prefix(&output),
        "no trailing dangling ▌ prefix after heading; got: {:?}",
        output
    );
}

#[test]
fn test_blockquote_code_block_no_trailing_prefix() {
    let output = blockquote_output(&Markdown::new("> ```\n> code\n> ```"));
    assert!(
        output.contains("code"),
        "code content should appear; got: {:?}",
        output
    );
    assert!(
        !last_line_is_dangling_prefix(&output),
        "no trailing dangling ▌ prefix after code block; got: {:?}",
        output
    );
}

#[test]
fn test_blockquote_list_no_trailing_prefix() {
    let output = blockquote_output(&Markdown::new("> - Item 1\n> - Item 2"));
    assert!(
        output.contains("Item 1"),
        "list item 1 should appear; got: {:?}",
        output
    );
    assert!(
        !last_line_is_dangling_prefix(&output),
        "no trailing dangling ▌ prefix after list; got: {:?}",
        output
    );
}

#[test]
fn test_blockquote_hr_no_trailing_prefix() {
    let output = blockquote_output(&Markdown::new("> ---"));
    assert!(
        !last_line_is_dangling_prefix(&output),
        "no trailing dangling ▌ prefix after HR; got: {:?}",
        output
    );
}

#[test]
fn test_blockquote_multi_paragraph_no_trailing_prefix() {
    let output = blockquote_output(&Markdown::new("> First quote.\n>\n> Second quote."));
    assert!(
        output.contains("First quote."),
        "first paragraph should appear; got: {:?}",
        output
    );
    assert!(
        output.contains("Second quote."),
        "second paragraph should appear; got: {:?}",
        output
    );
    assert!(
        !last_line_is_dangling_prefix(&output),
        "no trailing dangling ▌ prefix after multi-paragraph blockquote; got: {:?}",
        output
    );
}

// -- Image OSC-8 link covers prefix glyph --------------------------------

/// When `hyperlinks=true`, the image's `🌆` prefix glyph must also carry the
/// OSC-8 link, not just the alt text.  rich wraps the whole image
/// representation (prefix + alt) in the hyperlink.
#[test]
fn test_image_hyperlinks_prefix_has_link() {
    let console = make_console(80);
    let md = Markdown::new("![Alt text](https://example.com/image.png)");
    let segments = render_segments(&console, &md);
    // Find the 🌆 prefix segment
    let prefix_seg = segments.iter().find(|s| s.text.contains('\u{1F306}'));
    assert!(
        prefix_seg.is_some(),
        "🌆 prefix segment should exist; segments: {:?}",
        segments.iter().map(|s| &s.text).collect::<Vec<_>>()
    );
    if let Some(seg) = prefix_seg {
        let link = seg.style().and_then(|s| s.link());
        assert!(
            link.is_some(),
            "🌆 prefix should have OSC-8 link when hyperlinks=true; got style: {:?}",
            seg.style()
        );
        assert_eq!(
            link,
            Some("https://example.com/image.png"),
            "🌆 prefix link should be the image URL"
        );
    }
    // Also verify the alt text segment has the link
    let alt_seg = segments.iter().find(|s| s.text.contains("Alt text"));
    assert!(alt_seg.is_some(), "alt text segment should exist");
    if let Some(seg) = alt_seg {
        let link = seg.style().and_then(|s| s.link());
        assert_eq!(
            link,
            Some("https://example.com/image.png"),
            "alt text should have OSC-8 link"
        );
    }
}
