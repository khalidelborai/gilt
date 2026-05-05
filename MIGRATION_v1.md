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

## Future phases

This document grows phase by phase. See [issue #20](https://github.com/khalidelborai/gilt/issues/20)
for the full v1.0 roadmap. Subsequent phases will document:

- Phase 2: RAII guards for Live/Status/Pager/Screen (`start()` returns a Drop guard)
- Phase 3: `Live::from_renderable<R: Renderable>(r)` and `Status::set(&str)` direct setters
- Phase 4: Markup-first `Table::add_row`, Tree, Columns
- Phase 5: Panel/Padding/Rule/Align unification
- Phase 6: Rust-native extensions consistency
- Phase 6.5: Standalone `Traceback` widget + deeper `Pretty`
- Phase 7: Derive macro polish
