//! Tests extracted from traceback.rs for readability.
//! Wired in via `#[path = ...] mod tests;`.

use super::*;
use crate::console::Console;

// -- Sample backtrace strings for testing ---------------------------------

const SAMPLE_BACKTRACE: &str = "\
   0: std::backtrace::Backtrace::force_capture
             at /rustc/abc123/library/std/src/backtrace.rs:331:18
   1: core::panicking::panic_fmt
             at /rustc/abc123/library/core/src/panicking.rs:72:14
   2: myapp::myfunction
             at ./src/main.rs:42:9
   3: myapp::main
             at ./src/main.rs:10:5";

const SINGLE_FRAME_BACKTRACE: &str = "\
   0: myapp::main
             at ./src/main.rs:10:5";

const EMPTY_BACKTRACE: &str = "";

const FRAME_NO_LOCATION: &str = "\
   0: unknown_function
   1: myapp::main
             at ./src/main.rs:10:5";

// -- Frame tests ----------------------------------------------------------

#[test]
fn test_frame_new() {
    let frame = Frame::new("src/main.rs", Some(42), "main");
    assert_eq!(frame.filename, "src/main.rs");
    assert_eq!(frame.lineno, Some(42));
    assert_eq!(frame.name, "main");
    assert!(frame.source_line.is_none());
}

#[test]
fn test_frame_with_source_line() {
    let frame =
        Frame::new("src/main.rs", Some(42), "main").with_source_line("    println!(\"hello\");");
    assert_eq!(
        frame.source_line,
        Some("    println!(\"hello\");".to_string())
    );
}

#[test]
fn test_frame_display_with_lineno() {
    let frame = Frame::new("src/main.rs", Some(42), "main");
    let display = format!("{}", frame);
    assert!(display.contains("main"));
    assert!(display.contains("src/main.rs"));
    assert!(display.contains("42"));
}

#[test]
fn test_frame_display_without_lineno() {
    let frame = Frame::new("src/main.rs", None, "main");
    let display = format!("{}", frame);
    assert!(display.contains("main"));
    assert!(display.contains("src/main.rs"));
    assert!(!display.contains(':'));
}

#[test]
fn test_frame_equality() {
    let a = Frame::new("src/main.rs", Some(42), "main");
    let b = Frame::new("src/main.rs", Some(42), "main");
    assert_eq!(a, b);

    let c = Frame::new("src/lib.rs", Some(42), "main");
    assert_ne!(a, c);
}

#[test]
fn test_frame_clone() {
    let frame = Frame::new("src/main.rs", Some(42), "main").with_source_line("let x = 1;");
    let cloned = frame.clone();
    assert_eq!(frame, cloned);
}

// -- Parse backtrace tests ------------------------------------------------

#[test]
fn test_parse_backtrace_multiple_frames() {
    let frames = parse_backtrace(SAMPLE_BACKTRACE);
    assert_eq!(frames.len(), 4);

    assert_eq!(frames[0].name, "std::backtrace::Backtrace::force_capture");
    assert_eq!(
        frames[0].filename,
        "/rustc/abc123/library/std/src/backtrace.rs"
    );
    assert_eq!(frames[0].lineno, Some(331));

    assert_eq!(frames[1].name, "core::panicking::panic_fmt");
    assert_eq!(
        frames[1].filename,
        "/rustc/abc123/library/core/src/panicking.rs"
    );
    assert_eq!(frames[1].lineno, Some(72));

    assert_eq!(frames[2].name, "myapp::myfunction");
    assert_eq!(frames[2].filename, "./src/main.rs");
    assert_eq!(frames[2].lineno, Some(42));

    assert_eq!(frames[3].name, "myapp::main");
    assert_eq!(frames[3].filename, "./src/main.rs");
    assert_eq!(frames[3].lineno, Some(10));
}

#[test]
fn test_parse_backtrace_single_frame() {
    let frames = parse_backtrace(SINGLE_FRAME_BACKTRACE);
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].name, "myapp::main");
    assert_eq!(frames[0].filename, "./src/main.rs");
    assert_eq!(frames[0].lineno, Some(10));
}

#[test]
fn test_parse_backtrace_empty() {
    let frames = parse_backtrace(EMPTY_BACKTRACE);
    assert!(frames.is_empty());
}

#[test]
fn test_parse_backtrace_frame_without_location() {
    let frames = parse_backtrace(FRAME_NO_LOCATION);
    assert_eq!(frames.len(), 2);
    // First frame has no location info
    assert_eq!(frames[0].name, "unknown_function");
    assert!(frames[0].filename.is_empty());
    assert!(frames[0].lineno.is_none());
    // Second frame has location
    assert_eq!(frames[1].name, "myapp::main");
    assert_eq!(frames[1].filename, "./src/main.rs");
    assert_eq!(frames[1].lineno, Some(10));
}

#[test]
fn test_parse_backtrace_with_column() {
    let bt = "   0: myapp::handler\n             at ./src/handler.rs:15:23";
    let frames = parse_backtrace(bt);
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].filename, "./src/handler.rs");
    assert_eq!(frames[0].lineno, Some(15));
}

// -- Traceback constructors -----------------------------------------------

#[test]
fn test_from_backtrace() {
    let tb = Traceback::from_backtrace(SAMPLE_BACKTRACE);
    assert_eq!(tb.title, "Backtrace");
    assert_eq!(tb.frames.len(), 4);
    assert!(tb.message.is_empty());
}

#[test]
fn test_from_backtrace_empty() {
    let tb = Traceback::from_backtrace(EMPTY_BACKTRACE);
    assert_eq!(tb.title, "Backtrace");
    assert!(tb.frames.is_empty());
}

#[test]
fn test_from_panic() {
    let tb = Traceback::from_panic(
        "thread 'main' panicked at 'index out of bounds'",
        SAMPLE_BACKTRACE,
    );
    assert_eq!(tb.title, "Panic");
    assert!(tb.message.contains("index out of bounds"));
    assert_eq!(tb.frames.len(), 4);
}

#[test]
fn test_from_error_simple() {
    #[derive(Debug)]
    struct SimpleError;
    impl std::fmt::Display for SimpleError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "something went wrong")
        }
    }
    impl std::error::Error for SimpleError {}

    let err = SimpleError;
    let tb = Traceback::from_error(&err);
    assert!(!tb.title.is_empty());
    assert!(tb.message.contains("something went wrong"));
    assert!(tb.frames.is_empty());
}

#[test]
fn test_from_error_chain() {
    #[derive(Debug)]
    struct InnerError;
    impl std::fmt::Display for InnerError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "inner failure")
        }
    }
    impl std::error::Error for InnerError {}

    #[derive(Debug)]
    struct OuterError {
        source: InnerError,
    }
    impl std::fmt::Display for OuterError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "outer failure")
        }
    }
    impl std::error::Error for OuterError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Some(&self.source)
        }
    }

    let err = OuterError { source: InnerError };
    let tb = Traceback::from_error(&err);
    assert!(tb.message.contains("outer failure"));
    assert!(tb.message.contains("inner failure"));
    assert!(tb.message.contains("Caused by"));
}

// -- Builder methods ------------------------------------------------------

#[test]
fn test_builder_methods() {
    let tb = Traceback::new()
        .with_title("Custom Error")
        .with_message("details here")
        .with_show_locals(true)
        .with_width(120)
        .with_extra_lines(5)
        .with_theme("base16-mocha.dark")
        .with_word_wrap(false)
        .with_max_frames(50);

    assert_eq!(tb.title, "Custom Error");
    assert_eq!(tb.message, "details here");
    assert!(tb.show_locals);
    assert_eq!(tb.width, Some(120));
    assert_eq!(tb.extra_lines, 5);
    assert_eq!(tb.theme, "base16-mocha.dark");
    assert!(!tb.word_wrap);
    assert_eq!(tb.max_frames, 50);
}

#[test]
fn test_default_values() {
    let tb = Traceback::new();
    assert!(tb.title.is_empty());
    assert!(tb.message.is_empty());
    assert!(tb.frames.is_empty());
    assert!(!tb.show_locals);
    assert!(tb.width.is_none());
    assert_eq!(tb.extra_lines, 3);
    assert_eq!(tb.theme, "base16-ocean.dark");
    assert!(tb.word_wrap);
    assert_eq!(tb.max_frames, 100);
}

#[test]
fn test_default_trait() {
    let tb = Traceback::default();
    assert!(tb.title.is_empty());
    assert!(tb.frames.is_empty());
}

// -- Display / Debug traits -----------------------------------------------

#[test]
fn test_display_empty() {
    let tb = Traceback::new();
    let display = format!("{}", tb);
    assert!(display.is_empty());
}

#[test]
fn test_display_with_title_and_message() {
    let tb = Traceback::new()
        .with_title("Error")
        .with_message("something failed");
    let display = format!("{}", tb);
    assert!(display.contains("Error"));
    assert!(display.contains("something failed"));
}

#[test]
fn test_display_with_frames() {
    let tb = Traceback::from_backtrace(SAMPLE_BACKTRACE);
    let display = format!("{}", tb);
    assert!(display.contains("myapp::main"));
    assert!(display.contains("myapp::myfunction"));
}

#[test]
fn test_debug_trait() {
    let tb = Traceback::new().with_title("Debug Test");
    let debug = format!("{:?}", tb);
    assert!(debug.contains("Debug Test"));
}

// -- Max frames -----------------------------------------------------------

#[test]
fn test_max_frames_limit() {
    let mut tb = Traceback::new().with_max_frames(2);
    for i in 0..10 {
        tb.frames.push(Frame::new(
            "src/main.rs",
            Some(i + 1),
            &format!("func_{}", i),
        ));
    }

    // When rendering the content, only max_frames should be shown
    let content = tb.render_content();
    let plain = content.plain().to_string();
    // Should mention omitted frames
    assert!(plain.contains("omitted"));
}

#[test]
fn test_max_frames_not_truncated_when_under_limit() {
    let mut tb = Traceback::new().with_max_frames(10);
    tb.frames.push(Frame::new("src/main.rs", Some(1), "func_a"));
    tb.frames.push(Frame::new("src/main.rs", Some(2), "func_b"));

    let content = tb.render_content();
    let plain = content.plain().to_string();
    assert!(!plain.contains("omitted"));
    assert!(plain.contains("func_a"));
    assert!(plain.contains("func_b"));
}

// -- Renderable trait -----------------------------------------------------

#[test]
fn test_renderable_produces_segments() {
    let tb = Traceback::from_backtrace(SAMPLE_BACKTRACE);
    let console = Console::builder()
        .width(80)
        .no_color(true)
        .markup(false)
        .build();
    let options = console.options();
    let segments = tb.gilt_console(&console, &options);
    assert!(!segments.is_empty());
}

#[test]
fn test_renderable_contains_title() {
    let tb = Traceback::new()
        .with_title("TestError")
        .with_message("test message");
    let console = Console::builder()
        .width(80)
        .no_color(true)
        .markup(false)
        .build();
    let options = console.options();
    let segments = tb.gilt_console(&console, &options);
    let output: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(output.contains("TestError"));
    assert!(output.contains("test message"));
}

#[test]
fn test_renderable_contains_frame_info() {
    let mut tb = Traceback::new().with_title("Error");
    tb.frames.push(
        Frame::new("/some/path/file.rs", Some(42), "my_func").with_source_line("    let x = 1;"),
    );
    let console = Console::builder()
        .width(80)
        .no_color(true)
        .markup(false)
        .build();
    let options = console.options();
    let segments = tb.gilt_console(&console, &options);
    let output: String = segments.iter().map(|s| s.text.as_str()).collect();
    assert!(output.contains("file.rs"));
    assert!(output.contains("42"));
    assert!(output.contains("my_func"));
}

#[test]
fn test_renderable_wrapped_in_panel() {
    let tb = Traceback::new().with_title("PanelTest");
    let console = Console::builder()
        .width(40)
        .no_color(true)
        .markup(false)
        .build();
    let options = console.options();
    let segments = tb.gilt_console(&console, &options);
    let output: String = segments.iter().map(|s| s.text.as_str()).collect();
    // Panel uses ROUNDED box chars by default
    assert!(output.contains('\u{256d}') || output.contains('\u{2500}')); // top-left or horizontal
}

#[test]
fn test_renderable_with_width() {
    let tb = Traceback::new().with_title("WidthTest").with_width(60);
    let console = Console::builder()
        .width(120)
        .no_color(true)
        .markup(false)
        .build();
    let options = console.options();
    let segments = tb.gilt_console(&console, &options);
    // Segments should be produced
    assert!(!segments.is_empty());
}

#[test]
fn test_renderable_empty_traceback() {
    let tb = Traceback::new();
    let console = Console::builder()
        .width(80)
        .no_color(true)
        .markup(false)
        .build();
    let options = console.options();
    let segments = tb.gilt_console(&console, &options);
    // Even empty tracebacks should produce panel segments
    assert!(!segments.is_empty());
    let output: String = segments.iter().map(|s| s.text.as_str()).collect();
    // Should contain "Traceback" as default title
    assert!(output.contains("Traceback"));
}

// -- Error type name extraction -------------------------------------------

#[test]
fn test_error_type_name_from_debug() {
    #[derive(Debug)]
    struct MyCustomError(String);
    impl std::fmt::Display for MyCustomError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl std::error::Error for MyCustomError {}

    let err = MyCustomError("test".to_string());
    let name = error_type_name(&err);
    assert_eq!(name, "MyCustomError");
}

// -- Frame read_source_line -----------------------------------------------

#[test]
fn test_frame_read_source_line_nonexistent_file() {
    let mut frame = Frame::new("/nonexistent/path/foo.rs", Some(1), "main");
    frame.read_source_line();
    // Should remain None since the file doesn't exist
    assert!(frame.source_line.is_none());
}

#[test]
fn test_frame_read_source_line_no_lineno() {
    let mut frame = Frame::new("src/main.rs", None, "main");
    frame.read_source_line();
    assert!(frame.source_line.is_none());
}

#[test]
fn test_frame_read_source_line_zero_lineno() {
    let mut frame = Frame::new("src/main.rs", Some(0), "main");
    frame.read_source_line();
    assert!(frame.source_line.is_none());
}

#[test]
fn test_frame_read_source_line_already_set() {
    let mut frame = Frame::new("src/main.rs", Some(1), "main");
    frame.source_line = Some("existing".to_string());
    frame.read_source_line();
    assert_eq!(frame.source_line, Some("existing".to_string()));
}

// -- Traceback with manually constructed frames ---------------------------

#[test]
fn test_traceback_manual_frames() {
    let tb = Traceback::new()
        .with_title("ManualError")
        .with_message("manual test");

    let mut tb = tb;
    tb.frames.push(Frame::new("src/lib.rs", Some(10), "foo"));
    tb.frames.push(Frame::new("src/main.rs", Some(20), "bar"));

    assert_eq!(tb.frames.len(), 2);
    let display = format!("{}", tb);
    assert!(display.contains("foo"));
    assert!(display.contains("bar"));
    assert!(display.contains("ManualError"));
}

// -- Backtrace with varying whitespace ------------------------------------

#[test]
fn test_parse_backtrace_varying_whitespace() {
    let bt =
        "  0: func_a\n            at /path/a.rs:1:1\n  1: func_b\n            at /path/b.rs:2:2";
    let frames = parse_backtrace(bt);
    assert_eq!(frames.len(), 2);
    assert_eq!(frames[0].name, "func_a");
    assert_eq!(frames[1].name, "func_b");
}

// -- Backtrace with extra text between frames ----------------------------

#[test]
fn test_parse_backtrace_with_noise() {
    let bt = "stack backtrace:\n   0: func_a\n             at /path/a.rs:1:1\n   1: func_b\n             at /path/b.rs:2:2\nnote: Some additional info";
    let frames = parse_backtrace(bt);
    assert_eq!(frames.len(), 2);
    assert_eq!(frames[0].name, "func_a");
    assert_eq!(frames[1].name, "func_b");
}

// -- Traceback Display with all components --------------------------------

#[test]
fn test_display_complete_traceback() {
    let mut tb = Traceback::new()
        .with_title("RuntimeError")
        .with_message("division by zero");
    tb.frames
        .push(Frame::new("src/math.rs", Some(15), "divide"));
    tb.frames.push(Frame::new("src/main.rs", Some(8), "main"));

    let display = format!("{}", tb);
    assert!(display.contains("RuntimeError"));
    assert!(display.contains("division by zero"));
    assert!(display.contains("divide"));
    assert!(display.contains("main"));
    assert!(display.contains("src/math.rs"));
    assert!(display.contains("15"));
}

// -- Wave 5C: suppress_paths -------------------------------------------

fn tb_with_frames(filenames: &[&str]) -> Traceback {
    let mut tb = Traceback::new().with_title("E").with_message("m");
    for f in filenames {
        tb.frames.push(Frame::new(f, Some(1), "fn"));
    }
    tb
}

#[test]
fn suppress_paths_filters_matching_frames() {
    let tb = tb_with_frames(&[
        "/home/u/proj/src/main.rs",
        "/.cargo/registry/src/index/tokio-1.0/lib.rs",
        "/home/u/proj/src/lib.rs",
    ])
    .with_suppress(vec!["/.cargo/registry/src/".to_string()]);

    let visible = tb.visible_frames();
    assert_eq!(visible.len(), 2);
    for f in visible {
        assert!(
            !f.filename.contains("/.cargo/registry/src/"),
            "should have filtered: {}",
            f.filename
        );
    }
}

#[test]
fn suppress_all_frames_shows_placeholder() {
    let tb = tb_with_frames(&["/.cargo/registry/src/a.rs", "/.cargo/registry/src/b.rs"])
        .with_suppress(vec!["/.cargo/registry/src/".to_string()]);

    // Render via Display (which routes through gilt_console + Panel).
    let out = format!("{}", tb);
    assert!(
        out.contains("suppressed 2 frame"),
        "expected placeholder, got:\n{}",
        out
    );
}

#[test]
fn suppress_empty_passes_all_frames() {
    let tb = tb_with_frames(&["/a.rs", "/b.rs", "/c.rs"]);
    assert!(tb.suppress_paths.is_empty());
    assert_eq!(tb.visible_frames().len(), 3);
}

#[test]
fn suppress_path_matches_anywhere_in_filename() {
    let tb = tb_with_frames(&["/path/to/tokio-runtime/lib.rs", "/path/to/myapp/main.rs"])
        .with_suppress(vec!["tokio-".to_string()]);
    let visible = tb.visible_frames();
    assert_eq!(visible.len(), 1);
    assert!(visible[0].filename.contains("myapp"));
}
