# gilt

**Rich terminal formatting for Rust** — a port of Python's [rich](https://github.com/Textualize/rich) library.

[![CI](https://github.com/khalidelborai/gilt/actions/workflows/ci.yml/badge.svg)](https://github.com/khalidelborai/gilt/actions)
[![Crates.io](https://img.shields.io/crates/v/gilt.svg)](https://crates.io/crates/gilt)
[![Documentation](https://docs.rs/gilt/badge.svg)](https://docs.rs/gilt)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

gilt brings beautiful terminal output to Rust with styles, tables, trees, syntax highlighting, progress bars, and more — all rendered as ANSI escape sequences.

## Quick Start

```toml
[dependencies]
gilt = "0.1"
```

```rust
use gilt::prelude::*;

fn main() {
    let mut console = Console::new();
    console.print_text("Hello, [bold magenta]gilt[/bold magenta]!");
}
```

## Features

### Core Widgets
- **Text** — Rich text with markup, styles, wrapping, alignment
- **Table** — Unicode box-drawing tables with column alignment and row striping
- **Panel** — Bordered content panels with titles
- **Tree** — Hierarchical tree display with guide lines
- **Columns** — Multi-column layout
- **Layout** — Flexible split-pane layouts

### Terminal Features
- **Syntax** — Code highlighting via syntect (150+ languages)
- **Markdown** — Terminal-rendered Markdown
- **JSON** — Pretty-printed JSON with highlighting
- **Progress** — Multi-bar progress display with ETA, speed, spinner
- **Live** — Live-updating terminal display
- **Status** — Spinner with status message

### Rust-Native Extensions
- **Gradients** — True-color RGB gradient text
- **Stylize trait** — `"hello".bold().red()` method chaining
- **Iterator progress** — `iter.progress()` adapter
- **`#[derive(Table)]`** — Auto-generate tables from structs
- **Environment detection** — `NO_COLOR`, `FORCE_COLOR`, `CLICOLOR` support
- **Inspect** — Debug any value with rich formatting

### Error Reporting Integration
- **miette** — Diagnostic reporting with gilt styling
- **eyre** — Error reporting with gilt styling
- **tracing** — Log subscriber with colored output

## Optional Features

```toml
[dependencies]
gilt = { version = "0.1", features = ["tracing", "derive", "miette", "eyre"] }
```

| Feature | Description |
|---------|-------------|
| `tracing` | `tracing` subscriber with gilt formatting |
| `derive` | `#[derive(Table)]` proc macro |
| `miette` | `miette::ReportHandler` implementation |
| `eyre` | `eyre::EyreHandler` implementation |

## Examples

```bash
# Basic examples
cargo run --example table
cargo run --example panel
cargo run --example syntax
cargo run --example progress
cargo run --example markdown

# Rust-native features
cargo run --example gradient
cargo run --example inspect_demo
cargo run --example styled_string

# Feature-gated examples
cargo run --example derive_table --features derive
cargo run --example miette_demo --features miette
cargo run --example tracing_demo --features tracing
```

See the [examples/](examples/) directory for all 51 examples.

## Global Console

```rust
// Print with markup
gilt::print_text("Hello, [bold]world[/bold]!");

// Print JSON
gilt::print_json(r#"{"name": "gilt"}"#);

// Inspect any Debug value
gilt::inspect(&vec![1, 2, 3]);
```

## License

MIT
