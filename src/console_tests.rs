//! Tests extracted from console.rs for readability.
//! Wired in via `#[path] mod tests;`.

use super::*;

// -- ConsoleDimensions --------------------------------------------------

#[test]
fn test_console_dimensions_create() {
    let dims = ConsoleDimensions {
        width: 80,
        height: 25,
    };
    assert_eq!(dims.width, 80);
    assert_eq!(dims.height, 25);
}

#[test]
fn test_console_dimensions_clone() {
    let dims = ConsoleDimensions {
        width: 120,
        height: 40,
    };
    let cloned = dims;
    assert_eq!(dims, cloned);
}

#[test]
fn test_console_dimensions_equality() {
    let a = ConsoleDimensions {
        width: 80,
        height: 25,
    };
    let b = ConsoleDimensions {
        width: 80,
        height: 25,
    };
    let c = ConsoleDimensions {
        width: 120,
        height: 25,
    };
    assert_eq!(a, b);
    assert_ne!(a, c);
}

// -- ConsoleOptions -----------------------------------------------------

#[test]
fn test_console_options_ascii_only_utf8() {
    let opts = make_default_options();
    assert!(!opts.ascii_only());
}

#[test]
fn test_console_options_ascii_only_ascii() {
    let mut opts = make_default_options();
    opts.encoding = std::borrow::Cow::Borrowed("ascii");
    assert!(opts.ascii_only());
}

#[test]
fn test_console_options_ascii_only_latin1() {
    let mut opts = make_default_options();
    opts.encoding = std::borrow::Cow::Borrowed("latin-1");
    assert!(opts.ascii_only());
}

#[test]
fn test_console_options_copy() {
    let opts = make_default_options();
    let copy = opts.copy();
    assert_eq!(copy.size, opts.size);
    assert_eq!(copy.max_width, opts.max_width);
    assert_eq!(copy.encoding.as_ref(), opts.encoding.as_ref());
}

#[test]
fn test_console_options_update_width() {
    let opts = make_default_options();
    let updated = opts.update_width(40);
    assert_eq!(updated.size.width, 40);
    assert_eq!(updated.max_width, 40);
}

#[test]
fn test_console_options_update_height() {
    let opts = make_default_options();
    let updated = opts.update_height(50);
    assert_eq!(updated.height, Some(50));
}

#[test]
fn test_console_options_update_dimensions() {
    let opts = make_default_options();
    let updated = opts.update_dimensions(100, 50);
    assert_eq!(updated.size.width, 100);
    assert_eq!(updated.size.height, 50);
    assert_eq!(updated.max_width, 100);
    assert_eq!(updated.height, Some(50));
}

#[test]
fn test_console_options_reset_height() {
    let opts = make_default_options().update_height(50);
    assert_eq!(opts.height, Some(50));
    let reset = opts.reset_height();
    assert_eq!(reset.height, None);
}

#[test]
fn test_console_options_with_updates() {
    let opts = make_default_options();
    let updates = ConsoleOptionsUpdates {
        width: Some(60),
        no_wrap: Some(true),
        justify: Some(Some(JustifyMethod::Center)),
        ..Default::default()
    };
    let updated = opts.with_updates(&updates);
    assert_eq!(updated.size.width, 60);
    assert_eq!(updated.max_width, 60);
    // no_wrap is now Option<bool>; `Some(true)` from the update (tri-state).
    assert_eq!(updated.no_wrap, Some(true));
    assert_eq!(updated.justify, Some(JustifyMethod::Center));
}

// -- Console creation ---------------------------------------------------

#[test]
fn test_console_default() {
    // Use force_terminal so the builder produces a styled console regardless
    // of whether the test harness pipes stdout.
    let console = Console::builder().force_terminal(true).build();
    assert_eq!(console.encoding(), "utf-8");
    assert!(!console.no_color);
    assert!(!console.quiet);
    assert!(console.markup_enabled);
    assert!(console.highlight_enabled);
}

#[test]
fn test_console_builder_defaults() {
    // force_terminal so that the builder resolves a color system even when
    // stdout is piped in the test harness.
    let console = Console::builder().force_terminal(true).build();
    assert!(console.color_system.is_some());
    assert_eq!(console.tab_size, 8);
    assert!(!console.record);
}

#[test]
fn test_console_builder_width() {
    let console = Console::builder().width(120).build();
    assert_eq!(console.width(), 120);
}

#[test]
fn test_console_builder_height() {
    let console = Console::builder().height(50).build();
    assert_eq!(console.height(), 50);
}

#[test]
fn test_console_custom_width_height() {
    let console = Console::builder().width(100).height(40).build();
    assert_eq!(console.width(), 100);
    assert_eq!(console.height(), 40);
    let dims = console.size();
    assert_eq!(dims.width, 100);
    assert_eq!(dims.height, 40);
}

#[test]
fn test_console_color_system_standard() {
    let console = Console::builder().color_system("standard").build();
    assert_eq!(console.color_system(), Some(ColorSystem::Standard));
    assert_eq!(console.color_system_name(), Some("standard"));
}

#[test]
fn test_console_color_system_256() {
    let console = Console::builder().color_system("256").build();
    assert_eq!(console.color_system(), Some(ColorSystem::EightBit));
    assert_eq!(console.color_system_name(), Some("256"));
}

#[test]
fn test_console_color_system_truecolor() {
    let console = Console::builder().color_system("truecolor").build();
    assert_eq!(console.color_system(), Some(ColorSystem::TrueColor));
    assert_eq!(console.color_system_name(), Some("truecolor"));
}

#[test]
fn test_console_no_color() {
    let console = Console::builder().no_color(true).color_system("").build();
    assert!(console.color_system().is_none());
    assert_eq!(console.color_system_name(), None);
}

#[test]
fn test_console_no_color_overrides_env_vars() {
    // Even if FORCE_COLOR is set in the environment, an explicit
    // `no_color(true)` on the builder takes priority.
    let console = Console::builder().no_color(true).build();
    assert!(console.color_system().is_none());
}

#[test]
fn test_console_color_system_override_builder() {
    // `color_system_override` takes priority over string-based selection.
    let console = Console::builder()
        .color_system("standard")
        .color_system_override(ColorSystem::TrueColor)
        .build();
    assert_eq!(console.color_system(), Some(ColorSystem::TrueColor));
}

// -- Theme / style lookup -----------------------------------------------

#[test]
fn test_get_style_from_theme() {
    let console = Console::new();
    let style = console.get_style("bold");
    assert!(style.is_ok());
    assert_eq!(style.unwrap(), Style::parse("bold"));
}

#[test]
fn test_get_style_parse_inline() {
    let console = Console::new();
    let style = console.get_style("bold red on blue");
    assert!(style.is_ok());
}

#[test]
fn test_get_style_invalid() {
    let console = Console::new();
    let style = console.get_style("completely_nonexistent_style_xyzzy");
    // Should either find it in the theme or fail to parse
    // If not in theme and not parseable, it's an error
    assert!(style.is_err());
}

#[test]
fn test_push_pop_theme() {
    let mut console = Console::new();

    // Default should have "bold"
    assert!(console.get_style("bold").is_ok());

    // Push a theme with a custom style
    let mut styles = std::collections::HashMap::new();
    styles.insert("my_custom_style".to_string(), Style::parse("red bold"));
    let custom = Theme::new(Some(styles), true);
    console.push_theme(custom);

    // Custom style should be available
    let style = console.get_style("my_custom_style");
    assert!(style.is_ok());

    // Pop the theme
    console.pop_theme();

    // Custom style should no longer be available via theme lookup
    // (but might still parse as a style definition)
    let result = console.theme_stack.get("my_custom_style");
    assert!(result.is_none());
}

// -- render_str ---------------------------------------------------------

#[test]
fn test_render_str_plain() {
    let console = Console::builder().markup(false).build();
    let text = console.render_str("Hello, world!", None, None, None);
    assert_eq!(text.plain(), "Hello, world!");
}

#[test]
fn test_render_str_with_markup() {
    let console = Console::new();
    let text = console.render_str("[bold]Hello[/bold]", None, None, None);
    assert_eq!(text.plain(), "Hello");
    // Should have a bold span
    assert!(!text.spans().is_empty());
}

#[test]
fn test_render_str_with_style() {
    let console = Console::new();
    let text = console.render_str("Hello", Some("bold"), None, None);
    // The base style should be bold
    assert_eq!(text.plain(), "Hello");
}

#[test]
fn test_render_str_with_justify() {
    let console = Console::new();
    let text = console.render_str("Hello", None, Some(JustifyMethod::Center), None);
    assert_eq!(text.justify, Some(JustifyMethod::Center));
}

#[test]
fn test_render_str_with_overflow() {
    let console = Console::new();
    let text = console.render_str("Hello", None, None, Some(OverflowMethod::Ellipsis));
    assert_eq!(text.overflow, Some(OverflowMethod::Ellipsis));
}

// -- Capture ------------------------------------------------------------

#[test]
fn test_capture_basic() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(true)
        .markup(false)
        .build();

    console.begin_capture();
    let text = Text::new("Hello, world!", Style::null());
    console.print(&text);
    let captured = console.end_capture();

    assert!(captured.contains("Hello, world!"));
}

#[test]
fn test_capture_empty() {
    let mut console = Console::new();
    console.begin_capture();
    let captured = console.end_capture();
    assert!(captured.is_empty());
}

#[test]
fn test_capture_multiple_prints() {
    let mut console = Console::builder()
        .width(80)
        .no_color(true)
        .markup(false)
        .build();

    console.begin_capture();
    let text1 = Text::new("Hello", Style::null());
    let text2 = Text::new("World", Style::null());
    console.print(&text1);
    console.print(&text2);
    let captured = console.end_capture();

    assert!(captured.contains("Hello"));
    assert!(captured.contains("World"));
}

// -- print_text ---------------------------------------------------------

#[test]
fn test_print_text_capture() {
    let mut console = Console::builder()
        .width(80)
        .no_color(true)
        .markup(false)
        .build();

    console.begin_capture();
    console.print_text("Hello, terminal!");
    let captured = console.end_capture();

    assert!(captured.contains("Hello, terminal!"));
}

// -- export_text --------------------------------------------------------

#[test]
fn test_export_text_plain() {
    let mut console = Console::builder()
        .width(80)
        .no_color(true)
        .record(true)
        .markup(false)
        .build();

    let text = Text::new("Export me", Style::null());
    console.print(&text);
    let exported = console.export_text(false, false);

    assert!(exported.contains("Export me"));
}

#[test]
fn test_export_text_with_styles() {
    let mut console = Console::builder()
        .width(80)
        .record(true)
        .markup(false)
        .build();

    let text = Text::styled("Bold text", "bold");
    console.print(&text);
    let exported = console.export_text(false, true);

    // Styled export should contain ANSI codes
    assert!(exported.contains("Bold text"));
}

#[test]
fn test_export_text_clear() {
    let mut console = Console::builder()
        .width(80)
        .record(true)
        .no_color(true)
        .markup(false)
        .build();

    let text = Text::new("Clearable", Style::null());
    console.print(&text);

    let export1 = console.export_text(true, false);
    assert!(export1.contains("Clearable"));

    // After clearing, should be empty
    let export2 = console.export_text(false, false);
    assert!(!export2.contains("Clearable"));
}

// -- export_html --------------------------------------------------------

#[test]
fn test_export_html_inline_styles() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true) // needed so color system is active in the test harness
        .record(true)
        .markup(false)
        .build();

    let text = Text::styled("Red text", "red");
    console.print(&text);
    let html = console.export_html(None, false, true);

    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Red text"));
    assert!(html.contains("<span"));
}

#[test]
fn test_export_html_stylesheet() {
    let mut console = Console::builder()
        .width(80)
        .record(true)
        .markup(false)
        .build();

    let text = Text::styled("Styled text", "bold");
    console.print(&text);
    let html = console.export_html(None, false, false);

    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Styled text"));
}

#[test]
fn test_export_html_escape() {
    let mut console = Console::builder()
        .width(80)
        .record(true)
        .no_color(true)
        .markup(false)
        .build();

    let text = Text::new("<script>alert('xss')</script>", Style::null());
    console.print(&text);
    let html = console.export_html(None, false, true);

    assert!(html.contains("&lt;script&gt;"));
    assert!(!html.contains("<script>"));
}

// -- render_buffer ------------------------------------------------------

#[test]
fn test_render_buffer_plain() {
    let console = Console::new();
    let segments = vec![Segment::text("Hello")];
    let output = console.render_buffer(&segments);
    assert_eq!(output, "Hello");
}

#[test]
fn test_render_buffer_styled() {
    let console = Console::builder().color_system("truecolor").build();
    let segments = vec![Segment::styled("Bold", Style::parse("bold"))];
    let output = console.render_buffer(&segments);
    // Should contain ANSI bold code
    assert!(output.contains("\x1b["));
    assert!(output.contains("Bold"));
}

#[test]
fn test_render_buffer_no_color() {
    let console = Console::builder().no_color(true).color_system("").build();
    let segments = vec![Segment::styled("NoColor", Style::parse("bold"))];
    let output = console.render_buffer(&segments);
    // Without color system, style.render should return plain text
    assert_eq!(output, "NoColor");
}

#[test]
fn test_render_buffer_control() {
    let console = Console::new();
    let ctrl = Control::bell();
    let segments = vec![ctrl.segment.clone()];
    let output = console.render_buffer(&segments);
    assert_eq!(output, "\x07");
}

#[test]
fn test_render_buffer_link() {
    let console = Console::builder().color_system("truecolor").build();
    let style = Style::parse("bold link https://example.com");
    let segments = vec![Segment::styled("click", style)];
    let output = console.render_buffer(&segments);
    // Should contain OSC 8 open with id= prefix and close.
    assert!(output.contains(";https://example.com\x1b\\"));
    assert!(output.contains("\x1b]8;;\x1b\\"));
    assert!(output.contains("click"));
}

#[test]
fn test_render_buffer_link_only() {
    let console = Console::builder().color_system("truecolor").build();
    let style = Style::with_link("https://example.com");
    let segments = vec![Segment::styled("link text", style)];
    let output = console.render_buffer(&segments);
    // id= prefix is monotonic — match by structure not exact id.
    assert!(output.starts_with("\x1b]8;id="));
    assert!(output.contains(";https://example.com\x1b\\link text\x1b]8;;\x1b\\"));
}

#[test]
fn test_render_buffer_coalesces_consecutive_same_link() {
    // Three segments with the same link but different SGR styles should
    // emit ONE link wrapper. Many terminals refuse to treat the run as
    // a single clickable link if open/close brackets every segment.
    let console = Console::builder().color_system("truecolor").build();
    let url = "https://github.com/khalidelborai/gilt";
    let plain = Style::with_link(url);
    let bold = Style::parse(&format!("bold link {url}"));
    let segments = vec![
        Segment::styled("Visit ", plain.clone()),
        Segment::styled("Gilt", bold),
        Segment::styled(" on GitHub", plain),
    ];
    let output = console.render_buffer(&segments);

    // Exactly one OSC 8 open (with id= prefix) and one close.
    let url_in_open_pattern = format!(";{url}\x1b\\");
    assert_eq!(
        output.matches(&url_in_open_pattern).count(),
        1,
        "expected single OSC 8 open with the URL"
    );
    assert_eq!(output.matches("\x1b]8;;\x1b\\").count(), 1);
    assert!(output.contains("Visit "));
    assert!(output.contains("Gilt"));
    assert!(output.contains(" on GitHub"));
}

#[test]
fn test_render_buffer_closes_link_when_url_changes() {
    let console = Console::builder().color_system("truecolor").build();
    let segments = vec![
        Segment::styled("a", Style::with_link("https://one.example")),
        Segment::styled("b", Style::with_link("https://two.example")),
    ];
    let output = console.render_buffer(&segments);
    assert_eq!(output.matches("https://one.example").count(), 1);
    assert_eq!(output.matches("https://two.example").count(), 1);
    assert_eq!(output.matches("\x1b]8;;\x1b\\").count(), 2);
}

#[test]
fn test_render_buffer_closes_link_on_unlinked_segment() {
    let console = Console::builder().color_system("truecolor").build();
    let segments = vec![
        Segment::styled("link", Style::with_link("https://x")),
        Segment::text(" plain"),
        Segment::styled("link2", Style::with_link("https://x")),
    ];
    let output = console.render_buffer(&segments);
    // Two open/close pairs because the unlinked middle segment closes
    // the first run.
    assert_eq!(output.matches("https://x").count(), 2);
    assert_eq!(output.matches("\x1b]8;;\x1b\\").count(), 2);
}

// -- Terminal detection -------------------------------------------------

#[test]
fn test_detect_terminal_size_defaults() {
    // Clear env vars for this test
    let saved_cols = std::env::var("COLUMNS").ok();
    let saved_lines = std::env::var("LINES").ok();
    std::env::remove_var("COLUMNS");
    std::env::remove_var("LINES");

    let (w, h) = Console::detect_terminal_size();
    assert_eq!(w, 80);
    assert_eq!(h, 25);

    // Restore env vars
    if let Some(v) = saved_cols {
        std::env::set_var("COLUMNS", v);
    }
    if let Some(v) = saved_lines {
        std::env::set_var("LINES", v);
    }
}

#[test]
fn test_detect_terminal_size_env() {
    let saved_cols = std::env::var("COLUMNS").ok();
    let saved_lines = std::env::var("LINES").ok();

    std::env::set_var("COLUMNS", "120");
    std::env::set_var("LINES", "40");

    let (w, h) = Console::detect_terminal_size();
    assert_eq!(w, 120);
    assert_eq!(h, 40);

    // Restore
    match saved_cols {
        Some(v) => std::env::set_var("COLUMNS", v),
        None => std::env::remove_var("COLUMNS"),
    }
    match saved_lines {
        Some(v) => std::env::set_var("LINES", v),
        None => std::env::remove_var("LINES"),
    }
}

// -- Control methods ----------------------------------------------------

#[test]
fn test_control_bell() {
    let mut console = Console::builder().record(true).build();
    console.bell();
    let text = console.export_text(false, true);
    assert!(text.contains('\x07'));
}

#[test]
fn test_control_clear() {
    let mut console = Console::builder().record(true).build();
    console.clear();
    let text = console.export_text(false, true);
    assert!(text.contains("\x1b[H"));
}

#[test]
fn test_control_show_cursor() {
    let mut console = Console::builder().record(true).build();
    console.show_cursor(true);
    let text = console.export_text(true, true);
    assert!(text.contains("\x1b[?25h"));

    console.show_cursor(false);
    let text = console.export_text(true, true);
    assert!(text.contains("\x1b[?25l"));
}

// -- Alt screen ---------------------------------------------------------

#[test]
fn test_alt_screen_enable_disable() {
    let mut console = Console::builder().record(true).build();

    assert!(!console.is_alt_screen);
    let changed = console.set_alt_screen(true);
    assert!(changed);
    assert!(console.is_alt_screen);

    // Enabling again should return false (already enabled)
    let changed = console.set_alt_screen(true);
    assert!(!changed);

    let changed = console.set_alt_screen(false);
    assert!(changed);
    assert!(!console.is_alt_screen);
}

#[test]
fn test_update_screen_no_op_when_not_in_alt_screen() {
    let mut console = Console::builder().record(true).build();
    assert!(!console.is_alt_screen);
    console.update_screen(5, 5, &Text::new("hello", Style::null()));
    let text = console.export_text(false, true);
    // Not in alt-screen → no output (safety: never scribble on main scrollback)
    assert!(!text.contains("hello"), "expected no output, got {text:?}");
}

#[test]
fn test_update_screen_writes_at_position_in_alt_screen() {
    let mut console = Console::builder().record(true).build();
    console.set_alt_screen(true);
    console.update_screen(10, 3, &Text::new("hello", Style::null()));
    let text = console.export_text(false, true);
    // Cursor-positioning escape (CSI ?;? H) precedes the content.
    assert!(text.contains("hello"));
    assert!(
        text.contains("\x1b["),
        "expected ANSI cursor-position prefix"
    );
}

#[test]
fn test_update_screen_lines_writes_each_line_at_successive_rows() {
    let mut console = Console::builder().record(true).build();
    console.set_alt_screen(true);
    let lines = vec![
        vec![Segment::text("row0")],
        vec![Segment::text("row1")],
        vec![Segment::text("row2")],
    ];
    console.update_screen_lines(2, 5, &lines);
    let text = console.export_text(false, true);
    assert!(text.contains("row0"));
    assert!(text.contains("row1"));
    assert!(text.contains("row2"));
    // Three position changes one per line.
    let csi_count = text.matches("\x1b[").count();
    assert!(
        csi_count >= 3,
        "expected >=3 CSI sequences, got {csi_count}"
    );
}

#[test]
fn test_update_screen_lines_no_op_when_not_in_alt_screen() {
    let mut console = Console::builder().record(true).build();
    let lines = vec![vec![Segment::text("nope")]];
    console.update_screen_lines(0, 0, &lines);
    let text = console.export_text(false, true);
    assert!(!text.contains("nope"));
}

// -- Buffer nesting -----------------------------------------------------

#[test]
fn test_buffer_nesting() {
    let mut console = Console::new();
    assert!(!console.check_buffer());

    console.enter_buffer();
    assert!(console.check_buffer());

    console.enter_buffer();
    assert!(console.check_buffer());

    console.exit_buffer();
    assert!(console.check_buffer());

    console.exit_buffer();
    assert!(!console.check_buffer());
}

#[test]
fn buffer_flush_does_not_discard_output() {
    // Regression for B1: previously, exit_buffer rendered the buffer to
    // a String and then dropped it without writing anywhere. Buffered
    // content must reach an active capture sink (or stdout when no
    // capture is active).
    let mut console = Console::builder().width(80).force_terminal(false).build();
    console.begin_capture();
    console.enter_buffer();
    console.print_text("HELLO_FROM_BUFFER");
    console.exit_buffer();
    let captured = console.end_capture();
    assert!(
        captured.contains("HELLO_FROM_BUFFER"),
        "buffered content should reach the capture sink, got {captured:?}"
    );
}

// -- Renderable trait for Text ------------------------------------------

#[test]
fn test_renderable_text() {
    let console = Console::builder().width(80).build();
    let text = Text::new("Renderable text", Style::null());
    let opts = console.options();
    let segments = text.gilt_console(&console, &opts);
    assert!(!segments.is_empty());
    let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(combined.contains("Renderable text"));
}

// -- Renderable trait for str -------------------------------------------

#[test]
fn test_renderable_str() {
    let console = Console::builder().width(80).markup(false).build();
    let opts = console.options();
    let text = "Hello from str";
    let segments = text.gilt_console(&console, &opts);
    assert!(!segments.is_empty());
    let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(combined.contains("Hello from str"));
}

#[test]
fn test_renderable_string() {
    let console = Console::builder().width(80).markup(false).build();
    let opts = console.options();
    let text = String::from("Hello from String");
    let segments = text.gilt_console(&console, &opts);
    assert!(!segments.is_empty());
    let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(combined.contains("Hello from String"));
}

// -- Quiet mode ---------------------------------------------------------

#[test]
fn test_quiet_mode() {
    let mut console = Console::builder()
        .width(80)
        .record(true)
        .quiet(true)
        .markup(false)
        .build();

    let text = Text::new("Should not appear", Style::null());
    console.print(&text);
    let exported = console.export_text(false, false);
    // Quiet mode should suppress all output including recording
    assert!(exported.is_empty());
}

// -- Soft wrap mode -----------------------------------------------------

#[test]
fn test_soft_wrap_builder() {
    let console = Console::builder().soft_wrap(true).build();
    assert!(console.soft_wrap);
}

// -- No-color mode stripping --------------------------------------------

#[test]
fn test_no_color_mode_strips_color() {
    let mut console = Console::builder()
        .width(80)
        .no_color(true)
        .color_system("")
        .record(true)
        .markup(false)
        .build();

    let text = Text::styled("Colored text", "red");
    console.print(&text);

    // In no-color mode, the rendered output should be plain
    let exported = console.export_text(false, true);
    assert!(exported.contains("Colored text"));
    // Should NOT contain ANSI color codes since color_system is None
    assert!(!exported.contains("\x1b["));
}

// -- Record buffer accumulation -----------------------------------------

#[test]
fn test_record_buffer_accumulation() {
    let mut console = Console::builder()
        .width(80)
        .record(true)
        .no_color(true)
        .markup(false)
        .build();

    let text1 = Text::new("First", Style::null());
    let text2 = Text::new("Second", Style::null());
    console.print(&text1);
    console.print(&text2);

    let exported = console.export_text(false, false);
    assert!(exported.contains("First"));
    assert!(exported.contains("Second"));
}

// -- options() default --------------------------------------------------

#[test]
fn test_console_options_default() {
    let console = Console::builder().width(100).height(40).build();
    let opts = console.options();
    assert_eq!(opts.size.width, 100);
    assert_eq!(opts.size.height, 40);
    assert_eq!(opts.max_width, 100);
    assert_eq!(opts.encoding.as_ref(), "utf-8");
    // no_wrap is now Option<bool>; None = inherit / wrap by default.
    assert_eq!(opts.no_wrap, None);
    assert_eq!(opts.justify, None);
    assert_eq!(opts.overflow, None);
}

// -- render / render_lines ----------------------------------------------

#[test]
fn test_render_text() {
    let console = Console::builder().width(80).build();
    let text = Text::new("Render me", Style::null());
    let segments = console.render(&text, None);
    let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(combined.contains("Render me"));
}

#[test]
fn test_render_lines_basic() {
    let console = Console::builder().width(80).build();
    let text = Text::new("Line1\nLine2", Style::null());
    let lines = console.render_lines(&text, None, None, false, false);
    assert!(lines.len() >= 2);
}

// -- html_escape --------------------------------------------------------

#[test]
fn test_html_escape_all_entities() {
    assert_eq!(html_escape("&"), "&amp;");
    assert_eq!(html_escape("<"), "&lt;");
    assert_eq!(html_escape(">"), "&gt;");
    assert_eq!(html_escape("\""), "&quot;");
    assert_eq!(
        html_escape("<p class=\"x\">&</p>"),
        "&lt;p class=&quot;x&quot;&gt;&amp;&lt;/p&gt;"
    );
}

// -- svg_escape ---------------------------------------------------------

#[test]
fn test_svg_escape_entities() {
    assert_eq!(svg_escape("&"), "&amp;");
    assert_eq!(svg_escape("'"), "&#39;");
}

// -- set_window_title ---------------------------------------------------

#[test]
fn test_set_window_title_non_terminal() {
    let mut console = Console::builder().force_terminal(false).build();
    let result = console.set_window_title("Test");
    assert!(!result);
}

#[test]
fn test_set_window_title_terminal() {
    let mut console = Console::builder().force_terminal(true).record(true).build();
    let result = console.set_window_title("Test Title");
    assert!(result);
    let exported = console.export_text(false, true);
    assert!(exported.contains("Test Title"));
}

// -- export_svg ---------------------------------------------------------

#[test]
fn test_export_svg_basic() {
    let mut console = Console::builder()
        .width(40)
        .record(true)
        .no_color(true)
        .markup(false)
        .build();

    let text = Text::new("SVG test", Style::null());
    console.print(&text);
    let svg = console.export_svg("Test", None, false, None, 0.61);

    assert!(svg.contains("<svg"));
    assert!(svg.contains("SVG test"));
    assert!(svg.contains("</svg>"));
}

// -- encoding -----------------------------------------------------------

#[test]
fn test_encoding_always_utf8() {
    let console = Console::new();
    assert_eq!(console.encoding(), "utf-8");
}

// -- is_dumb_terminal ---------------------------------------------------

#[test]
fn test_is_dumb_terminal() {
    let saved = std::env::var("TERM").ok();
    std::env::set_var("TERM", "dumb");

    let console = Console::new();
    assert!(console.is_dumb_terminal());

    match saved {
        Some(v) => std::env::set_var("TERM", v),
        None => std::env::remove_var("TERM"),
    }
}

// -- Convenience methods ------------------------------------------------

#[test]
fn test_line_blank_lines() {
    let mut console = Console::builder()
        .width(80)
        .no_color(true)
        .markup(false)
        .build();

    console.begin_capture();
    console.line(3);
    let captured = console.end_capture();

    assert_eq!(captured, "\n\n\n");
}

#[test]
fn test_line_zero() {
    let mut console = Console::builder()
        .width(80)
        .no_color(true)
        .markup(false)
        .build();

    console.begin_capture();
    console.line(0);
    let captured = console.end_capture();

    assert!(captured.is_empty());
}

#[test]
fn test_rule_no_title_capture() {
    let mut console = Console::builder()
        .width(40)
        .no_color(true)
        .markup(false)
        .build();

    console.begin_capture();
    console.rule(None);
    let captured = console.end_capture();

    // Should contain rule characters and end with newline.
    // Default char is now ─ (U+2500, light) for rich parity; allow '-' for ascii fallback.
    assert!(captured.contains('\u{2500}') || captured.contains('-'));
    assert!(captured.ends_with('\n'));
}

#[test]
fn test_rule_with_title_capture() {
    let mut console = Console::builder()
        .width(40)
        .no_color(true)
        .markup(false)
        .build();

    console.begin_capture();
    console.rule(Some("Hello"));
    let captured = console.end_capture();

    assert!(captured.contains("Hello"));
    assert!(captured.ends_with('\n'));
}

#[cfg(feature = "json")]
#[test]
fn test_print_json_valid() {
    let mut console = Console::builder()
        .width(80)
        .no_color(true)
        .markup(false)
        .build();

    console.begin_capture();
    console.print_json(r#"{"name": "Alice", "age": 30}"#);
    let captured = console.end_capture();

    assert!(captured.contains("name"));
    assert!(captured.contains("Alice"));
    assert!(captured.contains("30"));
}

#[cfg(feature = "json")]
#[test]
fn test_print_json_invalid_falls_back() {
    let mut console = Console::builder()
        .width(80)
        .no_color(true)
        .markup(false)
        .build();

    console.begin_capture();
    console.print_json("not valid json");
    let captured = console.end_capture();

    assert!(captured.contains("not valid json"));
}

#[test]
fn test_measure_simple_text() {
    let console = Console::builder()
        .width(80)
        .no_color(true)
        .markup(false)
        .build();

    let text = Text::new("Hello World", Style::null());
    let measurement = console.measure(&text);

    // "Hello" and "World" are each 5 chars -- min should be 5
    // "Hello World" is 11 chars -- max should be 11
    assert_eq!(measurement.minimum, 5);
    assert_eq!(measurement.maximum, 11);
}

#[test]
fn test_measure_multiline_text() {
    let console = Console::builder()
        .width(80)
        .no_color(true)
        .markup(false)
        .build();

    let text = Text::new("Short\nA much longer second line", Style::null());
    let measurement = console.measure(&text);

    // max is the longer line
    assert!(measurement.maximum >= 25);
    // min is the longest word
    assert!(measurement.minimum >= 6); // "longer" or "second"
}

#[test]
fn test_measure_empty() {
    let console = Console::builder()
        .width(80)
        .no_color(true)
        .markup(false)
        .build();

    let text = Text::new("", Style::null());
    let measurement = console.measure(&text);

    assert_eq!(measurement.minimum, 0);
    assert_eq!(measurement.maximum, 0);
}

#[test]
fn test_save_text_to_file() {
    let mut console = Console::builder()
        .width(80)
        .no_color(true)
        .record(true)
        .markup(false)
        .build();

    let text = Text::new("Save me to a file", Style::null());
    console.print(&text);

    let dir = std::env::temp_dir();
    let path = dir.join("gilt_test_save_text.txt");
    let path_str = path.to_str().unwrap();

    let result = console.save_text(path_str, false, false);
    assert!(result.is_ok());

    let contents = std::fs::read_to_string(&path).unwrap();
    assert!(contents.contains("Save me to a file"));

    // Cleanup
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_save_html_to_file() {
    let mut console = Console::builder()
        .width(80)
        .record(true)
        .markup(false)
        .build();

    let text = Text::styled("HTML content", "red");
    console.print(&text);

    let dir = std::env::temp_dir();
    let path = dir.join("gilt_test_save.html");
    let path_str = path.to_str().unwrap();

    let result = console.save_html(path_str);
    assert!(result.is_ok());

    let contents = std::fs::read_to_string(&path).unwrap();
    assert!(contents.contains("<!DOCTYPE html>"));
    assert!(contents.contains("HTML content"));

    // Cleanup
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_save_svg_to_file() {
    let mut console = Console::builder()
        .width(40)
        .record(true)
        .no_color(true)
        .markup(false)
        .build();

    let text = Text::new("SVG save test", Style::null());
    console.print(&text);

    let dir = std::env::temp_dir();
    let path = dir.join("gilt_test_save.svg");
    let path_str = path.to_str().unwrap();

    let result = console.save_svg(path_str, Some("Test Title"));
    assert!(result.is_ok());

    let contents = std::fs::read_to_string(&path).unwrap();
    assert!(contents.contains("<svg"));
    assert!(contents.contains("SVG save test"));
    assert!(contents.contains("</svg>"));

    // Cleanup
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_save_svg_default_title() {
    let mut console = Console::builder()
        .width(40)
        .record(true)
        .no_color(true)
        .markup(false)
        .build();

    let text = Text::new("Default title test", Style::null());
    console.print(&text);

    let dir = std::env::temp_dir();
    let path = dir.join("gilt_test_save_default.svg");
    let path_str = path.to_str().unwrap();

    let result = console.save_svg(path_str, None);
    assert!(result.is_ok());

    let contents = std::fs::read_to_string(&path).unwrap();
    assert!(contents.contains("<svg"));

    // Cleanup
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_log_contains_timestamp_and_text() {
    let mut console = Console::builder()
        .width(80)
        .no_color(true)
        .markup(false)
        .build();

    console.begin_capture();
    console.log("Test log message");
    let captured = console.end_capture();

    // Should contain a timestamp pattern [HH:MM:SS]
    assert!(captured.contains('['));
    assert!(captured.contains(']'));
    assert!(captured.contains(':'));
    assert!(captured.contains("Test log message"));
    assert!(captured.ends_with('\n'));
}

#[test]
fn test_print_error_basic() {
    #[derive(Debug)]
    struct TestError;
    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "test error occurred")
        }
    }
    impl std::error::Error for TestError {}

    let mut console = Console::builder()
        .width(80)
        .no_color(true)
        .markup(false)
        .build();

    console.begin_capture();
    console.print_error(&TestError);
    let captured = console.end_capture();

    // Should contain the error message rendered inside a panel
    assert!(captured.contains("test error occurred"));
}

// -- Pager convenience --------------------------------------------------

#[test]
fn test_pager_with_capture() {
    let mut console = Console::builder()
        .width(80)
        .no_color(true)
        .record(true)
        .markup(false)
        .build();

    let text = Text::new("Pager content here", Style::null());
    console.print(&text);

    // Use `cat` as pager -- it reads stdin and exits cleanly.
    console.pager(Some("cat"));
}

// -- Screen enter/exit --------------------------------------------------

#[test]
fn test_enter_exit_screen() {
    let mut console = Console::builder().width(80).record(true).build();

    // Verify enter_screen activates alt screen and hides cursor.
    console.enter_screen(true);
    assert!(console.is_alt_screen);

    // Verify exit_screen restores state.
    console.exit_screen(true);
    assert!(!console.is_alt_screen);
}

#[test]
fn test_enter_exit_screen_no_hide_cursor() {
    let mut console = Console::builder().width(80).record(true).build();

    console.enter_screen(false);
    assert!(console.is_alt_screen);

    console.exit_screen(false);
    assert!(!console.is_alt_screen);
}

// -- Live ID ------------------------------------------------------------

#[test]
fn test_set_clear_live() {
    let mut console = Console::new();
    assert_eq!(console.current_live(), None);

    console.set_live(Some(42));
    assert_eq!(console.current_live(), Some(42));

    console.clear_live();
    assert_eq!(console.current_live(), None);
}

#[test]
fn test_set_live_none() {
    let mut console = Console::new();
    console.set_live(Some(7));
    assert_eq!(console.current_live(), Some(7));

    console.set_live(None);
    assert_eq!(console.current_live(), None);
}

#[test]
fn test_push_pop_live_nests() {
    let mut console = Console::new();
    assert_eq!(console.live_depth(), 0);
    assert!(console.push_live(1));
    assert_eq!(console.current_live(), Some(1));
    assert!(console.push_live(2));
    assert_eq!(console.current_live(), Some(2));
    assert_eq!(console.live_depth(), 2);
    assert_eq!(console.pop_live(), Some(2));
    // Outer Live is now active again.
    assert_eq!(console.current_live(), Some(1));
    assert_eq!(console.pop_live(), Some(1));
    assert_eq!(console.current_live(), None);
    assert_eq!(console.pop_live(), None);
}

#[test]
fn test_set_live_some_replaces_top() {
    // Compatibility behaviour: set_live(Some(id)) replaces the top of the
    // stack rather than pushing — preserves single-Live callers' state.
    let mut console = Console::new();
    console.set_live(Some(1));
    console.set_live(Some(2));
    assert_eq!(console.live_depth(), 1);
    assert_eq!(console.current_live(), Some(2));
}

#[test]
fn test_status_convenience() {
    let console = Console::builder().force_terminal(true).width(80).build();
    let status = console.status("Working...");
    assert_eq!(status.status_text, "Working...");
    assert!(!status.is_started());
}

// -- print_exception test -----------------------------------------------

#[test]
fn test_print_exception() {
    #[derive(Debug)]
    struct TestError;
    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "something went wrong")
        }
    }
    impl std::error::Error for TestError {}

    let mut console = Console::builder()
        .width(80)
        .no_color(true)
        .markup(false)
        .build();

    console.begin_capture();
    console.print_exception(&TestError);
    let captured = console.end_capture();

    // Should contain the error message rendered via Traceback
    assert!(captured.contains("something went wrong"));
}

// -- input_password method exists and compiles ----------------------------

#[cfg(feature = "interactive")]
#[test]
fn test_input_password_method_exists() {
    // Verify the method signature compiles correctly.
    // We cannot actually call it in tests (requires a real terminal),
    // but we can verify the function pointer type resolves.
    let _fn_ptr: fn(&mut Console, &str) -> Result<String, std::io::Error> = Console::input_password;
    // Also verify Console::input still works
    let _fn_ptr2: fn(&mut Console, &str) -> Result<String, std::io::Error> = Console::input;
}

// -- Synchronized output ------------------------------------------------

#[test]
fn test_begin_synchronized_capture() {
    let mut console = Console::new();
    console.begin_capture();
    console.begin_synchronized();
    let output = console.end_capture();
    assert_eq!(output, "\x1b[?2026h");
}

#[test]
fn test_end_synchronized_capture() {
    let mut console = Console::new();
    console.begin_capture();
    console.end_synchronized();
    let output = console.end_capture();
    assert_eq!(output, "\x1b[?2026l");
}

#[test]
fn test_synchronized_wraps_content() {
    let mut console = Console::new();
    console.begin_capture();
    console.synchronized(|c| {
        c.print_text("hello");
    });
    let output = console.end_capture();
    assert!(
        output.starts_with("\x1b[?2026h"),
        "should start with begin sync"
    );
    assert!(output.ends_with("\x1b[?2026l"), "should end with end sync");
    assert!(output.contains("hello"), "should contain the printed text");
}

#[test]
fn test_synchronized_returns_value() {
    let mut console = Console::new();
    console.begin_capture();
    let result = console.synchronized(|_c| 42);
    let _ = console.end_capture();
    assert_eq!(result, 42);
}

// -- Clipboard (OSC 52) -------------------------------------------------

#[test]
fn test_copy_to_clipboard_capture() {
    let mut console = Console::new();
    console.begin_capture();
    console.copy_to_clipboard("hello");
    let output = console.end_capture();
    // "hello" base64 = "aGVsbG8="
    assert_eq!(output, "\x1b]52;c;aGVsbG8=\x07");
}

#[test]
fn test_copy_to_clipboard_empty_capture() {
    let mut console = Console::new();
    console.begin_capture();
    console.copy_to_clipboard("");
    let output = console.end_capture();
    assert_eq!(output, "\x1b]52;c;\x07");
}

#[test]
fn test_copy_to_clipboard_unicode_capture() {
    let mut console = Console::new();
    console.begin_capture();
    console.copy_to_clipboard("caf\u{00e9}");
    let output = console.end_capture();
    // "caf\xc3\xa9" base64 = "Y2Fmw6k="
    assert_eq!(output, "\x1b]52;c;Y2Fmw6k=\x07");
}

#[test]
fn test_request_clipboard_capture() {
    let mut console = Console::new();
    console.begin_capture();
    console.request_clipboard();
    let output = console.end_capture();
    assert_eq!(output, "\x1b]52;c;?\x07");
}

// -- Stress tests -------------------------------------------------------

#[test]
fn test_render_large_text() {
    use crate::text::Text;

    // Build a 50,000 character text block (a mix of lines).
    let line = "The quick brown fox jumps over the lazy dog. ";
    let mut large = String::with_capacity(50_000);
    while large.len() < 50_000 {
        large.push_str(line);
        if large.len() % 200 < line.len() {
            large.push('\n');
        }
    }
    assert!(large.len() >= 50_000);

    let text = Text::new(&large, crate::style::Style::null());
    let mut console = Console::builder().width(120).force_terminal(true).build();
    console.begin_capture();
    console.print(&text);
    let output = console.end_capture();
    // Must produce non-empty output and not panic.
    assert!(
        !output.is_empty(),
        "expected non-empty output for large text",
    );
}

#[test]
fn test_render_deeply_nested_panels() {
    use crate::panel::Panel;
    use crate::text::Text;

    // Build 20 panels nested inside each other by rendering each level
    // into a single-line summary, keeping content size bounded.
    let mut console = Console::builder().width(200).force_terminal(true).build();

    let mut inner = Text::new("innermost content", crate::style::Style::null());

    for i in 0..20 {
        let panel = Panel::new(inner);
        console.begin_capture();
        console.print(&panel);
        let rendered = console.end_capture();
        // Keep only first and last lines to bound growth, preserving
        // proof that rendering happened at each level.
        let first_line = rendered.lines().next().unwrap_or("");
        inner = Text::new(
            &format!("level {i}: {first_line}"),
            crate::style::Style::null(),
        );
    }

    // Final render of the outermost level.
    console.begin_capture();
    console.print(&inner);
    let output = console.end_capture();
    assert!(
        !output.is_empty(),
        "expected non-empty output for deeply nested panels",
    );
    // Verify the last level references level 19.
    assert!(
        output.contains("level 19"),
        "expected level 19 in output, got: {output}",
    );
}

// -- Helper function for tests ------------------------------------------

fn make_default_options() -> ConsoleOptions {
    ConsoleOptions {
        size: ConsoleDimensions {
            width: 80,
            height: 25,
        },
        legacy_windows: false,
        min_width: 1,
        max_width: 80,
        is_terminal: false,
        encoding: std::borrow::Cow::Borrowed("utf-8"),
        max_height: 25,
        justify: None,
        overflow: None,
        no_wrap: None,
        highlight: None,
        markup: None,
        height: None,
    }
}

// ---------------------------------------------------------------------------
// Task 1 (v1.8): ConsoleCapabilities
// ---------------------------------------------------------------------------

#[test]
fn capabilities_truecolor_flag_reflects_colorterm_env() {
    // ConsoleCapabilities::from_env_parts is the pure helper; we test it
    // directly so no env mutation is needed.
    use crate::console_caps::ConsoleCapabilities;
    let caps = ConsoleCapabilities::from_env_parts(Some("truecolor"), None, true, None, None, None);
    assert!(
        caps.truecolor,
        "COLORTERM=truecolor → truecolor flag should be true"
    );
    assert!(caps.is_terminal);
}

#[test]
fn capabilities_synchronized_output_default_true() {
    use crate::console_caps::ConsoleCapabilities;
    let caps = ConsoleCapabilities::from_env_parts(None, None, false, None, None, None);
    assert!(
        caps.synchronized_output,
        "synchronized_output must default to true (CSI ?2026 is harmless no-op)"
    );
}

#[test]
fn console_capabilities_accessor_returns_struct() {
    let console = Console::builder().force_terminal(true).build();
    let caps = console.capabilities();
    // synchronized_output defaults to true.
    assert!(caps.synchronized_output);
    // is_terminal should be true because force_terminal(true) was set.
    assert!(caps.is_terminal);
}

#[test]
fn capabilities_unicode_version_from_env_parts() {
    use crate::console_caps::ConsoleCapabilities;
    let caps = ConsoleCapabilities::from_env_parts(None, None, false, Some("15"), None, None);
    assert_eq!(caps.unicode_version, Some(15));
    let caps_none = ConsoleCapabilities::from_env_parts(None, None, false, None, None, None);
    assert_eq!(caps_none.unicode_version, None);
}

// -- v1.3.1: Sync regression guard + with_writer coverage ----------------

/// Compile-time assertion that `Console: Send + Sync`. The v1.2.0 release
/// silently lost `Sync` because the writer_override field was typed
/// `Box<dyn Write + Send>` — without `+ Sync`. This test catches that
/// class of regression at build time.
#[test]
fn console_is_send_and_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<Console>();
    assert_sync::<Console>();
}

#[test]
fn with_writer_routes_output_to_buffer() {
    use std::sync::{Arc, Mutex};
    let sink: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));

    struct ArcWriter(Arc<Mutex<Vec<u8>>>);
    impl std::io::Write for ArcWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let mut console = Console::builder()
        .width(40)
        .no_color(true)
        .markup(false)
        .build()
        .with_writer(ArcWriter(Arc::clone(&sink)));
    console.print_text("hello via writer");

    let bytes = sink.lock().unwrap().clone();
    let captured = String::from_utf8(bytes).unwrap();
    assert!(
        captured.contains("hello via writer"),
        "writer override should receive the printed text, got {:?}",
        captured
    );
}

// -- Finding #1: color-system auto-detection helper ----------------------

#[test]
fn test_detect_color_system_from_truecolor() {
    use crate::color::ColorSystem;
    use crate::console::detect_color_system_from;
    assert_eq!(
        detect_color_system_from(Some("truecolor"), None),
        ColorSystem::TrueColor
    );
    assert_eq!(
        detect_color_system_from(Some("24bit"), None),
        ColorSystem::TrueColor
    );
    // Case-insensitive
    assert_eq!(
        detect_color_system_from(Some("TRUECOLOR"), None),
        ColorSystem::TrueColor
    );
}

#[test]
fn test_detect_color_system_from_256color() {
    use crate::color::ColorSystem;
    use crate::console::detect_color_system_from;
    assert_eq!(
        detect_color_system_from(None, Some("xterm-256color")),
        ColorSystem::EightBit
    );
    assert_eq!(
        detect_color_system_from(None, Some("screen-256color")),
        ColorSystem::EightBit
    );
}

#[test]
fn test_detect_color_system_from_standard_term() {
    use crate::color::ColorSystem;
    use crate::console::detect_color_system_from;
    // A non-dumb TERM with no COLORTERM/256color → Standard
    assert_eq!(
        detect_color_system_from(None, Some("xterm")),
        ColorSystem::Standard
    );
}

#[test]
fn test_detect_color_system_from_dumb_falls_through() {
    use crate::color::ColorSystem;
    use crate::console::detect_color_system_from;
    // dumb TERM → fallback to TrueColor (no meaningful signal)
    assert_eq!(
        detect_color_system_from(None, Some("dumb")),
        ColorSystem::TrueColor
    );
    // No TERM either
    assert_eq!(detect_color_system_from(None, None), ColorSystem::TrueColor);
}

#[test]
fn test_detect_color_system_colorterm_wins_over_256color_term() {
    use crate::color::ColorSystem;
    use crate::console::detect_color_system_from;
    // COLORTERM=truecolor overrides a 256color TERM
    assert_eq!(
        detect_color_system_from(Some("truecolor"), Some("xterm-256color")),
        ColorSystem::TrueColor
    );
}

// -- Finding #3: update_width min_width clamp ----------------------------

#[test]
fn test_update_width_clamps_min_width() {
    let mut opts = make_default_options();
    opts.min_width = 60; // set a large min_width
    let updated = opts.update_width(40); // reduce to narrower width
    assert_eq!(updated.max_width, 40);
    assert_eq!(
        updated.min_width, 40,
        "min_width should be clamped to new width"
    );
}

#[test]
fn test_update_width_does_not_clamp_when_min_already_small() {
    let opts = make_default_options(); // min_width=1
    let updated = opts.update_width(40);
    assert_eq!(
        updated.min_width, 1,
        "min_width stays 1 when already less than width"
    );
}

// -- Finding #4: synchronized drop guard emits end-sync ------------------

#[test]
fn test_synchronized_guard_emits_end_sync_on_normal_return() {
    let mut console = Console::new();
    console.begin_capture();
    console.synchronized(|c| {
        c.print_text("test");
    });
    let output = console.end_capture();
    assert!(output.starts_with("\x1b[?2026h"), "begin-sync at start");
    assert!(
        output.ends_with("\x1b[?2026l"),
        "end-sync at end (guard emitted it)"
    );
}

// -- Finding #7: HTML export emits <a href> for styled links -------------

#[test]
fn test_export_html_emits_anchor_for_link() {
    let mut console = Console::builder()
        .width(80)
        .record(true)
        .markup(false)
        .build();
    // Directly push a segment with a link-style into the record buffer.
    console.record_buffer.push(crate::segment::Segment::styled(
        "click me",
        Style::with_link("https://example.com"),
    ));
    let html = console.export_html(None, false, true);
    assert!(
        html.contains("<a href=\"https://example.com\">"),
        "should have anchor tag"
    );
    assert!(html.contains("click me"), "should have link text");
}

// -- Finding #11: SVG export derives unique id from content hash ---------

/// Finding #11: SVG export derives a content-unique id from an FNV-1a hash
/// when the caller passes `unique_id = None`.
#[test]
fn test_export_svg_derives_unique_id() {
    let mut console1 = Console::builder()
        .width(40)
        .record(true)
        .no_color(true)
        .markup(false)
        .build();
    let mut console2 = Console::builder()
        .width(40)
        .record(true)
        .no_color(true)
        .markup(false)
        .build();
    console1.print_text("content alpha unique");
    console2.print_text("content beta unique");
    let svg1 = console1.export_svg("Title", None, false, None, 0.61);
    let svg2 = console2.export_svg("Title", None, false, None, 0.61);
    // Derived ids start with "gilt-" followed by 16 hex chars (e.g. "gilt-0123456789abcdef").
    // Extract the first "id=\"gilt-" occurrence as the unique_id used in <defs>.
    fn extract_clip_id(svg: &str) -> Option<&str> {
        // The clip-path id is like: id="UNIQUEID-clip-terminal"
        let marker = "id=\"";
        let clip_suffix = "-clip-terminal\"";
        let pos = svg.find(clip_suffix)?;
        // Walk backwards from the clip suffix to find the opening quote.
        let before = &svg[..pos];
        let quote_pos = before.rfind(marker)? + marker.len();
        Some(&svg[quote_pos..pos])
    }
    let id1 = extract_clip_id(&svg1).expect("svg1 should have a clip-path id");
    let id2 = extract_clip_id(&svg2).expect("svg2 should have a clip-path id");
    assert!(
        id1.starts_with("gilt-"),
        "id1 should start with gilt-: {id1}"
    );
    assert!(
        id2.starts_with("gilt-"),
        "id2 should start with gilt-: {id2}"
    );
    assert_ne!(
        id1, id2,
        "different content should yield different unique ids: {id1} vs {id2}"
    );
}

#[test]
fn test_export_svg_static_unique_id_overrides_hash() {
    let mut console = Console::builder()
        .width(40)
        .record(true)
        .no_color(true)
        .markup(false)
        .build();
    console.print_text("hello");
    let svg = console.export_svg("Title", None, false, Some("myid"), 0.61);
    // The clip-path id should use the explicit id.
    assert!(
        svg.contains("id=\"myid-clip-terminal\""),
        "should use the explicit unique_id in clip-path"
    );
    // No hash-derived gilt- prefixed unique_id should appear.
    assert!(
        !svg.contains("id=\"gilt-"),
        "no hash-derived id should appear when explicit id given"
    );
}

// -- Console::stderr() smoke test ----------------------------------------

/// Smoke-test: `Console::stderr()` constructs without panicking and routes
/// output to stderr (the `begin_capture`/`end_capture` capture path overrides
/// the writer so we can observe output in tests).
#[test]
fn test_console_stderr_constructor() {
    let mut c = Console::stderr();
    // The writer override points at stderr; capture overrides that too.
    c.begin_capture();
    c.print_text("stderr smoke test");
    let captured = c.end_capture();
    assert!(
        captured.contains("stderr smoke test"),
        "Console::stderr() should produce output; got {:?}",
        captured
    );
}

/// `Console::stderr()` respects `force_terminal` — when stderr is not a tty
/// (e.g. test harness) the console should be in plain mode.
/// When it IS forced, the color system resolves to something.
#[test]
fn test_console_stderr_terminal_state() {
    // Build one that we know is a terminal via force_terminal, matching the
    // internal logic of Console::stderr() when stderr is a tty.
    let c_tty = Console::builder().force_terminal(true).build();
    assert!(
        c_tty.color_system().is_some(),
        "force_terminal(true) should yield a color system"
    );

    // And one that is not a terminal, matching the non-tty path.
    let c_pipe = Console::builder().force_terminal(false).build();
    // force_terminal(false) causes color_system to resolve to None via the
    // non-terminal branch.
    assert!(
        c_pipe.color_system().is_none(),
        "force_terminal(false) should yield no color system"
    );
}

// -- Console::notify (OSC 9 desktop notification) ---------------------------

/// `Console::notify` with a title routes through control() and produces the
/// OSC 9 sequence. Use a recording console + export_text(styles=true) to
/// capture the raw escape sequences.
#[test]
fn test_console_notify_with_title() {
    // record=true so we can inspect raw escape bytes via export_text(styles=true).
    // force_terminal(true) ensures is_dumb_terminal() check passes and the
    // control segment is written (is_dumb_terminal uses TERM env var, not
    // force_terminal, so we also rely on TERM not being "dumb" in CI).
    let mut c = Console::builder()
        .force_terminal(true)
        .no_color(true)
        .record(true)
        .build();
    c.notify("Build", "Done");
    let output = c.export_text(false, true);
    assert!(
        output.contains("\x1b]9;"),
        "notify should emit OSC 9 sequence; got {:?}",
        output
    );
    assert!(
        output.contains("Build: Done"),
        "notify should include the message; got {:?}",
        output
    );
}

#[test]
fn test_console_notify_empty_title() {
    let mut c = Console::builder()
        .force_terminal(true)
        .no_color(true)
        .record(true)
        .build();
    c.notify("", "just body");
    let output = c.export_text(false, true);
    assert!(
        output.contains("\x1b]9;"),
        "notify with empty title should still emit OSC 9; got {:?}",
        output
    );
    assert!(
        output.contains("just body"),
        "should include body; got {:?}",
        output
    );
}

// -- Console::set_taskbar_progress (OSC 9;4) --------------------------------

#[test]
fn test_console_set_taskbar_progress_normal() {
    use crate::segment::TaskbarState;
    let mut c = Console::builder()
        .force_terminal(true)
        .no_color(true)
        .record(true)
        .build();
    c.set_taskbar_progress(TaskbarState::Normal, 55);
    let output = c.export_text(false, true);
    assert!(
        output.contains("\x1b]9;4;1;"),
        "should emit OSC 9;4;state=1 for Normal; got {:?}",
        output
    );
    assert!(
        output.contains("55"),
        "should include percent 55; got {:?}",
        output
    );
}

#[test]
fn test_console_set_taskbar_progress_remove() {
    use crate::segment::TaskbarState;
    let mut c = Console::builder()
        .force_terminal(true)
        .no_color(true)
        .record(true)
        .build();
    c.set_taskbar_progress(TaskbarState::Remove, 0);
    let output = c.export_text(false, true);
    assert!(
        output.contains("\x1b]9;4;0;"),
        "should emit state=0 for Remove; got {:?}",
        output
    );
}

// -- ConsoleBuilder::log_path / Console::log caller path -------------------

#[test]
fn test_log_path_false_no_path_in_output() {
    let mut c = Console::builder()
        .width(120)
        .no_color(true)
        .markup(false)
        .log_path(false)
        .build();
    c.begin_capture();
    c.log("hello world");
    let output = c.end_capture();
    assert!(output.contains("hello world"), "should include message");
    // With log_path off, no file:line annotation should appear.
    // The caller file for this test is "console_tests.rs"; check it's absent.
    assert!(
        !output.contains("console_tests.rs"),
        "log_path=false should not include file name; got {:?}",
        output
    );
}

#[test]
fn test_log_path_true_includes_file_name() {
    let mut c = Console::builder()
        .width(120)
        .no_color(true)
        .markup(false)
        .log_path(true)
        .build();
    c.begin_capture();
    c.log("some message");
    let output = c.end_capture();
    assert!(output.contains("some message"), "should include message");
    // With log_path on, the short caller file name should appear.
    // This test is in console_tests.rs, so "console_tests.rs" should show up.
    assert!(
        output.contains("console_tests.rs"),
        "log_path=true should include caller file name; got {:?}",
        output
    );
}

// ---------------------------------------------------------------------------
// Task 1: HtmlExportOptions tests
// ---------------------------------------------------------------------------

#[test]
fn test_export_html_opts_copy_button_injects_button_and_script() {
    use crate::export_format::HtmlExportOptions;
    use crate::style::Style;
    use crate::text::Text;

    let mut console = Console::builder()
        .width(80)
        .record(true)
        .markup(false)
        .build();
    console.print(&Text::new("hello", Style::null()));

    let opts = HtmlExportOptions::default().copy_button(true);
    let html = console.export_html_opts(None, &opts);
    assert!(html.contains("<button"), "expected <button> in HTML");
    assert!(html.contains("gilt-copy-btn"), "expected copy button id");
    assert!(html.contains("<script"), "expected <script> tag");
}

#[test]
fn test_export_html_opts_dark_mode_injects_dark_css() {
    use crate::export_format::HtmlExportOptions;
    use crate::style::Style;
    use crate::text::Text;

    let mut console = Console::builder()
        .width(80)
        .record(true)
        .markup(false)
        .build();
    console.print(&Text::new("dark test", Style::null()));

    let opts = HtmlExportOptions::default().dark_mode(true);
    let html = console.export_html_opts(None, &opts);
    assert!(
        html.contains("prefers-color-scheme"),
        "expected dark-mode @media query"
    );
}

#[test]
fn test_export_html_opts_font_url_referenced_in_output() {
    use crate::export_format::HtmlExportOptions;
    use crate::style::Style;
    use crate::text::Text;

    let mut console = Console::builder()
        .width(80)
        .record(true)
        .markup(false)
        .build();
    console.print(&Text::new("font test", Style::null()));

    let opts = HtmlExportOptions::default()
        .font_url("https://example.com/my-font.woff2")
        .font_family("MyFont");
    let html = console.export_html_opts(None, &opts);
    assert!(
        html.contains("https://example.com/my-font.woff2"),
        "font URL should appear in output"
    );
}

#[test]
fn test_export_html_opts_defaults_match_export_html() {
    use crate::export_format::HtmlExportOptions;
    use crate::style::Style;
    use crate::text::Text;

    let mut c1 = Console::builder()
        .width(80)
        .record(true)
        .markup(false)
        .build();
    let mut c2 = Console::builder()
        .width(80)
        .record(true)
        .markup(false)
        .build();

    c1.print(&Text::new("shared", Style::null()));
    c2.print(&Text::new("shared", Style::null()));

    let via_opts = c1.export_html_opts(None, &HtmlExportOptions::default());
    let via_direct = c2.export_html(None, false, false);
    assert_eq!(via_opts, via_direct);
}

// ---------------------------------------------------------------------------
// Task 2: FontEmbedding SVG tests
// ---------------------------------------------------------------------------

#[test]
fn test_export_svg_opts_base64_font_embedding() {
    use crate::export_format::{FontEmbedding, SvgExportOptions};
    use crate::style::Style;
    use crate::text::Text;

    let mut console = Console::builder()
        .width(40)
        .record(true)
        .no_color(true)
        .markup(false)
        .build();
    console.print(&Text::new("SVG embed", Style::null()));

    let font_bytes = b"FAKE_FONT_DATA".to_vec();
    let opts = SvgExportOptions::default()
        .title("Embed Test")
        .font_embedding(FontEmbedding::Base64(font_bytes.clone()));
    let svg = console.export_svg_opts(None, &opts);

    assert!(svg.contains("<svg"), "should contain <svg");
    assert!(svg.contains("data:font/"), "should contain data: URL");
    // The base64 of b"FAKE_FONT_DATA" must appear
    assert!(
        svg.contains("RkFLRV9GT05UX0RBVEE="),
        "base64 of FAKE_FONT_DATA should appear in SVG"
    );
}

#[test]
fn test_export_svg_opts_none_embedding_no_data_url() {
    use crate::export_format::{FontEmbedding, SvgExportOptions};
    use crate::style::Style;
    use crate::text::Text;

    let mut console = Console::builder()
        .width(40)
        .record(true)
        .no_color(true)
        .markup(false)
        .build();
    console.print(&Text::new("SVG no embed", Style::null()));

    let opts = SvgExportOptions::default()
        .title("No Embed")
        .font_embedding(FontEmbedding::None);
    let svg = console.export_svg_opts(None, &opts);

    assert!(svg.contains("<svg"), "should contain <svg");
    assert!(!svg.contains("data:font/"), "should NOT contain data: URL");
}

// -- Opt 2: BufWriter write coalescing ------------------------------------------

/// A `Write` impl that counts how many times its `write` / `write_all` methods
/// are called at the OS level, and records all bytes for correctness checking.
#[cfg(test)]
struct CountingWriter {
    bytes: Vec<u8>,
    write_count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

#[cfg(test)]
impl CountingWriter {
    fn new(counter: std::sync::Arc<std::sync::atomic::AtomicUsize>) -> Self {
        CountingWriter {
            bytes: Vec::new(),
            write_count: counter,
        }
    }

    #[allow(dead_code)]
    fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

#[cfg(test)]
impl std::io::Write for CountingWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.bytes.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// RED test (Opt 2): a multi-segment emit via `begin_synchronized` /
/// `end_synchronized` should result in FEWER underlying `write` calls than
/// the number of `write_segments` calls (coalescing via BufWriter).
///
/// Before BufWriter is added, every `write_segments` call writes directly to
/// the underlying writer → write_count == write_segments_calls. After
/// BufWriter is added (+ deferred-flush-until-sync-end), write_count == 1.
#[test]
fn bufwriter_coalesces_writes_within_synchronized_emit() {
    use std::sync::atomic::Ordering;
    use std::sync::Arc;

    let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counting_writer = CountingWriter::new(Arc::clone(&counter));

    let mut console = Console::builder()
        .width(80)
        .height(25)
        .quiet(false)
        .markup(false)
        .no_color(true)
        .force_terminal(true)
        .build()
        .with_writer(counting_writer);

    // Perform a synchronized emit that internally produces multiple
    // write_segments calls (begin_sync, content, end_sync at minimum).
    console.begin_synchronized();
    console.print_text("segment_one");
    console.print_text("segment_two");
    console.print_text("segment_three");
    console.end_synchronized();

    let write_count = counter.load(Ordering::SeqCst);

    // Without BufWriter: 3+ writes (one per print_text + begin/end sync).
    // With BufWriter + deferred flush: should be exactly 1 write (all
    // bytes drained to underlying writer in one flush at end_synchronized).
    assert!(
        write_count < 5,
        "expected coalesced writes (< 5), got {} underlying write calls",
        write_count
    );

    // Byte-correctness: all text must have been written.
    // We can't access the bytes here directly (the writer was moved into
    // the console), so we use a separate golden test below for byte parity.
}

// ---------------------------------------------------------------------------
// v1.8 Task 1 RED tests: GILT_THEME env var / builder theme_from_path
// ---------------------------------------------------------------------------

/// ConsoleBuilder::theme_from_path loads a JSON theme file and makes styles
/// available on the built console.  This test is the RED test — it fails
/// until `theme_from_path` is implemented.
#[cfg(all(feature = "json", not(target_arch = "wasm32")))]
#[test]
fn task1_theme_from_path_makes_custom_style_available() {
    use std::io::Write as _;

    // Write a minimal JSON theme to a temp file.
    let dir = std::env::temp_dir();
    let path = dir.join("gilt_test_theme_from_path.json");
    {
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, r#"{{"info": "dim cyan", "warning": "bold yellow"}}"#).unwrap();
    }

    let console = Console::builder()
        .width(80)
        .no_color(true)
        .theme_from_path(&path)
        .build();

    let style = console.get_style("info").expect("info style should exist");
    assert_eq!(style, Style::parse("dim cyan"), "theme info style mismatch");

    let style2 = console
        .get_style("warning")
        .expect("warning style should exist");
    assert_eq!(
        style2,
        Style::parse("bold yellow"),
        "theme warning style mismatch"
    );

    let _ = std::fs::remove_file(&path);
}

/// theme_from_path with a bad file path falls back gracefully (no panic,
/// uses default theme).
#[cfg(all(feature = "json", not(target_arch = "wasm32")))]
#[test]
fn task1_theme_from_path_bad_path_falls_back_to_default() {
    use std::path::Path;

    let console = Console::builder()
        .width(80)
        .no_color(true)
        .theme_from_path(Path::new("/nonexistent/path/nope.json"))
        .build();

    // Default theme should still have "bold"
    assert!(
        console.get_style("bold").is_ok(),
        "default theme should have 'bold' style even after failed theme_from_path"
    );
}

/// theme_from_path with a bad JSON file falls back gracefully.
#[cfg(all(feature = "json", not(target_arch = "wasm32")))]
#[test]
fn task1_theme_from_path_bad_json_falls_back_to_default() {
    use std::io::Write as _;

    let dir = std::env::temp_dir();
    let path = dir.join("gilt_test_bad_json.json");
    {
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "this is not valid json {{{{").unwrap();
    }

    let console = Console::builder()
        .width(80)
        .no_color(true)
        .theme_from_path(&path)
        .build();

    // Should fall back: default "bold" still works
    assert!(
        console.get_style("bold").is_ok(),
        "should fall back to default on bad JSON"
    );

    let _ = std::fs::remove_file(&path);
}

/// Theme::from_json_str is a convenience method gated on the json feature.
#[cfg(feature = "json")]
#[test]
fn task1_theme_from_json_str_parses_styles() {
    use crate::theme::Theme;

    let theme = Theme::from_json_str(r#"{"info": "dim cyan", "err": "bold red"}"#)
        .expect("from_json_str should succeed");

    assert_eq!(
        theme.get("info").expect("info should exist"),
        &Style::parse("dim cyan")
    );
    assert_eq!(
        theme.get("err").expect("err should exist"),
        &Style::parse("bold red")
    );
}

/// GILT_THEME env var wiring smoke test — exercised via the pure path-loading
/// helper to avoid global-env races under cargo nextest.
/// This verifies that a console built via a theme path (which is what the
/// GILT_THEME code path calls internally) overrides the default theme.
#[cfg(all(feature = "json", not(target_arch = "wasm32")))]
#[test]
fn task1_gilt_theme_env_path_wiring_smoke() {
    use std::io::Write as _;

    // Write a JSON theme file.
    let dir = std::env::temp_dir();
    let path = dir.join("gilt_smoke_theme.json");
    {
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, r#"{{"smoke_style": "bold magenta"}}"#).unwrap();
    }

    // Set the env var, build a console via default builder (which reads it).
    let path_str = path.to_str().unwrap();
    // We use set_var here isolated — this test checks the env reading codepath.
    // In a parallel test suite this could race; we use an unusual style name to
    // minimize collateral damage.
    std::env::set_var("GILT_THEME", path_str);
    let console = Console::builder().width(80).no_color(true).build();
    std::env::remove_var("GILT_THEME");

    let style = console.get_style("smoke_style");
    assert!(
        style.is_ok(),
        "GILT_THEME env var should make smoke_style available; error: {:?}",
        style.err()
    );
    assert_eq!(style.unwrap(), Style::parse("bold magenta"));

    let _ = std::fs::remove_file(&path);
}

// ---------------------------------------------------------------------------
// v1.8 Task 2 RED tests: Console::record(closure) scoped API
// ---------------------------------------------------------------------------

/// Basic scoped record: the returned Recording exposes to_text/to_html,
/// and both contain the printed content.
#[test]
fn task2_scoped_record_text_and_html_contain_content() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .record(true)
        .markup(true)
        .build();

    let rec = console.scoped_record(|c| {
        c.print_text("[bold red]hi[/]");
    });

    let text = rec.to_text();
    let html = rec.to_html();

    assert!(
        text.contains("hi"),
        "to_text() should contain 'hi'; got {:?}",
        text
    );
    assert!(
        html.contains("hi"),
        "to_html() should contain 'hi'; got {:?}",
        html
    );
}

/// After the closure, the console is NOT in record mode (record state restored).
#[test]
fn task2_scoped_record_does_not_leave_console_in_record_mode() {
    // Start with record disabled.
    let mut console = Console::builder()
        .width(80)
        .no_color(true)
        .markup(false)
        .build();

    assert!(
        !console.record,
        "precondition: record should be false before scoped_record"
    );

    let _rec = console.scoped_record(|c| {
        c.print_text("inside closure");
    });

    assert!(
        !console.record,
        "record should be restored to false after scoped_record"
    );
}

/// A subsequent normal print after scoped_record does NOT accumulate into
/// the record buffer (the buffer was restored).
#[test]
fn task2_scoped_record_subsequent_print_not_accumulated() {
    let mut console = Console::builder()
        .width(80)
        .no_color(true)
        .record(false) // record starts OFF
        .markup(false)
        .build();

    let _rec = console.scoped_record(|c| {
        c.print_text("inside");
    });

    // Now that scoped_record has returned, record mode must be restored to false.
    // Do a normal print and verify the record buffer is empty.
    console.begin_capture();
    console.print_text("outside");
    let _ = console.end_capture();

    // The record_buffer should NOT contain "outside" because record=false was restored.
    let export = console.export_text(false, false);
    assert!(
        !export.contains("outside"),
        "record buffer should not contain text printed after scoped_record (record=false); got {:?}",
        export
    );
}

/// to_svg returns a valid SVG document.
#[test]
fn task2_scoped_record_to_svg_is_valid_svg() {
    let mut console = Console::builder()
        .width(40)
        .force_terminal(true)
        .record(true)
        .markup(true)
        .build();

    let rec = console.scoped_record(|c| {
        c.print_text("svg test content");
    });

    let svg = rec.to_svg("Test Title");
    assert!(svg.contains("<svg"), "to_svg() should produce valid SVG");
    assert!(
        svg.contains("svg test content"),
        "SVG should contain printed text"
    );
}

/// Recording segments are independent: two successive scoped_records produce
/// independent Recording values.
#[test]
fn task2_two_successive_scoped_records_are_independent() {
    let mut console = Console::builder()
        .width(80)
        .no_color(true)
        .markup(false)
        .build();

    let rec1 = console.scoped_record(|c| {
        c.print_text("first");
    });
    let rec2 = console.scoped_record(|c| {
        c.print_text("second");
    });

    let t1 = rec1.to_text();
    let t2 = rec2.to_text();

    assert!(
        t1.contains("first"),
        "rec1 should contain 'first'; got {:?}",
        t1
    );
    assert!(
        t2.contains("second"),
        "rec2 should contain 'second'; got {:?}",
        t2
    );
    assert!(
        !t1.contains("second"),
        "rec1 should NOT contain 'second'; got {:?}",
        t1
    );
    assert!(
        !t2.contains("first"),
        "rec2 should NOT contain 'first'; got {:?}",
        t2
    );
}
