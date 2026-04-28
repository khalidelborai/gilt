# gilt v1.0.0 Ergonomics Overhaul

## Goal

Reach rich-level API simplicity for entry-level usage. Specifically: rewrite all paired examples (~30) so that total LOC is **≤1.3× rich's equivalent** — currently 3× on entry-level examples, ~1.07× overall but with huge per-example variance.

Ship as a single breaking release: `gilt 1.0.0` + `gilt-derive 1.0.0`.

## Constraints

- **Mandate granted: "no matter how much rewrite"** — full mandate to break any API, but every break must reduce LOC or improve safety with a measurable case.
- **Idiomatic Rust** stays sacred: no anti-patterns to chase line count (e.g., panicking constructors, Stringly-typed builders that lose type safety, hidden global state).
- **MSRV stays 1.82.0** — no feature requiring newer Rust.
- **Lockstep release**: `gilt 1.0.0` + `gilt-derive 1.0.0` published together.
- **Migration guide is mandatory**, not optional. Every break documented with before/after.

## Key Decisions (from AskUserQuestion 2026-04-29)

1. **Release strategy**: v1.0.0 big bang. No staged deprecations; one migration cost.
2. **Patterns adopted (all four)**:
   - Lossy parsers + smart defaults (`Style::parse(s) -> Style`, `Console::default()`, etc.)
   - Markup-first cell APIs (`Table::add_row(&["[red]hi[/]"])` parses markup)
   - RAII guards everywhere (`Live::start() -> LiveGuard` with `Drop = stop`)
   - Allow `rand` / `chrono` in examples (drops ~80 lines of inline LCG/time formatting)
3. **Scope**: full sweep — entry widgets, live family, derive macros, Rust-native extensions.
4. **Success metric**: ≤1.3× rich line count across 10 paired examples (objective gate).

## Roadmap

Each phase = its own PR + verification gate. Phases 1-7 land additively on a `v1.0` integration branch; phase 8 (examples rewrite) validates the LOC metric; phase 9 (docs); phase 10 (release).

### Phase 1: Foundation — Errors, defaults, parsers
- `Console::default()` (auto-detect TTY, color, no_color from env)
- `Style::parse(s) -> Style` (lossy; null on bad input). Move strict to `Style::parse_strict(s) -> Result<Style, ParseError>`.
- `Text::styled(s: impl Into<String>, style: &str)` — single ergonomic constructor; parses markup if `[...]` present, else literal.
- Markup parsing API consolidation: one entry point, no per-widget reimplementation.
- **Verification**: `cargo test --lib` green; `Console::default().print_text("hello")` smoke test.

### Phase 2: RAII guards
- `LiveGuard`, `StatusGuard`, `PagerGuard`, `ScreenGuard` returned from `start()` methods.
- Original `start()/stop()` deleted. The guard's `Drop` impl is the only stop path.
- **Verification**: panic-safety test for each (`std::panic::catch_unwind` shows terminal restored).

### Phase 3: Live-rendering family simplification
- `Live::from_renderable<R: Renderable>(r: R)` — accepts any renderable, no capture/from_ansi roundtrip.
- `Status::set(&self, msg: &str)` direct setter (current `update().status().apply().unwrap()` chain deleted).
- `Progress::add_task(desc: &str) -> TaskId` simplification; collapse the 3-arg variants.
- **Verification**: rewrite `top_lite_simulator.rs`, target ~80 lines (rich is 79).

### Phase 4: Table + Tree + Columns markup-first
- `Table::columns([("PID", Right), ("Cmd", Left), ...])` builder shorthand replaces 7× `add_column(_, _, ColumnOptions { .. })`.
- `Table::add_row(&[&str])` parses markup per cell. Drop `add_row_styled`/`add_row_text`/`add_row_text_styled` overload set; one method, markup-driven.
- Same treatment for Tree node labels, Columns items.
- **Verification**: rewrite `table.rs` ≤25 lines (rich: 21), `columns.rs` ≤35 (rich: 28).

### Phase 5: Panel, Padding, Rule, Align
- `Panel::new(title, body)` accepts `&str`/`Text`/markup uniformly.
- Default constructors for all (drop builder-only patterns).
- **Verification**: paired examples within 1.3× rich.

### Phase 6: Rust-native extensions audit
- Gradient, Sparkline, Canvas, Diff, Figlet, CsvTable: consistency pass. Keep functionality, align constructor names with Phase 1-5 patterns.
- **No rewrites without specific friction** — these are gilt's unique value, don't churn for churn's sake.

### Phase 7: Derive macros polish
- Small additive helpers consistent with new ergonomics.
- `#[derive(Renderable)]` from `fmt::Display` if cheap.

### Phase 8: Examples rewrite (the verification phase)
- Rewrite all 30 paired examples + audit the gilt-only 75.
- Add `rand`, `chrono` to `[dev-dependencies]`.
- **Hard gate**: total paired LOC ≤ 1.3× rich's. If we miss, identify which API still leaks ceremony, return to Phase 1-5.

### Phase 9: Documentation
- Migration guide (`MIGRATION_v1.md`) — every break documented with before/after.
- README rewrite for new API.
- CHANGELOG: full v1.0.0 entry organized by phase.

### Phase 10: Release
- Pre-publish gauntlet (cargo fmt/test/clippy/doc/build-examples all green).
- Publish gilt-derive 1.0.0 then gilt 1.0.0.
- Tag, GitHub release, crates.io verification.

## State
- Done:
  - Decisions captured (AskUserQuestion 2026-04-29)
- Now: [→] Drafting Phase 1 design (need user go-signal before implementation)
- Next: Phase 1 implementation
- Remaining:
  - [ ] Phase 1: Foundation
  - [ ] Phase 2: RAII guards
  - [ ] Phase 3: Live family
  - [ ] Phase 4: Table/Tree/Columns markup
  - [ ] Phase 5: Panel/Padding/Rule/Align
  - [ ] Phase 6: Rust-native extensions
  - [ ] Phase 7: Derive macros
  - [ ] Phase 8: Examples rewrite (LOC gate)
  - [ ] Phase 9: Documentation
  - [ ] Phase 10: Release

## Decisions (round 2 — 2026-04-29)

5. **Branch model**: long-lived `v1.0` integration branch off `main`. Each phase = PR into `v1.0`. Final merge to `main` at Phase 10. `main` stays at 0.13.x for any hotfixes.
6. **Markup escape**: backslash, rich-compatible. `add_row(&["\\[red]not-markup\\[/]"])` renders literal brackets. Familiar to anyone migrating from rich.
7. **Renderable trait**: keep current `fn render(&self, console, options) -> Vec<Segment>`. Streaming variant (`impl Iterator<Item = Segment>`) deferred to v1.x — explicitly documented in the v1.0 CHANGELOG as a known future consideration so users don't expect it.
8. **Deps policy**:
   - `compact_str` + `arc-swap` stay as direct deps (load-bearing for perf).
   - Add `rand` and `chrono` to `[dev-dependencies]` for examples.
   - Audit feature-gated deps (tracing-subscriber, miette, eyre, syntect tree size). Land any reductions in Phase 6 or earlier where natural.

## Open Questions
- UNCONFIRMED: Progress per-task customization (visible columns, units, formatters) — collapse to one builder or keep ColumnOptions-style flexibility? Decide in Phase 3.

## Working Set
- Integration branch: `v1.0` off `main` (long-lived; created at Phase 1 start)
- Per-phase feature branches: `v1.0/phase-N-<name>` off `v1.0`, PR into `v1.0`
- Final: `v1.0` merges to `main` at Phase 10
- Verification commands per phase:
  - `cargo fmt --check`
  - `cargo test --lib && cargo test --doc`
  - `cargo test --lib --features "tracing derive miette eyre"`
  - `cargo test -p gilt-derive --lib`
  - `cargo test -p gilt-derive --test trybuild`
  - `cargo clippy -p gilt --features "tracing derive miette eyre" -- -D warnings`
  - `cargo clippy -p gilt-derive --all-targets -- -D warnings`
  - `cargo build --features derive --examples` (Phase 8: also measure LOC)
- LOC measurement command:
  ```bash
  for f in table status columns top_lite_simulator tree layout log live_progress padding rainbow; do
    r=$(wc -l < ../rich/examples/$f.py 2>/dev/null || echo 0)
    g=$(wc -l < examples/$f.rs 2>/dev/null || echo 0)
    echo "$f rich:$r gilt:$g ratio:$(awk "BEGIN{print $g/$r}")"
  done
  ```
