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

### Phase 6.5: Close rich feature gaps (decision: 2026-04-29 round 4)

Two structural gaps surfaced in the deep audit, both real (not API ergonomics):

- **`Traceback` widget**: rich's `traceback.py` exposes a standalone widget that captures and renders an arbitrary error/panic as an embeddable panel. gilt has tracing/eyre/miette handlers wrapping the `?` propagation path but no widget you can put inside a Panel/Layout. Build `gilt::traceback::Traceback` (~400-500 lines: frame capture, source-line read, syntect highlighting reuse).
- **Deeper `Pretty`**: rich's `pretty.py` is 948 lines of recursive pretty-printing with cycle detection, dataclass introspection, and a `__rich_repr__` discovery protocol. gilt's `utils/pretty.rs` is a stub. Flesh out: recursive struct/enum/Vec/HashMap rendering with depth limit + cycle detection. Reuse the `Inspect` derive for the introspection protocol.

**Verification gate**: rewrite `examples/exception.rs` (currently no equivalent) and `examples/repr.rs` (depends on deeper Pretty) using the new widgets. Both must come within 1.5× of the rich originals.

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
  - [x] Decisions captured (AskUserQuestion 3 rounds 2026-04-29)
  - [x] v1.0 integration branch created and pushed
  - [x] Tracking issue #20 opened
  - [x] Phase 8 LOC baseline measured and locked (2.22; target ≤1.30)
  - [x] Roadmap committed to main
- Now: [→] Phase 1 implementation
- Next: Phase 1 PR review + merge into v1.0
- Remaining:
  - [ ] Phase 1: Foundation
  - [ ] Phase 2: RAII guards
  - [ ] Phase 3: Live family
  - [ ] Phase 4: Table/Tree/Columns markup
  - [ ] Phase 5: Panel/Padding/Rule/Align
  - [ ] Phase 6: Rust-native extensions
  - [ ] Phase 6.5: Close rich feature gaps (Traceback widget + deeper Pretty)
  - [ ] Phase 7: Derive macros
  - [ ] Phase 8: Examples rewrite (LOC gate)
  - [ ] Phase 9: Documentation
  - [ ] Phase 10: Release

## Decisions (round 3 — 2026-04-29, advisor-flagged)

9. **Text::styled scope**: `Text::styled(content: impl Into<String>, style: &str)`. Content is **literal** (never parses markup). Style arg is parsed via `parse_lossy`. Cell-level markup stays scoped to `Table::add_row` (Phase 4) and existing `console.print_text`. Avoids the "user input containing `[foo]` becomes accidental markup" footgun.
10. **Migration doc lands inside Phase 1, not Phase 9**: `MIGRATION_v1.md` ships in the Phase 1 PR with the `Style::parse` lossy-vs-strict before/after entry. Reason: lossy parse is a silent-failure trap on upgrade — users who strip `?` to compile will get null styles on typos. Each subsequent phase adds its own migration entry in its own PR.
11. **Phase 1 verification gate includes a paired-example rewrite**: rewrite `examples/status.rs` end-to-end using only Phase 1 + existing APIs. Target: ≤35 lines (current 49; full ~14 unlocked after Phase 2 RAII + Phase 3 `Status::set`). Proves the new API actually composes; "cargo test green" doesn't.

## Phase 8 baseline + recalibrated success metric (round 4 — 2026-04-29)

The original 10-example sample understated the gap. Deep audit (research agent
2026-04-29) showed the corpus-wide reality:

| Scope | rich | gilt | ratio |
|---|---:|---:|---:|
| Library source LOC (no tests) | 38,515 | 67,718 | 1.76× |
| 10 cherry-picked examples (entry-level) | 397 | 883 | 2.22× |
| **All 36 paired examples (avg per file)** | **42.5** | **154.2** | **3.63×** |

The 1.76× lib-level ratio is the *floor* set by Rust verbosity; the example
ratio cannot honestly drop below it.

### Recalibrated two-tier success metric (decision: option B)

**Tier 1 — Easy: ≤1.30× on 12 single-widget entry examples**
(`table`, `status`, `columns`, `tree`, `padding`, `rainbow`, `link`, `log`,
`spinners`, `attrs`, `repr`, `rule`)

**Tier 2 — Full: ≤2.00× on the 36-example paired corpus**

Both must pass for Phase 8 to ship. Tier 1 protects the new-user impression;
Tier 2 protects the honest overall claim.

**Status (baseline)**: Tier 1 sample needs measurement. Tier 2 sits at ~3.63×
today; needs to drop to ≤2.0× → roughly halving total example LOC across the
36 paired files. Ergonomics phases (1-7) cover most; structural feature gaps
(Traceback widget, deeper Pretty) handled in Phase 6.5 if scoped in.

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
