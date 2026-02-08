//! Traceback formatting module for terminal display.
//!
//! Provides the `Traceback` struct for rendering Rust backtraces, error chains,
//! and panic messages with syntax highlighting and source context. Adapted from
//! Python rich's `traceback.py` for Rust-specific backtrace formats.

use regex::Regex;

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::panel::Panel;
use crate::segment::Segment;
use crate::style::Style;
#[cfg(feature = "syntax")]
use crate::syntax::Syntax;
use crate::text::{Text, TextPart};

// ---------------------------------------------------------------------------
// Frame
// ---------------------------------------------------------------------------

/// A single frame in a backtrace/traceback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// File path where this frame originates.
    pub filename: String,
    /// Line number within the file (1-based), if known.
    pub lineno: Option<usize>,
    /// The function or method name.
    pub name: String,
    /// The source line at the error location, if available.
    pub source_line: Option<String>,
}

impl Frame {
    /// Create a new frame with the given details.
    pub fn new(filename: &str, lineno: Option<usize>, name: &str) -> Self {
        Frame {
            filename: filename.to_string(),
            lineno,
            name: name.to_string(),
            source_line: None,
        }
    }

    /// Set the source line for this frame.
    #[must_use]
    pub fn with_source_line(mut self, line: &str) -> Self {
        self.source_line = Some(line.to_string());
        self
    }

    /// Try to read the source line from the file system if we have a valid
    /// local path and line number.
    pub fn read_source_line(&mut self) {
        if self.source_line.is_some() {
            return;
        }
        if let Some(lineno) = self.lineno {
            if lineno == 0 {
                return;
            }
            let path = std::path::Path::new(&self.filename);
            if path.is_absolute() || self.filename.starts_with("./") {
                if let Ok(contents) = std::fs::read_to_string(path) {
                    if let Some(line) = contents.lines().nth(lineno - 1) {
                        self.source_line = Some(line.to_string());
                    }
                }
            }
        }
    }
}

impl std::fmt::Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.lineno {
            Some(n) => write!(f, "  {} ({}:{})", self.name, self.filename, n),
            None => write!(f, "  {} ({})", self.name, self.filename),
        }
    }
}

// ---------------------------------------------------------------------------
// Traceback
// ---------------------------------------------------------------------------

/// A formatted traceback that renders error information with syntax
/// highlighting and source context, similar to Python rich's Traceback.
#[derive(Debug, Clone)]
pub struct Traceback {
    /// Title displayed at the top (error type or custom title).
    pub title: String,
    /// Error message displayed at the bottom.
    pub message: String,
    /// Stack frames, ordered from outermost to innermost.
    pub frames: Vec<Frame>,
    /// Reserved for future use: display local variables.
    pub show_locals: bool,
    /// Optional fixed width for the output.
    pub width: Option<usize>,
    /// Number of context lines to show around the highlighted source line.
    pub extra_lines: usize,
    /// Syntax highlighting theme name (e.g. "base16-ocean.dark").
    pub theme: String,
    /// Whether to word-wrap long source lines.
    pub word_wrap: bool,
    /// Maximum number of frames to display.
    pub max_frames: usize,
}

impl Traceback {
    /// Create a new empty traceback with default settings.
    pub fn new() -> Self {
        Traceback {
            title: String::new(),
            message: String::new(),
            frames: Vec::new(),
            show_locals: false,
            width: None,
            extra_lines: 3,
            theme: "base16-ocean.dark".to_string(),
            word_wrap: true,
            max_frames: 100,
        }
    }

    // -- Constructors -------------------------------------------------------

    /// Parse a `std::backtrace::Backtrace` string into a `Traceback`.
    ///
    /// Expects the format produced by `Backtrace::force_capture().to_string()`:
    /// ```text
    ///    0: std::backtrace::Backtrace::force_capture
    ///              at /rustc/.../backtrace.rs:331:18
    ///    1: myapp::main
    ///              at ./src/main.rs:10:5
    /// ```
    pub fn from_backtrace(bt: &str) -> Self {
        let frames = parse_backtrace(bt);
        Traceback {
            title: "Backtrace".to_string(),
            message: String::new(),
            frames,
            ..Traceback::new()
        }
    }

    /// Create a `Traceback` from an error chain.
    ///
    /// Walks the chain via `.source()` to collect all nested errors. The
    /// outermost error becomes the title, and nested errors are appended
    /// to the message.
    pub fn from_error(error: &dyn std::error::Error) -> Self {
        let title = format!("{}", error);
        let mut chain_messages: Vec<String> = Vec::new();
        let mut current = error.source();
        while let Some(cause) = current {
            chain_messages.push(format!("{}", cause));
            current = cause.source();
        }
        let message = if chain_messages.is_empty() {
            String::new()
        } else {
            format!("Caused by:\n  {}", chain_messages.join("\n  "))
        };
        Traceback {
            title: error_type_name(error),
            message: format!(
                "{}{}{}",
                title,
                if message.is_empty() { "" } else { "\n" },
                message
            ),
            frames: Vec::new(),
            ..Traceback::new()
        }
    }

    /// Create a `Traceback` from a panic message and a backtrace string.
    pub fn from_panic(message: &str, backtrace: &str) -> Self {
        let frames = parse_backtrace(backtrace);
        Traceback {
            title: "Panic".to_string(),
            message: message.to_string(),
            frames,
            ..Traceback::new()
        }
    }

    // -- Builder methods ----------------------------------------------------

    /// Set the title.
    #[must_use]
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    /// Set the error message.
    #[must_use]
    pub fn with_message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }

    /// Set whether to show locals (reserved for future use).
    #[must_use]
    pub fn with_show_locals(mut self, show: bool) -> Self {
        self.show_locals = show;
        self
    }

    /// Set a fixed width.
    #[must_use]
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the number of extra context lines around the source line.
    #[must_use]
    pub fn with_extra_lines(mut self, lines: usize) -> Self {
        self.extra_lines = lines;
        self
    }

    /// Set the syntax highlighting theme.
    #[must_use]
    pub fn with_theme(mut self, theme: &str) -> Self {
        self.theme = theme.to_string();
        self
    }

    /// Set whether to word-wrap source code.
    #[must_use]
    pub fn with_word_wrap(mut self, wrap: bool) -> Self {
        self.word_wrap = wrap;
        self
    }

    /// Set the maximum number of frames to display.
    #[must_use]
    pub fn with_max_frames(mut self, max: usize) -> Self {
        self.max_frames = max;
        self
    }

    // -- Internal rendering -------------------------------------------------

    /// Build the inner content `Text` that goes inside the Panel.
    ///
    /// This produces a simple text-only representation of the frames. The
    /// full `Renderable` implementation adds syntax highlighting on top.
    #[allow(dead_code)]
    fn render_content(&self) -> Text {
        let mut parts: Vec<TextPart> = Vec::new();

        // Determine how many frames to show
        let frame_count = self.frames.len();
        let show_count = frame_count.min(self.max_frames);
        let truncated = frame_count > self.max_frames;

        // Collect frame indices to display
        let indices: Vec<usize> = if truncated {
            let half = self.max_frames / 2;
            let mut idx: Vec<usize> = (0..half).collect();
            idx.extend(frame_count - half..frame_count);
            idx
        } else {
            (0..frame_count).collect()
        };

        let mut inserted_ellipsis = false;

        for (pos, &frame_idx) in indices.iter().enumerate() {
            // Insert the ellipsis marker at the split point
            if truncated && !inserted_ellipsis && frame_idx >= self.max_frames / 2 {
                inserted_ellipsis = true;
                let omitted = frame_count - show_count;
                let msg = format!("\n  ... {} frames omitted ...\n", omitted);
                parts.push(TextPart::Styled(
                    msg,
                    Style::parse("dim italic").unwrap_or_else(|_| Style::null()),
                ));
            }

            let frame = &self.frames[frame_idx];

            // File location line
            let location = match frame.lineno {
                Some(n) => format!("{}:{}", frame.filename, n),
                None => frame.filename.clone(),
            };

            parts.push(TextPart::Styled(
                format!("  File \"{}\"", location),
                Style::parse("green").unwrap_or_else(|_| Style::null()),
            ));
            parts.push(TextPart::Styled(
                format!(", in {}", frame.name),
                Style::parse("magenta").unwrap_or_else(|_| Style::null()),
            ));
            parts.push(TextPart::Raw("\n".to_string()));

            // Source line if available
            if let Some(ref source) = frame.source_line {
                let trimmed = source.trim();
                if !trimmed.is_empty() {
                    parts.push(TextPart::Raw(format!("    {}", trimmed)));
                    parts.push(TextPart::Raw("\n".to_string()));
                }
            }

            // Add a blank line between frames (except after the last one)
            if pos + 1 < indices.len() {
                parts.push(TextPart::Raw("\n".to_string()));
            }
        }

        Text::assemble(&parts, Style::null())
    }
}

impl Default for Traceback {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Traceback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.title.is_empty() {
            writeln!(f, "{}", self.title)?;
        }
        for frame in &self.frames {
            writeln!(f, "{}", frame)?;
        }
        if !self.message.is_empty() {
            write!(f, "{}", self.message)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Renderable
// ---------------------------------------------------------------------------

impl Renderable for Traceback {
    fn rich_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        #[cfg(feature = "syntax")]
        let panel_width = self.width.unwrap_or(options.max_width);

        // Build the inner content
        let mut content_parts: Vec<TextPart> = Vec::new();

        // Determine how many frames to show
        let frame_count = self.frames.len();
        let show_count = frame_count.min(self.max_frames);
        let truncated = frame_count > self.max_frames;

        let frames_to_show: Vec<&Frame> = if truncated {
            let half = self.max_frames / 2;
            let mut combined: Vec<&Frame> = self.frames.iter().take(half).collect();
            combined.extend(self.frames.iter().skip(frame_count - half));
            combined
        } else {
            self.frames.iter().collect()
        };

        let actual_show = frames_to_show.len();
        let half_mark = if truncated {
            self.max_frames / 2
        } else {
            actual_show + 1
        };

        for (i, frame) in frames_to_show.iter().enumerate() {
            // Insert ellipsis marker at the halfway point for truncated traces
            if truncated && i == half_mark {
                let omitted = frame_count - show_count;
                let msg = format!("\n... {} frames omitted ...\n\n", omitted);
                content_parts.push(TextPart::Styled(
                    msg,
                    Style::parse("dim italic").unwrap_or_else(|_| Style::null()),
                ));
            }

            // File location line
            let location = match frame.lineno {
                Some(n) => format!("{}:{}", frame.filename, n),
                None => frame.filename.clone(),
            };

            content_parts.push(TextPart::Styled(
                format!("File \"{}\"", location),
                Style::parse("green").unwrap_or_else(|_| Style::null()),
            ));
            content_parts.push(TextPart::Styled(
                format!(", in {}", frame.name),
                Style::parse("magenta").unwrap_or_else(|_| Style::null()),
            ));
            content_parts.push(TextPart::Raw("\n".to_string()));

            // Source context: try to read the file and show context lines
            #[allow(unused_mut)]
            let mut showed_syntax = false;

            #[cfg(feature = "syntax")]
            if let Some(lineno) = frame.lineno {
                if lineno > 0 {
                    let path = std::path::Path::new(&frame.filename);
                    if (path.is_absolute() || frame.filename.starts_with("./")) && path.exists() {
                        if let Ok(file_contents) = std::fs::read_to_string(path) {
                            let total_lines = file_contents.lines().count();
                            if lineno <= total_lines {
                                let start = lineno.saturating_sub(self.extra_lines).max(1);
                                let end = (lineno + self.extra_lines).min(total_lines);

                                let context: String = file_contents
                                    .lines()
                                    .enumerate()
                                    .filter(|(i, _)| {
                                        let n = i + 1;
                                        n >= start && n <= end
                                    })
                                    .map(|(_, line)| line)
                                    .collect::<Vec<_>>()
                                    .join("\n");

                                // Determine language from file extension
                                let ext =
                                    path.extension().and_then(|e| e.to_str()).unwrap_or("txt");

                                let syntax = Syntax::new(&context, ext)
                                    .with_theme(&self.theme)
                                    .with_line_numbers(true)
                                    .with_start_line(start)
                                    .with_highlight_lines(vec![lineno])
                                    .with_word_wrap(self.word_wrap);

                                let syntax_segments = syntax.rich_console(
                                    console,
                                    &options.update_width(panel_width.saturating_sub(4)),
                                );
                                if !syntax_segments.is_empty() {
                                    // Collect syntax output as a styled text block
                                    for seg in &syntax_segments {
                                        content_parts.push(TextPart::Raw(seg.text.to_string()));
                                    }
                                    showed_syntax = true;
                                }
                            }
                        }
                    }
                }
            }

            // Fallback: show the single source line if we didn't render syntax
            if !showed_syntax {
                if let Some(ref source) = frame.source_line {
                    let trimmed = source.trim();
                    if !trimmed.is_empty() {
                        content_parts.push(TextPart::Raw(format!("    {}\n", trimmed)));
                    }
                }
            }

            // Blank line between frames
            if i + 1 < actual_show {
                content_parts.push(TextPart::Raw("\n".to_string()));
            }
        }

        // Error message at the bottom
        if !self.message.is_empty() {
            content_parts.push(TextPart::Raw("\n".to_string()));
            content_parts.push(TextPart::Styled(
                self.message.clone(),
                Style::parse("bold").unwrap_or_else(|_| Style::null()),
            ));
        }

        let content_text = Text::assemble(&content_parts, Style::null());

        // Wrap in a Panel
        let title_text = if self.title.is_empty() {
            Text::styled(
                "Traceback",
                Style::parse("bold red").unwrap_or_else(|_| Style::null()),
            )
        } else {
            Text::styled(
                &self.title,
                Style::parse("bold red").unwrap_or_else(|_| Style::null()),
            )
        };

        let panel = Panel::new(content_text)
            .title(title_text)
            .border_style(Style::parse("red").unwrap_or_else(|_| Style::null()))
            .expand(true);

        let panel_opts = if let Some(w) = self.width {
            options.update_width(w)
        } else {
            options.clone()
        };

        panel.rich_console(console, &panel_opts)
    }
}

// ---------------------------------------------------------------------------
// Backtrace parsing
// ---------------------------------------------------------------------------

/// Parse a Rust backtrace string into a Vec of Frames.
///
/// Handles the standard Rust backtrace format:
/// ```text
///    0: rust_begin_unwind
///              at /rustc/hash/library/std/src/panicking.rs:652:5
///    1: core::panicking::panic_fmt
///              at /rustc/hash/library/core/src/panicking.rs:72:14
///    2: myapp::myfunction
///              at ./src/main.rs:42:9
/// ```
fn parse_backtrace(bt: &str) -> Vec<Frame> {
    // Use a lazy regex that matches frame entries
    let frame_re = Regex::new(r"(?m)^\s*(\d+):\s+(.+?)$").expect("invalid frame regex");
    let location_re =
        Regex::new(r"(?m)^\s+at\s+(.+?):(\d+)(?::(\d+))?\s*$").expect("invalid location regex");

    let lines: Vec<&str> = bt.lines().collect();
    let mut frames = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        if let Some(captures) = frame_re.captures(line) {
            let name = captures
                .get(2)
                .map(|m| m.as_str())
                .unwrap_or("")
                .trim()
                .to_string();

            // Check if the next line has location info
            let mut filename = String::new();
            let mut lineno = None;

            if i + 1 < lines.len() {
                if let Some(loc_captures) = location_re.captures(lines[i + 1]) {
                    filename = loc_captures
                        .get(1)
                        .map(|m| m.as_str())
                        .unwrap_or("")
                        .to_string();
                    lineno = loc_captures
                        .get(2)
                        .and_then(|m| m.as_str().parse::<usize>().ok());
                    i += 1; // consume the location line
                }
            }

            let mut frame = Frame::new(&filename, lineno, &name);
            frame.read_source_line();
            frames.push(frame);
        }
        i += 1;
    }

    frames
}

/// Extract a short type name from an error reference.
///
/// Since Rust does not have built-in runtime type names for trait objects, we
/// use a simple heuristic based on the Debug output. For well-known error
/// types, this produces a reasonable label.
fn error_type_name(error: &dyn std::error::Error) -> String {
    let debug = format!("{:?}", error);
    // Try to extract a type name from the Debug output.
    // Many errors format as `TypeName { ... }` or `TypeName(...)`.
    if let Some(paren) = debug.find('(') {
        let brace = debug.find('{').unwrap_or(debug.len());
        let end = paren.min(brace);
        let name = debug[..end].trim();
        if !name.is_empty() && !name.contains(' ') {
            return name.to_string();
        }
    } else if let Some(brace) = debug.find('{') {
        let name = debug[..brace].trim();
        if !name.is_empty() && !name.contains(' ') {
            return name.to_string();
        }
    }
    "Error".to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
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
        let frame = Frame::new("src/main.rs", Some(42), "main")
            .with_source_line("    println!(\"hello\");");
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
        let segments = tb.rich_console(&console, &options);
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
        let segments = tb.rich_console(&console, &options);
        let output: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(output.contains("TestError"));
        assert!(output.contains("test message"));
    }

    #[test]
    fn test_renderable_contains_frame_info() {
        let mut tb = Traceback::new().with_title("Error");
        tb.frames.push(
            Frame::new("/some/path/file.rs", Some(42), "my_func")
                .with_source_line("    let x = 1;"),
        );
        let console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build();
        let options = console.options();
        let segments = tb.rich_console(&console, &options);
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
        let segments = tb.rich_console(&console, &options);
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
        let segments = tb.rich_console(&console, &options);
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
        let segments = tb.rich_console(&console, &options);
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
        let bt = "  0: func_a\n            at /path/a.rs:1:1\n  1: func_b\n            at /path/b.rs:2:2";
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
}
