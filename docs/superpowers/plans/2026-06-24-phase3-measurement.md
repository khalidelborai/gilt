# Phase 3 — Measurement Protocol (detailed plan)

> REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use `- [ ]`.

**Goal:** `Console::measure` dispatches to a `Renderable::gilt_measure` protocol (width without a full render), matching rich's `__rich_measure__`. Closes audit gaps #2, #3, #4, #11, #14.

**Design:** add `fn gilt_measure(&self, &Console, &ConsoleOptions) -> Measurement` to `Renderable` with a DEFAULT impl = the current full-render fallback (NON-breaking — existing impls keep compiling). Override it on widgets that have a real `measure()`. Add `Measurement::get` + `measure_renderables`. Rewire `Console::measure` through `Measurement::get`. Fix empty case (#3) and `Align` width (#14).

**Base commit:** 77048de. **Branch:** parity-2.0. **2.0 — small correctness-breaking changes (Align::measure return, Console::measure empty case) allowed; flag in CHANGELOG.**

## Global Constraints
- MSRV 1.82; WASM-safe (no new deps); clippy `--all-features -D warnings`; fmt; `cargo nextest run --lib` + `cargo test --doc`.
- **Circular-import (CRITICAL):** `console.rs` already imports `Measurement` from `measure.rs`. Do NOT put `Measurement::get`/`measure_renderables` in `measure.rs` (would need `Console`/`Renderable` → cycle). Put them as free fns in `console_render.rs` (or a new `src/measure_protocol.rs`), re-export from `crate::measure` via `pub use`.
- **Verbatim-copy (CRITICAL):** the default `gilt_measure` body MUST be the EXACT current `Console::measure` width-derivation logic (read `src/console_render.rs:503-522` and copy it), except the empty-case constant. Any drift silently changes measurements for ~30 un-overridden types.
- Reference: `../research_doc/05-cells-and-measure.md`, `06-protocols-and-abc.md`. `Measurement(minimum, maximum)`.

**Order:** 3.1 foundation → 3.2 high-value overrides → 3.3 container overrides → 3.4 Align fix (#14) → 3.5 Console::measure rewire (#2/#3/#11) → 3.6 leaf-widget overrides. 3.5 is HIGHEST RISK and must land after 3.1–3.4.

---

### Task 3.1: `Renderable::gilt_measure` default + `Measurement::get` / `measure_renderables`

**Files:** `src/console.rs` (Renderable trait ~254-272: add default method); `src/console_render.rs` or new `src/measure_protocol.rs` (free fns `measurement_get`/`measure_renderables`); `src/measure.rs` (`pub use` re-export). Test: inline / `console_tests.rs`.

**Produces:**
```rust
// Renderable trait (default impl = current Console::measure body, copied verbatim except empty-case):
fn gilt_measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement { /* full-render fallback; empty → Measurement::new(0, options.max_width) */ }
// free fns (avoid measure.rs cycle), re-exported as crate::measure::{measurement_get, measure_renderables}:
pub fn measurement_get<R: Renderable + ?Sized>(console: &Console, options: &ConsoleOptions, r: &R) -> Measurement // = r.gilt_measure(..).normalize().with_maximum(options.max_width)
pub fn measure_renderables<R: Renderable + ?Sized>(console, options, &[&R]) -> Measurement // fold per rich's max(minimums)/sum-or-max(maximums) — match rich's contract
```

- [ ] **Step 1 — failing tests** (a custom `Renderable` whose `gilt_console` emits known text; assert default `gilt_measure` returns the same min/max the current `Console::measure` would; assert `measurement_get` clamps to max_width). FIRST read `src/console_render.rs:503-522` and base the expected values on its EXACT logic.
- [ ] **Step 2 — run, expect FAIL** (`gilt_measure`/`measurement_get` missing).
- [ ] **Step 3 — implement:** add the default trait method (verbatim copy of current measure body + empty→`(0, max_width)`); add the free fns in console_render.rs/new module; re-export from measure.rs. Resolve imports (`cell_len`, `Measurement`) without creating a cycle.
- [ ] **Step 4 — run, expect PASS**; full `cargo nextest run --lib`; clippy.
- [ ] **Step 5 — commit:** `feat(measure): add Renderable::gilt_measure default + measurement_get + measure_renderables`

**Breaking:** none (additive default method). gilt-tui inherits the default (old behavior) — safe.

---

### Task 3.2: override `gilt_measure` on Text, Panel, Tree, Table (delegate to existing `measure()`)

**Files:** `src/console.rs` (impl Renderable for Text ~274), `src/panel.rs` (~331), `src/tree.rs` (~207), `src/widgets/table/render.rs` (~8). Standalone `measure()` methods are KEPT (public, tested); `gilt_measure` just delegates.

**Consumes:** `Text::measure(&self)` (`text/core.rs:323`, no-console), `Panel::measure(&Console,&ConsoleOptions)` (`panel.rs:229`), `Tree::measure(..)` (`tree.rs:160`), `Table::measure(..)` (`widgets/table/core.rs:1599`).

- [ ] **Step 1 — failing tests:** for each widget, `widget.gilt_measure(&console,&opts) == widget.measure(..)` (Text: `== text.measure()`). e.g.
```rust
#[test]
fn text_gilt_measure_matches_standalone() {
    let console = Console::builder().width(80).build();
    let text = Text::new("Hello World", Style::null());
    assert_eq!(text.gilt_measure(&console, &console.options()), text.measure());
}
```
- [ ] **Step 2 — run, expect FAIL** (resolves to default, differs from standalone).
- [ ] **Step 3 — implement:** add `fn gilt_measure` to each `impl Renderable` block delegating to the standalone `measure()` (Text/Syntax use the no-console variant).
- [ ] **Step 4 — run, expect PASS**; full suite.
- [ ] **Step 5 — commit:** `feat(measure): override gilt_measure on Text, Panel, Tree, Table`

**Breaking:** none.

---

### Task 3.3: override `gilt_measure` on Columns, Padding, Align, Constrain, Group, Bar, Styled, Renderables

**Files:** `src/columns.rs` (~227), `src/utils/padding.rs` (~133), `src/utils/align_widget.rs` (~172), `src/utils/constrain.rs` (~64), `src/utils/group.rs` (~125), `src/utils/bar.rs` (~143), `src/utils/styled.rs` (~38), `src/utils/containers.rs` (~67).

**Note:** Padding/Constrain/Group/Bar/Styled/Renderables delegate to their existing `measure()`. **Columns has NO standalone `measure()`** — its `gilt_measure` must derive from the same per-column logic `Columns::gilt_console` uses (min = widest item `r.measure().maximum`; max = sum of item maximums + inter-column padding, clamped to `options.max_width`). Read `Columns::gilt_console` and mirror its width accounting without rendering.

- [ ] **Step 1 — failing tests** (Padding/Group delegate-match; Columns max == sum-of-item-maxes + padding). 
- [ ] **Step 2 — run, expect FAIL.**
- [ ] **Step 3 — implement** the overrides; Columns gets the custom derivation.
- [ ] **Step 4 — run, expect PASS**; full suite.
- [ ] **Step 5 — commit:** `feat(measure): override gilt_measure on Columns, Padding, Align, Constrain, Group, Bar, Styled, Renderables`

**Breaking:** none (Align override delegates to `Align::measure`, which is fixed in 3.4).

---

### Task 3.4: fix `Align::measure` (#14) — content width, not `max_width`

**Files:** `src/utils/align_widget.rs` (~117-119, the standalone `measure` body).

**Current bug:** `Measurement::new(content_width, options.max_width)` — maximum is ALWAYS `max_width`. **Fix:** when `self.width` is `None`, maximum = `content.measure().maximum.min(options.max_width)`; when `Some(w)`, maximum = `w`. minimum = `content.measure().minimum.min(maximum)`.

- [ ] **Step 1 — failing test:** `Align::left(Text::new("Hi")).measure(&console80,&opts).maximum == 2` (not 80); explicit-width path honored.
- [ ] **Step 2 — run, expect FAIL** (`maximum == 80`).
- [ ] **Step 3 — implement** the fix in the standalone `Align::measure`. (The 3.3 `gilt_measure` override delegates here.)
- [ ] **Step 4 — run, expect PASS**; `cargo nextest run --lib align`.
- [ ] **Step 5 — commit:** `fix(align): measure uses content width not max_width (#14)`

**Breaking:** changes the public `Align::measure()` return (old was a bug). Flag in CHANGELOG.

---

### Task 3.5: rewire `Console::measure` through the protocol + empty-case (#3) — HIGHEST RISK

**Files:** `src/console_render.rs` (~503-522: replace body with `measurement_get(self,&opts,renderable)`); `src/console_tests.rs` (~1147-1160: update `test_measure_empty` from `(0,0)` to `(0, max_width)`).

- [ ] **Step 1 — baseline:** record full `cargo nextest run --lib` green; **`cargo test --doc`** green (console_render.rs has measurement doctests asserting `(5,11)` for "Hello World" — must stay green).
- [ ] **Step 2 — failing tests:** `console.measure(&panel) == panel.measure(&console,&opts)` (routes through gilt_measure); `console.measure(&Text::new("")).maximum == 80` (was 0 — #3).
- [ ] **Step 3 — run, expect FAIL.**
- [ ] **Step 4 — implement:** replace `Console::measure` body with the `measurement_get` dispatch; update `test_measure_empty`.
- [ ] **Step 5 — run full `cargo nextest run --lib` + `cargo test --doc`** (the (5,11) doctest must pass — Text::gilt_measure → Text::measure yields it); clippy.
- [ ] **Step 6 — commit:** `fix(console): Console::measure uses gilt_measure protocol (#2 #3 #11)`

**Breaking:** empty-renderable measure changes `(0,0)`→`(0,max_width)`. Flag in CHANGELOG; check gilt-tui for `maximum==0` empty sentinels.

---

### Task 3.6: leaf-widget `gilt_measure` overrides — CsvTable, Figlet, Sparkline, Diff, Canvas, ProgressBar, Syntax

**Files:** `src/csv_table.rs` (~284), `src/figlet.rs` (~456), `src/sparkline.rs` (~196), `src/diff.rs` (~593), `src/canvas.rs` (~491), `src/progress_bar.rs` (~280), `src/syntax.rs` (~746/Renderable impl). All have a correct standalone `measure()`; `gilt_measure` delegates.

- [ ] **Step 1 — failing tests:** per widget, `gilt_measure(..) == measure(..)` (Syntax: `== self.measure()` no-console).
- [ ] **Step 2 — run, expect FAIL.**
- [ ] **Step 3 — implement** delegating overrides.
- [ ] **Step 4 — run, expect PASS**; full suite (+ `--all-features` for syntax/canvas).
- [ ] **Step 5 — commit:** `feat(measure): gilt_measure overrides for CsvTable, Figlet, Sparkline, Diff, Canvas, ProgressBar, Syntax`

**Breaking:** none.

---

### Phase 3 gate
- [ ] full `cargo nextest run --lib` + `--all-features` + `cargo test --doc` green
- [ ] clippy `--all-features -D warnings` + fmt clean
- [ ] `--no-default-features` + wasm32 build
- [ ] CHANGELOG `[Unreleased]`: Added (`Renderable::gilt_measure`, `measurement_get`/`measure_renderables`); Changed-Breaking (`Align::measure` width, `Console::measure` empty case); Fixed (#2 #3 #4 #11 #14). gilt-tui note.
