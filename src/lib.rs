//! # gilt — Rich terminal formatting for Rust
//!
//! gilt is a Rust port of Python's [rich](https://github.com/Textualize/rich) library,
//! providing beautiful terminal output with styles, tables, trees, syntax highlighting,
//! progress bars, and more — all rendered as ANSI escape sequences.
//!
//! ## Quick Start
//!
//! ```rust
//! use gilt::prelude::*;
//!
//! let mut console = Console::builder().width(80).build();
//! console.begin_capture();
//! console.print_text("Hello, [bold magenta]gilt[/bold magenta]!");
//! let output = console.end_capture();
//! assert!(output.contains("Hello"));
//! ```
//!
//! ## Global Console
//!
//! For quick one-off output, use the global convenience functions:
//!
//! ```rust,no_run
//! gilt::print_text("Hello, [bold]world[/bold]!");
//! # #[cfg(feature = "json")]
//! gilt::print_json(r#"{"name": "gilt"}"#);
//! gilt::inspect(&vec![1, 2, 3]);
//! ```
//!
//! ## Core Modules
//!
//! | Module | Description |
//! |--------|-------------|
//! | [`console`] | Console output with color detection, capture, and export |
//! | [`style`] | Text styles (bold, italic, colors, extended underlines) |
//! | [`text`] | Rich text with markup parsing and word wrapping |
//! | [`table`] | Unicode box-drawing tables with alignment and striping |
//! | [`panel`] | Bordered content panels with titles |
//! | [`tree`] | Hierarchical tree display with guide lines |
//! | [`syntax`] | Code highlighting via syntect (150+ languages) |
//! | [`markdown`] | Terminal-rendered Markdown |
//! | [`progress`] | Multi-bar progress display with ETA and speed |
//! | [`live`] | Live-updating terminal display |
//! | [`gradient`] | True-color RGB gradient text |
//! | [`styled_str`] | Stylize trait for `"text".bold().red()` chaining |
//! | [`mod@inspect`] | Debug any value with rich formatting |
//!
//! ## Feature Flags
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `json` | Yes | JSON pretty-printing (`serde`, `serde_json`) |
//! | `markdown` | Yes | Terminal-rendered Markdown (`pulldown-cmark`) |
//! | `syntax` | Yes | Syntax highlighting (`syntect`) |
//! | `interactive` | Yes | Password input (`rpassword`) |
//! | `tracing` | No | [`tracing`](https://docs.rs/tracing) subscriber with gilt formatting |
//! | `derive` | No | `#[derive(Table)]` proc macro for struct-to-table |
//! | `miette` | No | [`miette`](https://docs.rs/miette) diagnostic report handler |
//! | `eyre` | No | [`eyre`](https://docs.rs/eyre) error report handler |
//! | `anstyle` | No | Bidirectional `From` conversions with [`anstyle`](https://docs.rs/anstyle) |

pub mod accessibility;
pub mod align_widget;
pub mod ansi;
#[cfg(feature = "anstyle")]
pub mod anstyle_adapter;
pub mod bar;
pub mod box_chars;
pub mod cells;
pub mod color;
pub mod color_env;
pub mod color_triplet;
pub mod columns;
pub mod console;
pub mod constrain;
pub mod containers;
pub mod control;
pub mod default_styles;
pub mod emoji;
pub mod emoji_codes;
pub mod emoji_replace;
pub mod errors;
pub mod export_format;
#[cfg(feature = "eyre")]
pub mod eyre_handler;
pub mod filesize;
pub mod gradient;
pub mod group;
pub mod highlighter;
pub mod inspect;
#[cfg(feature = "json")]
pub mod json;
pub mod layout;
pub mod live;
pub mod live_render;
pub mod logging_handler;
#[cfg(feature = "markdown")]
pub mod markdown;
pub mod markup;
pub mod measure;
#[cfg(feature = "miette")]
pub mod miette_handler;
pub mod padding;
pub mod pager;
pub mod palette;
pub mod panel;
pub mod prelude;
pub mod pretty;
pub mod progress;
pub mod progress_bar;
pub mod prompt;
pub mod ratio;
pub mod region;
pub mod rule;
pub mod scope;
pub mod screen;
pub mod segment;
pub mod spinner;
pub mod spinners;
pub mod status;
pub mod style;
pub mod styled;
pub mod styled_str;
#[cfg(feature = "syntax")]
pub mod syntax;
pub mod table;
pub mod terminal_theme;
pub mod text;
pub mod theme;
pub mod traceback;
#[cfg(feature = "tracing")]
pub mod tracing_layer;
pub mod tree;
pub mod wrap;

#[cfg(feature = "derive")]
pub use gilt_derive::Table;

use std::sync::LazyLock;
use std::sync::Mutex;

/// Global default console instance, protected by a mutex for thread safety.
static DEFAULT_CONSOLE: LazyLock<Mutex<console::Console>> =
    LazyLock::new(|| Mutex::new(console::Console::new()));

/// Access the global default console.
///
/// Locks the mutex and calls the provided closure with a mutable reference
/// to the console. Panics if the mutex is poisoned.
pub fn with_console<F, R>(f: F) -> R
where
    F: FnOnce(&mut console::Console) -> R,
{
    let mut c = DEFAULT_CONSOLE.lock().expect("console mutex poisoned");
    f(&mut c)
}

/// Print a renderable to the default console.
///
/// This is the Rust equivalent of Python rich's `rich.print()`.
pub fn print(renderable: &dyn console::Renderable) {
    with_console(|c| c.print(renderable));
}

/// Print a text string to the default console, processing markup.
pub fn print_text(text: &str) {
    with_console(|c| c.print_text(text));
}

/// Pretty-print JSON to the default console.
#[cfg(feature = "json")]
pub fn print_json(json: &str) {
    with_console(|c| c.print_json(json));
}

/// Inspect a value in the default console.
///
/// Displays the type name, Debug representation, and optional docs
/// in a styled panel.
pub fn inspect<T: std::fmt::Debug + 'static>(value: &T) {
    with_console(|c| c.inspect(value));
}
