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

## Future phases

This document grows phase by phase. See [issue #20](https://github.com/khalidelborai/gilt/issues/20)
for the full v1.0 roadmap.
- Phase 4: Markup-first `Table::add_row`, Tree, Columns
- Phase 5: Panel/Padding/Rule/Align unification
- Phase 6: Rust-native extensions consistency
- Phase 6.5: Standalone `Traceback` widget + deeper `Pretty`
- Phase 7: Derive macro polish
