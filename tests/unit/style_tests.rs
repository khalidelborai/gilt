//! Style module tests
//!
//! Extracted from src/style.rs

use super::*;
use crate::color::{Color, ColorSystem};
use crate::style::*;

// Display tests
#[test]
fn test_display_not_bold() {
    let style = Style::new(
        None,
        None,
        Some(false),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(style.to_string(), "not bold");
}

#[test]
fn test_display_not_bold_with_color() {
    let style = Style::new(
        Some("red"),
        None,
        Some(false),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(style.to_string(), "not bold red");
}

#[test]
fn test_display_null() {
    let style = Style::null();
    assert_eq!(style.to_string(), "none");
}

#[test]
fn test_display_bold() {
    let style = Style::new(
        None,
        None,
        Some(true),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(style.to_string(), "bold");
}

#[test]
fn test_display_bold_red_on_black() {
    let style = Style::new(
        Some("red"),
        Some("black"),
        Some(true),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(style.to_string(), "bold red on black");
}

#[test]
fn test_display_link() {
    let style = Style::new(
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some("foo"),
    )
    .unwrap();
    assert_eq!(style.to_string(), "link foo");
}

#[test]
fn test_display_all_attributes() {
    let style = Style::new(
        Some("red"),
        Some("black"),
        Some(true),
        Some(true),
        Some(true),
        Some(true),
        Some(true),
        Some(true),
        Some(true),
        Some(true),
        Some(true),
        Some(true),
        Some(true),
        Some(true),
        Some(true),
        None,
    )
    .unwrap();
    let s = style.to_string();
    assert!(s.contains("bold"));
    assert!(s.contains("dim"));
    assert!(s.contains("italic"));
    assert!(s.contains("underline"));
    assert!(s.contains("blink"));
    assert!(s.contains("blink2"));
    assert!(s.contains("reverse"));
    assert!(s.contains("conceal"));
    assert!(s.contains("strike"));
    assert!(s.contains("underline2"));
    assert!(s.contains("frame"));
    assert!(s.contains("encircle"));
    assert!(s.contains("overline"));
    assert!(s.contains("red"));
    assert!(s.contains("on black"));
}

// Equality tests
#[test]
fn test_equality_same() {
    let style1 = Style::new(
        Some("red"),
        None,
        Some(true),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    let style2 = Style::new(
        Some("red"),
        None,
        Some(true),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(style1, style2);
}

#[test]
fn test_equality_different_color() {
    let style1 = Style::new(
        Some("red"),
        None,
        Some(true),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    let style2 = Style::new(
        Some("green"),
        None,
        Some(true),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_ne!(style1, style2);
}

// is_null tests
#[test]
fn test_is_null_true() {
    let style = Style::null();
    assert!(style.is_null());
}

#[test]
fn test_is_null_false_with_bold() {
    let style = Style::new(
        None,
        None,
        Some(true),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert!(!style.is_null());
}

#[test]
fn test_is_null_false_with_color() {
    let style = Style::new(
        Some("red"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert!(!style.is_null());
}

// Parse tests
#[test]
fn test_parse_empty() {
    let style = Style::parse("").unwrap();
    assert!(style.is_null());
}

#[test]
fn test_parse_red() {
    let style = Style::parse("red").unwrap();
    assert_eq!(style.color().unwrap().name, "red");
}

#[test]
fn test_parse_not_bold() {
    let style = Style::parse("not bold").unwrap();
    assert_eq!(style.bold(), Some(false));
}

#[test]
fn test_parse_bold_red_on_black() {
    let style = Style::parse("bold red on black").unwrap();
    assert_eq!(style.bold(), Some(true));
    assert_eq!(style.color().unwrap().name, "red");
    assert_eq!(style.bgcolor().unwrap().name, "black");
}

#[test]
fn test_parse_bold_link() {
    let style = Style::parse("bold link https://example.org").unwrap();
    assert_eq!(style.bold(), Some(true));
    assert_eq!(style.link(), Some("https://example.org"));
}

#[test]
fn test_parse_error_on_alone() {
    let result = Style::parse("on");
    assert!(result.is_err());
}

#[test]
fn test_parse_error_on_invalid_color() {
    let result = Style::parse("on nothing");
    assert!(result.is_err());
}

#[test]
fn test_parse_error_rgb_out_of_range() {
    let result = Style::parse("rgb(999,999,999)");
    assert!(result.is_err());
}

#[test]
fn test_parse_error_not_unknown_attribute() {
    let result = Style::parse("not monkey");
    assert!(result.is_err());
}

#[test]
fn test_parse_error_link_alone() {
    let result = Style::parse("link");
    assert!(result.is_err());
}

// Render tests
#[test]
fn test_render_no_color_system() {
    let style = Style::parse("red").unwrap();
    assert_eq!(style.render("foo", None), "foo");
}

#[test]
fn test_render_empty_text() {
    let style = Style::parse("red").unwrap();
    assert_eq!(style.render("", Some(ColorSystem::TrueColor)), "");
}

#[test]
fn test_render_null_style() {
    let style = Style::null();
    assert_eq!(style.render("foo", Some(ColorSystem::TrueColor)), "foo");
}

#[test]
fn test_render_bold_red_on_black() {
    let style = Style::parse("bold red on black").unwrap();
    let rendered = style.render("foo", Some(ColorSystem::TrueColor));
    assert!(rendered.contains("\x1b[1;31;40m"));
    assert!(rendered.contains("foo"));
    assert!(rendered.contains("\x1b[0m"));
}

#[test]
fn test_render_all_attributes() {
    let style = Style::parse(
        "bold dim italic underline blink blink2 reverse conceal strike underline2 frame encircle overline red on black"
    ).unwrap();
    let rendered = style.render("foo", Some(ColorSystem::TrueColor));
    assert!(rendered.contains("1;2;3;4;5;6;7;8;9;21;51;52;53;31;40"));
}

// Add tests
#[test]
fn test_add_with_none() {
    let style = Style::parse("red").unwrap();
    let result = style.clone() + None;
    assert_eq!(result, style);
}

#[test]
fn test_add_styles() {
    let style1 = Style::parse("red").unwrap();
    let style2 = Style::parse("bold").unwrap();
    let result = style1 + style2;
    assert_eq!(result.color().unwrap().name, "red");
    assert_eq!(result.bold(), Some(true));
}

#[test]
fn test_add_override_color() {
    let style1 = Style::parse("red").unwrap();
    let style2 = Style::parse("blue").unwrap();
    let result = style1 + style2;
    assert_eq!(result.color().unwrap().name, "blue");
}

// StyleStack tests
#[test]
fn test_style_stack_new() {
    let stack = StyleStack::new(Style::parse("red").unwrap());
    assert_eq!(stack.current().color().unwrap().name, "red");
}

#[test]
fn test_style_stack_push() {
    let mut stack = StyleStack::new(Style::parse("red").unwrap());
    stack.push(Style::parse("bold").unwrap());
    assert_eq!(stack.current().color().unwrap().name, "red");
    assert_eq!(stack.current().bold(), Some(true));
}

#[test]
fn test_style_stack_pop() {
    let mut stack = StyleStack::new(Style::parse("red").unwrap());
    stack.push(Style::parse("bold").unwrap());
    stack.pop().unwrap();
    assert_eq!(stack.current().color().unwrap().name, "red");
    assert_eq!(stack.current().bold(), None);
}

#[test]
fn test_style_stack_pop_error() {
    let mut stack = StyleStack::new(Style::null());
    let result = stack.pop();
    assert!(result.is_err());
}

// HTML style tests
#[test]
fn test_get_html_style_complex() {
    let style =
        Style::parse("reverse dim red on blue bold italic underline strike overline").unwrap();
    let html = style.get_html_style(None);
    // With reverse: blue becomes fg, red becomes bg
    // With dim: blend blue (0,0,128) with red (128,0,0) at 50% = (64,0,64) = #400040
    assert!(html.contains("color: #400040"));
    assert!(html.contains("text-decoration-color: #400040"));
    assert!(html.contains("background-color: #800000"));
    assert!(html.contains("font-weight: bold"));
    assert!(html.contains("font-style: italic"));
    assert!(html.contains("text-decoration: underline line-through overline"));
}

#[test]
fn test_get_html_style_simple() {
    let style = Style::parse("bold red").unwrap();
    let html = style.get_html_style(None);
    assert!(html.contains("color: #800000"));
    assert!(html.contains("font-weight: bold"));
}

// without_color tests
#[test]
fn test_without_color() {
    let style = Style::parse("bold red on blue").unwrap();
    let without = style.without_color();
    assert_eq!(without.bold(), Some(true));
    assert!(without.color().is_none());
    assert!(without.bgcolor().is_none());
}

// background_style tests
#[test]
fn test_background_style() {
    let style = Style::parse("bold yellow on red").unwrap();
    let bg = style.background_style();
    assert!(bg.color().is_none());
    assert_eq!(bg.bgcolor().unwrap().name, "red");
    assert_eq!(bg.bold(), None);
}

// clear_meta_and_links tests
#[test]
fn test_clear_meta_and_links() {
    let style = Style::parse("bold red link https://example.org").unwrap();
    let cleared = style.clear_meta_and_links();
    assert_eq!(cleared.bold(), Some(true));
    assert_eq!(cleared.color().unwrap().name, "red");
    assert!(cleared.link().is_none());
}

// Combine tests
#[test]
fn test_combine_empty() {
    let result = Style::combine(&[]);
    assert!(result.is_null());
}

#[test]
fn test_combine_multiple() {
    let styles = vec![
        Style::parse("red").unwrap(),
        Style::parse("bold").unwrap(),
        Style::parse("on blue").unwrap(),
    ];
    let result = Style::combine(&styles);
    assert_eq!(result.color().unwrap().name, "red");
    assert_eq!(result.bold(), Some(true));
    assert_eq!(result.bgcolor().unwrap().name, "blue");
}

// Attribute aliases tests
#[test]
fn test_parse_attribute_alias_b() {
    let style = Style::parse("b").unwrap();
    assert_eq!(style.bold(), Some(true));
}

#[test]
fn test_parse_attribute_alias_i() {
    let style = Style::parse("i").unwrap();
    assert_eq!(style.italic(), Some(true));
}

#[test]
fn test_parse_attribute_alias_u() {
    let style = Style::parse("u").unwrap();
    assert_eq!(style.underline(), Some(true));
}

#[test]
fn test_parse_attribute_alias_s() {
    let style = Style::parse("s").unwrap();
    assert_eq!(style.strike(), Some(true));
}

#[test]
fn test_parse_attribute_alias_uu() {
    let style = Style::parse("uu").unwrap();
    assert_eq!(style.underline2(), Some(true));
}

#[test]
fn test_parse_attribute_alias_o() {
    let style = Style::parse("o").unwrap();
    assert_eq!(style.overline(), Some(true));
}

// Hash test
#[test]
fn test_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    let style1 = Style::parse("bold red").unwrap();
    let style2 = Style::parse("bold red").unwrap();
    let style3 = Style::parse("bold blue").unwrap();

    set.insert(style1);
    assert!(set.contains(&style2));
    set.insert(style3);
    assert_eq!(set.len(), 2);
}

// Copy test
#[test]
fn test_copy() {
    let style = Style::parse("bold red").unwrap();
    let copied = style.copy();
    assert_eq!(style, copied);
}

// from_color tests
#[test]
fn test_from_color() {
    let color = Color::parse("red").unwrap();
    let bgcolor = Color::parse("blue").unwrap();
    let style = Style::from_color(Some(color), Some(bgcolor));
    assert_eq!(style.color().unwrap().name, "red");
    assert_eq!(style.bgcolor().unwrap().name, "blue");
    assert!(style.bold().is_none());
}

#[test]
fn test_from_color_none() {
    let style = Style::from_color(None, None);
    assert!(style.color().is_none());
    assert!(style.bgcolor().is_none());
}

// -- with_link builder tests --------------------------------------------

#[test]
fn test_with_link() {
    let style = Style::with_link("https://example.com");
    assert_eq!(style.link(), Some("https://example.com"));
    assert!(style.color().is_none());
    assert!(style.bold().is_none());
}

#[test]
fn test_with_link_is_not_null() {
    let style = Style::with_link("https://example.com");
    assert!(!style.is_null());
}

// -- link=URL parse syntax tests ----------------------------------------

#[test]
fn test_parse_link_equals_syntax() {
    let style = Style::parse("link=https://example.com").unwrap();
    assert_eq!(style.link(), Some("https://example.com"));
}

#[test]
fn test_parse_bold_link_equals_syntax() {
    let style = Style::parse("bold link=https://example.com").unwrap();
    assert_eq!(style.bold(), Some(true));
    assert_eq!(style.link(), Some("https://example.com"));
}

#[test]
fn test_parse_link_equals_empty_error() {
    let result = Style::parse("link=");
    assert!(result.is_err());
}

// -- link rendering tests -----------------------------------------------

#[test]
fn test_render_link_only() {
    let style = Style::with_link("https://example.com");
    let rendered = style.render("click", Some(ColorSystem::TrueColor));
    assert_eq!(
        rendered,
        "\x1b]8;;https://example.com\x1b\\click\x1b]8;;\x1b\\"
    );
}

#[test]
fn test_render_bold_with_link() {
    let style = Style::parse("bold link https://example.com").unwrap();
    let rendered = style.render("click", Some(ColorSystem::TrueColor));
    // Should have OSC 8 wrapping around the ANSI-styled text
    assert!(rendered.starts_with("\x1b]8;;https://example.com\x1b\\"));
    assert!(rendered.ends_with("\x1b]8;;\x1b\\"));
    assert!(rendered.contains("\x1b[1m"));
    assert!(rendered.contains("click"));
}

#[test]
fn test_render_link_no_color_system() {
    // With no color system, render returns plain text (no link wrapping)
    let style = Style::with_link("https://example.com");
    let rendered = style.render("click", None);
    assert_eq!(rendered, "click");
}

// -- link combine tests -------------------------------------------------

#[test]
fn test_add_link_override() {
    let style1 = Style::parse("link https://a.com").unwrap();
    let style2 = Style::parse("link https://b.com").unwrap();
    let result = style1 + style2;
    assert_eq!(result.link(), Some("https://b.com"));
}

#[test]
fn test_add_link_preserved() {
    let style1 = Style::parse("link https://a.com").unwrap();
    let style2 = Style::parse("bold").unwrap();
    let result = style1 + style2;
    assert_eq!(result.link(), Some("https://a.com"));
    assert_eq!(result.bold(), Some(true));
}

#[test]
fn test_combine_link() {
    let styles = vec![
        Style::parse("red").unwrap(),
        Style::with_link("https://example.com"),
        Style::parse("bold").unwrap(),
    ];
    let result = Style::combine(&styles);
    assert_eq!(result.link(), Some("https://example.com"));
    assert_eq!(result.color().unwrap().name, "red");
    assert_eq!(result.bold(), Some(true));
}

// -- Underline enhancement tests ----------------------------------------

#[test]
fn test_underline_style_setter_getter() {
    let mut style = Style::null();
    assert!(style.underline_style().is_none());
    style.set_underline_style(Some(UnderlineStyle::Curly));
    assert_eq!(style.underline_style(), Some(UnderlineStyle::Curly));
}

#[test]
fn test_underline_color_setter_getter() {
    let mut style = Style::null();
    assert!(style.underline_color().is_none());
    let red = Color::parse("red").unwrap();
    style.set_underline_color(Some(red));
    assert!(style.underline_color().is_some());
    assert_eq!(style.underline_color().unwrap().name, "red");
}

#[test]
fn test_underline_style_is_not_null() {
    let mut style = Style::null();
    style.set_underline_style(Some(UnderlineStyle::Double));
    assert!(!style.is_null());
}

#[test]
fn test_underline_color_is_not_null() {
    let mut style = Style::null();
    style.set_underline_color(Some(Color::parse("red").unwrap()));
    assert!(!style.is_null());
}

#[test]
fn test_underline_style_display() {
    let mut style = Style::null();
    style.set_underline_style(Some(UnderlineStyle::Curly));
    assert!(style.to_string().contains("curly"));
}

#[test]
fn test_underline_color_display() {
    let mut style = Style::null();
    style.set_underline_color(Some(Color::parse("red").unwrap()));
    assert!(style.to_string().contains("underline_color(red)"));
}

#[test]
fn test_underline_style_add() {
    let mut s1 = Style::null();
    s1.set_underline_style(Some(UnderlineStyle::Curly));
    let mut s2 = Style::null();
    s2.set_underline_style(Some(UnderlineStyle::Dashed));
    let result = s1 + s2;
    assert_eq!(result.underline_style(), Some(UnderlineStyle::Dashed));
}

#[test]
fn test_underline_color_add() {
    let mut s1 = Style::null();
    s1.set_underline_color(Some(Color::parse("red").unwrap()));
    let s2 = Style::parse("bold").unwrap();
    let result = s1 + s2;
    assert_eq!(result.underline_color().unwrap().name, "red");
}

#[test]
fn test_underline_style_render_curly() {
    let mut style = Style::null();
    style.set_underline_style(Some(UnderlineStyle::Curly));
    let rendered = style.render("foo", Some(ColorSystem::TrueColor));
    assert!(rendered.contains("4:3"));
}

#[test]
fn test_underline_style_render_dashed() {
    let mut style = Style::null();
    style.set_underline_style(Some(UnderlineStyle::Dashed));
    let rendered = style.render("foo", Some(ColorSystem::TrueColor));
    assert!(rendered.contains("4:5"));
}

#[test]
fn test_underline_color_render_truecolor() {
    let mut style = Style::null();
    style.set_underline(Some(true));
    style.set_underline_color(Some(Color::from_rgb(255, 0, 0)));
    let rendered = style.render("foo", Some(ColorSystem::TrueColor));
    // Should contain 58;2;255;0;0 for underline color
    assert!(rendered.contains("58;2;255;0;0"), "rendered: {}", rendered);
}

#[test]
fn test_without_color_preserves_underline_color() {
    let mut style = Style::parse("bold red on blue").unwrap();
    style.set_underline_color(Some(Color::parse("green").unwrap()));
    let without = style.without_color();
    assert!(without.color().is_none());
    assert!(without.bgcolor().is_none());
    assert!(without.underline_color().is_some());
    assert_eq!(without.underline_color().unwrap().name, "green");
}

#[test]
fn test_background_style_clears_underline() {
    let mut style = Style::parse("bold red on blue").unwrap();
    style.set_underline_color(Some(Color::parse("green").unwrap()));
    style.set_underline_style(Some(UnderlineStyle::Curly));
    let bg = style.background_style();
    assert!(bg.underline_color().is_none());
    assert!(bg.underline_style().is_none());
}

#[test]
fn test_underline_equality() {
    let mut s1 = Style::null();
    s1.set_underline_style(Some(UnderlineStyle::Curly));
    let mut s2 = Style::null();
    s2.set_underline_style(Some(UnderlineStyle::Curly));
    assert_eq!(s1, s2);

    let mut s3 = Style::null();
    s3.set_underline_style(Some(UnderlineStyle::Dashed));
    assert_ne!(s1, s3);
}

#[test]
fn test_public_setters() {
    let mut style = Style::null();
    style.set_bold(Some(true));
    style.set_dim(Some(true));
    style.set_italic(Some(true));
    style.set_underline(Some(true));
    style.set_blink(Some(true));
    style.set_reverse(Some(true));
    style.set_conceal(Some(true));
    style.set_strike(Some(true));

    assert_eq!(style.bold(), Some(true));
    assert_eq!(style.dim(), Some(true));
    assert_eq!(style.italic(), Some(true));
    assert_eq!(style.underline(), Some(true));
    assert_eq!(style.blink(), Some(true));
    assert_eq!(style.reverse(), Some(true));
    assert_eq!(style.conceal(), Some(true));
    assert_eq!(style.strike(), Some(true));
}
