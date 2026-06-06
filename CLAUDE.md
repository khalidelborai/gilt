# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

`gilt` is a Rust port of Python's [rich](https://github.com/Textualize/rich) library — terminal formatting (styles, tables, trees, syntax highlighting, progress bars, live displays, markdown) emitted as ANSI escape sequences. It is a published crate (`gilt` on crates.io, currently **v1.10.0**) in a workspace with three sibling crates under `crates/`: `gilt-derive` (proc-macro derives **+ the `text!` macro**), `gilt-cli` (an installable `gilt` binary — the gum pattern, published), and `gilt-vhs` (tape-as-code recording, `publish = false`). MSRV is **1.82.0** (relies on `std::sync::LazyLock`). It began as a rich port and now also ships Rust-native capabilities rich lacks: a compile-time `text!` macro, a WASM-safe build, lock-free async `Live`, inline terminal images, asciinema `.cast` export, and 7 widget derives.

API/behavior parity with upstream Python `rich` is an explicit goal — when porting or fixing a widget, the reference behavior is rich's. The rich source is not checked out here; the porting reference is **`../research_doc/`** — 30 module-by-module analysis docs of rich's source (segment, style, color, text, cells/measure, protocols, console, table, …) with Python→Rust patterns. Consult the matching doc before porting or changing a widget's behavior. Parity gaps against rich's test suite are tracked in `.review/04-test-parity.md`.

This repo is one of several sibling crates in the parent `rusty_rich/` workspace that together port the rich/Textual stack to Rust: `../gilt-tui` (a Textual port — CSS-styled TUI framework built on gilt) + `../gilt-tui-macros`, `../anstyle`, and `../textual_research/` (the Textual-analysis counterpart to `research_doc/`, feeding gilt-tui). Changes to gilt's public API can affect gilt-tui.

## Commands

The `justfile` defines the canonical workflows; `cargo test --lib` is what CI runs.

```bash
# Tests — prefer nextest, but it does NOT run doctests (run those with cargo test --doc)
cargo nextest run --lib                 # fast unit-test run (2300+ inline #[test])
cargo nextest run --lib <substring>     # run a single test / filtered subset
cargo test --doc                        # doctests — there are many in lib.rs; nextest skips these
cargo test --lib                        # what CI uses

just check         # fmt --check + clippy --all-features + test --lib + test --doc + doc + missing-docs
just check-all     # tests across default / no-default / all features + doctests + clippy
just check-minimal # cargo test --lib --no-default-features

# Lint / format / docs
cargo clippy --all-features -- -D warnings   # CI treats warnings as errors
cargo fmt                                     # apply; `just fmt-check` to verify
cargo doc --no-deps --all-features            # CI builds docs with RUSTDOCFLAGS=-D warnings

# Benchmarks (criterion, ~80 benches) — use the release-fast profile for local runs
cargo bench                                   # benches/benchmarks.rs + benches/live_threaded.rs

# Examples (107 of them under examples/)
cargo run --example showcase --all-features
cargo run --example derive_table --features derive
```

CI matrix (`.github/workflows/ci.yml`) gates on: `cargo test --lib` (stable + nightly), all-features tests, doctests, clippy `-D warnings` (base + features), `cargo fmt --check`, `cargo doc -D warnings`, **MSRV 1.82.0 `cargo check`**, and a **wasm32-unknown-unknown** build (`--no-default-features --features json,markdown,syntax`). Keep all of these green — especially MSRV and WASM, which are easy to break accidentally.

## Architecture

### The rendering pipeline (the core abstraction)

Everything renders through three types in `src/`:

- **`Renderable`** (trait, `console.rs`): a single method `gilt_console(&self, &Console, &ConsoleOptions) -> Vec<Segment>`. Every widget (`Table`, `Panel`, `Tree`, `Text`, even `str`/`String`) implements this. To add a widget, implement `Renderable`.
- **`Segment`** (`segment.rs`): the atomic unit of output — text + optional `Style` + optional control code. Width is computed in terminal *cells* (see Unicode below). All rendering flows through `Vec<Segment>`.
- **`Console`** (`console.rs` + sibling files): the orchestrator. Holds terminal capabilities (size, color system, markup on/off), drives `render()` → segments → ANSI string, and handles capture/record/export.

Flow: `console.print(&widget)` → `widget.gilt_console(...)` produces `Vec<Segment>` → console serializes segments to ANSI (or HTML/SVG when recording).

### Console is split across files via `#[path]` + multiple `impl Console` blocks

`console.rs` is the hub; methods are partitioned into sibling files included with `#[path = "..."] mod ...;`, each adding another `impl Console` block:
- `console_builder.rs` — `ConsoleBuilder`
- `console_capture.rs` — `begin_capture` / `end_capture`
- `console_render.rs` — `print`/`log`/`rule`/`render`/`measure`/`save_*` etc.
- `console_export.rs` — `export_text` / `export_html` / `export_svg`
- `console_tests.rs` — tests

When adding a `Console` method, put it in the file matching its concern and keep it inside that file's `impl Console` block — don't bloat `console.rs`.

### Module organization & backward-compat re-exports

Modules were reorganized into subdirectories (`color/`, `text/`, `utils/`, `widgets/`, `live/`, `progress/`, `error/`) but **`lib.rs` re-exports them at their old crate-root paths** (e.g. `pub use color::{accessibility, theme, ...}`, `pub use utils::{box_chars, emoji, inspect, ...}`, `pub use widgets::table`). So `gilt::theme`, `gilt::box_chars`, `gilt::table` still resolve. When moving a module, preserve its public path with a re-export rather than breaking it.

- `prelude.rs` — the curated `use gilt::prelude::*;` surface; add commonly-used new public types here.
- `text/` — `Text` (rich text = plain string + styled spans), wrapping, markup. `markup.rs` parses `[tag]...[/tag]` BBCode-style markup.
- `widgets/table/` — `Table` split into `core/render/column/row`.
- `progress/` and `progress/columns/` — multi-task progress with pluggable columns.
- `color/` — `Color`, `ColorSystem`, `ColorTriplet`, palettes, themes, WCAG `accessibility`.

### Feature gating

Default features: `json`, `markdown`, `syntax`, `interactive`, `logging`, `terminal-size`. Many opt-in features gate heavier or native-only capability: `derive` (+`text!`), `asciinema`, `inline-images` (`image`), `tty-select`/`terminal-query` (`crossterm`), `windows-vt` (`windows-sys`), `syntax-theme-file`, `tracing`, `miette`, `eyre`, `anstyle`, `csv`, `readline`, `async`, `http` (see `Cargo.toml`). Feature-gated modules use `#[cfg(feature = "...")]` on both the `pub mod` and the re-export. **The default, `--no-default-features`, and `wasm32` builds pull no `libc`/`crossterm`/terminal-syscall deps** — that is how WASM support is kept. A few **opt-in native features** (`tty-select`, `terminal-query`, `windows-vt`) do pull such deps, but they are never in `default` and never compiled for wasm (the wasm CI gate uses `--no-default-features --features json,markdown,syntax`). Any new public item must still compile under `--no-default-features` and on `wasm32`; native-only code goes behind an opt-in feature **and** `#[cfg(not(target_arch = "wasm32"))]`.

### Derive macros

`crates/gilt-derive/` provides 7 derives (`Table`, `Panel`, `Tree`, `Columns`, `Rule`, `Inspect`, `Renderable`). Non-colliding ones are re-exported at the crate root (`gilt::Table`); the three that collide with widget type names (`Columns`, `Inspect`, `Rule`) live ONLY under `gilt::derives::*`. It also provides the function-like **`text!` macro** (re-exported as `gilt::text!` under the `derive` feature), which validates gilt markup at compile time (its validator mirrors `Style::parse`/`Color::parse`). The derive crate version is pinned to the main crate version in `Cargo.toml`.

### Internal conventions worth knowing

- **Hashing**: use the single `utils::hash::fnv1a_64` / `fnv1a_64_extend` helper (link ids, content hashes, SVG ids) — do not roll a new FNV.
- **`Renderable::content_hash()`** is an additive default (`None`); implement it for cheap-to-hash leaf widgets so `Layout::render_with_cache` can skip unchanged children across Live frames.
- **`ConsoleCapabilities`** (`console_caps.rs`, env-derived, WASM-safe — no blocking probe) is the central place for terminal-feature flags; image-protocol and OSC-11 detection build on it.
- **Live** holds an `Arc<dyn Renderable + Send + Sync>` (re-rendered through its own console each frame), updates content lock-free via `ArcSwap`, and emits each frame wrapped in DEC-2026 synchronized output with line-diff repaint — preserve that emit discipline when touching `live/mod.rs::do_refresh`.

## Conventions

- **`Style::parse(&str) -> Style` is lossy/infallible** (not `Result`) — invalid tokens are silently dropped. This is a deliberate v1 ergonomic choice (see `MIGRATION_v1.md`). Markup parsing (`Text::from_markup`) does return `Result`.
- **Unicode correctness matters and is non-trivial.** Cell width uses `unicode-width`; cluster iteration uses `unicode-segmentation` (UAX #29 grapheme clusters). Truncation/cropping snaps to grapheme boundaries (never splits a ZWJ emoji cluster). v1.4 was a dedicated Unicode-correctness pass — `cells.rs` (`cell_len`, `set_cell_size`, `get_character_cell_size`) is the central width logic. The README "Unicode handling" section documents exactly what is in/out of scope; honor those boundaries.
- **Tests live both inline (`#[cfg(test)]` in `src/`, the bulk) and in `tests/`** (`tests/unit/`, `proptests.rs` with proptest, `integration.rs`, per-widget `*_unit.rs`). New widget logic should add inline unit tests; cross-cutting behavior goes in `tests/`.
- `CHANGELOG.md` is detailed and kept current per release; `MIGRATION_v1.md` documents the 0.13→1.0 API changes. Update the changelog when shipping user-visible changes.
- Release is automated via `just release <semver>` (bumps `Cargo.toml`, runs checks, tags, pushes, `gh release create`, `cargo publish`).

## Reference material

- **`../research_doc/`** — the canonical rich-port reference (start at `00-SUMMARY.md` / `07-QUICK-REFERENCE.md` / `INDEX.md`). Use it to find rich's exact behavior when porting or fixing a widget.
- **`.review/`** (coverage map, perf reports, code-quality, parity audit, v0.11 design) and **`thoughts/`** (research, audits, continuity ledgers) — in-repo prior analysis; consult before large refactors or perf work rather than re-deriving. Excluded from the published crate (`exclude` in `Cargo.toml`).
- **`.review/` strategy + decision docs** (read before proposing big features or perf work): `ultracode-review-2026-06-05.md` + `ultracode-fixes-2026-06-05.md` (the audit that drove the v1.5 correctness pass), `feature-roadmap-2026-06-05.md`, `landscape-and-strategy-2026-06-06.md` (cross-language landscape that shaped v1.8–v1.10), and **`v2-structural-decision-2026-06-06.md`** — the evidence-based ADR deferring the `StyleId`/`SegmentBuf`/SoA refactor (measured `Style`=48 B, win applies only to huge buffers gilt's print model doesn't build; has a documented trigger to revisit).
