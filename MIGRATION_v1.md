# Migrating to gilt v1.0

This document describes every breaking change between gilt 0.13.x and 1.0.0,
with concrete before/after code. Each phase of the v1.0 ergonomics overhaul
adds entries here as it lands; see [issue #20](https://github.com/khalidelborai/gilt/issues/20)
for the master plan.

> **TL;DR for users**: most code becomes shorter. The most common change is
> dropping `?` / `.unwrap()` / `.unwrap_or_else(|_| Style::null())` on
> `Style::parse(s)` calls — they're now infallible at the type level.

---

## Phase 1 — Foundation (lossy parsers, ergonomic constructors)

### `Style::parse` is now lossy (returns `Style`, not `Result`)

**Why this is a silent-failure trap on upgrade**: the old form returned
`Result<Style, StyleError>`. Code that wrote `Style::parse(s)?` will now
get a *type error* (good — forces a decision). But code that wrote
`Style::parse(s).unwrap()` or `Style::parse(s).unwrap_or_else(|_| ...)`
will quietly compile-fail with a method-not-found error and developers
*may* be tempted to "fix" it by stripping the `.unwrap()`. **Read the
guidance below before deleting**.

#### When to drop `.unwrap()` (the common case)

If your input is a static string literal you already trust to be valid
(`"bold red"`, `"on blue"`, etc.), drop the `.unwrap()`:

```rust
// Before (v0.13.x):
let style = Style::parse("bold red").unwrap();
let style = Style::parse("bold red").unwrap_or_else(|_| Style::null());

// After (v1.0):
let style = Style::parse("bold red");          // returns Style directly
```

This is what 95%+ of in-tree callsites were doing — gilt itself dropped
hundreds of these wrappers in the v1.0 sweep.

#### When to switch to `Style::parse_strict` (the dynamic case)

If your input is **user-supplied** (config file, CLI flag, environment
variable, network input) and you want to surface a syntax error instead
of silently rendering as null, use `Style::parse_strict`:

```rust
// Before (v0.13.x):
let style = Style::parse(&user_input)?;

// After (v1.0):
let style = Style::parse_strict(&user_input)?;
```

`parse_strict` returns the same `Result<Style, StyleError>` the old `parse`
returned. The function is unchanged; only its name is.

#### Quick decision table

| Old code | New code | Notes |
|---|---|---|
| `Style::parse("bold")?` | `Style::parse_strict("bold")?` | Strict if you propagate errors |
| `Style::parse("bold").unwrap()` | `Style::parse("bold")` | The boilerplate-free common case |
| `Style::parse("bold").unwrap_or_else(\|_\| Style::null())` | `Style::parse("bold")` | Identical behaviour, drops the wrapper |
| `Style::parse(x).is_err()` (in tests) | `Style::parse_strict(x).is_err()` | Strict for failure-case tests |
| `Style::parse(x).map_err(...)` | `Style::parse_strict(x).map_err(...)` | Strict where you transform errors |
| `Style::parse(x).ok()` | `Style::parse_strict(x).ok()` | Strict for the Option<Style> path |
| `Style::parse(x).expect("...")` | `Style::parse_strict(x).expect("...")` | Strict for explicit panics |

---

### `Text::styled` now takes a markup-string style

`Text::styled` previously took a pre-built `Style`. Most callsites were
typing `Text::styled(content, Style::parse("bold"))` — three layers of API
for one ergonomic concept. v1.0 collapses this:

```rust
// Before (v0.13.x):
let warn = Text::styled("watch out", Style::parse("bold yellow").unwrap());

// After (v1.0):
let warn = Text::styled("watch out", "bold yellow");
```

The content arg is treated as **literal** — markup tags inside the content
string are not parsed. To parse markup-in-content, use the existing
`Text::from_markup`, which is unchanged.

#### When to use the new `Text::styled_with`

If you already have a `Style` value (built via `Stylize`, derived from a
theme, computed by your code), use `Text::styled_with`:

```rust
// You have a Style you want to attach to text
let s: Style = ...;
let labelled = Text::styled_with("label", s);
```

The signature is `Text::styled_with(text: impl Into<String>, style: Style)`.
This is the renamed form of the old `Text::styled(text: &str, style: Style)`.

#### Quick decision table

| Old code | New code | Notes |
|---|---|---|
| `Text::styled(s, Style::parse("bold").unwrap())` | `Text::styled(s, "bold")` | Common case |
| `Text::styled(s, my_style)` | `Text::styled_with(s, my_style)` | When you have a Style value |
| `Text::styled(s, Style::null())` | `Text::styled_with(s, Style::null())` | Same |

---

### `Console::default()` is now the recommended entry point

`Console::default()` already existed in 0.13.x but its visibility was poor —
every example used `Console::builder().force_terminal(true).no_color(false).build()`.
v1.0 documents `Console::default()` prominently as the one-line entry point:

```rust
// Verbose (still works in v1.0):
let console = Console::builder()
    .force_terminal(true)
    .no_color(false)
    .build();

// Recommended (v1.0):
let console = Console::default();
```

`Console::default()` auto-detects:
- Terminal width from the underlying TTY (falls back to 80)
- Color support from `NO_COLOR` / `FORCE_COLOR` / `CLICOLOR` env vars
- Defaults to TrueColor when no environment override is present

Use `Console::builder()` only when you need explicit overrides
(custom width, recording for export, forcing a specific color system).

**No code changes are required** — this is purely a documentation and
example-style improvement. Existing `Console::builder()` callsites continue
to compile and behave identically.

---

---

## Phase 2 — Live family ergonomic setters

`Live` and `Status` already had `impl Drop` calling `stop()` — Phase 2 didn't
need to introduce `*Guard` wrapper types. The actual ergonomic gap was the
`update().X().apply().unwrap()` chain and explicit `start()` ceremony at
every call site. Phase 2 closes both with direct setters and auto-start
constructors.

### `Status::set` direct setter (replaces `update().status().apply().unwrap()`)

```rust
// Before (v0.13.x):
status.update().status("step 2").apply().unwrap();

// After (v1.0):
status.set("step 2");
```

`Status::update().status(...).apply()` still exists for atomic multi-field
updates (status text + spinner name + style + speed in one transaction).
`set` is the direct setter for the common single-field case.

### `Status::run` auto-start constructor

```rust
// Before (v0.13.x):
let mut status = Status::new("Loading...").with_console(Console::default());
status.start();

// After (v1.0):
let mut status = Status::run("Loading...");           // construction + start
// .with_console(...) etc. still chains on Status::new if you need overrides
```

`Status::run` is `Status::new` + `start` in one call. The returned value
implements `Drop`, so going out of scope automatically stops the display —
no explicit `stop()` call required.

### `Live::set` and `Live::run`

```rust
// Before:
let mut live = Live::new(initial);
live.start();
live.update(new_text, true);

// After:
let live = Live::run(initial);   // construction + start
live.set(new_text);              // no boolean noise
```

`Live::set(r)` is equivalent to `update(r, true)`. The original
`update(r, refresh)` is unchanged for callers who need explicit refresh
control.

### Implicit-Drop semantic, made explicit in docs

Both `Live` and `Status` already implemented `Drop` calling `stop()` in
v0.13.x — but rustdoc didn't make this prominent and most examples called
`stop()` manually. v1.0 documents this clearly: **you do not need to call
`stop()` if you let the value go out of scope normally.** Manual `stop()`
remains supported for cases where you want to stop early (e.g., before a
long synchronous operation that itself prints).

### Quick decision table

| Old code | New code | Notes |
|---|---|---|
| `Status::new(s); status.start()` | `Status::run(s)` | Auto-start constructor |
| `status.update().status(s).apply().unwrap()` | `status.set(s)` | Direct setter |
| `live.update(r, true)` | `live.set(r)` | Drops the boolean noise |
| `Live::new(r); live.start()` | `Live::run(r)` | Auto-start constructor |
| `status.stop()` (at end of fn) | (nothing) | Drop calls stop automatically |

---

---

## Phase 3 — `Live::from_renderable` for any widget

`Live::new` accepts only `Text`. v0.13.x users wanting to live-update a
`Table`, `Panel`, `Tree`, or `Layout` had to write a manual capture
roundtrip:

```rust
// Before (v0.13.x):
let mut tmp = Console::builder().force_terminal(true).record(true).build();
tmp.begin_capture();
tmp.print(&table);
let text = Text::from_ansi(&tmp.end_capture());
let live = Live::new(text).with_transient(true);
```

v1.0 wraps that roundtrip behind `Live::from_renderable<R: Renderable>(&R)`:

```rust
// After (v1.0):
let live = Live::from_renderable(&table).with_transient(true);
```

For tick updates of the same widget shape (e.g. a process table that
re-renders every 500ms), `Live::set_renderable_widget(&new_widget)` is
the matching setter.

The implementation is the same capture roundtrip — it just lives inside
`gilt` now instead of every caller. No performance change; one fewer thing
the user has to learn.

### Quick decision table

| Old code | New code | Notes |
|---|---|---|
| `Live::new(Text::from_ansi(&capture))` | `Live::from_renderable(&widget)` | Construction |
| `live.set(Text::from_ansi(&new_capture))` | `live.set_renderable_widget(&new_widget)` | Tick update |
| `Live::new(text)` | `Live::new(text)` | Unchanged for direct-Text use |

---

---

## Phase 4 — `Table::with_columns` shorthand + markup confirmation

### Markup in `Table::add_row` was already supported

`Table::add_row(&["[bold green]active[/]"])` already parses markup
correctly in v0.13.x — `CellContent::Plain` is resolved through
`console.render_str` which honors the markup-enabled flag. v1.0 adds
this fact to the rustdoc on `add_row` and the `Table::with_columns`
example below. **No code change is required at the call site.**

### `Table::with_columns` for the headers + justification case

The common pattern of *"declare headers and their alignment"* needed
seven lines per column in v0.13.x:

```rust
// Before:
let mut t = Table::new(&[]);
t.add_column("PID", "", ColumnOptions { justify: Some(JustifyMethod::Right), ..Default::default() });
t.add_column("Command", "", ColumnOptions { justify: Some(JustifyMethod::Left),  ..Default::default() });
t.add_column("CPU %",   "", ColumnOptions { justify: Some(JustifyMethod::Right), ..Default::default() });
```

v1.0 collapses this to a single iterable:

```rust
// After:
use gilt::text::JustifyMethod;
let mut t = Table::with_columns([
    ("PID",     JustifyMethod::Right),
    ("Command", JustifyMethod::Left),
    ("CPU %",   JustifyMethod::Right),
]);
```

`Table::new(&[...])` is unchanged for the no-justify case.
`add_column(_, _, ColumnOptions { … })` remains for advanced per-column
configuration (min_width, max_width, ratio, etc.) — `with_columns`
covers the >80% case.

### Why Tree didn't get markup-string labels

We considered changing `Tree::new` and `Tree::add` to parse markup from
a `&str`, but it conflicts with Phase 1's "content args are literal"
rule. The markup case stays opt-in:

```rust
let mut t = Tree::new(Text::from_markup("[bold]root[/]").unwrap());
t.add(Text::from_markup("[red]child[/]").unwrap());
```

Future phases may revisit if a clean two-method pattern emerges
(`Tree::new_markup` / `Tree::node_markup`) without violating the literal-content
rule for the structured `Text` overload.

---

---

## Phase 5 — Padding/Columns ergonomics + paired-example rewrites

### `PaddingDimensions` accepts tuples directly

Previously: `PaddingDimensions::Pair(2, 4)`. Now any of `2usize`,
`(2, 4)`, or `(2, 4, 2, 4)` works wherever `impl Into<PaddingDimensions>`
is accepted.

```rust
// All three are equivalent inputs to Padding::wrap
let p1 = Padding::wrap(text.clone(), 2);                  // uniform
let p2 = Padding::wrap(text.clone(), (2, 4));             // (vertical, horizontal)
let p3 = Padding::wrap(text,         (2, 4, 2, 4));       // (top, right, bottom, left)
```

### `Padding::wrap` simple constructor

Drops style + expand args from `Padding::new` for the most common case
(no styled padding background, expand to fill width):

```rust
// Before:
let p = Padding::new(content, PaddingDimensions::Pair(2, 4), Style::null(), true);
// After:
let p = Padding::wrap(content, (2, 4));
```

The full `Padding::new(content, pad, style, expand)` remains for advanced
use (styled padding background, fit-to-content).

### `Columns::from_items` constructor

Previously you constructed an empty `Columns::new()` and called
`add_renderable` per item. v1.0 collapses this:

```rust
// Before:
let mut c = Columns::new();
for item in items { c.add_renderable(&item); }

// After:
let c = Columns::from_items(items);
```

Accepts `IntoIterator<Item = impl Into<String>>`.

### Paired-example rewrites in this phase

| Example | v0.13 LOC | Phase 5 LOC | rich | ratio |
|---|---:|---:|---:|---:|
| padding.rs | 65 | 16 | 5 | 3.20× |
| link.rs | 23 | 7 | 4 | 1.75× |
| columns.rs | 91 | 23 | 28 | 0.82× ← gilt |
| rainbow.rs | 80 | 10 | 20 | 0.50× ← gilt |
| repr.rs | 56 | 47 | 23 | 2.04× |

### Tier gates achieved

- **Tier-1** (12 entry-level examples): **1.16×** vs target ≤1.30 ✓
- **Tier-2** (full 36-example corpus): **1.98×** vs target ≤2.00 ✓

Both gates pass. The remaining v1.0 phases focus on Rust-native
extensions polish, derive macros, the Traceback/Pretty feature gaps, and
final release prep — none of which are blocked on further example
ergonomics.

---

## Post-v1.0 releases

The roadmap items at issue [#20](https://github.com/khalidelborai/gilt/issues/20)
shipped or were dropped:

- **v1.1.0 (2026-05-05)** — paired-example coverage. Added
  `Columns::from_renderables` + `Panel::from_renderable`. The Traceback
  widget and deeper Pretty turned out to already exist (audit was
  wrong); paired examples cover them now. Additive, no migration.
- **v1.2.0 (2026-05-06)** — `Console::with_writer<W>` + internal
  reorganisation of `console.rs` (3814 → 1117 lines via six sibling
  files). Additive at the API level. Note: v1.2.0 silently dropped
  `Console: Sync` (the writer override was missing `+ Sync`); patched
  in v1.3.1.
- **v1.3.0 (2026-05-06)** — WebAssembly compatibility documentation
  + new CI job + `examples/wasm_export.rs`. No source change required.
- **v1.3.1 (2026-05-06)** — patch: restored `Console: Sync` and added
  a compile-time regression test (`console_is_send_and_sync`). Also
  bumped two stale `gilt = "0.10"` references in `src/lib.rs` doc
  comments. **If you were on v1.2.0 or v1.3.0 and your code uses
  `Arc<Console>`, upgrade to 1.3.1.**

For pre-v1.0 → v1.0 migration steps (the bulk of this document),
read on above.
