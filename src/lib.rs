//! # gilt — Rich terminal formatting for Rust
//!
//! gilt is a Rust port of Python's [rich](https://github.com/Textualize/rich) library,
//! providing beautiful terminal output with styles, tables, trees, syntax highlighting,
//! progress bars, and more.
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

pub mod color_triplet;
pub mod errors;
pub mod palette;
pub mod terminal_theme;
pub mod color;
pub mod cells;
pub mod style;
pub mod segment;
pub mod wrap;
pub mod text;
pub mod measure;
pub mod control;
pub mod default_styles;
pub mod theme;
pub mod markup;
pub mod export_format;
pub mod console;
pub mod region;
pub mod ratio;
pub mod box_chars;
pub mod padding;
pub mod align_widget;
pub mod rule;
pub mod panel;
pub mod tree;
pub mod table;
pub mod columns;
pub mod layout;
pub mod constrain;
pub mod styled;
pub mod styled_str;
pub mod containers;
pub mod group;
pub mod screen;
pub mod bar;
pub mod highlighter;
pub mod ansi;
pub mod json;
pub mod filesize;
pub mod emoji_codes;
pub mod emoji_replace;
pub mod emoji;
pub mod spinners;
pub mod spinner;
pub mod markdown;
pub mod syntax;
pub mod progress_bar;
pub mod live_render;
pub mod live;
pub mod status;
pub mod prompt;
pub mod progress;
pub mod pretty;
pub mod pager;
pub mod traceback;
pub mod logging_handler;
pub mod scope;
pub mod gradient;
pub mod color_env;
pub mod inspect;
#[cfg(feature = "tracing")]
pub mod tracing_layer;
pub mod prelude;
#[cfg(feature = "miette")]
pub mod miette_handler;
#[cfg(feature = "eyre")]
pub mod eyre_handler;

#[cfg(feature = "derive")]
pub use gilt_derive::Table;

use std::sync::Mutex;
use once_cell::sync::Lazy;

/// Global default console instance, protected by a mutex for thread safety.
static DEFAULT_CONSOLE: Lazy<Mutex<console::Console>> =
    Lazy::new(|| Mutex::new(console::Console::new()));

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
