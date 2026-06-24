# Phase 4 — Container Generalization (BREAKING) (detailed plan)

> REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use `- [ ]`.

**Goal:** container/layout widgets hold ANY `Renderable`, not just `Text` — enabling `Panel::new(Table)`, mixed `Group`, `Tree` with widget labels, nested-widget table cells. Closes audit gaps #13, #15, #16, #19, #21.

**Base commit:** f4787b1. **Branch:** parity-2.0. **2.0 — BREAKING public API.**

## Design decisions (locked)
- **Trait-object type:** `pub type RenderableArc = Arc<dyn Renderable + Send + Sync>` — matches `Layout`/`Live`/`CellContent`. `Arc<dyn T>` is `Clone` (refcount) → containers keep `#[derive(Clone)]`.
- **Debug:** the `Renderable` trait has NO `Debug` supertrait (adding one = breaking trait change, rejected). Use the established `Layout`/`CellContent` pattern: drop `#[derive(Debug)]`, add a manual `impl Debug` printing `"<renderable>"` for the trait-object field. Blast radius: 0 built-ins.
- **PartialEq/Hash:** none of the 7 container structs derive these → no trait-object issue.
- **content_hash:** preserved through `Arc::as_ref()` dispatch.
- **Ergonomics:** constructors become `impl Renderable + Send + Sync + 'static` (wrap in `Arc::new` internally). Existing `Panel::new(Text::new(..))` / `Panel::new("literal")` keep compiling. **Breaking:** direct `pub content: Text` field reads (e.g. `panel.content.plain()`) break — documented per task.
- **gilt-tui:** verified it depends on published `gilt = "0.7"` and does NOT use these gilt types (its "Panel"/"Align" are DOM strings). Task 4.7 = verify-only.

## Global Constraints
- MSRV 1.82; WASM-safe (`Arc` is std, `Send+Sync` doesn't regress wasm); clippy `--all-features -D warnings`; fmt (run `cargo fmt` + `cargo fmt --check` before each commit); `cargo nextest run --lib` + `--all-features` + `cargo test --doc`.
- Phase 3's `gilt_measure` is landed — nested widgets measure correctly. Containers measure children via `child.gilt_measure(console, options)` (NOT `Text`-specific `.measure()`/`.cell_len()`).
- Reference: `../research_doc/12-panel-and-box.md`, `13-tree.md`, `14-layout.md`, `15-containers-and-wrappers.md`.

**Order:** 4.1 foundation → 4.2 Panel (HIGHEST RISK) → 4.3 Tree → 4.4 Group/Renderables → 4.5 Align/Padding/Constrain/Styled → 4.6 Table cells (#19) → 4.7 gilt-tui verify → 4.8 gate.

---

### Task 4.1: foundation — `RenderableArc` alias + helper
**Files:** `src/console.rs` (after Renderable trait), `src/lib.rs` (re-export). Additive, non-breaking.
**Produces:** `pub type RenderableArc = Arc<dyn Renderable + Send + Sync>;` and `pub fn into_renderable_arc<R: Renderable + Send + Sync + 'static>(r: R) -> RenderableArc { Arc::new(r) }`. Re-export both from `lib.rs`.
- [ ] **RED:** test `RenderableArc` is Clone-able and `into_renderable_arc(Text::new(..))` renders. Run → compile error (alias missing).
- [ ] **IMPLEMENT:** add the alias + helper; re-export. (Check `Arc` is imported in console.rs.)
- [ ] **GREEN:** tests pass; full suite.
- [ ] **Commit:** `feat(core): add RenderableArc type alias + into_renderable_arc helper`

---

### Task 4.2: Panel — `content: RenderableArc` (HIGHEST RISK)
**Files:** `src/panel.rs`.
**Interfaces:** `pub content: RenderableArc`; `pub fn new(content: impl Renderable + Send + Sync + 'static) -> Self`; same for `fit`; `from_renderable<R: ...>(r: R) -> Self { Panel::new(r) }` (no pre-render); manual `impl Debug`.
- [ ] **RED:** `Panel::new(Table::new(&["Name","Score"]))` (with a row) renders the table content + headers inside the panel (assert "Alice"/"Score" present); a `Panel::new(inner_panel)` renders the inner; existing `Panel::new(Text::new(..))` still compiles+renders. Run → compile error (`new` takes Text).
- [ ] **IMPLEMENT:** field → `RenderableArc`; generic constructors (`Arc::new` inside); drop `derive(Debug)`, add manual Debug (`"<renderable>"`); in `measure()` replace `self.content.measure().maximum` → `self.content.gilt_measure(console, options).maximum`; in `gilt_console()` render child via `console.render_lines(self.content.as_ref(), Some(&child_opts), None, false, false)` at `inner_width`, keep the existing padding/height/border assembly. `from_renderable` → thin wrapper.
- [ ] **GREEN:** all 35+ existing panel tests + the 3 new ones; watch border-geometry/line-count regressions (triage: a changed assertion that encoded Text-only behavior may be legit, but a border off-by-one is a real bug — fix code). `cargo fmt`+check; clippy.
- [ ] **Commit:** `feat(panel): generalize content to RenderableArc; Panel::new accepts any Renderable`
**Breaking:** `pub content: Text` → `RenderableArc`; field reads break (migrate to `content.as_ref()` via Renderable, or downcast).

---

### Task 4.3: Tree — `label: RenderableArc`
**Files:** `src/tree.rs`.
**Interfaces:** `pub label: RenderableArc`; `new`/`add` take `impl Renderable + Send + Sync + 'static`; manual Debug.
- [ ] **RED:** `Tree::new(Panel::new(Text::new("root panel",..)))` renders "root panel"; `tree.add(Rule::new("divider"))` works. Run → compile error.
- [ ] **IMPLEMENT:** field → `RenderableArc`; generic constructors; manual Debug; render label via `console.render_lines(node.label.as_ref(), ...)`; thread `&Console,&ConsoleOptions` into `measure_recursive` and measure labels via `label.gilt_measure(console, options)`.
- [ ] **GREEN:** existing tree tests + new; fmt+clippy.
- [ ] **Commit:** `feat(tree): generalize label to RenderableArc; Tree::new/add accept any Renderable`
**Breaking:** `pub label: Text` → `RenderableArc`; `add()` signature.

---

### Task 4.4: Group + Renderables — `Vec<RenderableArc>`
**Files:** `src/utils/group.rs`, `src/utils/containers.rs`.
**Interfaces:** `Group { items: Vec<RenderableArc> }`; `new`/`fit(Vec<RenderableArc>)`; add `push(&mut self, item: impl Renderable + Send + Sync + 'static)`; `items() -> &[RenderableArc]`. `Renderables { items: Vec<RenderableArc> }`; `new(Vec<RenderableArc>)`; `append(impl Renderable + Send + Sync + 'static)`. Manual Debug both.
- [ ] **RED:** `Group::new(vec![Arc::new(Text), Arc::new(Rule), Arc::new(Panel)])` renders all; `Renderables` with an appended `Table` renders. Run → compile error.
- [ ] **IMPLEMENT:** fields → `Vec<RenderableArc>`; generic `push`/`append`; render items via `item.as_ref().gilt_console(...)`; measure via `item.gilt_measure(console, options)`; manual Debug.
- [ ] **GREEN:** existing + new; fmt+clippy.
- [ ] **Commit:** `feat(group,containers): generalize Group and Renderables to Vec<RenderableArc>`
**Breaking:** `new` takes `Vec<RenderableArc>` (migrate: wrap items in `Arc::new`); `items()` return type.

---

### Task 4.5: Align, Padding, Constrain, Styled — `RenderableArc`
**Files:** `src/utils/align_widget.rs`, `padding.rs`, `constrain.rs`, `styled.rs`.
**Interfaces:** each `content`/`renderable` field → `RenderableArc`; all constructors generic `impl Renderable + Send + Sync + 'static`; keep `derive(Clone)`, drop `derive(Debug)` + manual Debug.
- [ ] **RED:** `Align::center(Table::new(&["A"]))`, `Padding::wrap(Panel::new(Text), 1)` render the nested widget. Run → compile error.
- [ ] **IMPLEMENT:** fields → `RenderableArc`; generic constructors; manual Debug; render via `self.content.as_ref()` / `render_lines(self.content.as_ref(), ..)`; replace `Text`-specific measure (`cell_len()`/`.measure()`) with `self.content.gilt_measure(console, options)`. (Align already fixed in 3.4 — keep that logic, just change the source to `gilt_measure().maximum`.)
- [ ] **GREEN:** existing + new; fmt+clippy.
- [ ] **Commit:** `feat(layout): generalize Align, Padding, Constrain, Styled to RenderableArc`
**Breaking:** field types; construction via `Text::new`/`"literal"` still compiles.

---

### Task 4.6: Table cells — render `CellContent::Renderable` at column width (#19)
**Files:** `src/widgets/table/row.rs`, `render.rs`, `core.rs`.
**Problem:** `CellContent::Renderable` is pre-rendered at default width then re-wrapped, destroying nested-widget geometry. Fix: defer; render the cell renderable at the resolved COLUMN width.
- [ ] **RED:** a table cell holding `Panel::new(Text).with_width(20)` — assert panel border chars (`╭`/`+`) survive in the rendered table (not just flattened "inner"). Run → fails (border destroyed).
- [ ] **IMPLEMENT:** change `CellInfo::renderable: Text` → `RenderableArc` (it's `pub(crate)` — no public break); for the `Renderable` variant keep the `Arc` (no pre-render); in the render loop render with `console.render_lines(cell.renderable.as_ref(), Some(&col_opts), ..)` where `col_opts = options.update_width(column_width)`; `Plain`/`Styled` variants → `Arc::new(text)`.
- [ ] **GREEN:** existing table tests + new nested-panel test; fmt+clippy.
- [ ] **Commit:** `fix(table): render CellContent::Renderable at column width, not default width`
**Breaking:** none public (`CellInfo` is pub(crate); `CellContent::Renderable` already holds `Arc<dyn Renderable + Send + Sync>`).

---

### Task 4.7: gilt-tui verify
**Files:** `/Users/khaklidelborai/Data2/Velocity/rusty_rich/gilt-tui/`.
Investigation found gilt-tui pins published `gilt = "0.7"` and does NOT use gilt's container types (DOM strings only). This task VERIFIES, not rewrites.
- [ ] Point gilt-tui's `Cargo.toml` `gilt` dep at the local path (`path = "../../gilt/live-pause-resume"`) temporarily, run `cargo check -p gilt-tui`.
- [ ] If clean → no code changes needed; revert the Cargo.toml path change (or leave per user pref) and note the verdict. If errors → list + patch each call site (wrap in `Arc::new`). 
- [ ] **Commit (in gilt-tui if changes / else just report):** `chore(gilt-tui): verify against parity-2.0 container API (no breaking call sites)`

---

### Task 4.8: Phase 4 gate — composition integration tests
**Files:** `tests/` (new `container_composition.rs`), `src/lib.rs` doc example.
- [ ] Integration tests: Panel-of-Table renders headers+rows+title; Group of mixed widgets renders all; Tree with Panel label; `Live::new(Panel::new(Table))` constructs. (RED before the per-widget tasks; GREEN after.)
- [ ] full `cargo nextest run --lib` + `--all-features` + `cargo test --doc` green; clippy `--all-features -D warnings`; fmt; `--no-default-features` + wasm32 build.
- [ ] Add a `Panel::new(Table::new(..))` crate-doc example.
- [ ] CHANGELOG `[Unreleased]`: Added (`RenderableArc`); Changed-Breaking (Panel/Tree/Group/Renderables/Align/Padding/Constrain/Styled content fields → `RenderableArc`, generic constructors, manual Debug; field reads break); Fixed (#13 #15 #16 #19 #21).
- [ ] **Commit:** `test(phase4): cross-widget composition integration tests + gate`

## Highest-risk task
**4.2 (Panel)** — `measure()` and `gilt_console()` path changes (Text-specific → trait-object via `render_lines`/`gilt_measure`), 35+ pixel-precise tests, border-geometry off-by-one risk. Land it carefully; it sets the `render_lines(content.as_ref(), ..)` pattern the other containers copy.
