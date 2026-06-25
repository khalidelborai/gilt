# Plan 7.19 — Containers/Wrappers P2/P3 batch (gilt parity 2.0)

**Repo:** gilt (single root crate `gilt`; Rust port of Python `rich`). Base branch: parity-2.0.
**Reference (absolute):** `/Users/khaklidelborai/Data2/Velocity/rusty_rich/research_doc/` (text/lines, rule, containers docs).
Files: `src/text/lines.rs`, `src/rule.rs`, `src/utils/containers.rs`. NO `unsafe`; no new deps; WASM-safe; do NOT push. Read each cited fn fully first.

## Tasks (TDD each: failing test → RED → minimal impl → GREEN → conventional commit per task)

### Task 1 (P2): `Lines::justify(Full)` missing `console` param for adjacent-word style — `src/text/lines.rs` (~justify fn, ~line 59)
rich's full-justify picks the space style from the adjacent word. Add a `console: &Console` param (or thread the console) so `Full` justification can resolve the inter-word space style from the adjacent word's style. Match rich's `Lines.justify` full-mode. Fix call sites (this is breaking — run `cargo build --all-targets --all-features` to find them).
- Test: full-justified line distributes spaces correctly (width matches target).
- Commit: `fix(text): Lines::justify(Full) uses console for adjacent-word space style (Phase 7)`

### Task 2 (P2): `Rule` title truncation off-by-1 for left/right — `src/rule.rs` (~245, ~277)
For left/right-aligned rules the title truncation subtracts the wrong margin. Match rich's `Rule.__rich_console__`: the title area for left/right alignment should reserve only the chars rich reserves (read the reference). Fix the width calc (likely `width.saturating_sub(2)` vs an over-subtraction).
- Test: a long title with left alignment is truncated to the correct width (one more char fits than before).
- Commit: `fix(rule): correct left/right title truncation off-by-one (Phase 7)`

### Task 3 (P3): `Rule` missing `gilt_measure` — `src/rule.rs`
Add `fn gilt_measure(&self, _: &Console, _: &ConsoleOptions) -> Measurement { Measurement::new(1, 1) }` (a rule's measurement is 1..1 — it expands to width). Check the existing `Measurement` API + how other widgets override gilt_measure (Phase 3 pattern).
- Test: `rule.gilt_measure(...)` returns minimum=1.
- Commit: `feat(rule): gilt_measure override (Phase 7)`

### Task 4 (P3): `Renderables::measure` not wired into `gilt_measure` — `src/utils/containers.rs` (~52)
`Renderables` has a `measure()` but no `gilt_measure` override delegating to it. Add `fn gilt_measure(&self, console, options) -> Measurement` delegating to the existing measure. (Check if Phase 3 already did this — if so, note SKIP.)
- Test: `renderables.gilt_measure(...)` matches `renderables.measure(...)`.
- Commit: `feat(containers): Renderables gilt_measure delegation (Phase 7)`

## Final gates (ALL clean): `cargo nextest run --all-features` · `cargo test --doc` · `cargo clippy --all-features --all-targets -- -D warnings` · `cargo fmt --check` · `cargo build --all-targets --all-features`
Use your advisor tool before each commit.
