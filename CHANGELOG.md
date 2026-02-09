# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.4.0] - 2026-02-09

### Added
- `sparkline` module: inline Unicode bar charts using `▁▂▃▄▅▆▇█` blocks with linear interpolation resampling
- `canvas` module: Braille dot-matrix drawing (2×4 pixels per cell) with line, rect, fill_rect, circle primitives
- `diff` module: LCS-based line-level text diffing with unified and side-by-side rendering, colored output
- `figlet` module: large ASCII art text with built-in 5×7 block font (A-Z, a-z, 0-9, 34 punctuation chars)
- `csv_table` module: CSV-to-Table conversion with built-in parser (no deps) and optional `csv` crate integration
- `csv` feature gate for `csv` crate dependency
- 5 new examples: `sparkline`, `canvas`, `diff`, `figlet`, `csv_table`
- 119 new tests across the 5 modules

## [0.3.0] - 2026-02-08

### Added
- WCAG 2.1 accessibility module (`accessibility.rs`): `contrast_ratio()`, `meets_aa()`, `meets_aaa()`, `meets_aa_large()` for color contrast checking
- `REDUCE_MOTION` environment variable detection via `detect_reduce_motion()` in `color_env.rs`
- Feature gates: `json`, `markdown`, `syntax`, `interactive` (all default-on); use `default-features = false` for minimal builds
- `CompactString` for `Segment.text` providing inline storage for strings <=24 bytes, eliminating heap allocations for most terminal segments
- `Cow<str>` returns from `strip_control_codes`, `escape_control_codes`, `emoji_replace`, `set_cell_size` for zero-allocation when input needs no modification
- Comprehensive showcase example (`examples/showcase.rs`) demonstrating 21 feature sections
- 26 new criterion benchmarks covering control codes, cell sizing, emoji operations, segment operations, and export operations
- `just check-minimal` and `just check-all` justfile recipes for testing feature combinations

### Changed
- Replaced `once_cell` with `std::sync::LazyLock` (MSRV raised to 1.82)
- Replaced `format!()` with `write!()` in SGR rendering and HTML/SVG export loops to reduce intermediate string allocations
- `serde_json`, `pulldown-cmark`, `syntect`, `rpassword` are now optional dependencies behind default feature flags

### Removed
- `once_cell` dependency

## [0.2.1] - 2026-02-08

### Added
- `justfile` with dev and release recipes (format, lint, test, doc, publish workflows)

### Fixed
- Rustfmt formatting and module sort order in `anstyle_adapter.rs`

## [0.2.0] - 2026-02-08

### Added
- Extended underline support: curly, dotted, dashed, and double underline styles with underline color (SGR 4:N codes)
- `anstyle` feature flag for bidirectional `From` conversions between gilt and anstyle `Color`/`Style` types
- 100% rustdoc coverage: all 279 previously undocumented public items now have doc comments with examples
- Expanded crate-level documentation with core modules table, feature flags table, and global console examples
- Version specifier for `gilt-derive` dependency to support crates.io publishing

### Changed
- Excluded `.claude/` directory from published crate package

## [0.1.0] - 2026-02-08

### Added
- Initial release: full port of Python's rich library (65 modules, 51,000+ lines of Rust)
- 2,111 tests (2,066 unit + 37 tracing-gated + 4 miette + 4 eyre), 0 clippy warnings
- 45 examples covering every widget
- 30 Renderable implementations
- **Core text**: Rich `Text` with markup, styles, wrapping, alignment, and `Segment`-based rendering pipeline
- **Console**: Color system detection, capture mode, export to HTML/SVG/text, global console via `gilt::print()`, `gilt::print_text()`, `gilt::print_json()`
- **Widgets**: Table, Panel, Tree, Progress (multi-bar with ETA/speed/spinner), Live, Status, Columns, Layout, Rule, Bar, Group, Align, Constrain, Screen
- **Syntax highlighting**: 150+ languages via syntect
- **Markdown rendering**: Terminal-rendered Markdown via pulldown-cmark
- **JSON pretty-printing**: Highlighted JSON output
- **Prompt**: Interactive input with Select/MultiSelect, numbered-choice UI, "all" keyword, min/max validation
- **Gradient text**: True-color RGB interpolation with rainbow preset, `Renderable` + `Display`
- **Stylize trait**: `"hello".bold().red()` method chaining via `styled_str.rs`
- **Iterator progress**: `.progress()` adapter for any iterator via `ProgressIteratorExt`
- **`#[derive(Table)]`**: Proc macro for auto-generating tables from structs (feature: `derive`)
- **Inspect**: Debug any value with rich formatting, builder API
- **Pretty printer**: Type-annotated debug output with `infer_type_name()`
- **Highlighters**: Regex, ISO date, URL, UUID, JSON path highlighter types
- **Environment detection**: `NO_COLOR`, `FORCE_COLOR`, `CLICOLOR` 5-tier priority
- **OSC 52 clipboard**: Copy to clipboard via terminal escape sequence
- **Synchronized output**: Flicker-free rendering via DEC 2026 protocol
- **Pager**: Built-in terminal pager support
- **Logging handler**: `log` crate integration for styled log output
- **Traceback**: Rich error traceback display
- **Scope**: Variable scope inspection widget
- **miette integration**: Diagnostic reporting with gilt styling (feature: `miette`)
- **eyre integration**: Error reporting with gilt styling (feature: `eyre`)
- **tracing integration**: Log subscriber with colored output (feature: `tracing`)
- **Prelude module**: Ergonomic `use gilt::prelude::*` re-exports

[0.3.0]: https://github.com/khalidelborai/gilt/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/khalidelborai/gilt/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/khalidelborai/gilt/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/khalidelborai/gilt/releases/tag/v0.1.0
