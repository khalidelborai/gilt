# gilt

**Rich terminal formatting for Rust** — a port of Python's [rich](https://github.com/Textualize/rich) library.

[![CI](https://github.com/khalidelborai/gilt/actions/workflows/ci.yml/badge.svg)](https://github.com/khalidelborai/gilt/actions)
[![Crates.io](https://img.shields.io/crates/v/gilt.svg)](https://crates.io/crates/gilt)
[![Documentation](https://docs.rs/gilt/badge.svg)](https://docs.rs/gilt)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.82.0-blue)](https://blog.rust-lang.org/2024/10/17/Rust-1.82.0.html)

Beautiful terminal output for Rust: styles, tables, trees, syntax highlighting, progress bars, live displays, markdown rendering, and more — all rendered as ANSI escape sequences.

## Quick start

```toml
[dependencies]
gilt = "1.1"
```

```rust
use gilt::console::Console;

fn main() {
    let mut console = Console::default();
    console.print_text("Hello, [bold magenta]gilt[/bold magenta]!");
}
```

**Upgrading from 0.13.x?** See [MIGRATION_v1.md](MIGRATION_v1.md) — most code becomes shorter (lossy `Style::parse`, ergonomic `Text::styled`, `Status::run`, `Live::from_renderable`, `Padding::wrap`, …).

## Features

### Core widgets

`Text` · `Table` · `Panel` · `Tree` · `Columns` · `Layout` · `Padding` · `Align` · `Group`

### Terminal features

`Syntax` (150+ languages) · `Markdown` · `Json` · `Progress` (multi-bar with ETA, speed, spinner) · `Live` (in-place updates) · `Status`

### Rust-native extensions

`Gradient` · `Sparkline` · `Canvas` (Braille drawing) · `Diff` (unified + side-by-side) · `Figlet` · `CsvTable` · `Stylize` trait (`"hi".bold().red()`) · iterator `.progress()` · `Inspect` (any Debug value) · environment detection (`NO_COLOR`, `FORCE_COLOR`, `CLICOLOR`) · WCAG 2.1 contrast checking · extended underlines (curly, dotted, dashed, double) · bidirectional `anstyle` interop

### Derive macros (feature-gated)

```rust
use gilt::derives::{Table, Panel, Tree, Columns, Rule, Inspect, Renderable};
```

Auto-generate widget conversions from struct definitions. See [`crates/gilt-derive/README.md`](crates/gilt-derive/README.md).

### Optional integrations

`miette` · `eyre` · `tracing` · `anstyle`

## Documentation

| Resource | Where |
|----------|-------|
| API docs | [docs.rs/gilt](https://docs.rs/gilt) |
| Release notes | [CHANGELOG.md](CHANGELOG.md) |
| **v1.0 migration guide** | [MIGRATION_v1.md](MIGRATION_v1.md) |
| Derive macros | [crates/gilt-derive/](crates/gilt-derive/README.md) · [docs.rs/gilt-derive](https://docs.rs/gilt-derive) |
| Examples | [`examples/`](examples/) — run any with `cargo run --example <name>` |
| Feature flags & deps | [docs.rs/crate/gilt/latest/features](https://docs.rs/crate/gilt/latest/features) |

## Examples

```bash
cargo run --example showcase --all-features   # full feature tour
cargo run --example table                     # core widget demos
cargo run --example progress
cargo run --example markdown
cargo run --example derive_table --features derive
```

## Performance

`cargo bench` runs the criterion suite (~80 benchmarks). See [CHANGELOG.md](CHANGELOG.md) for v0.10.x → v0.11.0 perf wins (T8 lock-free Live `+21,000×`, table render `-46%`, etc.).

## Minimum Supported Rust Version

**1.82.0** (for `std::sync::LazyLock`).

## License

MIT — see [LICENSE](LICENSE).
