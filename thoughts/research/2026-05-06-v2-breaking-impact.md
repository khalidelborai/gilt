# v2.0 Breaking-Change Impact Analysis

Quantifies in-tree migration cost for the four v2.0-locked items in the v1.0 ergonomics overhaul ledger. All counts are from `rg` over `src/`, `examples/`, `tests/`, `benches/`.

## 1. Renderable streaming variant

Add `fn render_streaming(&self) -> impl Iterator<Item = Segment>` to `pub trait Renderable` at `src/console.rs:187`.

- **In-tree `impl Renderable for X` blocks: 45**
  - 41 in `src/`, 1 in `examples/testcard.rs` (`ColorBox`), 0 in `tests/`, 3 in doc comments / non-impl matches.
  - Concrete types: Text, str, String, Json, Panel, Markdown, Columns, Screen, Syntax, Gradient, Badge, Accordion, AccordionGroup, LiveRender, Figlet, Layout, Canvas, ProgressBar, Progress, Breadcrumbs, Diff, CsvTable, Toast, Sparkline, Rule, Spinner, Tree, Traceback, Table, Renderables, Lines, Bar, Inspect, StyledStr, Constrain, Emoji, RenderableBox, Group, Padding, Pretty, Styled, Scope, Align, ColorBox.
- **Migration regex:** none — adding a defaulted method (`fn render_streaming(&self) -> impl Iterator<Item = Segment> { self.gilt_console(...).into_iter() }`) is non-breaking *unless* the default is omitted. If trait gains a required method with no default, all 45 impls must be touched manually (no sed pattern; each impl needs a hand-written iterator path or `.into_iter()` shim).
- **Decision: stage to v2.1.** Shipping with a default impl in v2.0 (zero callsite churn) and tightening to required-method later avoids forcing a 45-impl edit during the v2.0 cut. If the streaming path needs to be the canonical one from day 1, a default that delegates to `gilt_console` keeps the change source-compatible.

## 2. Columns parallel-fields → enum

`src/columns.rs` currently exposes `pub renderables: Vec<String>` (line 24) and `pub widgets: Vec<Arc<dyn Renderable + Send + Sync>>` (line 27); render path at line 229 branches on `widgets.is_empty()`.

- **External callsite count: 0.** All references are internal to `src/columns.rs`:
  - 5 doc-comment / assert references in unit tests (`cols.renderables.is_empty()`, `cols.renderables.len()`, `cols.renderables[0]`) at lines 435, 449, 459, 460, 461.
  - 6 internal field uses inside the constructors and `gilt_console` at lines 24, 27, 61, 69–70, 84, 98, 229, 234, 239.
  - No `examples/`, `tests/`, or other `src/` file pokes these fields directly — all construction goes through `Columns::new`/`from_renderables`/`add`.
- **Migration regex:** internal-only. Replace `pub renderables: …` + `pub widgets: …` with `pub items: Vec<ColumnsItem>` and rewrite the 5 unit-test asserts. No external sed needed.
- **Decision: ship in v2.0.** Zero downstream callsite cost; the enum collapse is a pure internal refactor that only breaks the *public field signature*, not any caller. Cheapest item of the four.

## 3. `Console::capture_only()` lite constructor

`Console::from_builder` currently allocates: 4 env_var lookups (`detect_color_env`), `DEFAULT_STYLES.clone()` (153-entry HashMap), and `Arc::new(Mutex::new(StyleInterner::new()))` at `src/console.rs:413`.

- **Theme/interner reach in `console.rs`:** only 5 sites touch the two fields (`grep -n` lines 270, 296, 389, 406, 413, 548, 564–565, 570, 575). They cluster in:
  - **Render-mode only:** `get_style` (line 546, theme-stack lookup during markup parse), `style_interner()` accessor (564), `push_theme`/`pop_theme` (569/574).
  - **Capture-mode reachable:** none of `theme_stack` or `style_interner` is required by `begin_capture` / `end_capture` / `export_html` / `export_svg` / `export_text` themselves — those operate on `capture_buffer: Vec<Segment>`. However, **any `Renderable` whose `gilt_console` calls back into `console.get_style(...)` does need a theme stack**, and capture-mode renders go through the same `Renderable` trait. Renderables that parse markup (Text, Markdown, Panel, etc.) call `Style::parse_strict` directly without touching the theme; theme is only needed for *named* style lookups.
- **Genuinely 0-allocation?** No. Even a "lite" constructor must keep `capture_buffer: None`, the `ConsoleOptions` struct, and the dimensions probe. The savings are: skip 4 env_var syscalls, skip the 153-entry `HashMap::clone`, skip 1 `Arc<Mutex<_>>` allocation. Realistic claim: **"smaller and syscall-free", not zero-alloc.**
- **Migration regex:** N/A — additive constructor, existing `Console::default()` / `from_builder` callers untouched.
- **Decision: ship in v2.0** as additive (`Console::capture_only()`). Zero breakage. The "0-allocation" claim in the ledger should be softened to "no env probes, no theme clone" before release notes go out.

## 4. `Padding::wrap`-only + builder methods

`Padding::new(content, pad, style, expand)` at `src/utils/padding.rs:91` vs `Padding::wrap(content, pad)` at line 86 (defaults `Style::null()` + `expand: true`).

- **`Padding::new` callsites: 30** across 4 files.
  - `examples/padding_demo.rs`: 18
  - `src/utils/padding.rs`: 10 (1 in `Padding::indent`, 1 in `Padding::wrap` itself, 8 in unit tests)
  - `examples/padding.rs`: 1
  - `examples/showcase.rs`: 1
- **`Padding::wrap` callsites (existing): 0** outside the doc-test on line 84 — `wrap` is documented but unused in-tree.
- **Style/expand distribution of the 30 `new` sites:**
  - Non-default `style` (Style::parse) + `expand: true`: ~14 (need `Padding::wrap(c, pad).with_style(s)`).
  - Non-default `style` + `expand: false`: ~5 (need `.with_style(s).with_expand(false)`).
  - `Style::null()` + `expand: false`: ~2 (need `.with_expand(false)`).
  - `Style::null()` + `expand: true` (pure-wrap equivalent): ~1.
  - Internal (`indent`, `wrap`, tests): 9 — can stay on `pub(crate) fn new` if `new` is downgraded rather than removed.
- **Migration regex:** **manual.** Each call spans 4–7 lines with positional `Style::parse(...)` + `bool` arguments; no clean sed maps `Padding::new(c, p, Style::parse("on blue"), false)` → `Padding::wrap(c, p).with_style(Style::parse("on blue")).with_expand(false)` because the multi-line layout varies per site.
- **Decision: stage to v2.1.** 30 manual edits across 3 example files is the largest churn of the four. Recommended: in v2.0 keep `new` as `pub` but `#[deprecated]`, add `with_style` / `with_expand`, migrate examples in v2.0; flip `new` to `pub(crate)` only in v2.1 once external users have a release cycle to migrate.

## Summary

| Item | Callsites | Migration | v2.0 ship? |
|------|-----------|-----------|------------|
| 1. Renderable streaming | 45 impls | manual (none if defaulted) | v2.0 with default; v2.1 if required |
| 2. Columns enum | 0 external | internal sed | **v2.0** |
| 3. `capture_only()` | 0 (additive) | N/A | **v2.0** (soften "0-alloc" claim) |
| 4. Padding builder | 30 | manual | v2.0 deprecate, **v2.1 enforce** |

Recommendation: ship #2 and #3 in v2.0 (zero/low cost), defer #1 hard-break and #4 enforcement to v2.1 with deprecation warnings landing in v2.0.
