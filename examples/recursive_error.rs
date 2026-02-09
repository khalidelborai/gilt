//! Demonstrates gilt's Traceback rendering for deep error chains.
//!
//! Port of Python rich's `recursive_error.py`. In Python, `foo` and `bar`
//! call each other causing a RecursionError with a huge traceback that
//! Rich truncates with `max_frames`. Here we simulate this in Rust by:
//!
//! 1. Building a chain of nested errors (10 levels deep) to demonstrate
//!    `Traceback::from_error` with causal chains.
//! 2. Constructing a synthetic recursive call stack with manually built
//!    frames, then using `max_frames` to show truncation.
//!
//! Run with: `cargo run --example recursive_error --all-features`

use gilt::console::Console;
use gilt::rule::Rule;
use gilt::traceback::{Frame, Traceback};
use std::fmt;

// ---------------------------------------------------------------------------
// Custom error type that supports chaining (source)
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct RecursiveError {
    depth: usize,
    source: Option<Box<RecursiveError>>,
}

impl fmt::Display for RecursiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "recursion depth {} exceeded", self.depth)
    }
}

impl std::error::Error for RecursiveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e as &dyn std::error::Error)
    }
}

/// Build a chain of nested errors, `depth` levels deep.
fn build_error_chain(depth: usize) -> RecursiveError {
    let mut error = RecursiveError {
        depth: 0,
        source: None,
    };
    for d in 1..depth {
        error = RecursiveError {
            depth: d,
            source: Some(Box::new(error)),
        };
    }
    error
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let mut console = Console::builder()
        .width(100)
        .force_terminal(true)
        .no_color(false)
        .build();

    // ── 1. Error chain via Traceback::from_error ────────────────────────────

    console.print(&Rule::with_title("Error Chain (10 levels) via from_error"));

    let error = build_error_chain(10);
    console.print_error(&error);

    // ── 2. Simulated recursive call stack with max_frames truncation ────────

    console.print(&Rule::with_title(
        "Simulated Recursive Stack (100 frames, max_frames=20)",
    ));

    // Build 100 frames simulating foo -> bar -> foo -> bar ...
    let mut frames: Vec<Frame> = Vec::new();
    for i in 0..100 {
        let (func_name, file, line) = if i % 2 == 0 {
            ("recursive_error::foo", "examples/recursive_error.rs", 12)
        } else {
            ("recursive_error::bar", "examples/recursive_error.rs", 16)
        };
        frames.push(
            Frame::new(file, Some(line), func_name).with_source_line(if i % 2 == 0 {
                "    bar(n)"
            } else {
                "    foo(n)"
            }),
        );
    }

    let tb = Traceback {
        title: "RecursionError".to_string(),
        message: "maximum recursion depth exceeded\n\
                  \n\
                  The above error was caused by mutual recursion between foo() and bar().\n\
                  Rich (and gilt) can exclude frames in the middle to avoid huge tracebacks."
            .to_string(),
        frames,
        max_frames: 20,
        ..Traceback::new()
    };
    console.print(&tb);

    // ── 3. Smaller chain with source context ────────────────────────────────

    console.print(&Rule::with_title(
        "Realistic Error Chain with Source Context",
    ));

    let frames = vec![
        Frame::new("src/parser.rs", Some(145), "Parser::parse_expression")
            .with_source_line("    self.parse_expression()?  // recursive descent"),
        Frame::new("src/parser.rs", Some(89), "Parser::parse_term")
            .with_source_line("    let expr = self.parse_expression()?;"),
        Frame::new("src/parser.rs", Some(145), "Parser::parse_expression")
            .with_source_line("    self.parse_expression()?  // recursive descent"),
        Frame::new("src/parser.rs", Some(89), "Parser::parse_term")
            .with_source_line("    let expr = self.parse_expression()?;"),
        Frame::new("src/compiler.rs", Some(42), "Compiler::compile")
            .with_source_line("    let ast = parser.parse_expression()?;"),
        Frame::new("src/main.rs", Some(15), "main")
            .with_source_line("    compiler.compile(&source)?;"),
    ];

    let tb = Traceback {
        title: "StackOverflow".to_string(),
        message: "thread 'main' has overflowed its stack\n\
                  \n\
                  Caused by:\n  \
                  Infinite recursion in expression parser: \
                  parse_expression -> parse_term -> parse_expression -> ..."
            .to_string(),
        frames,
        ..Traceback::new()
    };
    console.print(&tb);
}
