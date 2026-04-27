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
gilt = "0.11"
```

```rust
use gilt::prelude::*;

fn main() {
    let mut console = Console::new();
    console.print_text("Hello, [bold magenta]gilt[/bold magenta]!");
}
```

## Migrating from 0.10.x to 0.11.0

`v0.11.0` collapses the `Color` struct into a 5-variant enum and converts
`Segment.style` from a public field to a method. Both changes are
mechanical at call sites; the type-level changes shrink Color from
~40 B + heap String to ~4 B inline `Copy`.

### Color: struct → enum

```rust
// Before:
Color { name: "red".into(), color_type: ColorType::Standard,
        number: Some(1), triplet: None }

// After:
Color::Standard(1)
// or
Color::parse("red")?
```

Field accessors are now methods:

| Before                    | After                                    |
|---------------------------|------------------------------------------|
| `color.name`              | `color.name()` → `Cow<'_, str>`          |
| `color.color_type`        | `color.kind()` → `ColorType`             |
| `color.number`            | `color.number()` → `Option<u8>`          |
| `color.triplet`           | `color.triplet()` → `Option<ColorTriplet>` |

Pattern-matching uses the variants directly:

```rust
// Before:
match color.color_type {
    ColorType::Standard => use_palette(color.number.unwrap()),
    ColorType::TrueColor => use_rgb(color.triplet.unwrap()),
    _ => {}
}

// After:
match color {
    Color::Standard(n) => use_palette(n),
    Color::TrueColor(t) => use_rgb(t),
    _ => {}
}
```

### Segment.style: field → method

```rust
// Before:                          // After:
seg.style.is_some()                 seg.style().is_some()
seg.style.clone()                   seg.style().cloned()
seg.style.as_ref()                  seg.style()
if let Some(ref s) = seg.style      if let Some(s) = seg.style()
seg.style = Some(s)                 seg.set_style(Some(s))
seg.style.clone().unwrap_or_else(Style::null)  seg.style_owned()
```

For struct-literal construction (`Segment { style: Some(s), … }`), use
`Segment::new(text, Some(s), control)` or `Segment::styled(text, s)`.

### Behavior changes (rare)

- **EightBit colors lose named round-trip**: `Color::parse("yellow4")?
  .name()` now returns `"color(106)"` instead of `"yellow4"`. Only the
  16 standard ANSI colors get canonical names.
- **`Segment::style_owned()` collapses None and Some(null)**: both map
  to `Style::null()`. Use `style()` directly to distinguish.

### What didn't change

Everything else: `Console::print`, the `Renderable` trait, markup parsing,
all widget APIs (`Panel`, `Tree`, `Table`, `Progress`, `Live`, `Status`),
`Style::parse` / `Style::combine` / `Style::render`, color parsing,
themes, layout, `Text` API. If you only consume widgets and never
construct `Color` or `Segment` directly, you likely don't need to
change anything.

### Why no L2 interner activation?

The original v0.11.0 design included a per-Segment style interner that
would swap `Segment.style` storage to a 4-byte `StyleId`. After L1
shrunk Color and made Style cheap to clone, activating the interner
empirically regressed `console_render/table_100_rows` from 600 µs to
884 µs (+47%) — the per-construction `Mutex<HashMap>` lock acquisition
exceeded the savings. See CHANGELOG `[0.11.0]` for the bench table.

The `StyleInterner` and `StyleId` types remain in `gilt::style_interner`
as dormant scaffolding for potential future use (e.g., a lock-free
DashMap-backed redesign).

## Recent releases

| Version  | Highlights                                                                                                  |
|----------|-------------------------------------------------------------------------------------------------------------|
| v0.11.0  | L1 Color enum collapse (~10× smaller, `Copy`); `Segment.style` field → method; dormant `StyleInterner` types |
| v0.10.3  | T8 lock-free `Live` writers — **+21,000×** under writer+renderer contention                                  |
| v0.10.2  | `Live::update_renderable: &self`; threaded contention bench                                                  |
| v0.10.1  | `Text::char_len` cache (~1 ns cached)                                                                        |
| v0.10.0  | Rich v15.0.0 sync — TTY env overrides, 4-tuple `Syntax.padding`, `Text::from_ansi` newline preservation     |
| v0.8.0   | Thread-safe Console, LRU style/color caching, iterator `.progress()`                                         |
| v0.7.0   | Builder standardization (`with_` prefix everywhere), comprehensive rustdoc                                   |
| v0.6.0   | Soundness fix in `live_render`, hardened API, expanded prelude                                               |

See [CHANGELOG.md](CHANGELOG.md) for details and [Migrating from 0.10.x to 0.11.0](#migrating-from-010x-to-0110) above.

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
gilt = "0.11"

# Minimal -- no heavy deps
gilt = { version = "0.11", default-features = false }

# Pick what you need
gilt = { version = "0.11", default-features = false, features = ["json", "syntax"] }
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

See the [examples/](examples/) directory for all 104 examples.

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

gilt includes a criterion benchmark suite (~80 benchmarks) covering text rendering, style application, table layout, segment operations, threaded contention, and more:

```bash
cargo bench                                       # full suite (~5 min)
cargo bench --bench benchmarks -- console_render  # focused
cargo bench --bench live_threaded                 # writer/renderer contention
```

Recent perf wins from the v0.10.x → v0.11.0 cycle:

| Bench                           | Before    | After     | Notes                                      |
|---------------------------------|-----------|-----------|--------------------------------------------|
| `console_render/table_100_rows` | 1.119 ms  | 600 µs    | -46%, T1/T2/T3/T9 + L1 Color enum          |
| `console_render/table_10_rows`  | 90.6 µs   | 50 µs     | -45%                                       |
| `text_operations/len_repeated_cached` | O(n) | ~1 ns     | T11 `Text::char_len` AtomicUsize cache    |
| `live_threaded/update_plus_render/1` | 43 ops/s | 904 K ops/s | **+21,000×**, T8 ArcSwap lock-free writers |
| `cell_len` (ASCII)              | 65.9 ns   | 4.9 ns    | -93%, ASCII fast path                      |

## Minimum Supported Rust Version

gilt requires **Rust 1.82.0** or later (for `std::sync::LazyLock`).

## License

MIT
