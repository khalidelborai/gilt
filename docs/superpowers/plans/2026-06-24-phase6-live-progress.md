# Phase 6 — Live/Progress Nesting + Screen Styling + Logging Layout (detailed plan)

> REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use `- [ ]`.

**Goal:** screen-mode Live preserves styling (#25); Live/Progress register with the console live-stack for correct nesting (#26, #27); log handlers use `Table::grid` for column alignment (#28).

**Base:** ed3916e. **Branch:** parity-2.0. **2.0.**

## Global Constraints
- MSRV 1.82; WASM-safe (Live/Progress core dep-free); clippy `--all-features -D warnings`; **`cargo fmt` + `--check` before each commit**; `cargo nextest run --lib` + `--all-features` + `cargo test --doc` + `cargo build --examples --all-features`.
- Preserve `live/mod.rs::do_refresh` emit discipline (DEC-2026 synchronized output + line-diff). This session already added `Live::pause()`/`resume()` — be consistent.
- Reference: `../research_doc/22-live.md`, `21-progress.md`, `25-logging.md`.

**Order:** 6.1 (screen styling) → 6.2 (live-stack wiring) → 6.3 (progress nesting; depends on 6.2) → 6.4 (logging Table::grid; HIGHEST RISK). 6.1/6.2/6.4 independent.

---

### Task 6.1: screen-mode `do_refresh` preserves styling (#25)
**Files:** `src/live/screen.rs` (Screen holds renderable), `src/live/mod.rs` (do_refresh is_screen branch ~597-609).
**Problem:** the screen path flattens styled segments to a plain `String` (`flat.push_str(&seg.text)`), stripping all SGR. **Fix:** `Screen` holds a `RenderableArc` (add `Screen::from_arc(RenderableArc)`; keep `Screen::new(impl Renderable…)` boxing to Arc); `Screen::gilt_console` renders via `self.renderable.as_ref()`. In `do_refresh` is_screen branch: `let screen = Screen::from_arc(content.clone()); s.console.print(&screen);` — drop the flatten loop, keep home()/begin_synchronized/end_synchronized.
- [ ] **RED:** screen-mode Live with `Style::parse("bold")` content, captured frame must contain an SGR escape (`\x1b[1m` / `\x1b[1;`), not just plain text. Run → fail (flattened).
- [ ] **IMPLEMENT:** Screen → RenderableArc + from_arc; simplify the is_screen branch.
- [ ] **GREEN:** suite + examples + doctests; clippy; fmt.
- [ ] **Commit:** `fix(live): screen-mode do_refresh preserves ANSI styling (#25)`
**Breaking:** `Screen.renderable` field type `Text` → `RenderableArc` (constructor `Screen::new(Text)` still works).

---

### Task 6.2: wire `Live::start`/`stop` to the console live-stack (#26)
**Files:** `src/live/mod.rs` (start ~285, stop ~358; pause/resume unchanged re: stack), `src/console.rs` (push_live/pop_live ~1353).
**Interface:** private `fn live_id(&self) -> usize { Arc::as_ptr(&self.state) as usize }`. `start`: after `show_cursor(false)`, `s.console.push_live(self.live_id())`. `stop`: before `show_cursor(true)`, `s.console.pop_live()`. **`pause` must NOT pop; `resume` must NOT push** (the Live stays "started"; only refresh halts).
- [ ] **RED:** after `start`, `console.live_depth() == 1`; after `stop`, `== 0`; a second test: `pause`/`resume` keep depth at 1 (start→1, pause→1, resume→1, stop→0). Run → fail (depth 0; push_live never called).
- [ ] **IMPLEMENT:** add `live_id()`; push in start, pop in stop; verify pause/resume leave the stack untouched.
- [ ] **GREEN:** all existing start/stop/pause/resume tests pass; clippy; fmt.
- [ ] **Commit:** `feat(live): wire Live start/stop to console live-stack (#26)`
**Breaking:** none (`live_id` private).

---

### Task 6.3: Progress live-stack registration (#27) — depends on 6.2
**Files:** `src/progress/core.rs` (start/stop ~697). Mostly FREE from 6.2 (Progress owns a private `Live`; `Progress::start` → `self.live.start()` → push_live).
- [ ] **RED:** after `progress.start()`, the internal live's `console.live_depth() == 1`; after `stop`, `== 0`. (Add a `#[cfg(test)]`/`pub(crate)` depth accessor on Progress since `live` is private.) Run → fail before 6.2.
- [ ] **IMPLEMENT:** confirm 6.2 merged; add the test accessor; verify `start`/`stop` thread through. Add an integration test: two Progress on separate consoles don't interfere (no panic).
- [ ] **GREEN:** suite; clippy; fmt.
- [ ] **Commit:** `feat(progress): live-stack registration via Live::start/stop (#27)`
**Breaking:** none.

---

### Task 6.4: logging uses `Table::grid()` for column alignment (#28) — HIGHEST RISK
**Files:** `src/error/logging_handler.rs` (`emit` ~267-310), `src/error/tracing_layer.rs` (`emit` ~228-293).
**Problem:** `emit` builds a flat `Text` with space separators → columns drift as the time string width varies. **Fix:** collect time/level/message/path as `Vec<Text>` cells; build `Table::grid(&headers)` with padding `(0,1,0,0)`; `add_row_text(&cells)`; `console.print(&grid)`. Mirror in `GiltLayer::emit` (time/level/message/fields/target). Right-justify the path column if the column API supports it (else leave left — alignment is the fix). Add `use crate::widgets::table::Table;`.
- [ ] **RED:** emit two records with different-length messages through a width-120 no_color record/capture console (use `with_omit_repeated_times(false)` so the time column is stable); split captured output on `\n`; assert the byte offset of `"INFO"` is identical on both lines (column aligned). Run → fail (flat concat drifts). (Add a `#[cfg(test)]` console accessor on the handler for capture.)
- [ ] **IMPLEMENT:** refactor both `emit`s to `Table::grid`; remove the space-separator appends.
- [ ] **GREEN:** all existing logging tests (level styles, time format, omit_repeated, link path) pass; clippy; fmt.
- [ ] **Commit:** `fix(logging): use Table::grid for column alignment in RichHandler and GiltLayer (#28)`
**Breaking:** none (`emit` private).

---

### Phase 6 gate
- [ ] full `cargo nextest run --lib` + `--all-features` + `cargo test --doc` + `cargo build --examples --all-features` green
- [ ] clippy `--all-features -D warnings` + fmt + `--no-default-features` + wasm32
- [ ] CHANGELOG `[Unreleased]`: Added (live-stack wiring, Screen::from_arc, progress nesting) / Changed-Breaking (Screen.renderable type) / Fixed (#25 #28).
