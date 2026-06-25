# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [2.3.1] - 2026-06-25

Two `Layout` rendering fixes reported by a downstream adopter (DeepForge) while
building an alt-screen TUI on gilt 2.0.

### Fixed

- **`Tree` nested in a fixed-size `Layout`/`Panel` region rendered only its root,
  dropping all child nodes.** Each node label was rendered with the ambient
  (region) height still set, so the root label was padded to fill the whole
  region and the children were cropped away. Node labels now render at their
  natural height (rich parity: `height=None` per label), so a `Tree` survives
  composition inside a fixed-size region.
- **Split `Layout` collapsed under `Live::with_screen(true)`** — the alt-screen
  frame was emitted with bare `\n` line separators (and a trailing `\n` from
  `console.print`). In the raw mode an interactive alt-screen TUI runs in, the
  tty's `ONLCR` (`\n`→`\r\n`) translation is off, so a bare LF moved the cursor
  down but not to column 0; full-width split-`Layout` rows staircased and
  collapsed (thin left column, empty middle, scattered borders). The print/export
  path was unaffected because it renders into a string buffer. `do_refresh` now
  emits the screen frame with CR-bearing separators (`application_mode`) and
  writes it directly (no trailing newline that would scroll a full-height frame).
  Geometry, the DEC-2026 synchronized-output wrapper, and line-diff/pause-resume
  are unchanged. Adds a screen-mode render test asserting no bare LF + intact
  row geometry.

## [2.3.0] - 2026-06-25

Inline images everywhere, two more charts, and richer Canvas demos. All
dep-free and WASM-safe (native image decoding stays behind `inline-images`).

### Added

- **Markdown inline images** — `Markdown` renders `![alt](src)` as a real inline
  image (auto-selecting Kitty / iTerm2 / Sixel / halfblock) when `src` is a local
  file and the `inline-images` feature is on; the text/alt placeholder is kept for
  remote URLs, missing files, and default/WASM builds.
- **`LineChart` widget** — a Braille line plot: multiple series with per-series
  styles, optional axes + y-range + legend, auto-scaling. Re-exported at the crate
  root and prelude; see `examples/linechart.rs`.
- **`Histogram` widget** — bins raw `f64` samples into a distribution rendered as
  vertical block columns (`with_bins`/`with_range`/`with_height`). Re-exported;
  see `examples/histogram.rs`.
- **`GILT_IMAGE_PROTOCOL` override + broader detection** — set
  `GILT_IMAGE_PROTOCOL=kitty|iterm|sixel|halfblock` to force the inline-image
  protocol; `ConsoleCapabilities` also recognizes more Sixel/Kitty/iTerm2-capable
  terminals from the environment.
- **`Image::with_background` + halfblock alpha compositing** — the halfblock
  renderer composites RGBA alpha over a configurable background (default opaque
  black); fully-opaque pixels are byte-identical to before.
- New Canvas examples: `canvas_plot` (function curves), `canvas_blitters` (the
  same shape across Braille/Octant/Sextant/HalfBlock), and `canvas_lissajous`.

### Fixed

- `cargo test --lib --no-default-features` now builds — the `print_json_opts_tests`
  module is gated behind the `json` feature (was `#[cfg(test)]` only).

## [2.2.0] - 2026-06-25

Visualizations — four beyond-rich Track A additions, each dep-free and WASM-safe.

### Added

- **`BarChart` widget** — horizontal bar chart: right-aligned labels, bars scaled
  to the data max (or an explicit `with_max`) with eighth-block sub-cell
  precision, and optional value labels. Builders for width, max, and bar/label/
  value styles; `Renderable` + `gilt_measure`. Re-exported at the crate root and
  in the prelude. See `examples/barchart.rs`.
- **`Heatmap` widget** — 2-D `f64` grid rendered as a coloured cell grid; each
  value is normalized and mapped through a configurable colour gradient to a
  background-coloured cell (`with_min`/`with_max`/`with_gradient`/`with_cell_width`).
  `Renderable` + `gilt_measure`. Re-exported at the crate root and in the prelude.
  See `examples/heatmap.rs`.
- **`Canvas` Octant blitter (Unicode 16)** — `Blitter::Octant` now renders real
  2×4 octant glyphs (higher density than Braille) instead of falling back to
  Braille. The full 256-pattern table is derived authoritatively from Unicode
  16.0 `UnicodeData.txt` (230 `BLOCK OCTANT-N` glyphs at U+1CD00..=U+1CDE5 plus
  26 reused legacy block/quadrant glyphs).

### Changed

- **`Sparkline` min/max markers** — additive `with_min_max_markers(bool)` plus
  `with_min_style`/`with_max_style` highlight the extreme data points. Fully
  backward-compatible: with markers off, output is unchanged.

## [2.1.0] - 2026-06-25

### Added

- **iTerm2 inline-image protocol (OSC 1337)** — `Image` now renders through the
  iTerm2 inline-image protocol when the terminal advertises it
  (`capabilities().iterm`), the console is not recording, and the `inline-images`
  feature is enabled (the RGBA buffer is PNG-encoded).
- **Sixel image protocol (DCS)** — `Image` renders real Sixel graphics when
  `capabilities().sixel` is true and not recording: a uniform 6×6×6 palette with
  banded, run-length-encoded sixel data. **Dep-free** (no `image` crate needed),
  so it works in any build. Protocol selection order is now recording→halfblock,
  Kitty, iTerm2, Sixel, halfblock.

### Fixed

- **Kitty / iTerm2 image escapes are no longer truncated** — the Kitty APC and
  iTerm2 OSC 1337 payloads are emitted as zero-width control segments so the
  render pipeline never width-crops or line-splits them. Previously a plain text
  segment truncated the base64 payload to the console width (a latent Kitty bug;
  its test only checked the introducer prefix).
- **Kitty images now honor `.width()`/`.height()`** — `render_kitty` sends the
  display cell-box size (`c`/`r` keys) and transmits at capped native resolution,
  so the image fills the requested cells. Previously it downscaled to
  `cols × rows*2` pixels with no `c`/`r`, rendering a tiny native-pixel thumbnail
  regardless of the requested size.

## [2.0.0] - 2026-06-25

Parity 2.0 — closing verified gaps against Python `rich` (see `.review/parity-audit-2026-06-24.md`). Phase 1: correctness fixes. Phase 2: render-time theme resolution. Phase 3: measurement protocol. Phase 4: container generalization. Phase 5: export correctness. Phase 6: live/progress nesting + logging layout. Phase 7: a broad P2/P3 parity sweep across 28 subsystems (Progress, Markdown, Pretty, Logging, Prompt, Traceback, Console, Control, Themes, Table, Panel/Box, Tree, Style, Markup, Syntax, Cells, Protocols, Layout, Containers, Text, Segment, Color/Palette, Highlighter, Public API, Windows, Inspect, Scope, Export).

### Added

- **Container widgets now hold ANY `Renderable`, not just `Text`** (#13, #15, #16, #19, #21) — `Panel`, `Tree` labels, `Group`/`Renderables` items, `Align`, `Padding`, `Constrain`, `Styled`, and `Table` `CellContent::Renderable` cells. Compose freely: `Panel::new(Table::new(..))`, a `Group` of mixed widgets, a `Tree` with a `Panel` label, a table cell holding a `Panel`. Constructors are generic (`impl Renderable + Send + Sync + 'static`), so `Panel::new(Text::new(..))` / `Panel::new("literal")` keep working.
- **`RenderableArc` type alias** (`Arc<dyn Renderable + Send + Sync>`) + **`into_renderable_arc`** helper — the shared container element type (also used by `Live`/`Layout`).
- **`Renderable::gilt_measure(&Console, &ConsoleOptions) -> Measurement`** — a measurement-protocol hook (rich's `__rich_measure__`) with a default that falls back to the previous full-render width derivation, overridden on every built-in widget (Text, Panel, Tree, Table, Columns, Padding, Align, Constrain, Group, Bar, Styled, Renderables, CsvTable, Figlet, Sparkline, Diff, Canvas, ProgressBar, Syntax) so width is computed without a full render.
- **`measurement_get` / `measure_renderables`** (re-exported from `gilt::measure`) — rich's `Measurement.get` / `measure_renderables` dispatch helpers.
- **`Span::named(start, end, name)`** — a span that defers style resolution to render time (carries a `style_name: Option<String>` instead of an eager `Style`).
- **`Text::render_themed(&Console)`** — resolves theme-named spans against the active console theme stack at render time (with a fast-path that delegates to `render()` when no span is named, preserving the zero-clone hot path).

### Changed (Breaking)

- **Phase 7 signature changes** (parity sweep): `Progress::add_task` gained a `start: bool` and `update` a `fields` parameter (and `TaskProgressColumn::with_separator` was removed); `Console::push_theme` and `Theme::from_file`/`read` gained an `inherit: bool`; `BoxChars::substitute` and `Style::render` gained a `legacy_windows: bool`; `Syntax::stylize_range` gained `style_before: bool` and `Syntax::highlight_code` was renamed to the public `Syntax::highlight`; `Syntax::measure` now takes `&Console, &ConsoleOptions`; `ConsoleOptionsUpdates::no_wrap` is now `Option<Option<bool>>` (so it can reset to `None`); `Text::justify`/`Lines::justify` and `Text::wrap` now take `&Console` (cascading into `Syntax`); `Layout::refresh_screen` now updates the screen in place instead of returning segments.
- **`Console::save_html` / `save_svg` gain forwarding parameters** — `save_html(path, theme, clear, inline_styles, code_format)` and `save_svg(path, title, theme, clear, unique_id, code_format)` (were `save_html(path)` / `save_svg(path, title)`). The five `export_*` methods now **panic** (assert) if called without `record` mode (matching rich's `assert self.record`), instead of silently returning an empty string — their `String` return types are unchanged.
- **Container content fields are now `RenderableArc` instead of `Text`** — `Panel.content`, `Tree.label`, `Align.content`, `Padding.content`, `Constrain.renderable`, `Styled.renderable`, and `Group`/`Renderables` items (`Vec<RenderableArc>`). Constructing via `Text`/`&str` still compiles, but **direct field reads that called `Text` methods** (e.g. `panel.content.plain()`) no longer compile — access through the `Renderable` trait or downcast the `Arc`. `Group::new`/`fit` take `Vec<RenderableArc>` (new `Group::push` / `Renderables::append` accept `impl Renderable`). These structs lost their derived `Debug` (now a manual impl printing `"<renderable>"`); `Clone` is preserved (cheap `Arc` clone).
- **`Panel::from_renderable` takes an owned `R`** (was `&R`); **`Styled::measure` takes `(&Console, &ConsoleOptions)`** (was no-arg).
- **Behavior break:** the five `export_*` methods now **panic** without `record` mode (see Fixed, #f) — callers relying on the old silent empty-string return must enable `.record(true)`. See `MIGRATION_v2.md`.
- **`Console::measure` now dispatches through `Renderable::gilt_measure`** instead of always rendering (#2, #11). For widgets with a measure override this skips the render. The empty-content case for the *default* (un-overridden) path now returns `(0, max_width)` instead of `(0, 0)` (#3), matching rich; `Text("")` still measures `(0, 0)` via its own override.
- **`Align::measure`** now returns the content's measured width (clamped to `max_width`), or the explicit width (also clamped), instead of always returning `max_width` (#14).
- **`Span` gains a `style_name: Option<String>` field** and its equality now includes it. Struct-literal construction (`Span { .. }`) must migrate to `Span::new(..)` or `Span::named(..)`.
- **`Renderable for Text` (and `Rule`/`Panel` title, `Progress`, `Spinner`) now use the Console theme** when rendering, so theme-named content that previously rendered unstyled now carries the themed style.

### Fixed

- **Phase 7 P2/P3 parity sweep (28 subsystems)** — closing the long tail of verified gaps against rich. Highlights: **Progress** lazy-start tasks, mergeable `fields`, `N%` task column, markup in `TextColumn`, per-column `max_refresh` rate-limiting, shared download units, ceil on `time_remaining`; **Markdown** OSC-8 hyperlinks, inline-code syntax highlighting, blockquote prefixes on all block types, styled table headers, right-aligned ordered lists (incl. nested), image-alt filename fallback; **Pretty** `pretty_repr`/`Console::pprint`, rich-style `+N` truncation, `max_depth`/`expand_all` on the Debug path, cell-width thresholds; **Logging** theme-driven keyword/level styles, configurable highlighter, `min_level` filtering, `GiltLayer` parity; **Prompt** typed `IntPrompt`/`FloatPrompt`/`Confirm`, `prompt.invalid` theming, styled password, `input` stream injection; **Traceback** PEP-678 notes, exception-group panels, `PanicHookConfig`, side-by-side locals, `code_width`; **Console** `render_str` emoji+highlight, `log` variadic objects, `soft_wrap`, real pager, `RenderHook` pipeline (via `print_with_hooks`), `print` sep/end; **Control** clear-only escape + `is_terminal` guards + `update_screen` region; **Themes** `inherit` + `use_theme` RAII guard + `;` INI comments; **Table** `leading` blank lines, `show_lines` without a box, `ratio_distribute` zero-guard + rich-faithful rounding; **Panel/Box** title-background compositing, `safe_box` inheritance, legacy-Windows box substitution; **Tree** `add_with` (style/guide/expanded/highlight), guide-style negation, default theme styles; **Style** `legacy_windows` OSC-8 guards, `normalize_str`, larger parse cache; **Markup** `@`-key meta tags + `@click(args)` + meta-span round-trip; **Syntax** `style_before`, `❱` pointer, shebang lexer guess, token-accurate line numbers; **Cells** `split_graphemes`/`split_text`; **Protocols** `GiltCast` rendering (via `CastWrapper`/`print_cast`); **Layout** in-place `refresh_screen`, `Columns` measurement via `gilt_measure`, longest-word `Align` minimum; **Color** `EightBit`→Windows/Standard downgrade passthrough + public ANSI-name accessors + `match_color` cache; **Highlighter** unbounded attribute names + Unicode whitespace; **Public API** `FileProxy::isatty`, `print_json` `data`/`ensure_ascii`, `VerticalCenter`, `log_time` toggle; **Windows** `legacy_windows` builder setter + auto-detection; **Inspect** `$PAGER`, pager `styles`, Jupyter/VSCode diagnose env; **Scope** per-cell key/equals styling + value highlighting; **Export** SVG per-line clip-paths wired to text. _Also:_ integration-suite tests made highlight-robust; no new dependencies introduced.
- **Screen-mode `Live` preserves styling** (#25) — alt-screen `do_refresh` no longer flattens styled segments to plain text (it renders the content through `Screen::from_arc`); bold/color/dim survive. `Screen.renderable` is now a `RenderableArc` (`Screen::new(Text::new(..))` still compiles).
- **`Live`/`Progress` nesting tracked via the console live-stack** (#26, #27) — `Live::start`/`stop` (and `Progress` through its internal `Live`) now `push_live`/`pop_live`, so nested live displays are tracked correctly. `pause`/`resume` leave the stack untouched.
- **Log handlers use `Table::grid()` for column alignment** (#28) — `RichHandler` and the `tracing` `GiltLayer` build an expanded grid (time/level/message/path columns) instead of flat space-concatenation, so columns align across records regardless of message length.
- **`export_*` methods panic when `record` mode is off** (#f) — previously returned an empty string silently; now `assert!(self.record, "export requires record mode — build the Console with .record(true)")` fires on all five base export methods (`export_text`, `export_html`, `export_html_opts`, `export_svg`, `export_svg_opts`), matching Python rich's `assert self.record`. Return types are unchanged (non-breaking for callers with record enabled).
- **SVG/HTML export correctness** (audit §3.10): SVG lines are cropped to the console width (#22); `reverse` swaps the background-rect fill and `dim` blends the foreground toward the theme background when a segment has no background (#23); per-line `<clipPath>` defs are emitted; traffic-light chrome geometry matches rich; all-space segments no longer emit `<text>`; class-mode HTML anchors are now wrapped *inside* the styled span. `save_html`/`save_svg` forward `theme`/`clear`/`inline_styles`/`unique_id`/`code_format`. _Deferred:_ wiring the per-line clip-paths to their `<text>` elements.
- **`save_html` / `save_svg` forward export parameters** (#e, #g) — `save_html(path)` is now `save_html(path, theme, clear, inline_styles, code_format)` and `save_svg(path, title)` is now `save_svg(path, title, theme, clear, unique_id, code_format)`. Both delegate to the underlying `export_*_opts` / `export_svg` methods. **Breaking:** existing call sites must add the new parameters.
- **Nested widgets in `Table` cells render at the column width** (#19) — a `CellContent::Renderable` (e.g. a `Panel`) was previously pre-rendered at the console's default width and re-wrapped, destroying its geometry; it now renders at the resolved column width with correct padding/alignment.
- **Measurement protocol (#2, #3, #4, #11, #14)** — see Added/Changed above; `Console::measure` no longer bypasses per-widget measurement, and the empty/Align width contracts now match rich.
- **Theme-named markup tags resolve at render time (#5, #6).** `[warning]`, `[repr.number]`, etc. previously collapsed to `Style::null()` at parse time (the theme name was lost); they now carry the name and resolve against the Console theme (or `DEFAULT_STYLES` with no console). _Known limitation:_ a single-word theme name that also parses as a literal style token (e.g. an overridden `red`) is still treated as a literal — full render-time resolution of every tag is a deferred enhancement.
- **`highlight_regex_with_groups` resolves group-name styles via `DEFAULT_STYLES`** (#7), not only `parse_strict` — so names like `repr.number` highlight correctly.
- **`RegexHighlighter` emits theme-overridable named spans** (#37) — a console theme override of a group style (e.g. `repr.number → italic yellow`) now wins at render time.
- **Color downgrade now applied at render time (P0).** `Style::render` previously emitted truecolor SGR (`38;2;r;g;b`) regardless of the console's color system; on a 16-color (`Standard`) or 256-color (`EightBit`) terminal that produced escapes the terminal can't render correctly. Each of `color`/`bgcolor`/`underline_color` is now downgraded to the console's `color_system` before emission, matching rich's `_make_ansi_codes`.
- **`Color::downgrade` is now an identity when the color is already at (or below) the target system**, instead of re-matching through a palette (which could shift an index, e.g. `Standard(8)` → `Standard(7)`).
- **`MONOKAI`, `DIMMED_MONOKAI`, and `NIGHT_OWLISH` terminal themes** now have 8 normal + 8 bright ANSI colors (were 9 + 7), fixing corrupted ANSI 8–15 in HTML/SVG export.
- **`pretty` debug-string truncation** places the `+N` overflow suffix outside the closing quote (`"kept"+N`, was `"kept+N"`).
- **`Prompt::ask_int` / `ask_float` terminate on EOF** instead of looping forever (the misleading "invalid number" message is no longer printed on EOF).
- **`Panel` fit mode** sizes to the longest content line (via `measure().maximum`) instead of the total character count, so multi-line panels are no longer over-wide.

## [1.11.0] - 2026-06-23

### Added

- **`Live::pause()` / `Live::resume()`** — first-class pause/resume for a live
  region. `pause` halts the background refresh and erases the current render in
  place (the same erase a transient `stop` does) while preserving the renderable
  and state; unlike a non-transient `stop` it emits **no trailing newline**, so
  the last render is not left behind in the scrollback. `resume` re-hides the
  cursor, redraws the preserved content at the cursor's current position
  (drawing downward, so output that scrolled in while paused is untouched), and
  restarts the refresh thread. Adds `Live::is_paused()`. This lets stacked live
  UIs (e.g. a sticky-footer `Live` + a child tree that renders its own `Live`)
  cleanly hand off the terminal's bottom row without callers toggling the
  `transient` flag or managing cursor erase by hand. `start`/`stop` behaviour is
  unchanged. TDD.

## [1.10.0] - 2026-06-06

Terminal-awareness + higher-resolution graphics, plus reproducible demos.
Completes the additive items from the landscape strategy. TDD.

### Added

- **OSC 11 background detection → auto dark/light** — `parse_osc11_response`
  and `is_dark_background` (dep-free, WASM-safe core); `Console::detect_background`
  and `ConsoleBuilder::auto_theme()` actively probe and pick a theme behind the
  opt-in `terminal-query` feature (native, crossterm, 200 ms timeout).
- **Canvas blitter ladder** — `Canvas::with_blitter(Blitter)`:
  `Braille` (default, 2×4), `Sextant` (2×3, U+1FB00), `HalfBlock`, and `Octant`
  (stubbed to Braille until Unicode 16 fonts are common). Same drawing API,
  higher cell density.
- **`gilt-vhs`** — a `publish = false` workspace crate for **tape-as-code**
  recording: `Tape::new().frame(renderable, delay)` → `to_cast()` (deterministic
  asciinema, injected clock — no sleeping) / `to_svg()`, for reproducible
  README/docs demos.
- **Examples for everything** — a runnable example per v1.8–v1.10 feature
  (`asciinema_export`, `scoped_record`, `themed_console`, `table_colspan`,
  `capabilities`, `inline_image`, `fuzzy_select`, `form`, `auto_theme`,
  `canvas_blitters`, `live_dirty_cache`).

## [1.9.0] - 2026-06-06

Fills the two largest structural gaps (inline images, interactive forms) and
opens a shell-scripting adoption surface — from the landscape strategy. TDD.

### Added

- **Inline terminal images** — `Image` renderable (`gilt::image`). `Image::from_rgba`
  renders anywhere via Unicode upper-half-block with truecolor (and exports to
  HTML/SVG); `from_path`/`from_bytes` decode PNG/JPEG behind the opt-in
  `inline-images` feature. Kitty graphics protocol used when the terminal
  supports it (and not recording); recording/export always uses halfblock. No
  `Segment` change. `ConsoleCapabilities` now detects kitty/iterm/sixel from env.
- **`Form` builder** (feature `interactive`) — chain `Input`/`Confirm`/`Select`
  fields with validation + re-prompt and an **accessibility fallback** (plain
  prompts on `NO_COLOR`/`TERM=dumb`), à la `huh`.
- **`FuzzySelect`** — a dep-free testable core (`FuzzySelectState<T>`: filter +
  navigate + select) plus an interactive driver behind the opt-in `tty-select`
  feature (crossterm, RAII raw-mode guard). crossterm is **not** in the default
  dep graph.
- **`gilt-cli`** — a new installable binary (`cargo install gilt-cli`) exposing
  gilt to shell scripts: `gilt print '[bold]hi[/]'`, `gilt style`, `gilt table <
  data.csv`, `gilt rule`, `gilt panel`, `gilt markdown`, `gilt json` (the `gum`
  pattern). Separate crate; the library is unaffected.

### Performance

- **Per-region dirty tracking** — `Renderable::content_hash()` (additive default
  `None`); `Layout::render_with_cache` reuses unchanged children's segments
  across frames, so static sections of a compound Live display aren't
  re-rendered each frame.

## [1.8.0] - 2026-06-06

Rendering correctness, performance, and table-stakes quality — guided by the
cross-language landscape study (`.review/landscape-and-strategy-2026-06-06.md`).
Built test-first.

### Added

- **`Console::export_asciinema()`** (feature `asciinema`) — export a recorded
  session to asciinema v2 `.cast` NDJSON, playable by `<asciinema-player>` (VS
  Code/Jupyter/docs). First in the Rust ecosystem. Injectable WASM-safe clock;
  `begin_asciinema_record()` / `with_asciinema_clock()` / `save_asciinema()`.
- **`Console::scoped_record(|c| {…}) -> Recording`** — one-call scoped capture
  returning `.to_text()` / `.to_html()` / `.to_svg()` (Spectre.Console pattern).
- **`GILT_THEME` env var** — `ConsoleBuilder` loads a JSON `Theme` file at
  construction and applies it (glamour's `GLAMOUR_STYLE` pattern; feature `json`,
  native). `Theme::from_json_str`/`from_json_reader`, `theme_from_path`.
- **`ConsoleCapabilities`** — env-derived terminal capability flags
  (`Console::capabilities()`); foundation for image-protocol detection.
- **Table column-span** — `Table::add_row_spanned(cells, spans)` / `Row.col_spans`.
- **Windows VT** — opt-in `windows-vt` feature enables
  `ENABLE_VIRTUAL_TERMINAL_PROCESSING` at `Console::new()` (native Windows).

### Changed / Performance

- **DEC mode 2026 synchronized output** — every Live frame is now wrapped in
  `CSI ?2026h`/`l`, so terminals apply it atomically (no flicker/tearing, esp.
  under tmux). Harmless no-op where unsupported.
- **Line-diff repaint** in `LiveRender` — unchanged lines get a cursor-move only,
  not an erase+rewrite; combined with frame-skip + sync output the Live display
  is now flicker-free and minimal-write.
- **BufWriter write coalescing** — segment writes within a synchronized frame are
  buffered and flushed once (O(1) syscalls/frame instead of O(segments)).
- **`cell_len` width cache** — thread-local LRU for non-ASCII character widths.

## [1.7.1] - 2026-06-06

Housekeeping — no API changes.

### Added

- Examples for recent features: `text_macro` (compile-time `text!`),
  `notifications` (OSC 9 + taskbar), `gradient_progress`
  (`BarColumn::with_gradient`), `export_themes` (`ThemeRegistry` + themed HTML).

### Internal

- Relocated the `Console` export methods (`export_text`/`export_html`/
  `export_html_with_theme`/`export_html_opts`/`export_svg`/`export_svg_opts`)
  from `console.rs` into `console_export.rs` beside their helpers, matching the
  crate's `#[path]` + `impl Console` split convention (`console.rs` 1779 → 1112
  lines). Pure relocation: same public paths, no behavior change.

## [1.7.0] - 2026-06-05

Rust-native differentiators + parity completions, led by a compile-time markup
macro. From the feature roadmap (`.review/feature-roadmap-2026-06-05.md`).

### Added

- **Compile-time `text!` macro** (feature `derive`) — `text!("[bold red]x[/]")`
  validates gilt markup at `cargo build` and expands to a `Text`. Unclosed
  brackets, mismatched/unclosed tags, and unknown style tokens are now compile
  errors. The validator mirrors `Style::parse` / `Color::parse` exactly, so any
  markup `Text::from_markup` accepts also compiles (guarded by an
  anti-false-positive suite + trybuild compile-fail tests). Impossible in Python
  rich.
- **Terminal escapes** — `Console::notify(title, body)` (OSC 9 desktop
  notification); `Console::set_taskbar_progress(state, pct)` + `TaskbarState`
  (OSC 9;4, ConEmu/Windows Terminal) with an opt-in `Progress::with_taskbar`;
  `ConsoleBuilder::log_path(bool)` to show the caller location in `log()`.
- **Theming** — `ThemeRegistry` with 5 built-in palettes (Dracula, Nord,
  Gruvbox, Monokai Pro, Solarized Dark) + `Console::export_html_with_theme(name)`;
  `serde` `Serialize`/`Deserialize` for `Color`/`Style`/`Theme` (feature `json`);
  `Syntax::with_syntect_theme()`, and `Syntax::load_theme_from_file()` (opt-in
  `syntax-theme-file` feature, native — kept out of default to stay dep-light).
- **Traceback `show_locals`** — now rendered: `Frame::with_local(s)` carry
  user-supplied name/value pairs shown via the scope renderer when enabled.
- **Markdown GFM task-lists** — `- [x]` / `- [ ]` render as ☑ / ☐.
- **Gradient progress bars** — `BarColumn::with_gradient(start, end)` fills the
  completed bar with a per-cell truecolor gradient.
- **`Pretty::from_serde<T: Serialize>`** (feature `json`).
- **Export builders** — `HtmlExportOptions` (`export_html_opts`): copy button,
  dark-mode CSS, custom font/`font_url`; `SvgExportOptions` with
  `FontEmbedding::Base64` for self-contained offline SVG.
- **Typed styling** — `Style::fg`/`bg`/`with_underline_color` and
  `Stylize::fg_color`/`bg_color`/`underline_color` take a typed `Color`.
- **Typed prompts** — `Prompt::with_converter` → `TypedPrompt<T>` (parse + retry
  loop for any type).

## [1.6.0] - 2026-06-05

A correctness + async + DX release: tty-aware color, a hardened async live
surface, finished rich-parity items, and CI that won't let any of it rot.
Informed by the feature roadmap in `.review/feature-roadmap-2026-06-05.md`.

### Added

- **tty-aware color.** `Console::is_terminal()` now uses `std::io::IsTerminal`
  (native; `TERM` fallback on wasm), and color **auto-disables when output is
  not a terminal** (piped/redirected) unless forced via
  `FORCE_COLOR`/`CLICOLOR_FORCE`/`force_terminal(true)`. `NO_COLOR` and explicit
  builder settings still win. New `Console::stderr()` constructor.
- **Span-level metadata.** `Span` carries optional `meta` (`Arc<HashMap<…>>`);
  markup `[@key=val]` / `[@flag]` tags now attach meta spans (previously
  dropped); new `Text::apply_meta()`, `Text::on(url)` (OSC-8 link), and
  `Text::get_meta_at()`.
- **`Live::print_above(renderable)`** — print scrollback output above a running
  live region without corrupting it.
- **Live cleanup guards** — `Console::capture_guard()` / `screen_guard()` RAII so
  capture/alternate-screen state can't leak under `?`/panic (`Console::is_capturing()`).
- **Async live surface** (feature `async`): `LiveAsync` is now **cancel-safe**
  (a synchronous `Drop` aborts the refresh task and restores the terminal even on
  a dropped `stop()` future, a lost `tokio::select!` branch, or a panic);
  `LiveAsync::new`/`update` are generic over any `Renderable`; new
  `async_run(live, hz, fut)` and `Live::watch(rx, f)` (drive a live view from a
  `tokio::sync::watch` channel).
- **Live & streaming guide** at [`docs/live-and-streaming.md`](docs/live-and-streaming.md).

### Changed

- **`Tree`** threads accumulated ancestor style into labels (a styled parent
  tints its subtree) and applies guide/background style to branch prefixes —
  rich parity.
- **`ConsoleOptions.no_wrap`** is now `Option<bool>` (`None` = inherit/wrap,
  `Some(true)` = no-wrap); behavior unchanged for existing callers.

### Performance

- **Segment-level frame-skip**: a `Live` refresh that would produce byte-identical
  output now performs zero terminal I/O.

### Fixed

- `tokio` `io-util` feature was missing, leaving the `async` `fs`
  read/write-with-progress helpers uncompilable under some feature sets.
- Prompt error/validation messages now render through the console instead of raw
  `eprintln!`.

### Internal

- CI hardened: `cargo test --all-features` (now exercises the `async`/`http`
  doctests + builds examples), `cargo clippy --all-features --all-targets
  -D warnings`, and a `--no-default-features` job. Cleaned 30 test/example lints.

## [1.5.3] - 2026-06-05

### Added

- Live-rendering examples showcasing v1.5.2's any-`Renderable` `Live`:
  - `examples/live_markdown.rs` — stream a Markdown document token-by-token
    (the LLM-streaming pattern), reflowing at terminal width each frame.
  - `examples/live_tree.rs` — a build/deploy pipeline as a `Tree` that fills
    in as steps run.
  - `examples/live_status_panel.rs` — a framed `Panel` installer whose body
    updates in place.

## [1.5.2] - 2026-06-05

### Changed

- **`Live` now displays any `Renderable`, rendered against its own console each
  frame.** Previously `Live` stored a flattened `Text`, and updating it with a
  widget flattened that widget through a throwaway default console — so a live
  `Markdown`/`Table`/`Tree`/`Panel` was laid out at the wrong width and ignored
  the live console's theme. `Live` now holds an `Arc<dyn Renderable + Send +
  Sync>` and re-renders it through the live console on every refresh, so width
  and theme are always correct and the display is resize-responsive. This makes
  `live.update_renderable(Markdown::new(md), true)` the faithful equivalent of
  rich's `live.update(Markdown(...))` for streaming output. The lock-free
  `ArcSwap` update path (no renderer contention) is preserved.

  API (technically breaking): `Live::new` / `update_renderable` / `update` /
  `set` / `run` / `set_renderable_widget` are now generic over
  `impl Renderable + Send + Sync + 'static` (passing `Text` still works);
  `Live::from_renderable` takes its renderable **by value** and stores it (was
  `&R`, snapshot-only); `with_get_renderable`'s closure returns
  `Arc<dyn Renderable + Send + Sync>`; added `current_renderable()` and
  `render_to_text()`. The same applies to `LiveRender`. `Console::render`,
  `render_lines`, `render_widget_to_text`, `measure`, `print`, and
  `print_styled` are now generic over `R: Renderable + ?Sized` (source-compatible).

## [1.5.1] - 2026-06-05

### Added

- **`terminal-size` feature (default-on, native only).** `Console::detect_terminal_size()`
  now queries the real terminal dimensions via `ioctl` (using the `terminal_size`
  crate) instead of being pinned to 80 columns when `COLUMNS` is not exported —
  which is the common case, since most shells do not pass `COLUMNS` to child
  processes. Resolution order is `COLUMNS`/`LINES` env vars → `ioctl` query →
  `80x25` fallback, so tests, CI, and piped/redirected output stay deterministic.
  The feature is excluded from `--no-default-features` (and therefore the wasm
  build), keeping those paths free of terminal syscalls; pass an explicit
  `Console::builder().width(..)` there.

## [1.5.0] - 2026-06-05

A correctness + parity + performance pass driven by a full multi-agent
audit against the Python `rich` reference (see
`.review/ultracode-review-2026-06-05.md` and
`.review/ultracode-fixes-2026-06-05.md`). ~95 confirmed findings fixed
across every subsystem.

> **Note:** this release contains API changes that are technically breaking
> (listed below). They are shipped under a minor version; pin `gilt = "=1.4.1"`
> if you depend on the old `Layout`/`ConsoleOptions`/`Prompt` signatures or the
> previous `Traceback`/`RichHandler`/`Rule` defaults.

### Changed (API — technically breaking)

- `ConsoleOptions.encoding`: `String` → `Cow<'static, str>`.
- `Layout.renderable`: `Option<String>` → `Option<Arc<dyn Renderable + Send + Sync>>`;
  panes can now hold any renderable. `effective_renderable()` returns
  `Option<&dyn Renderable>`. New: `update_renderable`, `with_renderable`,
  `refresh_screen`, `by_name`, `From<&str>/From<String>`.
- `Splitter::divide` takes `&[&Layout]` (was `&[Layout]`).
- `Syntax.highlight_lines`: `Vec<usize>` → `HashSet<usize>`.
- `Pretty::rebuild_json` gains a `max_width` parameter.
- `Prompt::ask*` take `&mut self`.
- `ProgressReader<R>` → `ProgressReader<'p, R>` (now sound — `unsafe` removed).
- Default changes: `Traceback::word_wrap` now `false`, `RichHandler::enable_link_path`
  now `true`, `Rule` default line is light `─`, log level colors realigned to rich.

### Added

- `Style::chain` and `Style::normalize`; parser accepts `underline_style`/
  `underline_color(...)` tokens (Display↔parse round-trip).
- `Panel::safe_box` + `with_safe_box`; `Tree::highlight` + `with_highlight`.
- `Syntax::with_padding`, `Syntax::stylize_range_linecol`.
- `Pretty::max_depth` + `with_max_depth`.
- `Prompt::confirm_with_default` / `confirm_with_input_and_default`.
- `console::detect_color_system_from`; `error::fmt_time_hms`.

### Fixed (parity with rich)

- Color: `blend_rgb` truncates (not rounds). Cells: `DEL` is 0-width;
  `chop_cells` is grapheme-aware. Text: column-aligned tab stops; `wrap`
  honors `overflow=Fold`; `append_text` keeps the base style; `highlight_words`
  drops non-rich `\b` anchors; `align` truncates before padding; whitespace
  `measure` returns `min=max_text_width`.
- Console: color-system detection honors `COLORTERM`/`TERM`; `control()` is
  silent on dumb terminals; HTML/SVG export emit hyperlinks; `SVG_EXPORT_THEME`
  restored to 8 normal + 8 bright.
- Table: collapsed padding `max(0, left−right)`; top/bottom cell padding emitted.
  Panel: `measure()` accounts for title + longest line; honors box substitution
  (ascii). Box: substitutes to ASCII. Rule: light default. Tree: independent
  min/max measure. Columns: preserve cell styling; no div-by-zero.
- Markup: no double-escape corruption; `:emoji:` shortcodes. Syntax: indent
  guides, padding, RGB line numbers, FontStyle bits, line/col `stylize_range`.
  Markdown: corrected hyperlink logic, styled blockquotes, syntax-highlighted
  code blocks, SIMPLE table box, `▌` quote border, image `🌆`, list indent.
- Live/Status: Status spinner animates and `set()` updates the display.
  Traceback: `max_frames=0` shows all frames; `suppress_paths` keeps the frame
  header (prefix match); source frames keep syntax colors. Logging: messages
  pass through `ReprHighlighter`.

### Fixed (correctness)

- `Syntax` no longer panics dedenting multi-byte leading whitespace.
- `Columns` no longer panics on `width=0 && padding=0`.
- `markup::render` no longer corrupts literal `\[`.
- `Console::synchronized` restores terminal sync state on panic (RAII guard).
- `ProgressReader` is memory-safe (lifetime-tied; all `unsafe` removed).
- `Layout` guards negative region coordinates; `ratio_resolve` is overflow-safe.

### Performance

- Live: content updates are lock-free (`ArcSwap`) — worker threads no longer
  stall on the renderer (T8). `Segment::divide` avoids a full slice clone (M-5);
  hot loops get capacity hints. `ConsoleOptions` no longer allocates `"utf-8"`
  per call (T7). `BarColumn` caches its `Console` (T5). Style/color parse caches
  enlarged; integer palette matching. JSON highlighter byte→char index built
  once. Traceback caches source files per render and hoists per-frame styles.
  `diff`/`json`/`sparkline`/`gradient` shed redundant clones (`Cow`).

### Internal

- Repaired 7 bit-rotted `async`/`http` doctests (only run under
  `--all-features`, which CI does not exercise).

## [1.4.1] - 2026-05-06

Patch release. Fixes a Unicode-spec-vs-terminal-reality divergence
that broke `Table` layouts on terminals without color-emoji + ZWJ
font support (most Linux xterm/tmux setups, headless CI).

### Changed

- **`cell_len(text)` now returns the per-codepoint width sum, not the
  cluster-aware `unicode_width::UnicodeWidthStr::width(text)` value.**
  The two differ only for ZWJ sequences:

  | Input | v1.4.0 | v1.4.1 | Most terminals render |
  |---|---:|---:|---:|
  | `👨‍👩‍👧` (ZWJ family) | 2 | 6 | 6 (3 separate emoji) |
  | `🇺🇸` (US flag) | 2 | 2 | 2 |
  | `café` (combining) | 4 | 4 | 4 |
  | `中` (CJK) | 2 | 2 | 2 |
  | `❤️` (VS-16 heart) | 2 | 1 | 1–2 (font-dependent) |

  Tables, columns, padding, and any layout that asks `cell_len("…")`
  now reserves space matching what the terminal actually draws on
  the majority of deployments. Terminals with full color-emoji + ZWJ
  support (kitty, iTerm2, Windows Terminal, alacritty + emoji-aware
  font) will see slightly over-reserved space on ZWJ sequences —
  trade-off chosen because over-reserved looks fine, under-reserved
  breaks layout.

### Fixed

- **`Table` columns containing ZWJ family emoji** (`👨‍👩‍👧`) no longer
  overflow into neighbouring columns on terminals without ZWJ font
  support. The user's bug report screenshot showed this directly.

### Test updates

- `family_zwj_emoji_is_2_cells_not_5_codepoints` →
  `family_zwj_emoji_is_6_cells_terminal_reality` (assertion flipped
  from `cell_len == 2` to `== 6`).
- `cell_len_variation_selector_heart_is_2` →
  `cell_len_variation_selector_heart_is_1` (per-codepoint sum, not
  emoji-presentation cluster width).
- `set_cell_size_truncates_before_zwj_cluster` →
  `set_cell_size_truncates_around_zwj_cluster` (budget bumped 4 → 6
  to fit the now-6-cell family).
- `text_truncate_keeps_zwj_family_intact_when_it_fits` budget
  bumped 4 → 6 for the same reason.

### Unchanged

- `set_cell_size`'s grapheme-cluster iteration (v1.4.0) is preserved.
  When a ZWJ cluster fits the budget, it's kept whole; when the
  budget cuts mid-cluster, the partial is replaced with whitespace
  (no orphan ZWJ joiners).
- Public API.

### gilt-derive

Lockstep version bump to 1.4.1; no source changes.

## [1.4.0] - 2026-05-06

Unicode-correctness pass. Visible width math now treats multi-codepoint
graphemes as single visible units; truncation never leaves a dangling
ZWJ joiner or splits a flag emoji's regional-indicator pair. Public API
unchanged — downstream code gets correct output without modifications.

### Added

- **`unicode-segmentation = "1"`** as a direct dependency (~50 KB
  compiled, MIT, zero transitive deps). Builds for `wasm32-unknown-unknown`.
- **README "Unicode handling" section** documenting what's supported
  (ASCII, CJK, single-emoji, ZWJ clusters, flag emoji, variation
  selectors, combining marks) and what's deferred (bidi, NFC/NFD).
- **`tests/unicode_edge_cases.rs`** — 11 end-to-end assertions
  covering ZWJ family emoji, US flag, combining acute, Hangul,
  variation-selector heart, and grapheme-safe `set_cell_size` /
  `Text::truncate`.

### Fixed

- **Five `.chars().count()` sites that miscounted multi-codepoint
  graphemes as visible width** now route through `cell_len()`:
  - `src/accordion.rs:503` icon width
  - `src/gradient.rs:175` gradient justify-padding
  - `src/error/logging_handler.rs:304` repeated-time blank padding
  - `src/utils/bar.rs:182` bar body width
  - `src/utils/bar.rs:188` bar prefix width
- **`set_cell_size` (and every caller — `Text::truncate`,
  `Text::right_crop`, etc.) iterates extended grapheme clusters
  instead of codepoints** when cropping. A 3-cell crop of
  `"👨‍👩‍👧 family"` no longer emits `"👨\u{200d}👩"` with a dangling
  ZWJ; it replaces the partial cluster with whitespace.

### Improves on rich (the upstream Python library)

Rich's `cells.py:194-199` hardcodes special-case logic for `\u200d`
(ZWJ) and `\ufe0f` (VS-16) only; flag emoji measure as 2 separate
cells in rich (silent breakage). gilt v1.4 uses true UAX #29
extended grapheme clusters everywhere — flag emoji, ZWJ families,
combining marks, and any future emoji-cluster format are handled
uniformly without per-cluster special cases.

### Performance

`cargo bench --bench benchmarks` against the v1.3.1 baseline:
**24 statistically-significant improvements (most in the 30-70%
range), 0 statistically-significant regressions.** Comparison archived
at `thoughts/research/2026-05-06-bench-comparison.txt`.

### gilt-derive

Lockstep version bump to 1.4.0; no source changes.

## [1.3.1] - 2026-05-06

Patch release. Fixes a `Console: !Sync` regression introduced in v1.2.0
plus stale doc-comment version refs in `src/lib.rs`.

### Fixed

- **`Console: Sync` restored.** The `writer_override` field added in
  v1.2.0 was typed `Option<Box<dyn Write + Send>>` (no `+ Sync`),
  which silently dropped the `Sync` impl on `Console` between v1.1.0
  and v1.2.0. Code that wrapped a `Console` in `Arc<…>` for
  cross-task sharing would have started failing to compile after
  upgrading. Field is now `Option<Box<dyn Write + Send + Sync>>`,
  matching the `+ Send + Sync` bounds on the public
  `Console::with_writer<W>` method (also tightened to require `Sync`).
- **`src/lib.rs:16`/`:573`** stale `gilt = "0.10"` strings in the
  crate-level docstring updated to `gilt = "1.3"`. These were
  rendering on docs.rs; users copying from there hit a
  six-major-versions-old version.

### Added (regression guards)

- `console::tests::console_is_send_and_sync` — compile-time assertion
  that `Console: Send + Sync`. This is the test that would have caught
  the v1.2.0 regression.
- `console::tests::with_writer_routes_output_to_buffer` — exercises
  the `with_writer` override end-to-end (was previously untested).

### Compatibility

`Console::with_writer<W>` now requires `W: Write + Send + Sync + 'static`
(was `Send + 'static`). This is technically a tightened trait bound,
but in practice any writer that was usable as `Send + 'static` and
not `Sync` would have been awkward to share across threads anyway.
The 0 in-tree callsites + the absence of `with_writer` from the public
v1.2.0 examples make this a no-op for downstream code.

### gilt-derive

Lockstep version bump to 1.3.1; no source changes.

## [1.3.0] - 2026-05-06

WebAssembly compatibility release. No source changes — gilt was already
WASM-friendly (no `libc`, `crossterm`, or terminal-syscall dependencies)
since v1.2.0; this release just documents the path and adds CI coverage.

### Added

- **`examples/wasm_export.rs`** — demo of the `record(true)` +
  `export_text(styles=true)` / `export_html` pipeline. Output suits
  xterm.js (ANSI) or direct DOM injection (HTML).
- **`README.md`** new "WebAssembly" section pointing at the demo.
- **CI**: new `WASM build` job that compiles gilt for
  `wasm32-unknown-unknown` with `--no-default-features --features
  json,markdown,syntax`.

### Verified

- `cargo build --target wasm32-unknown-unknown` clean with default
  features (logging + interactive included — `rpassword` and `log`
  both build for wasm32, the methods that would actually need stdin
  are unreachable in browser usage).

### gilt-derive

Lockstep version bump to 1.3.0; no source changes.

### Why this is its own release vs folded into 1.2.0

The 1.2.0 changelog already shipped before the WASM verification
landed. Documenting WASM support is non-trivial (changes README +
ships a new example + adds a CI job) and is a discoverability win —
worth a release note. Fully additive, no breakage.

## [1.2.0] - 2026-05-06

Additive release. One new public API; everything else is internal
file restructuring that downstream code doesn't see.

### Added

- **`Console::with_writer<W: std::io::Write + Send + 'static>(self, w: W) -> Self`**
  routes terminal output to any user-supplied sink instead of the
  default `std::io::stdout()`. Useful for log-to-file, in-memory
  testing, or piping into a network socket. Capture and record modes
  still take precedence when active.

  ```rust
  use gilt::console::Console;

  let buf: Vec<u8> = Vec::new();
  let mut console = Console::default().with_writer(buf);
  console.print_text("hello");          // bytes go to `buf`, not stdout
  ```

  Closes a friction surfaced in the v1.x async-surface audit
  (`thoughts/research/2026-05-06-async-surface.md`): `write_segments`
  and `flush_buffer` were hard-coded to `std::io::stdout()`, less
  swappable than Python rich's `Console(file=...)`.

### Internal (no API surface change)

- **`src/console.rs` reorganised** from 3814 lines to 1117 (-71%) by
  extracting six focused sibling files via `#[path] mod` declarations.
  Each sibling adds methods to `Console` through a separate
  `impl Console { ... }` block. Tests still pass at every original
  path; downstream `use gilt::console::Console` is unchanged. Files:
  - `src/console_tests.rs` — test module
  - `src/console_export.rs` — HTML/SVG generation helpers
  - `src/console_builder.rs` — `ConsoleBuilder`
  - `src/console_capture.rs` — `begin_capture`, `end_capture`,
    `render_widget_to_text`
  - `src/console_render.rs` — render path, print methods, segment
    output, buffering
- **5 large files split similarly** (sibling `_tests.rs` via
  `#[path]`):
  - `src/prompt.rs`: 2004 → 1084 (-920)
  - `src/style.rs`: 1938 → 1046 (-895)
  - `src/utils/pretty.rs`: 1525 → 702 (-826)
  - `src/markdown.rs`: 1346 → 691 (-655)
  - `src/error/traceback.rs`: 1398 → 789 (-612)
- **`crates/gilt-derive/src/lib.rs`**: 2084 → 196 (-1888) by
  extracting the 1887-line inline test module to
  `crates/gilt-derive/src/tests.rs`. Documented as planned in the
  v0.11.4 CHANGELOG.
- **Capture-and-from-ansi roundtrip deduplicated** across `Live`,
  `Panel`, and `Columns` callers via a single
  `Console::render_widget_to_text` method (was inlined three times
  with subtle differences).
- **Per-frame `Console::default()` allocation fix in `Columns`**:
  the widget-render path was creating a fresh `Console::default()`
  per widget (clones a 153-entry `DEFAULT_STYLES` HashMap + 4 env
  probes per call). Now reuses one `Console` per render — the only
  measurable perf bug found during the v1.0/v1.1 cleanup.

### Documentation

- Two memory artifacts captured under `thoughts/`:
  - `thoughts/research/2026-05-06-async-surface.md` — surface audit
    that surfaced the `with_writer` gap.
  - `thoughts/research/2026-05-06-v2-breaking-impact.md` — call-site
    counts and migration cost for v2.0 deferred items.
  - `thoughts/research/2026-05-06-novel-features.md` — WASM,
    flex-layout, theme-registry feasibility ranking.
  - `thoughts/research/2026-05-06-phase45-console-split.md` —
    rationale for the multi-`impl Console` pattern.

### gilt-derive

Lockstep version bump to 1.2.0; no source changes in the derive crate
since 1.1.0.

## [1.1.0] - 2026-05-05

Closes the three v1.0 deferred items (#28). Two of them turned out to
be discoverability problems rather than missing code — the Traceback
widget and a deeper Pretty already existed in v1.0; v1.1 adds the
paired examples that surface them.

### Added

- **`Columns::from_renderables<I, R>(I)`** — accepts an iterable of any
  `Renderable + Send + Sync + 'static` widget (Spinners, Panels, Tables,
  nested Columns, …). Internally stored as
  `Vec<Arc<dyn Renderable + Send + Sync>>` on the new `widgets` field;
  when non-empty, the render path uses these instead of the existing
  string list. The `add_renderable(&str)` / `from_items(I)` paths are
  unchanged.
- **`Panel::from_renderable(&R)`** — captures any Renderable through a
  temporary console and parses back to Text. Mirrors
  `Live::from_renderable`.
- **`examples/spinners.rs`** rewritten — 53 → 29 lines (1.26× rich), now
  matches rich's `Columns([Spinner(name) for name in SPINNERS])` pattern.
- **`examples/exception.rs`** new — 43 lines (1.05× rich's
  `exception.py`), demonstrates the existing `Traceback::from_error`
  widget on a fallible-divide loop.

### Internal

- `Columns` lost its `Debug` derive (`Arc<dyn Renderable>` doesn't
  implement Debug). `Clone` retained via Arc-clone semantics.
- gilt-derive bumps in lockstep to 1.1.0; codegen unchanged from
  1.0.0.

### Final tier metrics (improved from v1.0)

- Tier-1 (12 entry-level examples): **1.08×** rich (was 1.16× at v1.0)
- Tier-2 (full 36-example corpus): **1.94×** rich (was 1.98×)

### Discovery findings

- **`Traceback` widget** already existed in v1.0 at
  `src/error/traceback.rs` — fully-implemented `Renderable` with
  `from_backtrace` / `from_error` / `from_panic` constructors and
  syntax-highlighted source frames. The v1.0 audit's claim of a missing
  widget was a discoverability issue.
- **`Pretty`** at `src/utils/pretty.rs` is **1525 lines** vs rich's
  `pretty.py` at **1016 lines** — gilt's is *deeper*, not a stub. Same
  audit error.

Both findings reduced v1.1's scope from "build new widgets" to "add
paired examples" — no new widget code shipped.

## [1.0.0] - 2026-04-29

The ergonomics overhaul. v1.0 closes the rich-parity gap on entry-level
examples while keeping idiomatic Rust at the library level. Both
verification gates passed against rich's example corpus:

- **Tier-1** (12 entry-level examples): **1.16×** rich line count (target ≤1.30)
- **Tier-2** (full 36-example paired corpus): **1.98×** rich line count (target ≤2.00)

Single example to motivate: `examples/status.rs` is **13 lines** (rich
is 13). It was 49 lines in v0.13.x.

See [MIGRATION_v1.md](MIGRATION_v1.md) for the complete before/after
guide. Highlights:

### Breaking changes

- **`Style::parse(s)`** is now lossy: returns `Style`, falls back to
  `Style::null()` on bad input. Drops the
  `.unwrap()` / `.unwrap_or_else(|_| Style::null())` ceremony at every
  static-literal callsite. Use `Style::parse_strict(s) -> Result<…>`
  for user-supplied input where syntax errors should surface.
- **`Text::styled(content, "bold red")`** now takes a markup-string
  style. The previous `Text::styled(content, Style)` form is renamed
  to `Text::styled_with`.

### Additive (new ergonomic APIs)

- **`Console::default()`** — already existed; now the documented
  one-line entry point. Auto-detects color from `NO_COLOR` /
  `FORCE_COLOR` / `CLICOLOR`.
- **`Status::run(s) -> Self`** + **`Status::set(&str)`** — auto-start
  constructor and direct setter. Drops the
  `update().status().apply().unwrap()` chain.
- **`Live::run(t)`** + **`Live::set(t)`** — same shape for `Live`.
  Plus **`Live::from_renderable(&widget)`** + 
  **`Live::set_renderable_widget(&w)`** for live-updating any
  `Renderable` (Table, Panel, Tree, …) without manual capture
  roundtrips.
- **`Table::with_columns([(header, justify), …])`** — collapses 7-line
  per-column `add_column(_, _, ColumnOptions { justify: … })` pattern.
  Markup in `Table::add_row` already worked in 0.13.x; v1.0 documents
  this prominently.
- **`Padding::wrap(content, pad)`** — simple constructor. `pad`
  accepts `usize` (uniform), `(v, h)` tuple, or `(t, r, b, l)` tuple
  via new `From` impls on `PaddingDimensions`.
- **`Columns::from_items(I)`** — `IntoIterator<Item = impl Into<String>>`
  constructor.

### Documentation

- All paired examples rewritten to demonstrate the new ergonomic
  surface. `examples/columns.rs` and `examples/rainbow.rs` now have
  fewer lines than their rich equivalents.
- Per-method rustdoc updated to point users at the recommended v1.0
  entry points (`Console::default`, `Status::run`, `Live::run`,
  `Padding::wrap`, etc.) — the original `new()` / `builder()` /
  `start()` methods remain for advanced use.

### Internal

- Migration sweep across 110+ callsites of `Style::parse(s).unwrap()`
  and `.unwrap_or_else(|_| Style::null())` reduced gilt's library
  source by ~70 LOC before any example changes.
- gilt-derive bumps in lockstep to 1.0.0; codegen unchanged from
  0.13.x.

### Deferred to v1.1

- Standalone `Traceback` widget that renders an arbitrary error as an
  embeddable panel. v1.0's tracing/eyre/miette handlers still cover
  the `?`-propagation path.
- Recursive `Pretty` printer with `__rich_repr__`-equivalent protocol.
  Current `Pretty::from_debug` still works; deeper introspection
  arrives in v1.1.
- `Columns` accepting `Renderable` items (currently text-only). This
  unblocks rewriting `examples/spinners.rs` to match rich's pattern.

## [0.13.0] - 2026-04-27

Breaking release. Removes the three legacy derive aliases that were
deprecated in v0.12.0.

### Removed (breaking)

- **`gilt::DeriveColumns`** — use `gilt::derives::Columns`
- **`gilt::DeriveInspect`** — use `gilt::derives::Inspect`
- **`gilt::DeriveRule`** — use `gilt::derives::Rule`

The `gilt::derives::*` namespace (added in v0.11.3) has been the
recommended import surface for two minor releases. All seven derives
(`Table`, `Panel`, `Tree`, `Columns`, `Rule`, `Inspect`, `Renderable`)
remain available there.

### Migration

```rust
// Before (v0.12.x — deprecated):
use gilt::DeriveColumns;
#[derive(DeriveColumns)] struct Item { name: String }

// After (v0.13.0):
use gilt::derives::Columns;
#[derive(Columns)] struct Item { name: String }
```

The non-colliding derives (`Table`, `Panel`, `Tree`, `Renderable`)
continue to work from the crate root unchanged — only the three aliases
that overlapped with widget type names were affected.

### Internal

- `crates/gilt-derive/tests/compile_fail/inspect_on_union.rs` migrated
  to `gilt::derives::Inspect` (the `.stderr` golden continues to point
  at the union with the same message).
- `gilt-derive` itself is unchanged (lockstep version bump only).

## [0.12.0] - 2026-04-27

Closes the gilt-derive consolidation plan by deprecating the v0.11.x
legacy derive aliases.

### Deprecated

- **`gilt::DeriveColumns` / `gilt::DeriveInspect` / `gilt::DeriveRule`**
  top-level re-exports. These are still functional but emit
  deprecation warnings. Use the `gilt::derives::*` namespace (added in
  v0.11.3) instead:

  ```rust
  // Before:
  use gilt::DeriveColumns;
  #[derive(DeriveColumns)] struct ... { }

  // After:
  use gilt::derives::Columns;
  #[derive(Columns)] struct ... { }
  ```

### Removal timeline

- v0.12.0 (this release): warnings on use.
- v0.13.0+ (next minor or major): aliases removed entirely.

### Internal

- All in-tree examples (`derive_columns.rs`, `derive_inspect.rs`,
  `derive_rule.rs`, `showcase.rs`) migrated to `gilt::derives::*`.
- `gilt-derive` itself is unchanged (lockstep version bump only).

### gilt-derive consolidation plan: complete

| Phase | Shipped |
|---|---|
| 1: README + CHANGELOG | v0.11.2 |
| 2: panic-free + spans | v0.11.2 |
| 3: trybuild + insta | v0.11.3 |
| 4: `gilt::derives` namespace | v0.11.3 |
| 5: per-derive module split | v0.11.4 |
| 6: deprecate legacy aliases | **v0.12.0** (this) |

## [0.11.4] - 2026-04-27

Lockstep release with `gilt-derive 0.11.4` — internal file split of the
4667-line `crates/gilt-derive/src/lib.rs` into per-derive modules. No
`gilt` (main crate) source changes; this version exists to publish
alongside `gilt-derive 0.11.4` and pin the new minimum derive version.

### gilt-derive 0.11.4 highlights

- 7 per-derive modules + shared helpers (`crates/gilt-derive/src/`):
  `table.rs`, `panel.rs`, `tree.rs`, `columns.rs`, `rule.rs`,
  `inspect.rs`, `renderable.rs`, `shared.rs`. lib.rs reduced from 4667
  to 2085 lines.
- Insta snapshots byte-identical to v0.11.3 — codegen verified stable
  across the refactor.
- 113 unit tests + 6 trybuild tests still green.

See `crates/gilt-derive/CHANGELOG.md#0114---2026-04-27` for details.

## [0.11.3] - 2026-04-27

Adds the `gilt::derives` namespace and ships `gilt-derive 0.11.3` with
trybuild + insta test infrastructure.

### Added

- **`gilt::derives` module** — single namespace for all derive macros,
  sidestepping the `Columns` / `Inspect` / `Rule` name collisions with
  runtime widget types:
  ```rust
  use gilt::derives::{Columns, Inspect, Panel, Renderable, Rule, Table, Tree};

  #[derive(Columns)]   // unambiguous — refers to gilt_derive::Columns
  struct Item { name: String }
  ```
  Top-level `pub use gilt_derive::Columns as DeriveColumns;` etc. are
  kept for backward compatibility. **Planned deprecation:** v0.12.0.

### gilt-derive 0.11.3 highlights

- New `trybuild` test suite with compile-pass smoke tests (one per
  derive) and compile-fail tests with `.stderr` goldens (regression
  guards for the v0.11.2 panic-removal work).
- New `insta` snapshot tests catching codegen drift on the 7 derives.

See `crates/gilt-derive/CHANGELOG.md#0113---2026-04-27` for details.

### Deferred to v0.12.0

- Splitting `crates/gilt-derive/src/lib.rs` (4500 lines) into per-derive
  modules.
- Deprecating the legacy `DeriveColumns`/`DeriveInspect`/`DeriveRule`
  re-exports (one minor of overlap before deprecation warnings).

## [0.11.2] - 2026-04-27

Lockstep release with `gilt-derive 0.11.2` — derive crate robustness
improvements. No `gilt` (main crate) source changes; this version
exists to publish alongside `gilt-derive 0.11.2` and pin the new
minimum derive version.

### gilt-derive 0.11.2 highlights

- **Panic-free proc-macros.** All `.expect()` / `.unwrap()` paths in
  derive code replaced with `syn::Error` returns. Malformed input now
  produces actionable `compile_error!` diagnostics instead of
  proc-macro panics.
- **Better error spans.** `Renderable` derive's `via` attribute
  validation restructured to keep the `LitStr` in scope for spanned
  errors.
- **`crates/gilt-derive/README.md`** added — landing page for crates.io.
- **`crates/gilt-derive/CHANGELOG.md`** added — independent release
  history.

See `crates/gilt-derive/CHANGELOG.md#0112---2026-04-27` for the full
entry.

### Deferred to v0.11.3+

- `trybuild` compile-fail tests (per-derive error message regression
  guard) + `insta` snapshot tests for generated code.
- Splitting the 4500-line `crates/gilt-derive/src/lib.rs` into
  per-derive modules + shared utilities.
- Adding a `gilt::derives::*` namespace to sidestep the
  `DeriveColumns`/`DeriveInspect`/`DeriveRule` rename collisions with
  runtime widget types.

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
