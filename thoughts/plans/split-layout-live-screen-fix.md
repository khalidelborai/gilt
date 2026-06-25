# Fix plan — split `Layout` collapses under `Live::with_screen(true)`

**Status:** PLAN ONLY (root-caused + reproduced headlessly). No product code changed.
**Date:** 2026-06-25
**Crate:** `gilt` 2.3.0 (bug reported against 2.0)
**Reporter:** DeepForge (alt-screen coding-agent TUI)

---

## 1. Summary

A multi-region `Layout` (regions built with `split_row` / `split_column`, each a `Panel`)
renders with **correct geometry through the print/export path** but **collapses under
`Live::with_screen(true)`** (the alternate-screen path): content squishes into a thin left
column, the middle region looks empty, and panel borders scatter / float at the terminal edge.

**The geometry computation is NOT the bug.** The region map and per-row segment stream that the
screen path feeds to the terminal are *byte-for-byte identical* to the print/export path
(proven below). The collapse is an **emission defect**: the alt-screen frame is written as a
single `ESC[H` (cursor-home) followed by full-width lines separated by **bare `LF` (`\n`) with no
carriage return and no per-line absolute positioning**. That frame only renders correctly if the
host tty performs `LF`→`CRLF` translation (the `ONLCR` termios flag, i.e. *cooked* mode).
Interactive alt-screen TUIs run the tty in **raw mode** (`ONLCR` off — they read keystrokes
directly), so every line after the first begins at the column where the previous full-width line
ended → autowrap / vertical drift → the split `Layout` collapses.

The print/export path renders into a **string buffer** (no real cursor), so the missing `CR` is
invisible there — which is exactly why DeepForge's `export_text` unit tests pass while the live
alt-screen run is broken.

**Root cause (one line):** `src/live/mod.rs:607-608` — `do_refresh`'s screen branch emits the
whole frame via `console.print(&Screen::from_arc(content))` (LF-only separators, `Screen`
`application_mode` defaults `false` at `src/live/screen.rs:49`), positioning lines *only* with a
single `home()` and relying on tty `ONLCR`; it never positions each line absolutely the way the
non-screen path and `Console::update_screen_lines` already do.

**Reproduced headlessly:** yes.

---

## 2. Confirmed reproduction

All three observations below come from a throwaway harness (path-dependency on this crate; not
committed). The harness builds a DeepForge-shaped layout:
`root = split_column([ header(size 3), body = split_row([sidebar(30), main(ratio 1), metrics(30)]), footer(4) ])`,
each region a `Panel`, on a `Console::builder().width(W).height(H).force_terminal(true)`.

### 2a. Geometry is identical in both paths

Comparing `layout.gilt_console(&console, &opts)` (print path) against
`Screen::from_arc(layout).gilt_console(&console, &opts)` (the renderable the screen path wraps),
at `W=100, H=30`:

```
Total differing rows: 0 / 30
A widths: [100, 100, … 100]   (print path)
B widths: [100, 100, … 100]   (screen path)
```

Both paths emit the same 30 rows, each exactly 100 cells, borders at the same columns. **The
region math (`Layout::make_region_map` / `Layout::gilt_console`, `src/layout.rs:387-412` &
`:631-672`) is not where the bug lives.**

Why they match: `Console::options()` (`src/console.rs:1019-1036`) sets `max_width == size.width`
and `height == None`. `Layout::gilt_console` uses `options.max_width` + `options.size.height`;
`Screen::gilt_console` uses `options.size.width/height` then `render_lines` with the same
`max_width`. With a correctly-sized console the two resolve to the same `width × height`.

### 2b. The emitted alt-screen frame has no CR and no per-line positioning

Capturing the *raw bytes* of the screen frame (faithfully replicating the `do_refresh` screen
branch: `set_alt_screen(true)` → `begin_synchronized()` → `control(home())` →
`print(&Screen::from_arc(layout))` → `end_synchronized()`), at `W=40, H=12`, with escapes made
visible:

```
⟨ESC⟩[?2026h⟨ESC⟩[H╭───────…───╮⟨LF⟩
│ HEAD                                 │⟨LF⟩
…
│          ││                          │⟨LF⟩
⟨ESC⟩[?2026l

home() sequences         : 1      ← the ONLY cursor positioning in the whole frame
'H' (CUP) sequences      : 1
LF ('\n')                : 12     ← one bare LF after every line
CR ('\r')                : 0      ← never returns the column
Screen::from_arc default application_mode = false
```

Every line is positioned purely by the trailing `LF`. There is no `CR` and no
`ESC[row;colH` per line.

### 2c. The same bytes render correctly (cooked) and collapse (raw)

Feeding that exact captured frame through a minimal terminal-grid model twice — once with
`ONLCR` on (cooked: `LF` ⇒ `CR`+`LF`) and once with `ONLCR` off (raw: `LF` = line-feed only,
column preserved, `DECAWM` autowrap on):

```
COOKED tty (ONLCR on) — matches print/export, CORRECT
+----------------------------------------+
|╭──────────────────────────────────────╮|
|│ HEAD                                 │|
|│                                      │|
|╭──────────╮╭──────────────────────────╮|
|│ side     ││ MAIN                     │|
|│          ││                          │|
|│          ││                          │|   … (full 12-row frame, regions aligned)
+----------------------------------------+

RAW tty (ONLCR off) — what an alt-screen TUI gets, COLLAPSED
+----------------------------------------+
|╭──────────────────────────────────────╮|
|                                        |   ← blank: line 2 wrapped down a row
|│ HEAD                                 │|
|                                        |
|│                                      │|
|                                        |
|╭──────────╮╭──────────────────────────╮|
|                                        |
|│ side     ││ MAIN                     │|
|                                        |   ← frame runs off the bottom; lower
+----------------------------------------+      regions never appear → "empty middle"
```

The raw render reproduces the report: rows drift, blank lines appear, the lower half of the
layout is pushed off-screen so the middle/footer regions look empty, and full-width borders land
at unexpected columns. (Real terminals vary in *deferred-wrap* semantics — some smear the border
characters horizontally instead of inserting clean blank rows — but the failure mode is the same:
**bare-LF, full-width lines in raw mode do not stay column-aligned.**)

---

## 3. Root cause (precise divergence, with `file:line`)

### The single live-screen emit path
`Live::with_screen(true)` → every frame goes through `Live::do_refresh` (`src/live/mod.rs:576`).
Its screen branch (`src/live/mod.rs:593-609`):

```rust
if is_screen {
    let mut s = state.lock().unwrap();
    s.live_render.set_renderable(content.clone());
    s.live_render.vertical_overflow = vertical_overflow;
    s.console.begin_synchronized();
    let home_ctrl = crate::control::Control::home();   // :605  ← only positioning for the whole frame
    s.console.control(&home_ctrl);
    let screen = Screen::from_arc(content.clone());     // :607  ← application_mode defaults false
    s.console.print(&screen);                           // :608  ← dumps LF-separated lines
    s.console.end_synchronized();
}
```

`Screen::from_arc` (`src/live/screen.rs:45-51`) sets `application_mode: false`, so
`Screen::gilt_console` (`src/live/screen.rs:97-101`) joins lines with `Segment::line()` (bare
`\n`) rather than `\n\r`:

```rust
let new_line = if self.application_mode { Segment::text("\n\r") } else { Segment::line() };
```

`Console::set_alt_screen` (`src/console.rs:1316-1325`) only toggles the alternate buffer
(`ESC[?1049h` + home); gilt **never puts the tty into raw mode itself** (raw mode appears only in
transient helpers `src/terminal_bg.rs` and `src/fuzzy_select.rs`). So whether `ONLCR` is on is
entirely up to the host — and a keystroke-reading TUI like DeepForge has it off.

### Why it diverges from the path that works
- **Print/export** (`console.print(&layout)` then `export_text`): segments are serialized into a
  `String` buffer; `\n` is just a string newline, so column alignment is automatic and correct
  regardless of `CR`. → passes.
- **Non-screen `Live`** (`src/live/mod.rs:687-732`): already does the right thing — it positions
  every line with explicit control codes (`CarriageReturn` + `EraseInLine` + per-line
  `CursorDown`), so it is robust in raw mode. The screen branch is the **only** emit path that
  skipped this discipline.
- **`Console::update_screen_lines`** (`src/console.rs:1593-1602`) *also* already does it right —
  per line it emits `Control::move_to(x, y+i)` (absolute CUP) then the line. It is unused on the
  live path today.

So the bug is a **localized emission regression in the screen branch only**: it leans on tty
`ONLCR` instead of positioning lines absolutely like every sibling path already does.

### Why split `Layout` specifically (and why the Group+Columns workaround dodges it)
`Layout` pads **every** row to the full screen width (`Segment::set_shape` to region/screen width,
`src/layout.rs:450` / `src/live/screen.rs:94`). Full-width lines put the cursor at the right
margin on every row, so the raw-mode drift is maximal and uniform → total collapse. DeepForge's
workaround (`Group` + `Columns` + `Constrain`) produces narrower / non-full-width flow content
(and is naturally driven through the **non-screen** `Live` path, which positions each line
explicitly), so it does not depend on `ONLCR` and stays aligned. The fragility is the screen
branch, not the widget.

---

## 4. Fix approach

**Goal:** the `Live::with_screen` frame must position content with terminal-independent control
codes, exactly like the already-correct non-screen path and `update_screen_lines` — *not* rely on
the tty translating `LF`→`CRLF`. The geometry already matches, so the fix is purely in *how the
already-correct lines are emitted*.

### Recommended — Option A: absolute per-line positioning (rich/`update_screen_lines` parity)

Render the `Screen` to per-line segments and splat them with `Console::update_screen_lines`
(which already emits an absolute `move_to(0, i)` before each line and is a no-op outside
alt-screen). Replace the body of the screen branch (`src/live/mod.rs:593-609`):

```rust
if is_screen {
    let mut s = state.lock().unwrap();
    s.live_render.set_renderable(content.clone());
    s.live_render.vertical_overflow = vertical_overflow;

    // Render the same Screen renderable to per-line segments (geometry unchanged —
    // identical to today's `print(&screen)` content), then position each line
    // absolutely so the frame is independent of the tty's ONLCR/raw mode.
    let screen = Screen::from_arc(content.clone());
    let opts = s.console.options();
    let lines = s.console.render_lines(&screen, Some(&opts), None, true, false);

    s.console.begin_synchronized();
    s.console.update_screen_lines(0, 0, &lines); // per-line ESC[i+1;1H + line
    s.console.end_synchronized();
}
```

Notes:
- Drops the standalone `home()` — `update_screen_lines` positions every line, including row 0,
  so home is redundant.
- `render_lines(&screen, …)` reuses `Screen`'s width/height shaping and optional background style
  (preserves the #25 styling fix), then re-splits into `Vec<Vec<Segment>>`. The line content is
  the same bytes `print(&screen)` produced today (verified identical in §2a).
- Robust against the *deferred-wrap at the bottom row* edge case too: each line is repositioned,
  so a full-width final line cannot scroll the alt buffer.
- This is the rich-parity-correct shape for a full-screen buffer: position the frame, don't lean
  on newline translation. (Consult `../research_doc/14-layout.md` / a `*live*.md` doc — not
  checked out in this worktree — to confirm rich's `Screen.update`/`Control.home` emit ordering
  before finalizing; rich avoids the symptom mainly because its `Live` leaves the tty cooked, so
  gilt going absolute is a strict robustness improvement.)

### Minimal — Option B: enable `application_mode` on the screen frame

One-line change at `src/live/mod.rs:607`:

```rust
let screen = Screen::from_arc(content.clone()).with_application_mode(true);
```

This switches the separator to `\n\r` (`src/live/screen.rs:97-100`), so the `CR` returns the
column after every `LF`. Verified to fix the raw-mode collapse in the §2c model. It is safe in
cooked mode too (`ONLCR` turns `\n\r` into `\r\n\r` — the extra `CR` is a no-op).

**Trade-off:** Option B is a smaller diff but keeps relying on relative newline motion (no
absolute CUP), so it does not harden the bottom-row deferred-wrap case and is conceptually a
work-around. **Do NOT** fix this by flipping the `Screen::from_arc` default `application_mode` to
`true` (`src/live/screen.rs:49`) — that would change behavior for direct `console.print(&Screen)`
users (`examples/screen.rs`, `examples/fullscreen.rs`) and the `test_normal_mode_uses_lf`
contract. Keep any `application_mode` change local to `do_refresh`.

**Recommendation:** ship **Option A** (correct + reuses tested machinery + future-proof). Keep
Option B documented as the fallback if a regression in `update_screen_lines` line-splitting is
discovered during implementation.

---

## 5. Risk / blast radius

- **Scope:** one branch — `do_refresh`'s `is_screen` arm (`src/live/mod.rs:593-609`). The
  non-screen path, print/export, capture/record, SVG/HTML export, and direct `Screen` users are
  untouched.
- **Everything that renders through the screen path:** any `Live::with_screen(true)` user — e.g.
  the showcase finale (`examples/showcase.rs:1649-1835`), and DeepForge. After the fix these gain
  correct raw-mode rendering; cooked-mode rendering is unchanged (it was already correct).
- **The 2.0 `refresh_screen` in-place change** (`src/layout.rs:593-623`, `Layout::refresh_screen`
  → `update_screen_lines`) is **not on the live path** (no production callers — only tests). It is
  unaffected, but its `update_screen_lines` machinery is precisely what Option A reuses, so the
  fix is consistent with the 2.0 direction.
- **`update_screen_lines` no-op-outside-alt-screen guard** (`src/console.rs:1594`): on the live
  screen path the console *is* in alt-screen (entered in `Live::start`, `src/live/mod.rs:313`), so
  the guard passes. Confirm ordering: alt-screen must be entered before the first `do_refresh`
  (it is — `start()` sets alt-screen, then paints).
- **Synchronized-output wrapping** (DEC 2026) is preserved (kept around `update_screen_lines`).
- **Existing tests to re-run / watch:**
  - `src/live/screen.rs` unit tests (`test_normal_mode_uses_lf`, `test_application_mode_uses_cr`,
    `test_render_exact_dimensions`, …) — Option A does not touch `Screen`'s defaults, so these
    stay green.
  - `src/layout.rs` `refresh_screen_*` tests and `src/console_tests.rs`
    `test_update_screen_lines_*` — Option A leans on the same helper; keep green.
  - `tests/unit/console_tests.rs::test_alt_screen_enable_disable`.
  - `benches/live_threaded.rs` — sanity that the per-line emit doesn't regress frame cost
    (per-line `move_to` adds a few bytes/line; negligible, and synchronized-output already
    batches the frame).

---

## 6. Tests to add

The ASK explicitly wants a **screen-mode render test** (not just `export_text`).

1. **Screen-mode frame geometry (the headless equivalent of "look at the terminal").**
   Drive the real path: build a recording, alt-screen console
   (`force_terminal(true).record(true).width(W).height(H)` + `set_alt_screen(true)`), a split
   `Layout`, and a `Live::with_console(console).with_screen(true).with_auto_refresh(false)`; call
   `live.set(layout); live.refresh();`. Capture the emitted frame and **replay it through a tiny
   raw-mode grid model** (the ~60-line VT from this investigation: handle printable+autowrap,
   `CR`, `LF`-without-`CR`, and `ESC[row;colH`). Assert the resulting grid equals the print-path
   render of the same layout — i.e. regions land in the correct columns/rows and borders sit at
   region edges, **with `ONLCR` off**. This is the test that actually fails today and passes after
   the fix.

2. **No bare-LF positioning in the screen frame (cheap guard, no VT needed).**
   Capture the screen frame bytes and assert it does **not** depend on `ONLCR`: either every line
   is preceded by an absolute CUP (`ESC[…H`, Option A) — assert CUP count `== height` — or every
   `LF` is paired with a `CR` (Option B). Concretely for Option A: assert the frame contains no
   `"\n"` that is not part of an erase/positioning sequence, and that `frame.matches("\x1b[").count()`
   includes one CUP per row.

3. **Print-vs-screen geometry regression (guards §2a).**
   For the same split `Layout`, assert the per-line plaintext of `layout.gilt_console(...)` equals
   the per-line plaintext that the screen path feeds the terminal (`render_lines(&Screen::from_arc(layout), …)`).
   This locks in that the two paths never diverge in *geometry* again (it already holds; the test
   prevents a future region-vs-screen regression).

4. **Unit test for the emit helper** (if Option A): a focused test that `update_screen_lines(0, 0,
   &lines)` emits exactly one `move_to(0, i)` per line in order (extends existing
   `test_update_screen_lines_writes_each_line_at_successive_rows`).

Place 1–3 in `tests/` (cross-cutting live/screen behavior; e.g. `tests/live_screen_geometry.rs`)
and 4 inline in `src/console_tests.rs`.

---

## 7. Acceptance criteria

- A split `Layout` rendered via `Live::with_screen(true)` produces a frame that, replayed on a
  **raw-mode** (`ONLCR`-off) terminal grid, is identical to the print/export render of the same
  layout (regions column/row-aligned, borders at region edges, footer/lower regions present).
- The emitted screen frame no longer relies on tty `LF`→`CRLF`: every line is absolutely
  positioned (Option A) or every separator carries a `CR` (Option B).
- Print/export output for the same layout is byte-unchanged.
- All existing `live` / `screen` / `layout::refresh_screen` / `update_screen_lines` tests stay
  green; `just check` clean (fmt, clippy `-D warnings`, `--lib`, doctests, MSRV, wasm gate).
- New tests 1–3 (and 4 for Option A) added and passing.

## 8. Estimated effort

**Small — ~1–2 hours.** Option A is a ~10-line change in one branch plus ~3 tests. Most of the
time is the raw-mode grid test harness (reusable from this investigation's `vt` model). Option B
is a one-liner if a faster mitigation is needed first.

## 9. Relationship to task #24 (Tree-in-fixed-`Layout`)

**Distinct root cause; note but do not bundle.** Task #24 breaks in the **print/export** path too,
so its cause is in geometry / measurement (region sizing or `Tree` width measurement), evaluated
through `Layout::gilt_console` / `render` (`src/layout.rs:418-465`, `:631-672`). This bug is
**print/export-correct, screen-only**, and lives entirely in the alt-screen *emission* branch
(`src/live/mod.rs:593-609`). They touch the same `Layout`↔`Screen` rendering surface but fail in
different stages (geometry vs. emission). Fix them separately; a regression test from each side
(this plan's test 3 for geometry-parity, #24's own print-path assertion) will keep the shared
surface honest.

---

### Appendix — verified reference points

| What | Location |
|------|----------|
| Live screen emit branch (the bug) | `src/live/mod.rs:593-609` (home `:605`, `Screen::from_arc` `:607`, `print(&screen)` `:608`) |
| `Screen` separator choice (`\n` vs `\n\r`) | `src/live/screen.rs:97-101`; default `application_mode:false` `:40`,`:49` |
| Correct primitive — per-line absolute CUP | `Console::update_screen_lines` `src/console.rs:1593-1602` |
| Correct sibling — non-screen explicit positioning | `src/live/mod.rs:687-732` |
| `move_to` (0-indexed in → 1-indexed CUP) / `home` | `src/utils/control.rs:260-266`, `:188-190` |
| Geometry (identical both paths) | `Layout::gilt_console` `src/layout.rs:631-672`; `make_region_map` `:387-412` |
| `options()` (`max_width==size.width`, `height==None`) | `src/console.rs:1019-1036` |
| `set_alt_screen` (alt buffer only; no raw mode) | `src/console.rs:1316-1325` |
| alt-screen entered on `Live::start` | `src/live/mod.rs:313` |
