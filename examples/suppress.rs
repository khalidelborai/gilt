//! Demonstrates traceback rendering with frame suppression/filtering.
//!
//! Port of Python rich's `suppress.py`. In the Python original, rich's
//! `install(suppress=[click])` hides frames from the `click` library to
//! reduce noise. In Rust/gilt, we demonstrate the concept by:
//!
//! 1. Showing a full traceback with both application and library frames.
//! 2. Showing a filtered traceback with only application frames.
//! 3. Showing how `max_frames` can further reduce output size.
//!
//! This illustrates how library frames can be identified and excluded from
//! tracebacks to focus attention on application code.
//!
//! Run with: `cargo run --example suppress --all-features`

use gilt::console::Console;
use gilt::rule::Rule;
use gilt::style::Style;
use gilt::text::Text;
use gilt::traceback::{Frame, Traceback};

/// Simulated application frames (the code the developer wrote).
fn app_frames() -> Vec<Frame> {
    vec![
        Frame::new("src/main.rs", Some(15), "main")
            .with_source_line("    let result = cli::run()?;"),
        Frame::new("src/cli.rs", Some(42), "cli::run")
            .with_source_line("    let config = parse_args()?;"),
        Frame::new("src/cli.rs", Some(78), "cli::parse_args")
            .with_source_line("    let value: i64 = arg.parse()?;"),
    ]
}

/// Simulated library frames (third-party code, e.g. an argument parsing library).
fn library_frames() -> Vec<Frame> {
    vec![
        Frame::new(
            "/home/user/.cargo/registry/src/clap-4.5.0/src/parser.rs",
            Some(312),
            "clap::parser::Parser::parse",
        )
        .with_source_line("    self.parse_subcommand()?;"),
        Frame::new(
            "/home/user/.cargo/registry/src/clap-4.5.0/src/parser.rs",
            Some(445),
            "clap::parser::Parser::parse_subcommand",
        )
        .with_source_line("    let matches = self.get_matches_from(args)?;"),
        Frame::new(
            "/home/user/.cargo/registry/src/clap-4.5.0/src/build/app.rs",
            Some(189),
            "clap::build::App::get_matches_from",
        )
        .with_source_line("    self.try_get_matches_from(it)?"),
        Frame::new(
            "/home/user/.cargo/registry/src/clap-4.5.0/src/build/app.rs",
            Some(210),
            "clap::build::App::try_get_matches_from",
        )
        .with_source_line("    self._do_parse(&mut it)?"),
    ]
}

fn main() {
    let mut console = Console::builder()
        .width(100)
        .force_terminal(true)
        .no_color(false)
        .build();

    // ── 1. Full traceback (all frames) ──────────────────────────────────────

    console.print(&Rule::with_title(
        "Full Traceback (application + library frames)",
    ));

    let mut all_frames = Vec::new();
    all_frames.extend(app_frames());
    all_frames.extend(library_frames());

    let tb_full = Traceback {
        title: "ValueError".to_string(),
        message: "invalid digit found in string\n\
                  \n\
                  Note: All frames shown, including 4 library frames from `clap`.\n\
                  In a real traceback, library internals add noise and distract from the bug."
            .to_string(),
        frames: all_frames,
        ..Traceback::new()
    };
    console.print(&tb_full);

    // ── 2. Suppressed traceback (application frames only) ───────────────────

    console.print(&Rule::with_title(
        "Suppressed Traceback (library frames filtered out)",
    ));

    // Filter: only keep frames that are NOT in the cargo registry
    let filtered_frames: Vec<Frame> = app_frames()
        .into_iter()
        .chain(library_frames())
        .filter(|f| !f.filename.contains(".cargo/registry"))
        .collect();

    let tb_suppressed = Traceback {
        title: "ValueError".to_string(),
        message: "invalid digit found in string\n\
                  \n\
                  Note: Library frames from `clap` have been suppressed.\n\
                  Only application frames are shown, making the error location clear.\n\
                  (4 frames from clap::parser and clap::build were hidden)"
            .to_string(),
        frames: filtered_frames,
        ..Traceback::new()
    };
    console.print(&tb_suppressed);

    // ── 3. Explanation panel ────────────────────────────────────────────────

    console.print(&Rule::with_title("How Suppression Works"));

    let explanation_style = Style::parse("italic").unwrap_or_else(|_| Style::null());
    let highlight_style = Style::parse("bold cyan").unwrap_or_else(|_| Style::null());

    let mut explanation = Text::empty();
    explanation.append_str(
        "In Python's rich, you suppress library frames with:\n\n",
        Some(explanation_style.clone()),
    );
    explanation.append_str(
        "    install(suppress=[click])\n\n",
        Some(highlight_style.clone()),
    );
    explanation.append_str(
        "In Rust with gilt, you can filter frames before building a Traceback:\n\n",
        Some(explanation_style.clone()),
    );
    explanation.append_str(
        "    let frames: Vec<Frame> = all_frames\n",
        Some(highlight_style.clone()),
    );
    explanation.append_str("        .into_iter()\n", Some(highlight_style.clone()));
    explanation.append_str(
        "        .filter(|f| !f.filename.contains(\".cargo/registry\"))\n",
        Some(highlight_style.clone()),
    );
    explanation.append_str("        .collect();\n\n", Some(highlight_style));
    explanation.append_str(
        "This keeps tracebacks focused on YOUR code, hiding noisy library internals.\n\
         The error message can note how many frames were suppressed for transparency.",
        Some(explanation_style),
    );

    console.print(&explanation);

    // ── 4. Deep library stack with max_frames ──────────────────────────────

    console.print(&Rule::with_title(
        "Deep Stack with max_frames=8 (20 total frames)",
    ));

    let mut deep_frames = app_frames();
    // Add many library frames to simulate a deep call stack
    for i in 0..17 {
        deep_frames.push(
            Frame::new(
                &format!(
                    "/home/user/.cargo/registry/src/framework-2.0.0/src/layer{}.rs",
                    i % 5
                ),
                Some(100 + i * 13),
                &format!("framework::layer{}::process", i % 5),
            )
            .with_source_line("    next.call(req)?"),
        );
    }

    let tb_deep = Traceback {
        title: "TimeoutError".to_string(),
        message: "request timed out after 30s\n\
                  \n\
                  Note: max_frames=8 truncates the middle of the 20-frame stack,\n\
                  showing the first 4 and last 4 frames with an omission marker."
            .to_string(),
        frames: deep_frames,
        max_frames: 8,
        ..Traceback::new()
    };
    console.print(&tb_deep);
}
