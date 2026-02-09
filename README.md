# gilt

**Rich terminal formatting for Rust** -- a port of Python's [rich](https://github.com/Textualize/rich) library.

[![CI](https://github.com/khalidelborai/gilt/actions/workflows/ci.yml/badge.svg)](https://github.com/khalidelborai/gilt/actions)
[![Crates.io](https://img.shields.io/crates/v/gilt.svg)](https://crates.io/crates/gilt)
[![Documentation](https://docs.rs/gilt/badge.svg)](https://docs.rs/gilt)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.82.0-blue)](https://blog.rust-lang.org/2024/10/17/Rust-1.82.0.html)

gilt brings beautiful terminal output to Rust with styles, tables, trees, syntax highlighting, progress bars, and more -- all rendered as ANSI escape sequences.

## Quick Start

```toml
[dependencies]
gilt = "0.7"
```

```rust
use gilt::prelude::*;

fn main() {
    let mut console = Console::new();
    console.print_text("Hello, [bold magenta]gilt[/bold magenta]!");
}
```

## v0.7.0 Highlights

- **Builder standardization** -- all widget builders now use `with_` prefix (`Panel::new().with_title()`, `Table::new().with_box_chars()`, etc.)
- **Comprehensive rustdoc** -- 731-line crate-level guide with 132 doctests covering every feature
- **2,295 lib tests + 132 doctests** -- full coverage across all widgets

## v0.6.0 Highlights

- **Soundness fix** -- replaced unsafe interior mutability with `Cell` in live_render
- **Hardened API** -- 10 `unwrap()` calls replaced with graceful handling
- **Feature-gated `log`** -- `logging_handler` now behind `logging` feature (default on)
- **Expanded prelude** -- `Bar`, `Layout`, `Live`, `Status`, `Prompt`, `Json` added
- **Binary filesize** -- `filesize::binary()` for KiB/MiB/GiB units
- **Readline autocomplete** -- rustyline-based prompt completions (feature-gated)
- **Display on 5 more widgets** -- Constrain, Scope, Group, Align, Styled
- **2,334 tests** -- 85 new tests for CJK/emoji, boundary widths, stress scenarios
- **85 examples** including 4 cookbooks and 47-section showcase

## Features

### Core Widgets
- **Text** -- Rich text with markup, styles, wrapping, alignment
- **Table** -- Unicode box-drawing tables with column alignment and row striping
- **Panel** -- Bordered content panels with titles
- **Tree** -- Hierarchical tree display with guide lines
- **Columns** -- Multi-column layout
- **Layout** -- Flexible split-pane layouts

### Terminal Features
- **Syntax** -- Code highlighting via syntect (150+ languages)
- **Markdown** -- Terminal-rendered Markdown
- **JSON** -- Pretty-printed JSON with highlighting
- **Progress** -- Multi-bar progress display with ETA, speed, spinner
- **Live** -- Live-updating terminal display
- **Status** -- Spinner with status message

### Rust-Native Extensions
- **Gradients** -- True-color RGB gradient text
- **Sparkline** -- Inline Unicode bar charts
- **Canvas** -- Braille dot-matrix drawing with line, rect, circle primitives
- **Diff** -- Unified and side-by-side text diffs with colored output
- **Figlet** -- Large ASCII art text rendering
- **CsvTable** -- CSV to rich Table conversion
- **Stylize trait** -- `"hello".bold().red()` method chaining
- **Iterator progress** -- `iter.progress()` adapter
- **`#[derive(Table, Panel, Tree, Columns, Rule, Inspect, Renderable)]`** -- Auto-generate widgets from structs
- **Environment detection** -- `NO_COLOR`, `FORCE_COLOR`, `CLICOLOR` support
- **Inspect** -- Debug any value with rich formatting
- **Accessibility** -- WCAG 2.1 contrast checking, `REDUCE_MOTION` detection
- **Extended underlines** -- Curly, dotted, dashed, double styles with color
- **anstyle interop** -- Bidirectional conversion with `anstyle` types

### Integrations
- **miette** -- Diagnostic reporting with gilt styling
- **eyre** -- Error reporting with gilt styling
- **tracing** -- Log subscriber with colored output
- **anstyle** -- Convert between gilt and anstyle `Color`/`Style` types

## Feature Gates

All four heavy dependencies are **default-on**. Disable them for minimal builds:

```toml
# Full (default) -- includes json, markdown, syntax, interactive
gilt = "0.7"

# Minimal -- no heavy deps
gilt = { version = "0.7", default-features = false }

# Pick what you need
gilt = { version = "0.7", default-features = false, features = ["json", "syntax"] }
```

| Feature | Default | Description |
|---------|---------|-------------|
| `json` | yes | Pretty-printed JSON (`serde`, `serde_json`) |
| `markdown` | yes | Terminal Markdown rendering (`pulldown-cmark`) |
| `syntax` | yes | Syntax highlighting (`syntect`) |
| `interactive` | yes | Password prompts, select/multi-select (`rpassword`) |
| `tracing` | no | `tracing` subscriber with gilt formatting |
| `derive` | no | `#[derive(Table, Panel, Tree, ...)]` proc macros (7 derives) |
| `miette` | no | `miette::ReportHandler` implementation |
| `eyre` | no | `eyre::EyreHandler` implementation |
| `csv` | no | CSV file reading via `csv` crate (built-in parser always available) |
| `anstyle` | no | Bidirectional `From` conversions with `anstyle` types |

## Examples

```bash
# Showcase (runs all major widgets)
cargo run --example showcase --all-features

# Core widgets
cargo run --example table
cargo run --example panel
cargo run --example syntax
cargo run --example progress
cargo run --example markdown

# Rust-native features
cargo run --example gradient
cargo run --example inspect_demo
cargo run --example styled_string
cargo run --example sparkline

# Feature-gated examples
cargo run --example derive_table --features derive
cargo run --example derive_panel --features derive
cargo run --example derive_tree --features derive
cargo run --example derive_rule --features derive
cargo run --example derive_inspect --features derive
cargo run --example miette_demo --features miette
cargo run --example tracing_demo --features tracing
```

See the [examples/](examples/) directory for all 73 examples.

## Global Console

```rust
// Print with markup
gilt::print_text("Hello, [bold]world[/bold]!");

// Print JSON
gilt::print_json(r#"{"name": "gilt"}"#);

// Inspect any Debug value
gilt::inspect(&vec![1, 2, 3]);
```

## Performance

gilt includes a criterion benchmark suite (64 benchmarks) covering text rendering, style application, table layout, segment operations, and more:

```bash
cargo bench
```

## Minimum Supported Rust Version

gilt requires **Rust 1.82.0** or later (for `std::sync::LazyLock`).

## License

MIT
