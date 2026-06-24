# gilt → rich Parity 2.0 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the 191 verified parity gaps between gilt and Python rich (see `.review/parity-audit-2026-06-24.md`), culminating in a `gilt 2.0` release with full behavioral parity.

**Architecture:** Phased program. Correctness bugs first (non-breaking), then additive protocol work (measurement, theme resolution), then the breaking type-generalization refactor (`Text → Box<dyn Renderable>`), then export/live/sweep. Each phase is independently shippable and committed per fix.

**Tech Stack:** Rust 1.82 MSRV, `unicode-width`/`unicode-segmentation`, `arc-swap`. Tests via `cargo nextest run --lib` + `cargo test --doc`. Reference: `../research_doc/NN-*.md` (rich source analysis).

## Global Constraints

- **MSRV 1.82.0** — `cargo check` must pass on 1.82 (uses `std::sync::LazyLock`). Copied from CLAUDE.md.
- **WASM-safe core** — default, `--no-default-features`, and `wasm32-unknown-unknown --no-default-features --features json,markdown,syntax` must all build. No new `libc`/`crossterm`/terminal-syscall deps outside opt-in native features gated by `#[cfg(not(target_arch = "wasm32"))]`.
- **Clippy clean** — `cargo clippy --all-features -- -D warnings`. **fmt clean** — `cargo fmt --check`.
- **`Style::parse` stays lossy/infallible** (deliberate, MIGRATION_v1.md). Do not change its signature.
- **Breaking changes ALLOWED** (targeting 2.0). When a public signature/type changes, update `CHANGELOG.md`, `MIGRATION_v1.md` (add a v2 migration section), and note downstream impact on `../gilt-tui`.
- **TDD** — every fix starts with a failing test. **Hashing** uses `utils::hash::fnv1a_64`. New widget logic adds inline `#[cfg(test)]` tests.
- Reference behavior is rich's; consult the matching `../research_doc/NN-*.md` before changing a widget.

---

## Phase Roadmap

| Phase | Theme | Breaking? | Audit refs (P0/P1 backlog #) | Depends on |
|---|---|---|---|---|
| **1** | Correctness quick-wins | No | #1, #39, #24, #32, #38, #17 | — |
| **2** | Render-time theme resolution | No (additive) | #5, #6, #7, #29, #37 | — |
| **3** | Measurement protocol | No (additive default method) | #2, #3, #4, #11, #14 | — |
| **4** | Container/widget generalization (`Text → Box<dyn Renderable>`) | **Yes** | #13, #15, #16, #19, #21 | Phase 3 |
| **5** | Export correctness (SVG/HTML) | Mostly no | #22, #23, #24(done P1), export §3.10 | — |
| **6** | Live/progress nesting + screen styling | Some | #25, #26, #27, #28 | — |
| **7** | P2/P3 sweep (subsystem-by-subsystem) | Mixed | remaining ~150 in §3 of the audit | per-item |

Each phase is detailed just-in-time at execution (Phases 2–7 get their bite-sized task breakdown when reached, after Phase 1 lands, so they build on the actual landed code). Phase 1 is fully detailed below.

The published-crate touchpoints (CHANGELOG, MIGRATION_v1 v2 section, gilt-tui update, `just release 2.0.0`) are a final **Phase R (Release)** handled after Phase 7.

---

## Phase 1 — Correctness Quick-Wins

Five isolated, non-breaking correctness bugs. Highest value/risk ratio. All confirmed against source.

### Task 1.1: Color downgrade early-return (audit #39)

**Files:**
- Modify: `src/color/mod.rs:411` (`Color::downgrade`)
- Test: inline `#[cfg(test)]` in `src/color/mod.rs`

**Interfaces:**
- Consumes: `Color::system() -> ColorSystem`, `ColorSystem` (must be `PartialOrd`/`Ord` so `Standard < EightBit < TrueColor < Windows`-comparable; if not already, add a fidelity comparison helper rather than changing the enum's derive if that reorders).
- Produces: `Color::downgrade(system)` returns `*self` unchanged when the color's own system is already at or below `system` fidelity.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn downgrade_to_same_system_is_identity() {
    // A Standard color downgraded to Standard must NOT be re-matched through
    // the palette (which can change the index). It must return itself.
    let c = Color::Standard(5);
    assert_eq!(c.downgrade(ColorSystem::Standard), Color::Standard(5));
    // EightBit -> EightBit identity.
    assert_eq!(Color::EightBit(123).downgrade(ColorSystem::EightBit), Color::EightBit(123));
    // EightBit -> Standard still downgrades (different system, lower fidelity).
    // (no identity assertion here — just must not panic and must be Standard)
    assert!(matches!(Color::EightBit(123).downgrade(ColorSystem::Standard), Color::Standard(_)));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo nextest run --lib downgrade_to_same_system_is_identity`
Expected: FAIL — `Color::Standard(5).downgrade(Standard)` currently returns a palette-rematched index ≠ 5.

- [ ] **Step 3: Implement the early-return**

In `Color::downgrade`, after the `Color::Default` guard, add a guard that returns `*self` when the color is already at the target system (or strictly lower fidelity). Determine the color's system via the existing `system()`/variant and compare. Concretely: `Standard`/`Windows` colors returned unchanged for `Standard`/`Windows`/`EightBit`/`TrueColor` targets; `EightBit` returned unchanged for `EightBit`/`TrueColor`. Read the existing `Color::system()` to wire the comparison; if no total order exists, special-case by variant.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo nextest run --lib downgrade` — Expected: PASS. Then `cargo nextest run --lib color::` to ensure no regression in existing downgrade tests.

- [ ] **Step 5: Commit**

```bash
git add src/color/mod.rs
git commit -m "fix(color): downgrade() is identity when already at target system"
```

### Task 1.2: P0 — wire color downgrade into the render path (audit #1)

**Files:**
- Modify: `src/style.rs:557-624` (`Style::render_inner`)
- Test: inline `#[cfg(test)]` in `src/style.rs`

**Interfaces:**
- Consumes: `Color::downgrade(ColorSystem)` (Task 1.1), `Color::write_ansi_codes(foreground, buf)`.
- Produces: `Style::render`/`render_no_link` emit SGR codes appropriate to the supplied `color_system` (no truecolor on a Standard console).

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn render_downgrades_color_to_console_system() {
    // A truecolor style rendered for a Standard (16-color) console must emit a
    // 16-color SGR (30-37/90-97), never a truecolor "38;2;r;g;b".
    let st = Style::parse("#ff0000"); // bright red truecolor fg
    let out = st.render("X", Some(ColorSystem::Standard));
    assert!(!out.contains("38;2;"), "must not emit truecolor on Standard: {out:?}");
    assert!(out.contains("\x1b["), "must still emit an SGR: {out:?}");
    // EightBit console -> 256-color form, not truecolor.
    let out8 = st.render("X", Some(ColorSystem::EightBit));
    assert!(!out8.contains("38;2;"), "must not emit truecolor on EightBit: {out8:?}");
    assert!(out8.contains("38;5;"), "EightBit fg should use 38;5;N: {out8:?}");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo nextest run --lib render_downgrades_color_to_console_system`
Expected: FAIL — currently emits `38;2;255;0;0` for both systems.

- [ ] **Step 3: Implement the downgrade in render_inner**

In `render_inner`, `color_system` is already `Some` past the `is_none()` early return at line 563. Before each `write_ansi_codes`, downgrade the color to that system:

```rust
let sys = color_system.expect("color_system is Some past the early return");
if let Some(color) = &self.color {
    color.downgrade(sys).write_ansi_codes(true, &mut sgr);
}
if let Some(bgcolor) = &self.bgcolor {
    bgcolor.downgrade(sys).write_ansi_codes(false, &mut sgr);
}
if let Some(ul_color) = &self.underline_color {
    ul_color.downgrade(sys).write_underline_color_codes(&mut sgr);
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo nextest run --lib render_downgrades` then `cargo nextest run --lib style::` and `cargo nextest run --lib` (full) — Expected: all PASS. Watch for golden-output tests that assumed un-downgraded truecolor; update any that were asserting the *bug*.

- [ ] **Step 5: Commit**

```bash
git add src/style.rs
git commit -m "fix(style): downgrade colors to the console color system when rendering (P0)"
```

### Task 1.3: terminal_theme palettes 8 normal + 8 bright (audit #24)

**Files:**
- Modify: `src/color/terminal_theme.rs:119-194` (`MONOKAI`, `DIMMED_MONOKAI`, `NIGHT_OWLISH`)
- Test: inline `#[cfg(test)]` in `src/color/terminal_theme.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn builtin_themes_have_8_normal_and_8_bright() {
    for (name, theme) in [
        ("MONOKAI", &MONOKAI), ("DIMMED_MONOKAI", &DIMMED_MONOKAI), ("NIGHT_OWLISH", &NIGHT_OWLISH),
    ] {
        assert_eq!(theme.ansi_colors_normal().len(), 8, "{name} normal must be 8");
        assert_eq!(theme.ansi_colors_bright().len(), 8, "{name} bright must be 8");
    }
}
```
(Adjust accessor names to the actual `TerminalTheme` API after reading the struct; the invariant is 8 + 8.)

- [ ] **Step 2: Verify it fails** — Run: `cargo nextest run --lib builtin_themes_have_8`. Expected: FAIL (currently 9 + 7).
- [ ] **Step 3:** Re-derive each palette from rich's `rich/terminal_theme.py` (cross-check `../research_doc/09-themes.md`) so normal = indices 0–7, bright = 8–15. Fix the misplaced color.
- [ ] **Step 4: Verify pass** — `cargo nextest run --lib terminal_theme` — Expected: PASS. Add an HTML-export round-trip assertion if a golden exists.
- [ ] **Step 5: Commit** — `git commit -am "fix(themes): MONOKAI/DIMMED_MONOKAI/NIGHT_OWLISH use 8 normal + 8 bright ANSI colors"`

### Task 1.4: pretty truncate puts `+N` outside the quote (audit #32)

**Files:**
- Modify: `src/utils/pretty.rs:614-622` (`truncate_debug_strings` / quote handling)
- Test: inline `#[cfg(test)]` in `src/utils/pretty.rs`

- [ ] **Step 1: Failing test**

```rust
#[test]
fn truncate_debug_string_places_suffix_outside_quote() {
    // rich renders truncated strings as "kept"+N, not "kept+N".
    let out = truncate_debug_strings("\"abcdefghij\"", 5); // adjust to real fn signature
    assert!(out.ends_with("\"+5") || out.contains("\"+"), "suffix must be outside the closing quote: {out:?}");
    assert!(!out.contains("j+"), "suffix must not be inside the quote: {out:?}");
}
```
(Adapt to the actual function name/signature found at line ~614.)

- [ ] **Step 2: Verify fails** — Expected: FAIL (currently `"kept+N"`).
- [ ] **Step 3:** Move the `+N` suffix after the closing quote character. Cross-check `../research_doc/19-pretty.md:108-111`.
- [ ] **Step 4: Verify pass** — `cargo nextest run --lib pretty`.
- [ ] **Step 5: Commit** — `git commit -am "fix(pretty): place truncation +N suffix outside the closing quote"`

### Task 1.5: prompt ask_int/ask_float terminate on EOF (audit #38)

**Files:**
- Modify: `src/prompt.rs:607-647` (`ask_int`, `ask_float` read loops)
- Test: inline `#[cfg(test)]` in `src/prompt.rs` (drive with an empty/EOF reader, not stdin)

- [ ] **Step 1: Failing test** — construct a prompt over a reader that is already at EOF and assert `ask_int` returns an `Err`/`None` (per the function's signature) rather than looping. Read the function to find the injectable reader; if none exists, the minimal fix is to add an EOF branch that breaks the loop.

```rust
#[test]
fn ask_int_returns_on_eof_instead_of_looping() {
    // Build a Prompt reading from an empty cursor (immediate EOF).
    let mut input = std::io::Cursor::new(Vec::<u8>::new());
    // ... construct the prompt with `input` as its reader (use existing test ctor) ...
    let result = /* prompt */ .ask_int_from(&mut input); // adapt to real API
    assert!(result.is_err() || result.is_none(), "EOF must terminate, got {result:?}");
}
```

- [ ] **Step 2: Verify fails** — Expected: hang/FAIL (infinite loop ⇒ run with `--test-threads` timeout or assert the EOF branch is missing by inspection first, then write the breaking test).
- [ ] **Step 3:** In the read loop, when the reader returns `Ok(0)` (EOF), break with a terminating error/`None` instead of re-prompting. Cross-check `../research_doc/23-prompt.md:41-56` (rich raises `EOFError`).
- [ ] **Step 4: Verify pass** — `cargo nextest run --lib prompt`.
- [ ] **Step 5: Commit** — `git commit -am "fix(prompt): ask_int/ask_float terminate on EOF instead of looping"`

### Task 1.6: Panel fit-mode measures longest line, not total chars (audit #17)

**Files:**
- Modify: `src/panel.rs:354-356` (fit-mode width)
- Test: inline `#[cfg(test)]` in `src/panel.rs`

- [ ] **Step 1: Failing test**

```rust
#[test]
fn panel_fit_uses_longest_line_not_total_chars() {
    // Two short lines: longest line = 3 cells, total chars = 6+newline.
    // A fit panel should be ~ longest-line + borders + padding, NOT total-chars wide.
    let p = Panel::new(Text::new("abc\nde", Style::null())).fit(); // adapt to real fit ctor
    let console = Console::builder().width(80).markup(false).no_color(true).build();
    let measured = console.measure(&p, None); // or panel.measure(...)
    assert!(measured.maximum <= 3 + 4, "fit width must track longest line (3)+borders, got {}", measured.maximum);
}
```
(Adapt to the real fit API + measure call; the invariant: fit width derives from the longest rendered line.)

- [ ] **Step 2: Verify fails** — Expected: FAIL (uses `cell_len` of all content).
- [ ] **Step 3:** Replace the `cell_len()`-of-content computation with the maximum rendered-line width (longest line). If Phase 3's `measure()` isn't landed yet, compute the longest line directly by splitting on newlines and taking `max(cell_len(line))`. Cross-check `../research_doc/12-panel-and-box.md`.
- [ ] **Step 4: Verify pass** — `cargo nextest run --lib panel`.
- [ ] **Step 5: Commit** — `git commit -am "fix(panel): fit mode measures the longest line, not total character count"`

### Phase 1 gate

- [ ] `cargo nextest run --lib` — all green
- [ ] `cargo test --doc` — all green
- [ ] `cargo clippy --all-features -- -D warnings` — clean
- [ ] `cargo fmt --check` — clean
- [ ] `cargo check --no-default-features` and wasm32 build — clean
- [ ] Update `CHANGELOG.md` `[Unreleased]` with the 6 fixes
- [ ] Checkpoint with the user before Phase 2

---

## Phases 2–7 (scoped; detailed at execution)

- **Phase 2 — Render-time theme resolution:** thread `&Console`/theme through `markup::render` and `Text::gilt_console` so `[warning]`/`[repr.number]`/group-style names resolve against the theme at render time instead of `Style::null()` at parse time. Spans must store the *style name string* (or an unresolved marker), resolved in `Text::render`/`gilt_console`. Touches `markup.rs`, `text/core.rs`, `utils/highlighter.rs`. Additive (new internal plumbing; markup spans gain a name-carrying variant).

- **Phase 3 — Measurement protocol:** add `fn gilt_measure(&self, &Console, &ConsoleOptions) -> Measurement` to `Renderable` with a default that does the current full-render fallback (non-breaking). Add `Measurement::get` and `measure_renderables`; override `gilt_measure` for `Text`, `Table`, `Panel`, `Tree`, `Columns`, `Padding`, `Align`. Fix `Console::measure` empty case to return `(0, max_width)`.

- **Phase 4 — Container/widget generalization (BREAKING):** change `Group`, `Renderables`, `Align`, `Padding`, `Constrain`, `Styled`, `Panel.content`, `Tree.label`, and `Table` cell content from `Text` to `Box<dyn Renderable + ...>` (decide exact bound: likely `Box<dyn Renderable + Send + Sync>` to match `Live`). Provide `From<Text>`/`From<&str>` ergonomic conversions so most call sites compile unchanged. Depends on Phase 3 (measurement of nested widgets). Update gilt-tui.

- **Phase 5 — Export correctness:** SVG line cropping to width (`split_and_crop_lines`), `reverse` background handling, `dim` blend when bg is None, per-line clip-paths, `record=false` guard. `console_export.rs`.

- **Phase 6 — Live/progress nesting + screen styling:** wire `push_live`/`pop_live` into `Live::start`/`stop`; fix screen-mode `do_refresh` to preserve ANSI styling (render styled, not flattened); add Progress nested-stack support; `Table.grid()` log layout in logging handlers.

- **Phase 7 — P2/P3 sweep:** work through §3 of `.review/parity-audit-2026-06-24.md` subsystem-by-subsystem (≈150 items). Group into per-subsystem subagent tasks; each is a small TDD fix. Independent items can be dispatched in parallel.

- **Phase R — Release:** finalize `CHANGELOG.md`, add a v2 section to `MIGRATION_v1.md`, update `../gilt-tui` for the breaking container API, run `just check-all`, then `just release 2.0.0`.

---

## Self-Review notes

- **Spec coverage:** every P0/P1 backlog item (#1–#39) maps to a Phase (1: #1,#17,#24,#32,#38,#39; 2: #5,#6,#7,#29,#37; 3: #2,#3,#4,#11,#14; 4: #13,#15,#16,#19,#21; 5: #22,#23; 6: #25,#26,#27,#28). Remaining P2/P3 → Phase 7 sweep against §3.
- **Ordering:** Phase 3 (measurement) precedes Phase 4 (generalization) because nested-widget layout needs `gilt_measure`. Task 1.1 precedes 1.2 because the P0 render fix relies on a correct `downgrade`.
- **No placeholders in Phase 1.** Phases 2–7 are intentionally scoped (not bite-sized) and will be expanded into TDD tasks at execution, against the then-current code.
