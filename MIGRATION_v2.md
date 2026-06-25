# Migrating to gilt v2.0

gilt 2.0 is the **parity release**: it closes ~190 verified behavior gaps against
Python [rich](https://github.com/Textualize/rich) across every subsystem (see
`CHANGELOG.md` and `.review/parity-audit-2026-06-24.md`). Closing those gaps required
a batch of breaking API changes. This guide lists every one, with a before → after →
fix, grouped by area. Most upgrades are mechanical.

MSRV is unchanged (**1.82.0**). No new dependencies. The WASM-safe surface
(`--no-default-features --features json,markdown,syntax`) is unchanged.

---

## At a glance

| Area | Change | Severity |
|---|---|---|
| Containers | content fields are `RenderableArc`, not `Text` | compile |
| Containers | `Panel::from_renderable` takes `R` by value (was `&R`) | compile |
| Containers | `Group`/`Renderables` elements are `RenderableArc` (was `Text`) | compile |
| Export | `save_html`/`save_svg` gained forwarding params | compile |
| Export | `export_*` **panic** without `record` mode (was empty string) | runtime |
| Progress | `add_task` gained `start: bool`; `update` gained `fields` | compile |
| Progress | `TaskProgressColumn::with_separator` removed | compile |
| Themes | `push_theme`, `Theme::from_file`/`read` gained `inherit: bool` | compile |
| Box/Style | `BoxChars::substitute` & `Style::render` gained `legacy_windows: bool` | compile |
| Syntax | `stylize_range` gained `style_before`; `highlight_code`→`highlight`; `measure(&Console, &ConsoleOptions)` | compile |
| Console | `ConsoleOptionsUpdates::no_wrap` is `Option<Option<bool>>` | compile |
| Text | `Text::justify`/`Lines::justify`/`Text::wrap` take `&Console` | compile |
| Layout | `Layout::refresh_screen` updates in place (no return) | compile |
| Structs | `Span`, `PanicHookConfig`, `IntPrompt`, `FloatPrompt`, `Confirm`, `ConsoleOptionsUpdates` are `#[non_exhaustive]` | compile |
| Color | bright-color downgrade output changed (now rich-faithful) | behavior |

---

## Containers (the `RenderableArc` change)

Every container that held a `Text` body now holds a `RenderableArc`
(`Arc<dyn Renderable + Send + Sync>`), so containers can wrap **any** widget, not
just text (matching rich). Affected: `Panel`, `Tree` node labels, `Group`,
`Renderables`, `Align`, `Padding`, `Constrain`, `Styled`, and `Table` cells.

For the common "wrap a string/Text" case nothing changes — `&str`, `String`, and
`Text` all implement `Renderable` and are accepted directly:

```rust
Panel::new(Text::new("hi", Style::null()))   // still works
Panel::new("hi")                             // also works (str: Renderable)
```

### `Panel::from_renderable` now takes ownership

```rust
// 1.x
let p = Panel::from_renderable(&widget);
// 2.0 — by value (Arc-wrapped internally)
let p = Panel::from_renderable(widget);
```

### `Group` / `Renderables` element type

```rust
// 1.x: Vec<Text>
let g = Group::new(vec![Text::new("a", Style::null()), Text::new("b", Style::null())]);
// 2.0: Vec<RenderableArc> — use into_renderable_arc, or build via the helpers
use gilt::console::into_renderable_arc;
let g = Group::new(vec![into_renderable_arc("a"), into_renderable_arc(panel)]);
```

If you only ever put `Text` in a `Group`, wrap each item with
`into_renderable_arc(...)`.

---

## Export

`save_html`/`save_svg` now forward the same options as their `export_*` counterparts:

```rust
// 1.x
console.save_html("out.html");
console.save_svg("out.svg", "Title");
// 2.0
console.save_html("out.html", /*theme*/ None, /*clear*/ true, /*inline_styles*/ true, /*code_format*/ None);
console.save_svg("out.svg", "Title", None, true, /*unique_id*/ None, None);
```

**The five `export_*` methods now panic if `record` mode is off**, matching rich's
`assert self.record`. Previously they silently returned an empty string. Build the
console with `.record(true)`:

```rust
let mut c = Console::builder().record(true).build();   // required before export_*
```

---

## Progress

```rust
// 1.x
let id = progress.add_task("download", Some(100.0));
progress.update(id, Some(50.0), None, None, None);
// 2.0 — add_task gained `start`, update gained `fields`
let id = progress.add_task("download", Some(100.0), /*start*/ true);
progress.update(id, Some(50.0), None, None, None, /*fields*/ None);
```

`TaskProgressColumn::with_separator` was removed — the column now renders a percentage
(`50%`) to match rich; there is no separator to configure.

---

## Themes

```rust
// 1.x
console.push_theme(theme);
let t = Theme::from_file("theme.ini")?;
// 2.0 — `inherit` controls whether the base theme styles show through
console.push_theme(theme, /*inherit*/ true);
let t = Theme::from_file("theme.ini", /*inherit*/ true)?;
```

`Console::use_theme(theme, inherit)` is new — it returns a `ThemeGuard` (in the
prelude) that pops the theme on drop; render through the guard:

```rust
let mut guard = console.use_theme(theme, true);
guard.print_text("[info]rendered under the theme[/info]");   // theme active here
// guard drops -> theme popped
```

---

## Box / Style (`legacy_windows`)

```rust
// 1.x
let b = box_chars.substitute(ascii_only);
let s = style.render(text, color_system);
// 2.0
let b = box_chars.substitute(ascii_only, /*legacy_windows*/ false);
let s = style.render(text, color_system, /*legacy_windows*/ false);
```

Pass `false` unless you are targeting a legacy Windows console. The console now
auto-detects this (`ConsoleBuilder::with_legacy_windows(bool)` to override).

---

## Syntax

```rust
// 1.x
syntax.stylize_range(style, range);
let t = syntax.highlight_code(code);   // was private/renamed
let m = syntax.measure();
// 2.0
syntax.stylize_range(style, range, /*style_before*/ false);
let t = syntax.highlight(code);                       // public, renamed
let m = syntax.measure(&console, &options);
```

---

## Console options

`ConsoleOptionsUpdates::no_wrap` is now `Option<Option<bool>>` so it can explicitly
**reset** `no_wrap` to `None` (inherit), not only set/leave it:

```rust
// outer None  -> leave unchanged
// Some(None)  -> reset to None (inherit)
// Some(Some(b)) -> set to b
let updates = ConsoleOptionsUpdates { no_wrap: Some(Some(true)), ..Default::default() };
```

(`ConsoleOptionsUpdates` is now `#[non_exhaustive]` — see below.)

---

## Text wrapping / justification take `&Console`

Full-justify and tab-expansion now resolve adjacent-word styles against the console,
matching rich, so these take a `&Console`:

```rust
// 1.x
text.wrap(width, None, None, tab_size, false);
lines.justify(width, JustifyMethod::Full, overflow);
// 2.0
text.wrap(&console, width, None, None, tab_size, false);
lines.justify(&console, width, JustifyMethod::Full, overflow);
```

---

## Layout

`Layout::refresh_screen` now updates the screen region in place (matching rich)
instead of returning segments:

```rust
// 1.x
let segs = layout.refresh_screen(&mut console, "name");
console.write_segments(&segs);
// 2.0
layout.refresh_screen(&mut console, "name");   // writes directly
```

---

## `#[non_exhaustive]` structs

These structs gained fields in 2.0 and are now `#[non_exhaustive]`, so construct them
from `Default` (or a constructor) and set fields, rather than a struct literal:

`PanicHookConfig`, `IntPrompt`, `FloatPrompt`, `Confirm`, `Span`, `ConsoleOptionsUpdates`.

```rust
// instead of a struct literal:
let mut config = PanicHookConfig::default();
config.max_frames = 50;
config.suppress_paths = vec!["/.cargo/".into()];
Traceback::install_panic_hook_with_config(config);
```

`Span` additionally gained `style_name` and `meta` — build named/meta spans with
`Span::named(...)` / `Span::with_meta(...)` rather than `Span::new(...)` when you
need them.

---

## Color downgrade (behavior change, no code change)

Downgrading **bright** colors to the 8-color `Standard` system is now rich-faithful:
bright `EightBit`/`Windows` colors (indices 8–15) nearest-match into 0–7 (e.g.
`EightBit(9)` → `Standard(1)`) instead of producing an out-of-range index. Output for
those specific colors on 8-color terminals changes; no source change is required.

---

## New (additive) APIs worth knowing

Non-breaking, but new in 2.0: `pretty_repr` / `Console::pprint`; `Console::log_objects`;
`RenderHook` + `Console::print_with_hooks`; `Console::input_with_stream`; typed
`IntPrompt`/`FloatPrompt`/`Confirm`; `Style::normalize_str`; `ansi_color_name` /
`get_ansi_color_number`; `cells::split_graphemes` / `split_text`; `Segments` /
`SegmentLines`; `vertical_center`; `print_json_opts`; `FileProxy::isatty`;
`Screen::from_arc`; `markup::render_with_options`; `Console::with_log_time` /
`with_legacy_windows` / `with_highlighter`; `RichHandler`/`GiltLayer` theming +
`min_level`.
