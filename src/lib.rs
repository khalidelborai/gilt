//! # gilt -- Rich Terminal Formatting for Rust
//!
//! [![Crates.io](https://img.shields.io/crates/v/gilt.svg)](https://crates.io/crates/gilt)
//! [![Documentation](https://docs.rs/gilt/badge.svg)](https://docs.rs/gilt)
//! [![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/khalidelborai/gilt/blob/main/LICENSE)
//!
//! A Rust port of Python's [rich](https://github.com/Textualize/rich) library -- beautiful
//! terminal output with styles, tables, trees, syntax highlighting, progress bars, and
//! more, all rendered as ANSI escape sequences.
//!
//! # Quick Start
//!
//! Add gilt to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! gilt = "0.6"
//! ```
//!
//! Then use the [`prelude`] for convenient access to common types:
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
//! For quick one-off output, use the **global console** functions:
//!
//! ```rust,no_run
//! gilt::print_text("Hello, [bold]world[/bold]!");
//! gilt::inspect(&vec![1, 2, 3]);
//! ```
//!
//! # The Console
//!
//! [`Console`](console::Console) is the central type in gilt. It manages terminal capabilities,
//! drives the rendering pipeline, and handles output buffering, capture, and export.
//!
//! ## Creating a Console
//!
//! ```rust
//! use gilt::console::Console;
//!
//! // Defaults: auto-detect terminal size, TrueColor, markup on
//! let console = Console::new();
//!
//! // Builder pattern for full control
//! let console = Console::builder()
//!     .width(120)
//!     .color_system("truecolor")
//!     .record(true)       // enable export_html / export_svg
//!     .no_color(false)
//!     .build();
//! ```
//!
//! ## Printing
//!
//! ```rust
//! # use gilt::prelude::*;
//! let mut console = Console::builder().width(80).build();
//!
//! // Print with rich markup
//! console.print_text("[bold red]Error:[/bold red] something went wrong");
//!
//! // Print any Renderable (Panel, Table, Tree, ...)
//! let panel = Panel::new(Text::new("inside a box", Style::null()));
//! console.print(&panel);
//! ```
//!
//! ## Capture and Export
//!
//! Capture lets you collect console output as a plain string instead of writing
//! to the terminal:
//!
//! ```rust
//! # use gilt::console::Console;
//! let mut console = Console::builder().width(60).build();
//! console.begin_capture();
//! console.print_text("[bold]Captured![/bold]");
//! let plain = console.end_capture();
//! assert!(plain.contains("Captured!"));
//! ```
//!
//! For rich export, enable `record(true)` on the builder, then call
//! [`export_text`](console::Console::export_text),
//! [`export_html`](console::Console::export_html), or
//! [`export_svg`](console::Console::export_svg):
//!
//! ```rust
//! # use gilt::console::Console;
//! let mut console = Console::builder().width(60).record(true).build();
//! console.print_text("[green]Recorded[/green]");
//! let html = console.export_html(None, true, true);
//! assert!(html.contains("<span"));
//! ```
//!
//! # Text and Styling
//!
//! ## Style
//!
//! [`Style`](style::Style) represents a combination of foreground/background color, text attributes
//! (bold, italic, underline, ...), extended underline styles, and hyperlinks.
//!
//! ```rust
//! use gilt::style::Style;
//!
//! // Parse from a human-readable definition
//! let style = Style::parse("bold red on white").unwrap();
//! let subtle = Style::parse("dim italic #808080").unwrap();
//!
//! // Combine styles with the + operator (right side wins on conflicts)
//! let combined = style + subtle;
//! ```
//!
//! ## Markup
//!
//! gilt supports inline markup tags in strings, similar to BBCode:
//!
//! ```rust
//! # use gilt::console::Console;
//! let mut c = Console::builder().width(60).build();
//! c.begin_capture();
//! c.print_text("[bold]Bold[/bold], [italic green]green italic[/italic green]");
//! let out = c.end_capture();
//! assert!(out.contains("Bold"));
//! ```
//!
//! Closing tags match their opening tag, or use `[/]` to close the most recent:
//! `"[bold red]error[/] normal text"`.
//!
//! ## Text
//!
//! [`Text`](text::Text) is the fundamental rich-text type, storing a plain string plus styled spans.
//!
//! ```rust
//! use gilt::text::Text;
//! use gilt::style::Style;
//!
//! // Plain text
//! let text = Text::new("Hello, world!", Style::null());
//!
//! // Styled text (entire string has one style)
//! let bold = Text::styled("Important", Style::parse("bold").unwrap());
//!
//! // From markup (parses [tags])
//! let rich = Text::from_markup("[red]Error:[/red] file not found").unwrap();
//! assert!(rich.plain().contains("Error:"));
//! ```
//!
//! ## Stylize Trait
//!
//! The [`Stylize`](styled_str::Stylize) trait provides a fluent, Rust-idiomatic API for
//! building styled text directly from string literals:
//!
//! ```rust
//! use gilt::styled_str::Stylize;
//!
//! let styled = "Hello".bold().red().on_blue();
//! assert_eq!(styled.text, "Hello");
//! ```
//!
//! `StyledStr` implements [`Renderable`](console::Renderable), so it can be passed
//! directly to [`Console::print`](console::Console::print).
//!
//! # Widgets
//!
//! ## Panel
//!
//! [`Panel`](panel::Panel) wraps content in a bordered box with optional title and subtitle.
//!
//! ```rust
//! use gilt::prelude::*;
//! use gilt::box_chars::DOUBLE;
//!
//! // Expanding panel (fills available width)
//! let panel = Panel::new(Text::new("content", Style::null()))
//!     .with_title("My Panel")
//!     .with_box_chars(&DOUBLE)
//!     .with_border_style(Style::parse("blue").unwrap());
//!
//! // Fit-to-content panel
//! let compact = Panel::fit(Text::new("snug", Style::null()));
//! ```
//!
//! ## Table
//!
//! [`Table`](table::Table) renders data in Unicode box-drawing tables with column alignment,
//! row striping, headers, footers, and style control.
//!
//! ```rust
//! use gilt::table::Table;
//!
//! let mut table = Table::new(&["Name", "Age", "City"]);
//! table.add_row(&["Alice", "30", "Paris"]);
//! table.add_row(&["Bob", "25", "London"]);
//!
//! let output = format!("{}", table);
//! assert!(output.contains("Alice"));
//! ```
//!
//! Builder methods customise every aspect of the table:
//!
//! ```rust
//! use gilt::table::Table;
//! use gilt::box_chars::ROUNDED;
//!
//! let table = Table::new(&["Key", "Value"])
//!     .with_title("Config")
//!     .with_box_chars(Some(&ROUNDED))
//!     .with_border_style("cyan")
//!     .with_row_styles(vec!["".into(), "dim".into()])
//!     .with_expand(true);
//! ```
//!
//! Use [`Table::grid`](table::Table::grid) for borderless side-by-side layout:
//!
//! ```rust
//! use gilt::table::Table;
//!
//! let mut grid = Table::grid(&["A", "B"]);
//! grid.add_row(&["left", "right"]);
//! ```
//!
//! ## Tree
//!
//! [`Tree`](tree::Tree) renders hierarchical data with Unicode guide lines.
//!
//! ```rust
//! use gilt::prelude::*;
//!
//! let mut tree = Tree::new(Text::new("root", Style::null()));
//! let child = tree.add(Text::new("child 1", Style::null()));
//! child.add(Text::new("grandchild", Style::null()));
//! tree.add(Text::new("child 2", Style::null()));
//! ```
//!
//! Guide style affects line weight: default is thin lines, `bold` gives thick lines,
//! and `underline2` selects double lines.
//!
//! ## Rule
//!
//! [`Rule`](rule::Rule) draws a horizontal line, optionally with a centered title.
//!
//! ```rust
//! use gilt::rule::Rule;
//!
//! let rule = Rule::new();                           // plain line
//! let titled = Rule::with_title("Section Header");  // line with title
//! ```
//!
//! ## Columns
//!
//! [`Columns`](columns::Columns) lays out items in an auto-fitting multi-column grid.
//!
//! ```rust
//! use gilt::columns::Columns;
//!
//! let mut cols = Columns::new();
//! cols.add_renderable("one");
//! cols.add_renderable("two");
//! cols.add_renderable("three");
//! ```
//!
//! ## Layout
//!
//! [`Layout`](layout::Layout) splits the terminal into rows and columns with flexible or
//! fixed sizing, like a split-pane window manager.
//!
//! ```rust
//! use gilt::layout::Layout;
//!
//! let mut root = Layout::new(None, Some("root".into()), None, None, Some(1), None);
//! root.split_row(vec![
//!     Layout::new(Some("Left pane".into()), Some("left".into()), None, None, Some(1), None),
//!     Layout::new(Some("Right pane".into()), Some("right".into()), None, None, Some(2), None),
//! ]);
//! ```
//!
//! # Terminal Features
//!
//! ## Syntax Highlighting
//!
//! [`Syntax`](syntax::Syntax) highlights source code using [syntect](https://docs.rs/syntect)
//! with 150+ language grammars. *(Requires the `syntax` feature, enabled by default.)*
//!
//! ```rust
//! # #[cfg(feature = "syntax")]
//! # {
//! use gilt::syntax::Syntax;
//!
//! let code = r#"fn main() { println!("hello"); }"#;
//! let syntax = Syntax::new(code, "rust")
//!     .with_line_numbers(true)
//!     .with_theme("base16-ocean.dark");
//! # }
//! ```
//!
//! ## Markdown
//!
//! [`Markdown`](markdown::Markdown) renders Markdown to the terminal with headings,
//! lists, code blocks, and emphasis. *(Requires the `markdown` feature, enabled by default.)*
//!
//! ```rust
//! # #[cfg(feature = "markdown")]
//! # {
//! use gilt::markdown::Markdown;
//!
//! let md = Markdown::new("# Hello\n\nThis is **bold** and *italic*.");
//! # }
//! ```
//!
//! ## JSON Pretty-Printing
//!
//! [`Json`](json::Json) parses and syntax-highlights JSON data.
//! *(Requires the `json` feature, enabled by default.)*
//!
//! ```rust
//! # #[cfg(feature = "json")]
//! # {
//! use gilt::json::{Json, JsonOptions};
//!
//! let json = Json::new(r#"{"name": "gilt", "version": "0.6.0"}"#, JsonOptions::default()).unwrap();
//! # }
//! ```
//!
//! The global convenience function makes it even simpler:
//!
//! ```rust,no_run
//! # #[cfg(feature = "json")]
//! gilt::print_json(r#"{"key": "value"}"#);
//! ```
//!
//! ## Progress Bars
//!
//! [`Progress`](progress::Progress) provides a multi-task progress display with customisable columns
//! (text, bar, spinner, time, speed) and live terminal updates.
//!
//! ```rust,no_run
//! use gilt::progress::Progress;
//!
//! let mut progress = Progress::new(vec![]);
//! let task = progress.add_task("Downloading", Some(100.0));
//! progress.start();
//! for _ in 0..100 {
//!     progress.advance(task, 1.0);
//! }
//! progress.stop();
//! ```
//!
//! ### Iterator Progress
//!
//! The [`ProgressIteratorExt`](progress::ProgressIteratorExt) trait adds `.progress()` to any iterator:
//!
//! ```rust,no_run
//! use gilt::progress::ProgressIteratorExt;
//!
//! let items: Vec<i32> = (0..100).progress("Processing").collect();
//! ```
//!
//! ## Live Display
//!
//! [`Live`](live::Live) renders content that updates in-place using cursor control codes.
//!
//! ```rust,no_run
//! use gilt::live::Live;
//! use gilt::text::Text;
//! use gilt::style::Style;
//!
//! let mut live = Live::new(Text::new("Loading...", Style::null()));
//! live.start();
//! // ... update content ...
//! live.update_renderable(Text::new("Done!", Style::null()), true);
//! live.stop();
//! ```
//!
//! ## Status Spinner
//!
//! [`Status`](status::Status) displays a spinner animation with a status message.
//!
//! ```rust,no_run
//! use gilt::status::Status;
//!
//! let mut status = Status::new("Loading...");
//! status.start();
//! status.update().status("Processing...").apply();
//! status.stop();
//! ```
//!
//! # Rust-Native Features
//!
//! These features go beyond what Python's rich provides, taking advantage of
//! Rust's type system and ecosystem.
//!
//! ## Gradients
//!
//! [`Gradient`](gradient::Gradient) renders text with smoothly interpolated true-color gradients.
//!
//! ```rust
//! use gilt::gradient::Gradient;
//! use gilt::color::Color;
//!
//! let g = Gradient::two_color("Hello!", Color::from_rgb(255, 0, 0), Color::from_rgb(0, 0, 255));
//! let rainbow = Gradient::rainbow("All the colors!");
//! ```
//!
//! ## Sparkline
//!
//! [`Sparkline`](sparkline::Sparkline) renders numeric data as compact inline Unicode bar charts.
//!
//! ```rust
//! use gilt::sparkline::Sparkline;
//!
//! let spark = Sparkline::new(&[1.0, 3.0, 5.0, 7.0, 5.0, 3.0, 1.0]);
//! let output = format!("{}", spark);
//! assert!(!output.is_empty());
//! ```
//!
//! ## Canvas
//!
//! [`Canvas`](canvas::Canvas) provides a Braille dot-matrix for high-resolution terminal graphics.
//! Each character cell encodes a 2x4 pixel grid, giving sub-character resolution.
//!
//! ```rust
//! use gilt::canvas::Canvas;
//!
//! let mut c = Canvas::new(10, 5);
//! c.line(0, 0, 19, 19);   // diagonal line
//! c.rect(2, 2, 10, 10);   // rectangle
//! c.circle(10, 10, 8);    // circle
//! ```
//!
//! ## Diff
//!
//! [`Diff`](diff::Diff) computes and renders colored line-level diffs in
//! unified or side-by-side format.
//!
//! ```rust
//! use gilt::diff::{Diff, DiffStyle};
//!
//! let old = "line 1\nline 2\nline 3\n";
//! let new = "line 1\nline 2 modified\nline 3\nline 4\n";
//! let diff = Diff::new(old, new).with_labels("before", "after");
//! let sbs = Diff::side_by_side(old, new);
//! ```
//!
//! ## Figlet
//!
//! [`Figlet`](figlet::Figlet) renders large ASCII art text using a built-in 5x7 block font.
//!
//! ```rust
//! use gilt::figlet::Figlet;
//!
//! let banner = Figlet::new("HI");
//! let output = format!("{}", banner);
//! assert!(!output.is_empty());
//! ```
//!
//! ## Inspect
//!
//! [`Inspect`](inspect::Inspect) displays structured information about any
//! `Debug` value in a styled panel, showing the type name and formatted representation.
//!
//! ```rust
//! use gilt::inspect::Inspect;
//! use gilt::console::Console;
//!
//! let data = vec![1, 2, 3];
//! let widget = Inspect::new(&data)
//!     .with_title("My Data")
//!     .with_label("numbers");
//!
//! let mut c = Console::builder().width(80).force_terminal(true).build();
//! c.begin_capture();
//! c.print(&widget);
//! let output = c.end_capture();
//! assert!(output.contains("Vec"));
//! ```
//!
//! Or use the global shorthand:
//!
//! ```rust,no_run
//! gilt::inspect(&vec![1, 2, 3]);
//! ```
//!
//! ## CsvTable
//!
//! [`CsvTable`](csv_table::CsvTable) converts CSV data into a rich [`Table`](table::Table).
//!
//! ```rust
//! use gilt::csv_table::CsvTable;
//!
//! let csv = CsvTable::from_csv_str("name,age\nAlice,30\nBob,25").unwrap();
//! let table = csv.to_table();
//! ```
//!
//! # Derive Macros
//!
//! With the `derive` feature enabled, gilt provides seven proc-macro derives that
//! automatically generate widget conversions from struct definitions:
//!
//! | Derive | Generates | Method |
//! |--------|-----------|--------|
//! | `Table` | Table from a slice of structs | `Type::to_table(&items)` |
//! | `Panel` | Panel from a single struct | `value.to_panel()` |
//! | `Tree` | Tree from a struct | `value.to_tree()` |
//! | `Columns` | Columns from a struct | `value.to_columns()` |
//! | `Rule` | Rule from a struct | `value.to_rule()` |
//! | `Inspect` | Inspect panel from a struct | `value.to_inspect()` |
//! | `Renderable` | `Renderable` trait impl | `console.print(&value)` |
//!
//! ```rust,ignore
//! use gilt::Table;
//!
//! #[derive(Table)]
//! #[table(title = "Employees", box_style = "ROUNDED")]
//! struct Employee {
//!     #[column(header = "Full Name", style = "bold")]
//!     name: String,
//!     #[column(justify = "right")]
//!     age: u32,
//!     #[column(skip)]
//!     internal_id: u64,
//!     #[column(header = "Dept", style = "green")]
//!     department: String,
//! }
//!
//! let employees = vec![
//!     Employee { name: "Alice".into(), age: 30, internal_id: 1, department: "Eng".into() },
//! ];
//! let table = Employee::to_table(&employees);
//! ```
//!
//! Enable in `Cargo.toml`:
//!
//! ```toml
//! gilt = { version = "0.6", features = ["derive"] }
//! ```
//!
//! # Feature Gates
//!
//! | Feature | Default | Crate Dependencies | Description |
//! |---------|---------|-------------------|-------------|
//! | `json` | Yes | `serde`, `serde_json` | JSON pretty-printing via [`Json`](json::Json) |
//! | `markdown` | Yes | `pulldown-cmark` | Terminal Markdown via [`Markdown`](markdown::Markdown) |
//! | `syntax` | Yes | `syntect` | Syntax highlighting via [`Syntax`](syntax::Syntax) |
//! | `interactive` | Yes | `rpassword` | Password prompts and selection menus |
//! | `logging` | Yes | `log` | Logging handler |
//! | `tracing` | No | `tracing`, `tracing-subscriber` | [`GiltLayer`](tracing_layer::GiltLayer) subscriber |
//! | `derive` | No | `gilt-derive` | 7 proc-macro derives |
//! | `miette` | No | `miette` | [`GiltMietteHandler`](miette_handler::GiltMietteHandler) |
//! | `eyre` | No | `eyre` | [`GiltEyreHandler`](eyre_handler::GiltEyreHandler) |
//! | `anstyle` | No | `anstyle` | Bidirectional `From` conversions |
//! | `csv` | No | `csv` | CSV file reading (built-in parser always available) |
//! | `readline` | No | `rustyline` | Readline-based prompt completions |
//!
//! For a minimal build with no heavy dependencies:
//!
//! ```toml
//! gilt = { version = "0.6", default-features = false }
//! ```
//!
//! # Integrations
//!
//! ## miette -- Diagnostic Reporting
//!
//! Install gilt as the [miette](https://docs.rs/miette) report handler for beautifully
//! styled diagnostic output. *(Requires the `miette` feature.)*
//!
//! ```rust,ignore
//! gilt::miette_handler::install();
//! ```
//!
//! ## eyre -- Error Reporting
//!
//! Install gilt as the [eyre](https://docs.rs/eyre) report handler. *(Requires the `eyre` feature.)*
//!
//! ```rust,ignore
//! gilt::eyre_handler::install().unwrap();
//! ```
//!
//! ## tracing -- Structured Logging
//!
//! Use [`GiltLayer`](tracing_layer::GiltLayer) as a tracing subscriber layer for
//! colored, formatted log output. *(Requires the `tracing` feature.)*
//!
//! ```rust,ignore
//! use gilt::tracing_layer::GiltLayer;
//! use tracing_subscriber::prelude::*;
//!
//! tracing_subscriber::registry()
//!     .with(GiltLayer::new())
//!     .init();
//! ```
//!
//! ## anstyle -- Type Conversions
//!
//! With the `anstyle` feature, gilt [`Color`](color::Color) and [`Style`](style::Style)
//! types gain bidirectional `From` conversions with their
//! [anstyle](https://docs.rs/anstyle) counterparts, enabling interop with clap,
//! owo-colors, and the anstyle ecosystem.
//!
//! # Advanced
//!
//! ## Theme System
//!
//! gilt's [`Theme`](theme::Theme) maps style names (like `"table.header"` or `"bold red"`)
//! to [`Style`](style::Style) instances. The default theme provides sensible styling for all built-in
//! widgets. Push custom themes onto the console's theme stack:
//!
//! ```rust
//! use gilt::console::Console;
//! use gilt::theme::Theme;
//! use gilt::style::Style;
//! use std::collections::HashMap;
//!
//! let mut styles = HashMap::new();
//! styles.insert("info".to_string(), Style::parse("bold cyan").unwrap());
//!
//! let mut console = Console::new();
//! console.push_theme(Theme::new(Some(styles), true));
//! // All rendering now uses the custom "info" style
//! console.pop_theme();
//! ```
//!
//! ## Custom Renderables
//!
//! Implement the [`Renderable`](console::Renderable) trait to create your own widgets:
//!
//! ```rust
//! use gilt::console::{Console, ConsoleOptions, Renderable};
//! use gilt::segment::Segment;
//!
//! struct Greeting { name: String }
//!
//! impl Renderable for Greeting {
//!     fn rich_console(&self, _console: &Console, _options: &ConsoleOptions) -> Vec<Segment> {
//!         vec![
//!             Segment::text(&format!("Hello, {}!", self.name)),
//!             Segment::line(),
//!         ]
//!     }
//! }
//! ```
//!
//! ## Accessibility (WCAG Contrast)
//!
//! The [`accessibility`] module provides WCAG 2.1 contrast ratio calculations:
//!
//! ```rust
//! use gilt::accessibility::{contrast_ratio, meets_aa, meets_aaa};
//! use gilt::color_triplet::ColorTriplet;
//!
//! let black = ColorTriplet::new(0, 0, 0);
//! let white = ColorTriplet::new(255, 255, 255);
//! assert!((contrast_ratio(&black, &white) - 21.0).abs() < 0.1);
//! assert!(meets_aa(&black, &white));
//! assert!(meets_aaa(&black, &white));
//! ```
//!
//! ## Environment Variables
//!
//! gilt respects standard terminal environment variables with a 5-tier priority system:
//!
//! | Variable | Effect |
//! |----------|--------|
//! | `NO_COLOR` | Disables all color output ([no-color.org](https://no-color.org)) |
//! | `FORCE_COLOR` | Forces color output even when not a TTY |
//! | `CLICOLOR_FORCE` | Same as `FORCE_COLOR` |
//! | `CLICOLOR=0` | Disables color |
//! | `COLUMNS` / `LINES` | Overrides terminal size detection |
//!
//! Programmatic settings (via [`ConsoleBuilder`](console::ConsoleBuilder)) always take
//! priority over environment variables.
//!
//! ## Console Export (HTML, SVG, Text)
//!
//! When recording is enabled, the console can export its output in multiple formats:
//!
//! ```rust
//! # use gilt::console::Console;
//! let mut c = Console::builder().width(60).record(true).build();
//! c.print_text("[bold green]Success![/bold green]");
//!
//! let text = c.export_text(true, false);             // plain text
//! let html = c.export_html(None, true, true);         // HTML with inline styles
//! let svg  = c.export_svg("Title", None, true, None, 0.61); // SVG image
//! ```
//!
//! Files can be written directly with
//! [`save_text`](console::Console::save_text),
//! [`save_html`](console::Console::save_html), and
//! [`save_svg`](console::Console::save_svg).
//!
//! # Module Index
//!
//! | Module | Description |
//! |--------|-------------|
//! | [`console`] | Console engine: rendering, capture, export |
//! | [`style`] | Text styles: colors, attributes, hyperlinks |
//! | [`text`] | Rich text with markup parsing and word wrapping |
//! | [`table`] | Unicode box-drawing tables |
//! | [`panel`] | Bordered content panels |
//! | [`tree`] | Hierarchical tree display |
//! | [`rule`] | Horizontal rules with titles |
//! | [`columns`] | Auto-fitting multi-column layout |
//! | [`layout`] | Split-pane terminal layouts |
//! | [`progress`] | Multi-task progress bars with live display |
//! | [`live`] | Live-updating terminal display |
//! | [`status`] | Spinner with status message |
//! | [`gradient`] | True-color gradient text |
//! | [`sparkline`] | Inline Unicode sparkline charts |
//! | [`canvas`] | Braille dot-matrix graphics |
//! | [`diff`] | Colored unified and side-by-side diffs |
//! | [`figlet`] | Large ASCII art text |
//! | [`csv_table`] | CSV-to-Table conversion |
//! | [`styled_str`] | Stylize trait for `"text".bold().red()` chaining |
//! | [`mod@inspect`] | Debug any value with rich formatting |
//! | [`markup`] | Markup tag parser |
//! | [`color`] | Color types and parsing |
//! | [`segment`] | Low-level rendering segments |
//! | [`theme`] | Named style collections |
//! | [`accessibility`] | WCAG 2.1 contrast checking |
//! | [`highlighter`] | Regex-based and repr syntax highlighters |
//! | [`emoji`] | Emoji shortcode replacement |
//! | [`box_chars`] | 19 box-drawing character sets |
//! | [`prelude`] | Convenience re-exports |

pub mod accessibility;
pub mod align_widget;
pub mod ansi;
#[cfg(feature = "anstyle")]
pub mod anstyle_adapter;
pub mod bar;
pub mod box_chars;
pub mod canvas;
pub mod cells;
pub mod color;
pub mod color_env;
pub mod color_triplet;
pub mod columns;
pub mod console;
pub mod constrain;
pub mod containers;
pub mod control;
pub mod csv_table;
pub mod default_styles;
pub mod diff;
pub mod emoji;
pub mod emoji_codes;
pub mod emoji_replace;
pub mod errors;
pub mod export_format;
#[cfg(feature = "eyre")]
pub mod eyre_handler;
pub mod figlet;
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
#[cfg(feature = "logging")]
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
pub mod sparkline;
pub mod spinner;
pub mod spinners;
pub mod status;
pub mod style;
pub mod styled;
pub mod styled_str;

// Re-export cache management functions
pub use style::{clear_style_cache, style_cache_size};
pub use color::{clear_color_cache, color_cache_size};
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
pub use gilt_derive::Columns as DeriveColumns;
#[cfg(feature = "derive")]
pub use gilt_derive::Inspect as DeriveInspect;
#[cfg(feature = "derive")]
pub use gilt_derive::Panel;
#[cfg(feature = "derive")]
pub use gilt_derive::Renderable;
#[cfg(feature = "derive")]
pub use gilt_derive::Rule as DeriveRule;
#[cfg(feature = "derive")]
pub use gilt_derive::Table;
#[cfg(feature = "derive")]
pub use gilt_derive::Tree;

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
