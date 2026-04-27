# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.11.1] - 2026-04-27

Cleanup release. Removes long-deprecated items missed by the v0.11.0
break window plus the only `pub(crate)` dead method.

### Removed

- **`Style::copy()`** — deprecated alias for `Style::clone()`. Use
  `style.clone()` directly. (Was tagged `#[deprecated]`.)
- **`RenderableBox::downcast_ref()`** — deprecated since v0.10.0. Always
  returned `None` because `RenderableBox` doesn't store a `TypeId`. If
  you need downcasting, store the concrete type instead of erasing it.
- **`Tree::style()`, `Tree::guide_style()`, `Tree::expanded()`,
  `Tree::hide_root()`** — deprecated since 0.2.0 (~6 years). Use the
  `with_*` builder forms (`Tree::with_style()`, etc.).

### Internal

- Removed the unused `Table::render_table` wrapper (`pub(crate)`, no
  callers since v0.10.x T9 added `render_table_with_cells`).
- Audited remaining `#[allow(dead_code)]` suppressions: 3 in `Console`
  (`tab_size`, `soft_wrap`, `safe_box`) are kept — they have public
  builder setters and the comments accurately describe future use.
- Cleaned up workflow scratch from disk (44 MB freed).

### Migration from 0.11.0

```rust
// Before:                       // After:
style.copy()                     style.clone()
tree.style(s)                    tree.with_style(s)
tree.guide_style(s)              tree.with_guide_style(s)
tree.expanded(true)              tree.with_expanded(true)
tree.hide_root(true)             tree.with_hide_root(true)
```

`RenderableBox::downcast_ref` had no working alternative — the method
always returned `None` even before removal. If you somehow depended on
the no-op behavior, replace `box.downcast_ref::<T>()` with `None::<&T>`.

## [0.11.0] - 2026-04-26

Stable v0.11.0 release. Aggregates the alpha series:
**alpha.1** dormant `StyleInterner` types, **alpha.2** `Segment.style`
field → method conversion, **alpha.3** L1 Color enum collapse.

### Highlights

- **Color is now a 5-variant enum** (~40 B + heap → ~4 B inline,
  `Copy`). See alpha.3 entry for migration details.
- **`Segment.style` is a method, not a field.** See alpha.2 entry.
- **`StyleInterner` + `StyleId` types added** as foundation for future
  perf work. Currently dormant — no callers route through them yet.

### Performance

- `console_render/table_100_rows`: 612 µs → 600 µs (-2%, primarily
  from Color enum's smaller footprint reducing cache pressure).
- `Color::clone` is now a 4-byte memcpy (was: alloc + copy of `name`
  String + ~30 B struct).
- `Style::clone` is much cheaper since `Option<Color>` is `Copy`. The
  only remaining heap allocation is `link: Option<String>` (rarely
  set).

### L2 interner activation: deferred

The original v0.11.0 design (`.review/V0_11_DESIGN.md`) included a PR3
that would swap `Segment.style` storage to `StyleId` and route every
construction through the global interner. **This was attempted, then
reverted based on benchmark evidence:**

| Stage                          | table_100_rows |
|--------------------------------|----------------|
| Pre-L1 baseline (post-PR #5)   | 612 µs         |
| Post-L1 (this release)         | 600 µs         |
| Post-L2 activation (single-lock optimized) | 884 µs (**+47%**) |

The `Mutex<HashMap>::lock()` per `Segment::new` plus `Arc<Style>`
allocation cost more than the savings from cheaper clones. After L1,
Style is small enough that interning loses its leverage.

The `StyleInterner` skeleton stays in the codebase as deferred
research. Reactivation would need a different mechanism (DashMap,
thread-local cache + merge, or fundamentally lock-free).

The `StyleInterner` and `StyleId` types remain in
`gilt::style_interner` as dormant scaffolding for potential future
work (e.g., a lock-free DashMap-backed redesign).

### Migration from 0.10.x

See README's "Migrating from 0.10.x" section. TL;DR:

```rust
// Color: struct → enum
Color { name: "red".into(), color_type: ColorType::Standard,
        number: Some(1), triplet: None }
// becomes:
Color::Standard(1)  // or Color::parse("red")?

// Color field access → methods
color.name        → color.name()        // Cow<'_, str>
color.color_type  → color.kind()        // ColorType
color.number      → color.number()      // Option<u8>
color.triplet     → color.triplet()     // Option<ColorTriplet>

// Segment.style field is now a method (alpha.2)
seg.style.is_some()  → seg.style().is_some()
seg.style.clone()    → seg.style().cloned()
if let Some(ref s) = seg.style → if let Some(s) = seg.style()
```

### Behavior changes

- **EightBit colors lose named round-trip**: `Color::parse("yellow4")?
  .name()` now returns `"color(106)"` instead of `"yellow4"`. Only the
  16 standard ANSI colors get canonical names. `Color::parse("color(106)")`
  still works.
- **`Segment::style_owned()` collapses None and Some(null)**: both map
  to `Style::null()`. Use `style()` directly to distinguish.

## [0.11.0-alpha.3] - 2026-04-26

L1 Color enum (PR2 of the v0.11.0 break bundle). The largest single
break of the bundle. Color shrinks from ~40 B + heap String to **~4 B
inline** and becomes `Copy`.

### Changed (breaking)

- **`Color` is now a 5-variant enum** with one variant per color
  classification:
  ```rust
  pub enum Color {
      Default,
      Standard(u8),       // ANSI 0..16
      EightBit(u8),       // 16..=255
      TrueColor(ColorTriplet),
      Windows(u8),        // legacy Windows console palette
  }
  ```
- **`Color: Copy`** — pass and store by value; no more `.clone()` on
  every Color access.
- **`Color.name: String` field → `Color::name() -> Cow<'_, str>`
  method.** Borrowed for the 16 named ANSI colors and `"default"`;
  owned `format!("color({n})")` for EightBit/Windows numbered, owned
  hex for TrueColor.
- **`Color.color_type: ColorType` field → `Color::kind() -> ColorType`
  method.** `ColorType` enum is preserved for backward compatibility.
- **`Color.number: Option<u8>` field → `Color::number() -> Option<u8>`
  method.**
- **`Color.triplet: Option<ColorTriplet>` field → `Color::triplet() ->
  Option<ColorTriplet>` method.**

### Migration

```rust
// Before:
Color { name: "red".into(), color_type: ColorType::Standard,
        number: Some(1), triplet: None }
// After:
Color::Standard(1)
// or
Color::parse("red")?

// Before:                       // After:
color.name                       color.name()             // -> Cow<'_, str>
color.color_type                 color.kind()             // -> ColorType
color.number                     color.number()           // -> Option<u8>
color.triplet                    color.triplet()          // -> Option<ColorTriplet>
match color.color_type { ... }   match color.kind() { ... }
```

### Deliberate behavior change

EightBit colors no longer round-trip their named form through Display.
`Color::parse("yellow4")?.name()` was `"yellow4"`, now returns
`"color(106)"`. Only the 16 standard ANSI colors get canonical names
from the inverse table. `Color::parse("color(106)")` still works in
both directions. A future PR can build a comprehensive inverse map for
named EightBit colors if needed.

### Why keep Windows?

Per AskUserQuestion, kept the `Windows` variant rather than folding
into `Standard`. Windows colors have their own resolution palette in
`get_truecolor`. Folding would lose `system() == ColorSystem::Windows`
distinction. The 5-variant enum is still ~4-5 B vs the previous ~40 B
+ heap.

### Memory impact

For a 256-row table with 5 styled columns (~1280 Color values per
render): ~45 KB saved before counting heap deallocation of `name`
strings. The interner activation in PR3 will compound this further.

### Notes

- Kept `Style.color/bgcolor/underline_color: Option<Color>` for now;
  PR3 (interner activation) handles the next-level shrink.
- 27 internal call sites + 4 external (anstyle_adapter, syntax bridge,
  gradient, diagnose) migrated. ~30 test fixtures updated.

## [0.11.0-alpha.2] - 2026-04-26

PR1b of the v0.11.0 break bundle. **API change** to `Segment` so PR3 can
swap storage to `StyleId` without further churn at call sites.

### Changed (breaking)

- **`Segment.style: Option<Style>` field is now `pub(crate)`.** Access
  via the new methods below.
  - `Segment::style(&self) -> Option<&Style>` — borrowed accessor.
  - `Segment::style_mut(&mut self) -> &mut Option<Style>` — for the rare
    in-tree call sites that mutate after construction.
  - `Segment::set_style(&mut self, style: Option<Style>)` — replaces
    `seg.style = Some(...)`.
  - `Segment::style_owned(&self) -> Style` — returns owned `Style`,
    substituting `Style::null()` for the `None` case so callers that
    previously did `seg.style.clone().unwrap_or_else(Style::null)`
    collapse to one call.

### Migration

Most call sites collapse mechanically:

```rust
// Before:
seg.style.is_some()           seg.style.clone()
seg.style.as_ref()            if let Some(ref style) = seg.style { … }

// After:
seg.style().is_some()         seg.style().cloned()
seg.style()                   if let Some(style) = seg.style() { … }
```

For struct-literal construction (`Segment { style: Some(s), … }`), use
`Segment::new(text, Some(s), control)` or `Segment::styled(text, s)`.

### Deliberate semantics: `style_owned()` collapses None and Some(null)

The L2 interner (PR3) will only have `StyleId::NULL` for both. Tests
that need to distinguish should use `style()` directly, which returns
`Option<&Style>` faithfully.

### Notes

- Internal storage stays `Option<Style>` in alpha.2 — no perf change.
  PR3 will swap storage to `StyleId`, at which point the method
  signatures stay the same and call sites don't move again.
- 23 internal sites + 4 external (2 examples, 1 proptest) migrated.

## [0.11.0-alpha.1] - 2026-04-26

First slice of the v0.11.0 break bundle. **Pre-release:** API surface is
stable but `Segment.style` will change in `0.11.0-alpha.2` (PR1b).
Don't depend on alpha.1 unless you're tracking the bundle progress.

### Added

- **`gilt::style_interner`** module with [`StyleInterner`] and
  [`StyleId`]. Content-deduplicating interner for `Style` values backed
  by `HashMap<Arc<Style>, StyleId>` (works because `Arc<T>: Hash + Eq`
  delegates to `T`). `StyleId(u32)` is `Copy`; `StyleId::NULL` is
  pre-seeded with `Style::null()`. Foundation for L2 — see
  `.review/V0_11_DESIGN.md`.
- `Console::style_interner() -> &Arc<Mutex<StyleInterner>>` accessor.
- 6 unit tests covering null pre-seed, dedup, distinct ids, null
  intern, unknown lookup, id stability.

### Changed

- `Console` now holds a `style_interner: Arc<Mutex<StyleInterner>>`
  field. Construction allocates the interner and pre-seeds `NULL`. **No
  caller routes through the interner yet** — it is dormant. Per-Console
  cost: one `Arc<Mutex<…>>` allocation + a `HashMap` with one entry.

### Notes

The skeleton stays dormant on purpose. PR1b (`v0.11.0-alpha.2`) will
convert `Segment.style: Option<Style>` (a public field accessed at 246
sites) into a method that resolves through the interner. PR3 activates
interning at `Segment` construction sites and lands the actual perf
win. This split was chosen via AskUserQuestion to derisk the 246-site
mass rewrite — it stays in its own PR rather than mixing with the type
introduction.

## [0.10.3] - 2026-04-26

T8 lock-free `Live` writers. Realistic Progress workload throughput
improves by ~21,000× under writer + renderer contention.

### Performance

- **`Live::update_renderable` is now lock-free.** Pulled the hot
  `renderable: Text` field out of `Mutex<SharedState>` into
  `Arc<ArcSwap<Text>>`. Writers do an atomic pointer swap; the renderer
  loads atomically. Writers no longer queue behind the renderer's mutex
  hold during paint.

### Bench delta (apples-to-apples on `live_threaded` from v0.10.2)

| Bench                    | OLD        | NEW (this) |
|--------------------------|------------|------------|
| update_only_small/1      | 6.08 M/s   | 3.73 M/s   |
| update_only_small/8      | 1.96 M/s   | 1.33 M/s   |
| update_only_large/1      | 2.77 M/s   | 0.88 M/s   |
| **update_plus_render/1** | **43 op/s**| **904 K/s**|
| **update_plus_render/2** | 32 op/s    | 1.01 M/s   |
| **update_plus_render/4** | 49 op/s    | 1.07 M/s   |
| **update_plus_render/8** | ~700 op/s  | 1.31 M/s   |

The realistic workload — writer + renderer thread (the default
`auto_refresh: true` pattern used by every `Progress` instance) — speeds
up by **3-4 orders of magnitude**.

The single-writer regression on `update_only_*` (~3× on large payloads)
is the cost of the per-store `Arc::new(Text)` allocation. The previous
mutex-only path moved the `Text` into existing storage with zero alloc.
A candidate-3 design (mutex-only with tightened critical section) was
benchmarked and beats the v0.10.2 baseline on `update_only` but
catastrophically loses (200 op/s vs 1.31 M/s) on `update_plus_render`,
because writers still queue behind the renderer's lock hold.

Real `Live` instances always have a renderer, so the trade is correct.
The `update_only` numbers represent only `with_auto_refresh(false) +
manual refresh` — a niche pattern.

### Behavior change

- `Live::update_renderable(_, false)` no longer synchronously updates
  the internal `LiveRender`. The `LiveRender`'s stored renderable is
  refreshed on the next `refresh()` call (which is when it actually
  matters — for shape tracking and the next paint). Calling
  `live.live_render()` between an update and a refresh may show stale
  data; previously it showed the just-updated value. No production code
  depended on this; only one in-tree test, updated to match the new
  contract.

### Added

- `arc-swap = "1"` dependency.
- Bench groups `live_threaded/update_only_{small,large}` and
  `live_threaded/update_plus_render` with parameterised payload size, so
  the realistic Progress workload (writer + renderer + ~2 KB Text) is
  measurable.

## [0.10.2] - 2026-04-26

T8 prep release: enable shared-`Live` use across threads and ship the
threaded contention bench that justifies the v0.10.6 lock-free split.

### Changed

- **`Live::update_renderable` and `Live::update` now take `&self`** instead
  of `&mut self`. The body only touches the internal
  `Arc<Mutex<SharedState>>`; the previous `&mut` was an unnecessary API
  restriction that prevented sharing a `Live` across threads. Strict
  loosening — all existing callers continue to work; the only side effect
  is that some `let mut live` bindings now warn `unused_mut`.

### Added

- **`benches/live_threaded.rs`**: criterion bench measuring
  `update_renderable` throughput under N=1/2/4/8 writer threads (and a
  variant with a renderer thread also calling `refresh()`). Confirms
  negative scaling on writers due to single-mutex contention — at 8
  writers, throughput is ~40% of single-writer (5% efficiency vs linear
  ideal). Justifies the v0.10.6 lock-free `Live::SharedState` split.

### Notes

- **Q9 (table divider hoist) verified already optimal.** Audit found the
  divider-construction block at `widgets/table/core.rs:1301` is already
  hoisted outside the line/cell loops; the remaining `divider.clone()`
  per cell-per-line is unavoidable `Vec<Segment>` push semantics. No
  code change needed.

## [0.10.1] - 2026-04-26

Performance patch. No public API changes.

### Performance

- **T11 — `Text::char_len` cache**: `Text::len()` is now O(1) on the
  second-and-later call. Backed by an `AtomicUsize` (8 B, `Sync`-preserving)
  with `usize::MAX` as the uninitialized sentinel. Mutators that already
  know the new length (`set_plain`, `append_str`, `append_text`, `pad_left`,
  `pad_right`, `right_crop`, `extend_style`) re-prime the cache directly,
  avoiding the recompute. Cached `len()` measured at ~1 ns on a 10 KB
  Unicode-heavy `Text`. First step of the v0.11.0 perf-track de-risking.

## [0.10.0] - 2026-04-26

A rich-v15.0.0 sync release. Ports every behavioural fix from rich
v14.0.0–v15.0.0, adds significant new APIs surfaced by the deep-dive
review, and fixes three user-reported runtime bugs.

### Added

- **Rich v14.0.0 — TTY env-var overrides**: `TTY_COMPATIBLE=0/1` forces
  TTY mode independent of platform detection; `TTY_INTERACTIVE=0/1`
  forces interactive mode independent of TTY status. Surfaced via
  `Console::is_terminal()` (now consults `TTY_COMPATIBLE`) and the new
  `Console::is_interactive()` method. New `TtyOverride` enum and two
  `detect_tty_*` functions in `color::color_env`.
- **Rich v14.1.0 — `Syntax.padding` four-tuple**: changed from
  `(top, bottom)` to `(top, right, bottom, left)` matching the CSS
  shorthand. New `PaddingSpec` enum + `unpack_padding()` helper for
  ergonomic construction from any of the four shorthand forms.
- **Rich v15.0.0 — `Text::from_ansi` newline preservation**: input's
  trailing newlines are no longer stripped (`from_ansi("Hello\n")
  .plain() == "Hello\n"`).
- **`Progress::open_file(path, description)` /
  `Progress::wrap_file<R: Read+Seek>(reader, description)`** —
  one-call file-progress wiring (top-10 use case previously requiring
  6 lines of manual setup).
- **`Console::out(text, style)`** — raw print with no markup, emoji,
  highlight, or wrap parsing. Use when content already contains
  literal `[` brackets or `:tags:` that shouldn't be interpreted.
- **`Style::pick_first(&[Option<&Style>])`** — selects first non-null
  style from candidates; mirrors rich's helper used in render
  pipelines that layer theme/row/column/cell overrides.
- **OSC 8 hyperlink `id=` parameter** — every emitted hyperlink now
  carries a process-monotonic `id=` so iTerm2/Kitty/WezTerm group
  multi-line link runs as a single clickable target.
- **`RichHandler` config**: `with_omit_repeated_times(bool)` (default
  `true`) blanks duplicate timestamps; `with_enable_link_path(bool)`
  renders `module::path:line` as an OSC 8 `file://` link;
  `with_gilt_tracebacks(bool)` routes multi-line ERROR records
  through `Traceback::from_panic` for Panel-wrapped styled output.
- **`Traceback::with_suppress(Vec<String>)` and
  `Traceback::install_panic_hook[_with]`** — filter library frames by
  filename substring; install a `std::panic::set_hook` that prints
  styled tracebacks to stderr using `Backtrace::force_capture`.
- **`Table::add_row_renderable(Vec<Arc<dyn Renderable + Send + Sync>>)`**
  — cells can now hold any Renderable (Panel, Tree, nested Table)
  rather than only `&str` or `Text`. New `CellContent::Renderable`
  variant.
- **`Text::markup() -> String`** — round-trips a styled Text back to a
  parseable markup string, escaping literal brackets and emitting
  open/close tags for every span.
- **`Text::get_style_at_offset_themed(console, offset)`** — extends
  `get_style_at_offset` with theme-stack lookup so named styles like
  `"highlight"` resolve through `console.get_style()`.
- **`Syntax` default theme** changed to `"base16-mocha.dark"` (closest
  near-Monokai available in `syntect`'s bundled themes), aligning the
  visual baseline with rich's `"monokai"` default.
- **3 new test binaries** (`tests/table_unit.rs`, `progress_unit.rs`,
  `segment_unit.rs`) with 45 scenarios ported from rich's pytest
  suite plus inline filesize coverage.

### Fixed

- **Rich v14.0.0 — `NO_COLOR=""` semantics**: empty value no longer
  disables color (must be a non-empty value to opt out, per the
  no-color.org convention).
- **Rich v14.0.0 — `FORCE_COLOR=""` semantics**: empty value no
  longer force-enables color; falls through to no-override.
- **Rich v14.3.0 — Live spurious newline on stop**: `Live::stop()` no
  longer emits a trailing `\n` when the live region rendered nothing
  (gates on `last_render_height() > 0`).
- **Rich v14.3.0 — Table padding consistency**: `get_padding_width`
  (measure path) and per-cell padding application (render path) were
  computing `collapse_padding` differently — measure said `pad_left=0`
  always, render did `pad_left.saturating_sub(pad_right)`. Both paths
  now agree per rich v14.3.0: `pad_left=0` only for `column_index>0`.
  First-column text could previously overflow its measured width.
- **Rich v15.0.0 — Markdown table inline code dropped**: `Event::Code`
  inside a table cell was routing to the wrong accumulator and the
  styled inline code disappeared on close. `TableContext` now stores
  `Vec<Text>` rather than `Vec<String>`, and the event handler guards
  on `in_table_cell` to preserve styling end-to-end.
- **`Table::measure(width=Some(0))`** now returns `Measurement(0,0)`
  matching rich's fully-collapsed semantics.
- **`Progress` mutators don't refresh the live display**: `advance`,
  `update`, `start_task`, `stop_task` now call `self.refresh()` after
  mutating, so progress bars actually update on screen instead of
  showing only `[?25l[?25h]` (cursor hide/show with no content).
- **`SpinnerColumn` always rendered frame 0**: `Spinner::render(time)`
  was treating its first call's `time` as `start_time` and computing
  `elapsed = time - start_time = 0`, so a fresh-per-render Spinner
  (which is what `SpinnerColumn` constructs) never advanced. Stateless
  callers now have `time` interpreted as elapsed seconds directly.
- **Hyperlinks fragmented into per-segment OSC 8 wrappers**:
  `[link=URL]Visit [bold]X[/bold] now[/link]` was emitting three
  open/close pairs, breaking single-link recognition in many
  terminals. `Console::render_buffer` now coalesces consecutive
  same-link segments under one wrapper.
- Bit-rotted examples: `testcard` updated for the renamed `gilt_console`
  trait method; `stylize_safe` import path corrected. Three orphan
  example files (`themes`, `http_demo`, `async_demo`) removed —
  they referenced APIs that no longer exist.

### Changed (breaking)

- **`Renderable::rich_console` → `Renderable::gilt_console`**, and
  the related identifiers `RichCast` → `GiltCast`, `rich_cast` →
  `gilt_cast`, `rich_cast_impl!` → `gilt_cast_impl!`,
  `TextPart::Rich` → `TextPart::Inner`. The `__rich_*__` protocol
  hook names were renamed to `__gilt_*__` in doc comments. This
  removes any "rich" identity from gilt's public API surface.
- **`Syntax.padding`** type changed from `(usize, usize)` (top, bottom)
  to `(usize, usize, usize, usize)` (top, right, bottom, left).
- **`Style::render`** now emits OSC 8 with an `id=N;` prefix.
  Existing tests asserting the exact byte sequence need to match by
  structure rather than equality.

### Internal — quality

- Deleted 9 orphaned root-level files in `src/` (`file_proxy`,
  `spinner`, `spinners`, `align_widget`, `bar`, `padding`, `scope`,
  `styled`, `styled_str`) that were duplicates of files already
  declared under `utils/` or `status/` and never compiled.
- Promoted `traceback.rs`'s per-call regexes to module-level
  `LazyLock<Regex>`; collapsed two byte-identical regexes in
  `markup.rs`; replaced `Regex::new()` with `str::match_indices` in
  `Text::split` (literal-string only).
- Removed unnecessary `unsafe` in `async.rs::ProgressStream::poll_next`
  (the `S: Unpin` bound makes the safe `get_mut()` sufficient).
- `STYLE_CACHE` lookups now recover from a poisoned mutex via
  `into_inner` rather than panicking on access.
- `COLOR_CACHE` was declared with public `clear`/`size` functions in
  v0.8.0 but `Color::parse()` never used it — wired through so the
  documented LRU caching now actually works.
- `tests/unit/console_tests.rs` `set_var` calls now serialise via the
  same `ENV_LOCK` pattern used in `color_env`'s tests.
- Doc comments across 51 source files cleaned of "Port of Python
  rich/X.py" attribution and similar "Python rich" references.
- Version strings in `lib.rs` quickstart, `tracing_layer` docs, and
  README install examples updated from `"0.6"` / `"0.7"` / `"0.8"` /
  `"0.1"` to the current major.

## [0.8.0] - 2026-02-09

### Added
- **Thread-safe Console** - Interior mutability with `Arc<RwLock<ConsoleInner>>` for safe sharing across threads
- **LRU Cache for Style/Color parsing** - 256-entry cache for `Style::parse()`, 512-entry for `Color::parse()` for 2-5x speedup on repeated parses
- **New Progress Column Types**:
  - `SpinnerColumn` - Animated spinner with 80+ styles
  - `TimeElapsedColumn` - Shows elapsed time (H:MM:SS format)
  - `TimeRemainingColumn` - Shows estimated remaining time with `--:--` when unknown
  - `FileSizeColumn` - Human-readable file sizes (kB, MB, GB)
  - `DownloadColumn` - Shows `completed/total` with size formatting
  - `TransferSpeedColumn` - Shows transfer rate (MB/s, GB/s)
- **Iterator Progress Tracking** - `track()` function and `ProgressIteratorExt` trait for `.progress()` on any iterator
- **File Progress I/O** - `Progress::open()` and `ProgressFile` for reading files with progress bars
- **Live stdout/stderr redirection** - `FileProxy` for capturing print! output within Live displays
- **New Widgets**:
  - `Padding` widget - configurable padding on all sides
  - `Align` widget - horizontal and vertical alignment of content
  - `Group` container - combines multiple renderables with fit mode
- **Safe Stylize methods** - `try_styled()`, `try_fg()`, `try_bg()`, `try_attr()` for fallible style parsing
- **Improved Unicode cell width** - Proper handling of ZWJ sequences (👨‍👩‍👧‍👦), variation selectors (👍️), and regional indicators (🇺🇸)
- **12 new examples**: `align_demo`, `cache_demo`, `columns_demo`, `group_demo`, `padding_demo`, `panel_nested`, `progress_columns_demo`, `raii_guards`, `stylize_safe`, `thread_safe`, `track_demo`, `unicode_width_demo`

### Changed
- **Breaking**: `Panel` now accepts any `Renderable` (not just `Text`) via `Box<dyn Renderable>`
- **Breaking**: `Columns` now accepts any `Renderable` via `Vec<Box<dyn Renderable>>`
- **Breaking**: `Panel` and `Columns` no longer implement `Clone` due to trait object constraints
- Console methods changed from `&mut self` to `&self` for thread safety

### Fixed
- **Progress/Live display shape tracking** - Fixed over-clearing of lines caused by Text's trailing newline
- `Rule::with_characters()` now validates input (panics on empty string)
- `cell_len()` now correctly handles box-drawing characters

[0.8.0]: https://github.com/khalidelborai/gilt/compare/v0.7.0...v0.8.0

## [0.4.0] - 2026-02-09

### Added
- `sparkline` module: inline Unicode bar charts using `▁▂▃▄▅▆▇█` blocks with linear interpolation resampling
- `canvas` module: Braille dot-matrix drawing (2×4 pixels per cell) with line, rect, fill_rect, circle primitives
- `diff` module: LCS-based line-level text diffing with unified and side-by-side rendering, colored output
- `figlet` module: large ASCII art text with built-in 5×7 block font (A-Z, a-z, 0-9, 34 punctuation chars)
- `csv_table` module: CSV-to-Table conversion with built-in parser (no deps) and optional `csv` crate integration
- `csv` feature gate for `csv` crate dependency
- 5 new examples: `sparkline`, `canvas`, `diff`, `figlet`, `csv_table`
- 119 new tests across the 5 modules

## [0.3.0] - 2026-02-08

### Added
- WCAG 2.1 accessibility module (`accessibility.rs`): `contrast_ratio()`, `meets_aa()`, `meets_aaa()`, `meets_aa_large()` for color contrast checking
- `REDUCE_MOTION` environment variable detection via `detect_reduce_motion()` in `color_env.rs`
- Feature gates: `json`, `markdown`, `syntax`, `interactive` (all default-on); use `default-features = false` for minimal builds
- `CompactString` for `Segment.text` providing inline storage for strings <=24 bytes, eliminating heap allocations for most terminal segments
- `Cow<str>` returns from `strip_control_codes`, `escape_control_codes`, `emoji_replace`, `set_cell_size` for zero-allocation when input needs no modification
- Comprehensive showcase example (`examples/showcase.rs`) demonstrating 21 feature sections
- 26 new criterion benchmarks covering control codes, cell sizing, emoji operations, segment operations, and export operations
- `just check-minimal` and `just check-all` justfile recipes for testing feature combinations

### Changed
- Replaced `once_cell` with `std::sync::LazyLock` (MSRV raised to 1.82)
- Replaced `format!()` with `write!()` in SGR rendering and HTML/SVG export loops to reduce intermediate string allocations
- `serde_json`, `pulldown-cmark`, `syntect`, `rpassword` are now optional dependencies behind default feature flags

### Removed
- `once_cell` dependency

## [0.2.1] - 2026-02-08

### Added
- `justfile` with dev and release recipes (format, lint, test, doc, publish workflows)

### Fixed
- Rustfmt formatting and module sort order in `anstyle_adapter.rs`

## [0.2.0] - 2026-02-08

### Added
- Extended underline support: curly, dotted, dashed, and double underline styles with underline color (SGR 4:N codes)
- `anstyle` feature flag for bidirectional `From` conversions between gilt and anstyle `Color`/`Style` types
- 100% rustdoc coverage: all 279 previously undocumented public items now have doc comments with examples
- Expanded crate-level documentation with core modules table, feature flags table, and global console examples
- Version specifier for `gilt-derive` dependency to support crates.io publishing

### Changed
- Excluded `.claude/` directory from published crate package

## [0.1.0] - 2026-02-08

### Added
- Initial release: full port of Python's rich library (65 modules, 51,000+ lines of Rust)
- 2,111 tests (2,066 unit + 37 tracing-gated + 4 miette + 4 eyre), 0 clippy warnings
- 45 examples covering every widget
- 30 Renderable implementations
- **Core text**: Rich `Text` with markup, styles, wrapping, alignment, and `Segment`-based rendering pipeline
- **Console**: Color system detection, capture mode, export to HTML/SVG/text, global console via `gilt::print()`, `gilt::print_text()`, `gilt::print_json()`
- **Widgets**: Table, Panel, Tree, Progress (multi-bar with ETA/speed/spinner), Live, Status, Columns, Layout, Rule, Bar, Group, Align, Constrain, Screen
- **Syntax highlighting**: 150+ languages via syntect
- **Markdown rendering**: Terminal-rendered Markdown via pulldown-cmark
- **JSON pretty-printing**: Highlighted JSON output
- **Prompt**: Interactive input with Select/MultiSelect, numbered-choice UI, "all" keyword, min/max validation
- **Gradient text**: True-color RGB interpolation with rainbow preset, `Renderable` + `Display`
- **Stylize trait**: `"hello".bold().red()` method chaining via `styled_str.rs`
- **Iterator progress**: `.progress()` adapter for any iterator via `ProgressIteratorExt`
- **`#[derive(Table)]`**: Proc macro for auto-generating tables from structs (feature: `derive`)
- **Inspect**: Debug any value with rich formatting, builder API
- **Pretty printer**: Type-annotated debug output with `infer_type_name()`
- **Highlighters**: Regex, ISO date, URL, UUID, JSON path highlighter types
- **Environment detection**: `NO_COLOR`, `FORCE_COLOR`, `CLICOLOR` 5-tier priority
- **OSC 52 clipboard**: Copy to clipboard via terminal escape sequence
- **Synchronized output**: Flicker-free rendering via DEC 2026 protocol
- **Pager**: Built-in terminal pager support
- **Logging handler**: `log` crate integration for styled log output
- **Traceback**: Rich error traceback display
- **Scope**: Variable scope inspection widget
- **miette integration**: Diagnostic reporting with gilt styling (feature: `miette`)
- **eyre integration**: Error reporting with gilt styling (feature: `eyre`)
- **tracing integration**: Log subscriber with colored output (feature: `tracing`)
- **Prelude module**: Ergonomic `use gilt::prelude::*` re-exports

[0.3.0]: https://github.com/khalidelborai/gilt/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/khalidelborai/gilt/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/khalidelborai/gilt/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/khalidelborai/gilt/releases/tag/v0.1.0
