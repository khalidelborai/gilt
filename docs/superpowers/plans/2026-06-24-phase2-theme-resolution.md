# Phase 2 — Render-Time Theme Resolution (detailed plan)

> REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use `- [ ]`.

**Goal:** Theme-named markup/highlight styles (`[warning]`, `[repr.number]`, regex group styles) resolve against the active Console theme at RENDER time, instead of collapsing to `Style::null()` at parse time. Closes audit gaps #5, #6, #7, #37.

**Design decision:** Add `style_name: Option<String>` to `Span` (Option b — least churn; keeps `Span.style` for resolved styles, records the unresolved name for theme lookup). Resolution happens in a new `Text::render_themed(&Console)` (and a `DEFAULT_STYLES` fallback in standalone `render()`).

**Base commit:** 0e4d77d. **Branch:** parity-2.0. **2.0 target — breaking changes allowed.**

## Global Constraints
- MSRV 1.82; WASM-safe (no new deps); `Style::parse` stays lossy/infallible; clippy `--all-features -D warnings`; fmt clean; `cargo nextest run --lib` + `cargo test --doc`.
- Reference: `../research_doc/16-markup.md`, `04-text.md`, `20-highlighter-and-ansi.md`. In rich a `Span.style` is `str | Style`, resolved at render via `console.get_style`.
- `Console::get_style(name) -> Result<Style, _>` (`src/console.rs:959`) tries the ThemeStack then `parse_strict`; on unknown name it errs → render fallback is `Style::null()`.

**Linear order (respects deps):** 2.1 Span → 2.2 markup → 2.3 render_themed → 2.4 highlight_regex_with_groups → 2.5 RegexHighlighter named spans → 2.6 standalone render() fallback.

---

### Task 2.1: Add `style_name` to `Span` with named constructors

**Files:** Modify `src/text/span.rs` (field, constructors, accessors; propagate through `split`/`move_span`/`right_crop`/`extend`; update `PartialEq`/`Hash`). Test: inline.

**Interfaces produced:**
```rust
pub style_name: Option<String>,            // new field, init None in new()/with_meta()
pub fn named(start: usize, end: usize, name: impl Into<String>) -> Self  // style = Style::null(), style_name = Some(name)
pub fn named_with_meta(start, end, name, meta: Option<Arc<HashMap<String,String>>>) -> Self
pub fn style_name(&self) -> Option<&str>
pub fn is_named(&self) -> bool
```

- [ ] **Step 1 — failing tests** (inline `#[cfg(test)]`):
```rust
#[test]
fn named_span_carries_style_name() {
    let s = Span::named(0, 5, "warning");
    assert_eq!(s.style_name(), Some("warning"));
    assert!(s.is_named());
    assert!(s.style.is_null(), "named span has null resolved style initially");
}
#[test]
fn regular_span_has_no_style_name() {
    let s = Span::new(0, 5, Style::parse("bold"));
    assert_eq!(s.style_name(), None);
    assert!(!s.is_named());
}
#[test]
fn named_span_split_preserves_name() {
    let (left, right) = Span::named(0, 6, "repr.number").split(3);
    assert_eq!(left.style_name(), Some("repr.number"));
    assert_eq!(right.unwrap().style_name(), Some("repr.number"));
}
#[test]
fn named_span_move_preserves_name() {
    let moved = Span::named(0, 3, "repr.bool_true").move_span(10);
    assert_eq!(moved.style_name(), Some("repr.bool_true"));
    assert_eq!(moved.start, 10);
}
```
- [ ] **Step 2 — run, expect FAIL** (`cargo nextest run --lib text::span` / `span`): methods/field don't exist.
- [ ] **Step 3 — implement:** add `pub style_name: Option<String>`; init `None` in `new`/`with_meta`; add `named`/`named_with_meta` (`style: Style::null()`, `style_name: Some(name.into())`); add `style_name()`/`is_named()`; propagate `style_name` (clone) through `split`, `move_span`, `right_crop`, `extend`; include `style_name` in `PartialEq` and `Hash`.
- [ ] **Step 4 — run, expect PASS**; then `cargo nextest run --lib text::` for regressions.
- [ ] **Step 5 — commit:** `feat(span): add style_name field for deferred theme resolution (#5 #6)`

**Breaking:** new `Span` field → struct-literal construction breaks (migrate to `Span::new`/`Span::named`); `PartialEq` now includes `style_name`.

---

### Task 2.2: markup emits `Span::named` for unresolved theme tags (#5)

**Files:** Modify `src/markup.rs` — `resolve_tag_style` (line ~350) + the 5 `Span::new(start,end,tag_style)` call sites. Test: inline.

**Consumes:** `Span::named` (2.1). **Produces:** markup spans whose tag failed `parse_strict` now carry `style_name = Some(tag)` (not a null-style span).

- [ ] **Step 1 — failing tests:**
```rust
#[test]
fn theme_tag_span_carries_name() {
    let result = render("[warning]hello[/warning]", Style::null()).unwrap();
    assert_eq!(result.plain(), "hello");
    let span = &result.spans()[0];
    assert!(span.style.is_null());
    assert_eq!(span.style_name(), Some("warning"));
}
#[test]
fn literal_style_tag_has_no_style_name() {
    let result = render("[bold]hello[/bold]", Style::null()).unwrap();
    assert_eq!(result.spans()[0].style, Style::parse("bold"));
    assert_eq!(result.spans()[0].style_name(), None);
}
#[test]
fn repr_number_tag_carries_name() {
    let result = render("[repr.number]42[/repr.number]", Style::null()).unwrap();
    assert_eq!(result.spans()[0].style_name(), Some("repr.number"));
}
```
- [ ] **Step 2 — run, expect FAIL** (currently theme-name spans have `style_name == None`).
- [ ] **Step 3 — implement:** make `resolve_tag_style` distinguish literal-parse success from failure (e.g. return the resolved `Style` or the tag name). At each span construction: if `parse_strict` succeeded → `Span::new(start,end,style)`; else → `Span::named(start,end,tag_name)`.
- [ ] **Step 4 — run, expect PASS;** update existing `test_render_theme_name_fallback` to assert `style_name == Some("repr.number")` instead of null.
- [ ] **Step 5 — commit:** `feat(markup): preserve theme names as named spans instead of null styles (#5)`

---

### Task 2.3: `Text::render_themed(&Console)` resolves named spans at render time (#6) — HIGHEST RISK

**Files:** Modify `src/text/core.rs` (add `render_themed`); `src/console.rs` (`impl Renderable for Text { gilt_console }` at ~line 291 + wrapped-line path ~314 call `render_themed`, un-prefix `_console`). Test: inline / `console_tests.rs`.

**Consumes:** `Span::style_name` (2.1), `Console::get_style` (`console.rs:959`). **Produces:** `pub fn render_themed(&self, console: &Console) -> Vec<Segment>`.

**Lifetime note (critical):** `render()` builds `style_map: Vec<&Style>` referencing `self.spans`. For named spans, compute a `Vec<Style>` of resolved styles UPFRONT (one per span: `span.style.clone()` if not named, else `console.get_style(name).unwrap_or_else(|_| Style::null())`), then build `style_map` referencing THAT vec so the owned styles outlive the sweep-line loop.

- [ ] **Step 1 — failing tests** (adapt accessor names to the real API — `Theme::new`, `Console::builder().theme(..)`, `render_str`, `seg.style()`/`seg.text()` — verify and adjust):
```rust
#[test]
fn theme_name_span_resolved_at_render_time() {
    let mut styles = std::collections::HashMap::new();
    styles.insert("warning".to_string(), Style::parse("bold red"));
    let theme = crate::color::theme::Theme::new(Some(styles), true);
    let console = Console::builder().theme(theme).no_color(false).build();
    let text = console.render_str("[warning]x[/warning]", None, None, None);
    assert_eq!(text.spans()[0].style_name(), Some("warning"));
    let segments = text.render_themed(&console);
    let x_seg = segments.iter().find(|s| s.text().contains('x')).unwrap();
    let st = x_seg.style().unwrap();
    assert_eq!(st.bold(), Some(true));
    assert!(st.color().is_some_and(|c| c.name().contains("red")));
}
#[test]
fn render_themed_fallback_to_null_for_unknown_name() {
    let console = Console::builder().build();
    let mut text = Text::new("x", Style::null());
    text.spans_mut().push(Span::named(0, 1, "does.not.exist"));
    let segs = text.render_themed(&console);
    assert!(segs.iter().any(|s| s.text().contains('x'))); // no panic
}
```
- [ ] **Step 2 — run, expect FAIL** (`render_themed` missing).
- [ ] **Step 3 — implement** `render_themed` (mirror `render()` with upfront resolved-styles vec per the lifetime note); switch `Text`'s `gilt_console` to call it.
- [ ] **Step 4 — run, expect PASS; run full `cargo nextest run --lib`** and triage snapshot/capture tests whose markup theme tags now render styled instead of null — UPDATE tests that encoded the null bug (justify each); fix real regressions.
- [ ] **Step 5 — commit:** `feat(text): render_themed resolves named spans through Console theme (#6)`

---

### Task 2.4: `highlight_regex_with_groups` resolves names via DEFAULT_STYLES (#7)

**Files:** Modify `src/text/core.rs` `highlight_regex_with_groups` (~line 927-960). Test: inline.

- [ ] **Step 1 — failing test:**
```rust
#[test]
fn highlight_regex_with_groups_resolves_default_style_names() {
    let re = regex::Regex::new(r"(?P<number>\d+)").unwrap();
    let mut text = Text::new("count=42", Style::null());
    let count = text.highlight_regex_with_groups(&re, "repr.");
    assert_eq!(count, 1);
    // span covering "42" gets repr.number = bold not-italic cyan
    let plain = text.plain().to_string();
    let s = text.spans().iter().find(|sp| {
        let b = |n| plain.char_indices().nth(n).map(|(i,_)| i).unwrap_or(plain.len());
        &plain[b(sp.start)..b(sp.end)] == "42"
    }).unwrap();
    assert_eq!(s.style.bold(), Some(true));
    assert_eq!(s.style.italic(), Some(false));
    assert!(s.style.color().is_some_and(|c| c.name().contains("cyan")));
}
```
- [ ] **Step 2 — run, expect FAIL** (`parse_strict("repr.number")` fails → count 0).
- [ ] **Step 3 — implement:** at the group-style resolution, try `DEFAULT_STYLES.get(&style_str).cloned().or_else(|| Style::parse_strict(&style_str).ok())`. Import `crate::default_styles::DEFAULT_STYLES`.
- [ ] **Step 4 — run, expect PASS** + `cargo nextest run --lib text::`.
- [ ] **Step 5 — commit:** `fix(text): highlight_regex_with_groups resolves group names via DEFAULT_STYLES (#7)`

---

### Task 2.5: `RegexHighlighter` emits named spans for theme-overridable styles (#37)

**Files:** Modify `src/utils/highlighter.rs` `highlight_with_groups` (~line 37-72). Test: inline.

**Consumes:** `Span::named` (2.1), `render_themed` (2.3).

- [ ] **Step 1 — failing tests:**
```rust
#[test]
fn highlight_with_groups_produces_named_spans() {
    let re = regex::Regex::new(r"(?P<number>\d+)").unwrap();
    let mut text = Text::new("x=42", Style::null());
    let hl = RegexHighlighter { highlights: vec![re], base_style: "repr.".to_string() };
    hl.highlight(&mut text);
    assert!(text.spans().iter().any(|s| s.style_name() == Some("repr.number")));
}
#[test]
fn highlight_with_groups_themed_render_resolves_override() {
    let mut styles = std::collections::HashMap::new();
    styles.insert("repr.number".to_string(), Style::parse("italic yellow"));
    let theme = crate::color::theme::Theme::new(Some(styles), true);
    let console = Console::builder().theme(theme).no_color(false).build();
    let re = regex::Regex::new(r"(?P<number>\d+)").unwrap();
    let hl = RegexHighlighter { highlights: vec![re], base_style: "repr.".to_string() };
    let text = hl.apply("val=99");
    let seg = text.render_themed(&console).into_iter().find(|s| s.text().contains("99")).unwrap();
    let st = seg.style().unwrap();
    assert_eq!(st.italic(), Some(true));
    assert!(st.color().is_some_and(|c| c.name().contains("yellow")));
}
```
- [ ] **Step 2 — run, expect FAIL.**
- [ ] **Step 3 — implement:** in `highlight_with_groups`, when the style key exists in `DEFAULT_STYLES`, push the NAME (not the resolved style); apply via `Span::named(c_start, c_end, name)` into `text.spans_mut()`.
- [ ] **Step 4 — run, expect PASS;** verify `test_regex_highlighter_basic` still passes.
- [ ] **Step 5 — commit:** `feat(highlighter): emit named spans for theme-overridable group styles (#37)`

---

### Task 2.6: standalone `Text::render()` falls back to DEFAULT_STYLES for named spans

**Files:** Modify `src/text/core.rs` `render()`. Test: inline.

- [ ] **Step 1 — failing test:**
```rust
#[test]
fn standalone_render_resolves_named_span_against_default_styles() {
    let mut text = Text::new("42", Style::null());
    text.spans_mut().push(Span::named(0, 2, "repr.number"));
    let segs = text.render();
    let seg = segs.iter().find(|s| s.text().contains("42")).unwrap();
    let st = seg.style().unwrap();
    assert_eq!(st.bold(), Some(true));
    assert!(st.color().is_some_and(|c| c.name().contains("cyan")));
}
```
- [ ] **Step 2 — run, expect FAIL** (named span renders null without lookup).
- [ ] **Step 3 — implement:** in `render()`, build the per-span resolved-style vec; for `is_named()` spans use `DEFAULT_STYLES.get(name).cloned().unwrap_or_else(Style::null)`; non-named use `span.style.clone()`. (Same lifetime structure as 2.3.)
- [ ] **Step 4 — run, expect PASS;** full `cargo nextest run --lib`.
- [ ] **Step 5 — commit:** `feat(text): render() resolves named spans via DEFAULT_STYLES for no-console path`

---

### Task 2.7: sweep widget `gilt_console` to use `render_themed(console)`

**Files:** Modify `src/rule.rs` (title render ~228/255/289), `src/panel.rs` (`align_title_segments` ~294 — thread a `console` param; callers ~447/527 have it), `src/progress/core.rs` (~820), `src/status/spinner.rs` (~188), and `src/syntax.rs` (un-prefix `_console` at ~773 and thread to `render_syntax` ~685/707 where feasible). Tests: inline per widget.

**Why:** these `Renderable::gilt_console` impls receive `console: &Console` but call `text.render()` (discarding it), so theme-named spans (markup theme tags, RegexHighlighter group styles) render UNSTYLED there. Convert to `render_themed(console)`, threading the console parameter where a helper currently lacks it.

- [ ] **Step 1 — failing test (per widget that has a console):** build a Console whose theme maps a name (e.g. `repr.number` -> `italic yellow`); construct the widget with a `Text` carrying a `Span::named(.., "repr.number")` (or markup `[repr.number]`); render the widget through the console; assert the relevant segment carries the themed style (italic yellow), not null/default. Start with `Rule` (clearest: has a title Text + console). Example:
```rust
#[test]
fn rule_title_resolves_named_span_through_theme() {
    let mut styles = std::collections::HashMap::new();
    styles.insert("repr.number".to_string(), Style::parse("italic yellow"));
    let theme = crate::color::theme::Theme::new(Some(styles), true);
    let console = Console::builder().theme(theme).width(40).no_color(false).build();
    let mut title = Text::new("42", Style::null());
    title.spans_mut().push(crate::text::Span::named(0, 2, "repr.number"));
    let rule = Rule::new().with_title(title); // adapt to real Rule API
    let segs = console.render(&rule, None);
    assert!(segs.iter().any(|s| s.style().is_some_and(|st| st.italic()==Some(true))),
            "rule title named span must resolve to italic via theme");
}
```
- [ ] **Step 2 — run, expect FAIL** (sites use `.render()`, span renders null).
- [ ] **Step 3 — implement:** at each site, replace `text.render()` with `text.render_themed(console)`; for `align_title_segments` add a `console: &Console` parameter and pass it from the two call sites; for `syntax.rs` un-prefix `_console` and thread it into `render_syntax` (skip if a path genuinely has no console — note it).
- [ ] **Step 4 — run, expect PASS;** full `cargo nextest run --lib` + triage any snapshot changes (theme-named content now styled is the correct new behavior).
- [ ] **Step 5 — commit:** `fix(render): resolve named spans via render_themed in Rule/Panel-title/Progress/Spinner/Syntax (#6 cross-cutting)`

### Phase 2 gate
- [ ] full `cargo nextest run --lib` + `cargo test --doc` green
- [ ] clippy `--all-features -D warnings` + fmt clean
- [ ] `--no-default-features` + wasm32 build
- [ ] CHANGELOG `[Unreleased]` (Added: `Span::named`, `Text::render_themed`, theme-tag resolution; Changed-Breaking: Span field + eq; Fixed: #5 #6 #7 #37). Note gilt-tui impact (Span construction).
